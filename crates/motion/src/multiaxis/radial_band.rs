//! The radial cut BAND (#2774): a rotary op's top and bottom radii are two
//! CYLINDERS about the fitted axis, and the cut lives in the annulus between
//! them. This module reads the stated band, windows a contour's rings whole
//! ([`band_select`]), and trims a parallel kind's skin-following passes at
//! the band edges ([`band_trim`]).
//!
//! Provenance: 4-axis rotary band windowing from general machining practice;
//! the convexity-and-bisection edge solve is elementary geometry. Nothing of
//! any vendor's consulted.

use crate::cl::{Move, MoveKind, Toolpath};
use crate::geometry::Point3;
use crate::machining2d::{GenError, ARC_FIT_TOL};

use super::radial::RadialHeights;

/// The radial cut band the op states, `None` when it states neither edge
/// (#2774). Each supported one-sided band is honored: a Bottom radius alone
/// runs from that floor cylinder out to the skin's end, a Top radius alone
/// from the axis out to that ceiling cylinder - a stored radius the
/// generation would ignore is never kept. Both edges stated order the band
/// at the command (`radial_gate`); the generators say it again, because a
/// path made straight through the API skips it. A stated radius that is not
/// a distance above the axis refuses by name.
pub(super) fn band_window(radial: &RadialHeights, kind: &str) -> Result<Option<(f64, f64)>, GenError> {
    if let Some(lo) = radial.bottom
        && !(lo.is_finite() && lo >= 0.0)
    {
        return Err(GenError::Invalid(format!(
            "{kind} reads its bottom radius as a distance from the bar's axis: {lo:.3} mm is not a radius above it"
        )));
    }
    if let Some(hi) = radial.top
        && !(hi.is_finite() && hi >= 0.0)
    {
        return Err(GenError::Invalid(format!(
            "{kind} reads its top radius as a distance from the bar's axis: {hi:.3} mm is not a radius above it"
        )));
    }
    match (radial.bottom, radial.top) {
        (None, None) => Ok(None),
        (Some(lo), None) => Ok(Some((lo, f64::INFINITY))),
        (None, Some(hi)) => Ok(Some((0.0, hi))),
        (Some(lo), Some(hi)) if lo < hi => Ok(Some((lo, hi))),
        (Some(lo), Some(hi)) => Err(GenError::Invalid(format!(
            "{kind} cuts between two cylinders about the bar's axis: the band {lo:.3}..{hi:.3} mm does not order above the axis"
        ))),
    }
}

/// #2774: keep the whole cut STATIONS whose path touches the radial BAND
/// `lo`..=`hi` about the axis line the frame's origin stands on, when a
/// station rides at (near-)constant radius: the contour's waterline rings.
/// `rho` reads the TOOL TIP's radius about that axis at each stored ladder
/// point, in the kind's retip convention. A ring is kept or dropped whole:
/// the band windows the ring ladder ring by ring, and the radial relink
/// that follows owns every link between rings. Plunges, rapids and the
/// contour's axial lifts are dropped; the relink rebuilds every link.
/// Refuses by name when the band leaves no cut, never a silent empty or
/// uncut path. (A pass that follows the skin across the whole radial
/// profile - Rotary Parallel - needs [`band_trim`] instead: there the band
/// can only cut the pass back at its edges.)
pub(super) fn band_select(
    base: &Toolpath,
    band: (f64, f64),
    rho: &dyn Fn(f64, f64, f64) -> f64,
    kind: &str,
) -> Result<Toolpath, GenError> {
    let (lo, hi) = band;
    let ms = &base.moves;
    let link = |i: usize| -> bool {
        let m = &ms[i];
        match m.kind {
            MoveKind::Rapid | MoveKind::Plunge => true,
            MoveKind::Feed => {
                m.arc.is_none()
                    && m.from.x == m.to.x
                    && m.from.y == m.to.y
                    && m.to.z > m.from.z
                    && ms.get(i + 1).is_some_and(|n| n.kind == MoveKind::Rapid)
            }
        }
    };
    let mut out: Vec<Move> = Vec::with_capacity(ms.len());
    let (mut all_min, mut all_max) = (f64::MAX, f64::MIN);
    let mut i = 0usize;
    while i < ms.len() {
        if link(i) {
            i += 1;
            continue;
        }
        let s = i;
        while i < ms.len() && !link(i) {
            i += 1;
        }
        let (mut run_min, mut run_max) = (f64::MAX, f64::MIN);
        for mv in &ms[s..i] {
            for (a, b) in mv.segments3(ARC_FIT_TOL) {
                for p in [a, b] {
                    let q = rho(p.x, p.y, p.z);
                    run_min = run_min.min(q);
                    run_max = run_max.max(q);
                }
            }
        }
        all_min = all_min.min(run_min);
        all_max = all_max.max(run_max);
        if run_max >= lo && run_min <= hi {
            // The kept runs need a link between them, or the relink below
            // reads them as one pass: the Rapid is the segment marker there.
            if let (Some(prev), Some(next)) = (out.last().map(|m| m.to), ms.get(s))
                && prev != next.from
            {
                out.push(Move::new(MoveKind::Rapid, prev, next.from, None));
            }
            out.extend(ms[s..i].iter().cloned());
        }
    }
    if !out.iter().any(|m| m.kind == MoveKind::Feed) {
        return Err(GenError::Empty(format!(
            "{kind} cuts between cylinders at radius {} mm about the bar's axis, but its path stands {all_min:.3}..{all_max:.3} mm out - the band crosses no part skin",
            band_text(lo, hi)
        )));
    }
    let mut path = Toolpath::new(out);
    path.notes = base.notes.clone();
    Ok(path)
}

/// The band as the refusals read it: an open-ended one-sided band names the
/// skin's end, never an `inf`.
fn band_text(lo: f64, hi: f64) -> String {
    if hi.is_finite() {
        format!("{lo:.3}..{hi:.3}")
    } else {
        format!("{lo:.3}..the skin's end")
    }
}

/// #2774: trim a planning-frame ladder to the radial BAND `lo`..=`hi` about
/// the axis line the frame's origin stands on, for a kind whose passes
/// follow the skin across the whole radial profile (Rotary Parallel): a
/// pass at one azimuth covers every radius the skin reaches, so the band
/// can only cut each pass back at its edges. `rho` reads the TOOL TIP's
/// radius about that axis at each stored ladder point, in the kind's retip
/// convention. That read is convex along a straight segment (a hypot of an
/// affine map), so each edge comes from one ternary search for the segment's
/// minimum and bisection on each side of it. Each kept stretch of a run is
/// its own run: the radial relink that follows owns every entry, and the
/// band edge is the plunge point there. Plunges, rapids and the axial lifts
/// are dropped; the relink rebuilds every link. Refuses by name when the
/// band leaves no cut, never a silent empty or uncut path.
pub(super) fn band_trim(
    base: &Toolpath,
    band: (f64, f64),
    rho: &dyn Fn(f64, f64, f64) -> f64,
    kind: &str,
) -> Result<Toolpath, GenError> {
    let (lo, hi) = band;
    let ms = &base.moves;
    let link = |i: usize| -> bool {
        let m = &ms[i];
        match m.kind {
            MoveKind::Rapid | MoveKind::Plunge => true,
            MoveKind::Feed => {
                m.arc.is_none()
                    && m.from.x == m.to.x
                    && m.from.y == m.to.y
                    && m.to.z > m.from.z
                    && ms.get(i + 1).is_some_and(|n| n.kind == MoveKind::Rapid)
            }
        }
    };
    // A band that covers the path's whole reach trims nothing: the trim
    // hands the base path on unchanged. The per-span solve below would
    // still chop runs where the retip read dips past the axis - the read
    // is a flank convention, and a radius is a distance, so a point it
    // reads negative stands at the axis - and the relink would turn each
    // chop into a retract, index and plunge (#2774). The read clamped at
    // 0 stays convex, so one ternary search reads the chord's low point.
    let radius = |p: &Point3| rho(p.x, p.y, p.z).max(0.0);
    let chord_covered = |a: &Point3, b: &Point3| -> bool {
        let r = |t: f64| {
            let p = chord(a, b, t);
            radius(&p)
        };
        let (mut u, mut v) = (0.0f64, 1.0f64);
        for _ in 0..64 {
            let (m1, m2) = (u + (v - u) / 3.0, v - (v - u) / 3.0);
            if r(m1) <= r(m2) {
                v = m2;
            } else {
                u = m1;
            }
            if v - u < 1e-12 {
                break;
            }
        }
        // A convex read tops out at the chord's ends; its low point is
        // the search's. The edges are inclusive, as in the span solve.
        r((u + v) * 0.5) >= lo && r(0.0) <= hi && r(1.0) <= hi
    };
    let mut whole = true;
    for (i, mv) in ms.iter().enumerate() {
        if link(i) {
            continue;
        }
        for (a, b) in mv.segments3(ARC_FIT_TOL) {
            if !chord_covered(&a, &b) {
                whole = false;
                break;
            }
        }
        if !whole {
            break;
        }
    }
    if whole {
        let mut path = Toolpath::new(ms.clone());
        path.notes = base.notes.clone();
        return Ok(path);
    }
    let mut out: Vec<Move> = Vec::with_capacity(ms.len());
    let (mut min_r, mut max_r) = (f64::MAX, f64::MIN);
    let mut open = false;
    for (i, mv) in ms.iter().enumerate() {
        if link(i) {
            open = false;
            continue;
        }
        for (a, b) in mv.segments3(ARC_FIT_TOL) {
            for p in [a, b] {
                let q = rho(p.x, p.y, p.z);
                min_r = min_r.min(q);
                max_r = max_r.max(q);
            }
            each_band_span(&a, &b, lo, hi, rho, |t0, t1| {
                let (s, e) = (chord(&a, &b, t0), chord(&a, &b, t1));
                if s == e {
                    return;
                }
                // Two stretches touch when one ends where the next starts:
                // they are one cut. Consecutive Feed records keep them one
                // run for the relink below - never a link between them, and
                // never one straight chord across the polyline's corner
                // (#3490: extending the previous move's end cut every wrap
                // arc's corner straight through the skin, a 1.5 mm gouge
                // the envelope law rightly refused). Only a real gap - a
                // band edge or a dropped link move - takes a Rapid that
                // the relink below rebuilds.
                let touch = open && out.last().is_some_and(|m| m.to == s);
                if !touch && let Some(prev) = out.last().map(|m| m.to) {
                    out.push(Move::new(MoveKind::Rapid, prev, s, None));
                }
                out.push(Move::new(MoveKind::Feed, s, e, ms[i].feed));
                open = true;
            });
        }
    }
    if !out.iter().any(|m| m.kind == MoveKind::Feed) {
        return Err(GenError::Empty(format!(
            "{kind} cuts between cylinders at radius {} mm about the bar's axis, but its path stands {min_r:.3}..{max_r:.3} mm out - the band crosses no part skin",
            band_text(lo, hi)
        )));
    }
    let mut path = Toolpath::new(out);
    path.notes = base.notes.clone();
    Ok(path)
}

/// Call `f` on each stretch of the chord `a`→`b` whose tip radius stands
/// in `lo`..=`hi` (inclusive at the edges). A hypot of an affine map is
/// convex along the chord, so the chord has one radial low point and at
/// most two kept stretches, one on each side of it.
///
/// ONE closed-interval rule for every span shape (constant, tangent,
/// crossing): a stretch is kept when the half's own low point stands at or
/// under `hi` and its far endpoint stands at or above `lo`, and each edge
/// is solved only where the chord strictly crosses it - a chord that
/// STARTS or ENDS on a band cylinder (a constant-radius run on the bottom
/// wall, a tangent touch at the top) keeps that whole edge, so a span
/// never loses the end that stands inside its own stated band.
fn each_band_span(
    a: &Point3,
    b: &Point3,
    lo: f64,
    hi: f64,
    rho: &dyn Fn(f64, f64, f64) -> f64,
    mut f: impl FnMut(f64, f64),
) {
    let r = |t: f64| {
        let p = chord(a, b, t);
        rho(p.x, p.y, p.z)
    };
    let (mut u, mut v) = (0.0f64, 1.0f64);
    for _ in 0..64 {
        let (m1, m2) = (u + (v - u) / 3.0, v - (v - u) / 3.0);
        if r(m1) <= r(m2) {
            v = m2;
        } else {
            u = m1;
        }
        if v - u < 1e-12 {
            break;
        }
    }
    let tm = (u + v) * 0.5;
    let (rm, ra, rb) = (r(tm), r(0.0), r(1.0));
    // The left half falls (weakly) from `ra` to the chord's low point
    // `rm`; it is kept when its low point reaches under the ceiling and
    // its start stands at or above the floor. Both edges are inclusive.
    if rm <= hi && ra >= lo {
        let l0 = if ra > hi { bisect_edge(a, b, 0.0, tm, hi, false, rho) } else { 0.0 };
        let l1 = if rm < lo { bisect_edge(a, b, 0.0, tm, lo, false, rho) } else { tm };
        if l0 < l1 {
            f(l0, l1);
        }
    }
    // The right half rises (weakly) from `rm` to `rb`, kept by the same
    // closed rule read the other way.
    if rm <= hi && rb >= lo {
        let r0 = if rm < lo { bisect_edge(a, b, tm, 1.0, lo, true, rho) } else { tm };
        let r1 = if rb > hi { bisect_edge(a, b, tm, 1.0, hi, true, rho) } else { 1.0 };
        if r0 < r1 {
            f(r0, r1);
        }
    }
}

/// The parameter where the chord's tip radius crosses `edge` inside
/// `x0`..`x1`, on the half the radius moves monotonically toward. The
/// caller has proved the endpoints straddle the edge.
fn bisect_edge(a: &Point3, b: &Point3, x0: f64, x1: f64, edge: f64, rising: bool, rho: &dyn Fn(f64, f64, f64) -> f64) -> f64 {
    let (mut p, mut q) = (x0, x1);
    for _ in 0..60 {
        let t = (p + q) * 0.5;
        let c = chord(a, b, t);
        let rv = rho(c.x, c.y, c.z);
        let hold = if rising { rv <= edge } else { rv >= edge };
        if hold {
            p = t;
        } else {
            q = t;
        }
    }
    (p + q) * 0.5
}

fn chord(a: &Point3, b: &Point3, t: f64) -> Point3 {
    if t <= 0.0 {
        *a
    } else if t >= 1.0 {
        *b
    } else {
        Point3::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t, a.z + (b.z - a.z) * t)
    }
}

#[cfg(test)]
#[path = "radial_band_tests.rs"]
mod tests;
