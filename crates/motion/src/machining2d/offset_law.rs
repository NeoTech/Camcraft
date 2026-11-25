//! The ring depths of the Offset rings spiral (#3933): the step from one
//! ring to the next one out is the one that holds the optimal load's
//! contact angle on the ring's tightest round stretch, so the rings
//! crowd near a small core and open to the load's width on straight
//! walls. Provenance: the constant-engagement law for a circular tool
//! path inside a circular cleared boundary (Stori & Wright 2000,
//! "Constant engagement tool path generation for convex geometries",
//! J. Manuf. Systems 19(3)): a circle of radius `m` about a path point
//! at radius `q` meets a wall of radius `R` at `cos t = (R^2 - q^2 -
//! m^2) / (2 q m)`; held at the angle a straight wall `ae` deep reads on
//! the same circle, `cos t0 = (r - ae) / m`. The circle is the read's
//! rim (`m` a cell and a half outside the tool, `quality::rim_for`), so
//! the law and the plan's load read agree.
//! The curvature window, the corner rule and the floor are ours. No
//! vendor material consulted.

use crate::geo2d::{Point2, Region};

use super::face::GenError;
use super::ring::dist;

/// A ring vertex that turns more than this (radians, about 15 degrees)
/// is a corner, not a round stretch: the law does not read it (a sharp
/// corner reads as no radius at all; the plan's load read names it).
const CORNER: f64 = 0.26;

/// The curvature of a ring is read over windows of this share of the
/// tool radius, so a micron vertex step does not read as a tight arc.
const WINDOW_PER_R: f64 = 0.5;

/// No step is smaller than this share of the load's width: out of a
/// point-like core the law asks for steps near zero; the read names the
/// load the floor leaves.
const FLOOR_SHARE: f64 = 0.15;

/// A step that would leave the last ring closer to the wall than this
/// share of the step splits the rest in two instead.
const SLIVER_SHARE: f64 = 0.2;

/// On a round stretch the law aims at this share of the load: the read
/// on the planner's grid sees a curved wall a few percent heavier than
/// the law's circle (grid phase), and the plan must read within its
/// margin (`PLAN_TOL`), as the HSM plan's cuts do.
const CURVED_AIM: f64 = 0.96;

/// The core ring stands this far in from the area's deepest point: as
/// far as the law's step out of it is this share of the load width, so
/// the first revolutions do not crawl out of a point (at most
/// [`CORE_MAX_PER_R`] of the tool radius, so the core lap still reaches
/// over everything inside it).
const CORE_STEP_SHARE: f64 = 0.5;
const CORE_MAX_PER_R: f64 = 0.9;

/// Bisection rounds for the core radius, and for the deepest point (each
/// an offset of the area: 24 rounds put it within ae / 2^24).
const BISECT: usize = 40;
const DEPTH_BISECT: usize = 24;

/// Rings are capped (the Prime Directive's "never hang").
const MAX_RINGS: usize = 4000;

/// The depths (mm in from `center`'s edge, ascending from 0) of the
/// spiral's rings for the load width `ae` and tool radius `r`: the core
/// stands [`core_radius`] in from the area's deepest point, every ring
/// out from it stepped by [`law_step`] on its tightest round stretch.
pub(super) fn law_depths(center: &Region, ae: f64, (r, rim): (f64, f64)) -> Result<Vec<f64>, GenError> {
    if !(ae.is_finite() && ae > 0.0 && r.is_finite() && r > 0.0 && ae <= r && rim.is_finite() && rim >= r) {
        return Err(GenError::Invalid("the ring step must be positive and at most the tool radius".into()));
    }
    let Some((lo, hi)) = center.bounds() else {
        return Err(GenError::Empty("empty pocket region".into()));
    };
    // The deepest point: the last depth of the `ae` tiling that stands,
    // then bisected to the depth where the area vanishes.
    let cap = (dist(lo, hi) / ae + 10.0).max(16.0) as u32;
    let mut deep: Option<f64> = None;
    for k in 0..=cap {
        let d = f64::from(k) * ae;
        let ring = if k == 0 { center.clone() } else { center.offset(-d)? };
        if ring.is_empty() {
            break;
        }
        deep = Some(d);
    }
    let Some(mut stands) = deep.filter(|d| center.offset(-(d + ae)).is_ok_and(|g| g.is_empty())) else {
        return Err(GenError::Invalid("offset rings did not converge; check the optimal load".into()));
    };
    let mut gone = stands + ae;
    for _ in 0..DEPTH_BISECT {
        let mid = 0.5 * (stands + gone);
        if center.offset(-mid)?.is_empty() {
            gone = mid;
        } else {
            stands = mid;
        }
    }
    let mut d = (stands - core_radius((r, rim), ae)).max(0.0);
    let mut ring = if d <= 0.0 { center.clone() } else { center.offset(-d)? };
    if ring.is_empty() {
        (d, ring) = (0.0, center.clone());
    }
    let mut out = vec![d];
    for _ in 0..MAX_RINGS {
        if d <= 0.0 {
            out.reverse();
            return Ok(out);
        }
        let s = law_step(tightest(&ring, WINDOW_PER_R * r), (r, rim), ae).max(FLOOR_SHARE * ae);
        d = if d - s <= 0.0 {
            0.0
        } else if d - s < SLIVER_SHARE * s {
            0.5 * d
        } else {
            d - s
        };
        ring = if d <= 0.0 { center.clone() } else { center.offset(-d)? };
        out.push(d);
    }
    Err(GenError::Invalid("offset rings did not converge; check the optimal load".into()))
}

/// How far in from the deepest point the core ring stands: the least
/// path radius whose law step is [`CORE_STEP_SHARE`] of the width, at
/// most [`CORE_MAX_PER_R`] of the tool radius.
pub(super) fn core_radius((r, rim): (f64, f64), ae: f64) -> f64 {
    let (mut lo, mut hi) = (0.0f64, CORE_MAX_PER_R * r);
    if law_step(hi, (r, rim), ae) < CORE_STEP_SHARE * ae {
        return hi;
    }
    for _ in 0..BISECT {
        let mid = 0.5 * (lo + hi);
        if law_step(mid, (r, rim), ae) >= CORE_STEP_SHARE * ae {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    hi
}

/// The step out from a path of radius `p` (mm; infinite for a straight
/// one) that holds the contact angle of the load width `ae` for a tool
/// of radius `r`, read on the rim of radius `rim`: the next path's
/// radius `q` sees the wall the path left, `R = p + r`, at the straight
/// wall's angle `t0`, so `q = -m cos t0 + sqrt(R^2 - m^2 sin^2 t0)`
/// (`m` the rim). The spiral does not walk the circle `q`: it climbs one
/// step a turn, so its heading leans into the wall by `atan(s / (2 pi
/// q))`; the angle held is `t0` less that lean (two rounds of the fixed
/// point). Never more than `ae`.
pub(super) fn law_step(p: f64, (r, rim): (f64, f64), ae: f64) -> f64 {
    if !p.is_finite() || p > 1e6 {
        return ae;
    }
    let t0 = ((r - CURVED_AIM * ae) / rim).clamp(-1.0, 1.0).acos();
    let p = p.max(0.0);
    let wall = p + r;
    let mut s = ae;
    let mut lean = 0.0f64;
    for _ in 0..3 {
        let c = (t0 - lean).max(0.0).cos();
        let q = -rim * c + (wall * wall - rim * rim * (1.0 - c * c)).max(0.0).sqrt();
        s = (q - p).clamp(0.0, ae);
        lean = (s / (std::f64::consts::TAU * q.max(1e-9))).atan();
    }
    s
}

/// The tightest radius (mm) of the round stretches of `ring` that turn
/// into the area (convex: the material the next ring cuts lies outside
/// the turn), read over windows of arc length `window`; a window that
/// holds a corner is not read. Infinite when no stretch turns.
pub(super) fn tightest(ring: &Region, window: f64) -> f64 {
    let mut best = f64::INFINITY;
    for l in ring.loops() {
        let pts = l.points();
        let n = pts.len();
        if n < 3 {
            continue;
        }
        // Per vertex: the turn (left positive: into the area, whose loops
        // run with it on their left) and the length of the edge after it.
        let turn: Vec<f64> = (0..n)
            .map(|i| {
                let (a, b, c) = (pts[(i + n - 1) % n], pts[i], pts[(i + 1) % n]);
                let (u, v) = (Point2::new(b.x - a.x, b.y - a.y), Point2::new(c.x - b.x, c.y - b.y));
                (u.x * v.y - u.y * v.x).atan2(u.x * v.x + u.y * v.y)
            })
            .collect();
        let len: Vec<f64> = (0..n).map(|i| dist(pts[i], pts[(i + 1) % n])).collect();
        let total: f64 = len.iter().sum();
        if !(total > 0.0) {
            continue;
        }
        let span = window.min(total);
        for i in 0..n {
            let (mut run, mut bend, mut j) = (0.0f64, 0.0f64, i);
            let mut corner = false;
            for _ in 0..n {
                run += len[j];
                j = (j + 1) % n;
                if run >= span {
                    break;
                }
                corner |= turn[j].abs() > CORNER;
                bend += turn[j];
            }
            if !corner && bend > 1e-9 {
                best = best.min(run / bend);
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::law_step;

    /// The law's two ends: a straight path steps the full load width; a
    /// path round a small core steps less, and the step grows with the
    /// radius toward the width.
    #[test]
    fn the_step_is_the_load_width_on_a_straight_wall_and_less_round_a_core() {
        let (r, ae) = ((2.0, 2.0), 1.0);
        assert!((law_step(f64::INFINITY, r, ae) - ae).abs() < 1e-12);
        let (s1, s5, s50) = (law_step(1.0, r, ae), law_step(5.0, r, ae), law_step(50.0, r, ae));
        assert!(s1 < s5 && s5 < s50 && s50 < ae, "{s1} {s5} {s50}");
        // A path of radius 1 against the wall of radius 3 it left, held
        // at the 60-degree contact of a 25% load: q = -1 + sqrt(9 - 3) on
        // a circle, a little less on the spiral's climbing heading.
        let circle = 6.0f64.sqrt() - 2.0;
        assert!(s1 < circle && s1 > 0.8 * circle, "{s1} vs {circle}");
        // Read on a rim outside the tool, the step is smaller still.
        assert!(law_step(1.0, (2.0, 2.05), ae) < s1);
    }
}
