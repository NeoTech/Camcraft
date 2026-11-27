//! The start side in the order stage (#4177): Auto resolved in one place,
//! the sense in which the ring index grows (into the area or out to the
//! stock edge), and the side of each curve where the deeper curves lie, so
//! that the direction choice can put the uncut material on the right
//! (climb) or on the left (conventional) of the travel.
//! Provenance: our own design record `docs/decisions/4024-path-kernel-design.md`
//! "Stage 4: Order" and "#3934" (Outside walks from the stock edge, Inside
//! from the part wall); climb milling with the material on the right of a
//! clockwise spindle, as in any machining handbook.

use super::StartSide;
use crate::guide::{Curve, Family};
use crate::region::PlanarRegion;
use crate::{Loop, Point2, to_loops};

/// How far (mm) the side test steps off a curve.
const SIDE_STEP: f64 = 0.01;

/// Auto resolves to Outside when the region reaches a stock edge with air
/// beyond it (the tool can start outside the stock), else to Inside. The
/// stage never selects another guide.
pub(super) fn resolve(p: &PlanarRegion, side: StartSide) -> StartSide {
    match side {
        StartSide::Auto if !p.stock_edge().is_empty() && !p.air().is_empty() => StartSide::Outside,
        StartSide::Auto => StartSide::Inside,
        s => s,
    }
}

/// The mean distance of the points of `curves` to `loops`.
fn mean_to(curves: &[&Curve], loops: &[Loop]) -> f64 {
    let (mut sum, mut n) = (0.0, 0.0);
    for c in curves {
        for q in c.pts() {
            sum += to_loops(q.at(), loops);
            n += 1.0;
        }
    }
    if n > 0.0 { sum / n } else { f64::INFINITY }
}

/// True when the order cuts the deeper curves (higher index) first. A
/// single index has no deeper curve: its material is on the wall side, as
/// for a deeper-first walk. Else the index grows outward when the region
/// has a stock edge and the deepest curves lie nearer to it than the first
/// ones (rings round a part); it grows inward otherwise (a pocket). Inside
/// starts at the part wall or in the middle of a pocket; Outside starts at
/// the stock edge or at the pocket wall.
pub(super) fn deeper_first(p: &PlanarRegion, curves: &[Curve], side: StartSide) -> bool {
    let (Some(lo), Some(hi)) = (curves.iter().map(Curve::k).min(), curves.iter().map(Curve::k).max()) else {
        return true;
    };
    if lo == hi {
        return true;
    }
    let at = |k: u32| curves.iter().filter(|c| c.k() == k).collect::<Vec<_>>();
    let outward = !p.stock_edge().is_empty() && mean_to(&at(hi), p.stock_edge()) < mean_to(&at(lo), p.stock_edge());
    (side == StartSide::Outside) == outward
}

/// True when the deeper curves lie on the left of `c` as it runs, `None`
/// when the curve has no such side (a row: its direction is free). A
/// closed ring is a loop of a level area whose inside is deeper: the area
/// is on the left of its loops. An open piece of a ring round a part has
/// the deeper side away from the part walls.
pub(super) fn deeper_left(p: &PlanarRegion, c: &Curve) -> Option<bool> {
    if c.family() == Family::Row {
        return None;
    }
    if c.closed() {
        return Some(true);
    }
    if p.part_walls().is_empty() {
        return None;
    }
    let pts = c.pts();
    let (mut best, mut at) = (0.0, None);
    for w in pts.windows(2) {
        let (a, b) = (w[0].at(), w[1].at());
        let len = (b.x - a.x).hypot(b.y - a.y);
        if len > best {
            best = len;
            at = Some((a, b, len));
        }
    }
    let (a, b, len) = at?;
    let m = Point2::new(0.5 * (a.x + b.x), 0.5 * (a.y + b.y));
    let (nx, ny) = (-(b.y - a.y) / len * SIDE_STEP, (b.x - a.x) / len * SIDE_STEP);
    let left = to_loops(Point2::new(m.x + nx, m.y + ny), p.part_walls());
    let right = to_loops(Point2::new(m.x - nx, m.y - ny), p.part_walls());
    (left.is_finite() && right.is_finite() && left != right).then_some(left > right)
}
