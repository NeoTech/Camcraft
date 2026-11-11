//! The plasma cut's certification laws (#670, #672), run over the moves
//! just laid down, before the path leaves the generator:
//!
//! - **Kerf law.** Every lit move of a closed contour keeps its kerf off
//!   the part: no chord point lies inside the part grown by kerf / 2
//!   (less [`KERF_TOL`]). A cut that would reach the part is refused by
//!   name, never trimmed.
//! - **Cover law** (#676 decision 12, #672). Every edge of a closed
//!   contour is reached by its own cut's kerf: no point of the nominal
//!   contour stands farther than one kerf width from the lit centre line.
//!   An outward offset bridges a slot or notch narrower than the kerf
//!   (and an inward offset drops a keyway narrower than the kerf from a
//!   hole) without splitting or closing a loop, so the ring law in
//!   [`super::ring::kerf_ring`] accepts it and the feature is silently
//!   left in the material. Here it is refused by name. The allowance is
//!   one kerf width (2 r) plus the tolerance: an inside corner the torch
//!   rounds off leaves r (1 / sin(φ/2) - 1) at its vertex, under r for
//!   any opening φ of 60° or more - the fillet every round torch leaves -
//!   while a bridged slot's floor stands its whole depth away.
//!
//! Provenance: our own geometry over Tormach's public 1300PL manual
//! (UM10720: kerf ≈ 1.5x the orifice; a feature under the kerf cannot be
//! cut) and the LinuxCNC Plasma Primer; no vendor implementation read.

use crate::cl::{ContourRole, Toolpath, TorchSpan};
use crate::geo2d::{Point2, Region};
use crate::machining2d::GenError;

use super::emit::FIT_TOL;
use super::nest::Contour;
use super::ring::dist;
use super::KERF_TOL;

/// How far apart (mm) the contour and the chords are sampled.
const STEP: f64 = 0.25;

fn invalid(why: String) -> GenError {
    GenError::Invalid(format!("plasma profile: {why}"))
}

/// Every lit move of every closed contour keeps its kerf off the part.
pub(super) fn kerf_law(path: &Toolpath, spans: &[TorchSpan], forbidden: &Region) -> Result<(), GenError> {
    for sp in spans.iter().filter(|s| s.role != ContourRole::Open) {
        for i in sp.on..=sp.off {
            let Some(m) = path.moves.get(i) else { continue };
            for (a, b) in m.chords(FIT_TOL) {
                let k = (dist(a, b) / STEP).ceil().clamp(1.0, 4096.0) as u32;
                for j in 0..=k {
                    let f = f64::from(j) / f64::from(k);
                    let q = Point2::new(a.x + (b.x - a.x) * f, a.y + (b.y - a.y) * f);
                    if forbidden.contains(q) {
                        return Err(invalid(format!(
                            "contour C{}: move {i} would carry the kerf into the part near X{:.3} Y{:.3} - refused",
                            sp.contour, q.x, q.y
                        )));
                    }
                }
            }
        }
    }
    Ok(())
}

/// Every edge of every closed contour is reached by its own cut's kerf:
/// `spans[pos]` is the cut of `cs[order[pos]]`.
pub(super) fn cover_law(
    path: &Toolpath,
    spans: &[TorchSpan],
    cs: &[Contour],
    order: &[usize],
    kerf: f64,
) -> Result<(), GenError> {
    let reach = kerf + KERF_TOL;
    for (sp, &k) in spans.iter().zip(order) {
        let Some(c) = cs.get(k) else { continue };
        if c.role == ContourRole::Open || sp.role == ContourRole::Open {
            continue;
        }
        let chords: Vec<(Point2, Point2)> =
            path.moves.get(sp.on..=sp.off).into_iter().flatten().flat_map(|m| m.chords(FIT_TOL)).collect();
        let n = c.pts.len();
        for i in 0..n {
            let (a, b) = (c.pts[i], c.pts[(i + 1) % n]);
            let steps = (dist(a, b) / STEP).ceil().clamp(1.0, 65536.0) as u32;
            for j in 0..steps {
                let f = f64::from(j) / f64::from(steps);
                let q = Point2::new(a.x + (b.x - a.x) * f, a.y + (b.y - a.y) * f);
                let nearest = chords.iter().map(|(s, t)| pt_seg(q, *s, *t)).fold(f64::INFINITY, f64::min);
                if nearest.is_nan() || nearest > reach {
                    return Err(invalid(format!(
                        "contour C{}: the torch never comes within a kerf of the part edge near X{:.3} Y{:.3} - a slot, notch or inside corner narrower than the kerf ({kerf:.3} mm) is bridged by the offset, not cut; widen it or use a narrower kerf",
                        sp.contour, q.x, q.y
                    )));
                }
            }
        }
    }
    Ok(())
}

/// Distance from `p` to segment a-b.
fn pt_seg(p: Point2, a: Point2, b: Point2) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 0.0 { (((p.x - a.x) * dx + (p.y - a.y) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
    (p.x - (a.x + t * dx)).hypot(p.y - (a.y + t * dy))
}
