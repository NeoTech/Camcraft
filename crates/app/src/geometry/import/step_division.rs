//! The parameter grid a B-spline or NURBS STEP face is meshed on, split in
//! each direction only where that direction needs it (#1602).
//!
//! Provenance: adaptive tensor-product subdivision with a flatness test per
//! parameter direction, as in Piegl & Tiller, "The NURBS Book" (2nd ed.,
//! ch. 5 and the tessellation notes of ch. 12: a patch is flat in one
//! direction when its iso-curves in that direction stand within the
//! tolerance of their chords), and truck's published `parameter_division`
//! (ricosjp, Apache-2.0), whose acceptance test - the surface against the
//! bilinear patch of a cell's corners at a jittered interior point - every
//! cell of our grid still passes. Nothing from any CAM vendor.
//!
//! Why: truck's division marks BOTH parameter intervals of a cell whose
//! test fails, whichever direction caused it. Every sweep then halves
//! every interval of the other direction too, so a face curved in v only
//! (a thread flank: a ruled helical strip, straight across u, fourteen
//! turns along v) still gets 2^sweeps intervals across u. On the #1602
//! models that turned a strip that needs ~2,000 x 2 cells into ~2,000 x
//! 1,000: 2 M facets on one face, the shell's whole budget, and minutes of
//! import.
//!
//! How: per cell, one interior point is compared three ways. Against the
//! chord of its iso-v curve (the cell's u direction) and of its iso-u
//! curve (v): a far u chord splits the cell's u interval, a far v chord
//! its v interval. Only when both chords hold and the bilinear test still
//! fails (a twist) are both split, as truck does. The loop ends when no
//! cell splits, so every cell meets truck's own bilinear test, and the
//! grid is never coarser than the tolerance asks; it is only not finer
//! where nothing is curved.

use truck_stepio::r#in::alias::{EuclideanSpace, HashGen, MetricSpace, ParametricSurface, Point3};

/// How many sweeps the division may take. Each sweep halves a flagged
/// interval, so 64 sweeps reach below f64 resolution of any range: a
/// continuous face is divided long before (14 turns of a thread at 5 um
/// take 11). A face that is still splitting then has a cell that never
/// meets the tolerance - a jump - and is reported as not converged, never
/// returned as a grid: a jump split only across its own direction grows
/// one column a sweep, so the evaluation meter alone would let it through
/// as a wall that is not there.
pub(super) const MAX_SWEEPS: usize = 64;

/// Divide `(urange, vrange)` of `surface` into a grid whose every cell
/// stands within `tol` of the surface (truck's bilinear test), splitting a
/// direction only where it is curved. `tol` must be positive and finite;
/// anything else returns the range's corners alone. `None` when the grid
/// is still splitting after [`MAX_SWEEPS`] (the surface jumps).
pub(super) fn parameter_division<S>(
    surface: &S,
    (urange, vrange): ((f64, f64), (f64, f64)),
    tol: f64,
) -> Option<(Vec<f64>, Vec<f64>)>
where
    S: ParametricSurface<Point = Point3>,
{
    let mut udiv = vec![urange.0, urange.1];
    let mut vdiv = vec![vrange.0, vrange.1];
    if !(tol.is_finite() && tol > 0.0) {
        return Some((udiv, vdiv));
    }
    let tol2 = tol * tol;
    for _ in 0..MAX_SWEEPS {
        let (usplit, vsplit) = sweep(surface, &udiv, &vdiv, tol2);
        if !usplit.contains(&true) && !vsplit.contains(&true) {
            return Some((udiv, vdiv));
        }
        udiv = halve(&udiv, &usplit);
        vdiv = halve(&vdiv, &vsplit);
    }
    None
}

/// One sweep: which u intervals and which v intervals to halve.
fn sweep<S>(surface: &S, udiv: &[f64], vdiv: &[f64], tol2: f64) -> (Vec<bool>, Vec<bool>)
where
    S: ParametricSurface<Point = Point3>,
{
    let mut usplit = vec![false; udiv.len().saturating_sub(1)];
    let mut vsplit = vec![false; vdiv.len().saturating_sub(1)];
    for (u, ub) in udiv.windows(2).zip(usplit.iter_mut()) {
        for (v, vb) in vdiv.windows(2).zip(vsplit.iter_mut()) {
            if *ub && *vb {
                continue;
            }
            let (du, dv) = cell(surface, (u[0], u[1]), (v[0], v[1]), tol2);
            *ub |= du;
            *vb |= dv;
        }
    }
    (usplit, vsplit)
}

/// Whether one cell must split its u interval, its v interval.
fn cell<S>(surface: &S, (u0, u1): (f64, f64), (v0, v1): (f64, f64), tol2: f64) -> (bool, bool)
where
    S: ParametricSurface<Point = Point3>,
{
    // truck's jitter: an interior point 0.4..0.6 across the cell, drawn
    // from the surface at the centre, so a periodic feature cannot hide
    // between probes placed always at the middle.
    let centre = surface.subs((u0 + u1) / 2.0, (v0 + v1) / 2.0);
    let t = 0.5 + (0.2 * HashGen::hash1(centre) - 0.1);
    let (u, v) = (u0 + (u1 - u0) * t, v0 + (v1 - v0) * t);
    let p = surface.subs(u, v);
    let far = |q: Point3| p.distance2(q) > tol2;
    let lerp = |a: Point3, b: Point3| Point3::from_vec(a.to_vec() * (1.0 - t) + b.to_vec() * t);
    // The iso-v curve through p against its chord: curvature across u.
    let ufar = far(lerp(surface.subs(u0, v), surface.subs(u1, v)));
    // The iso-u curve through p against its chord: curvature along v.
    let vfar = far(lerp(surface.subs(u, v0), surface.subs(u, v1)));
    if ufar || vfar {
        return (ufar, vfar);
    }
    // Both chords hold: truck's own bilinear test catches a twist.
    let (p00, p01) = (surface.subs(u0, v0), surface.subs(u0, v1));
    let (p10, p11) = (surface.subs(u1, v0), surface.subs(u1, v1));
    let bilinear = Point3::from_vec(
        p00.to_vec() * (1.0 - t) * (1.0 - t)
            + p01.to_vec() * (1.0 - t) * t
            + p10.to_vec() * t * (1.0 - t)
            + p11.to_vec() * t * t,
    );
    let twist = far(bilinear);
    (twist, twist)
}

/// `div` with every flagged interval halved.
fn halve(div: &[f64], split: &[bool]) -> Vec<f64> {
    let mut out = Vec::with_capacity(div.len() * 2);
    if let Some(first) = div.first() {
        out.push(*first);
    }
    for (w, s) in div.windows(2).zip(split) {
        if *s {
            out.push((w[0] + w[1]) / 2.0);
        }
        out.push(w[1]);
    }
    out
}

#[cfg(test)]
#[path = "step_division_tests.rs"]
mod tests;
