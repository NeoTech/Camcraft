//! The checks between the parts of a planar region (K1, #4025): the loops
//! of the cutter-centre area are oriented by their nesting, and the part
//! walls and the stock edges partition its boundary.
//! Provenance: our own design record (`RECOMMENDED.md`, `## #4024`)
//! "Stage 1: Region" (the invariants); the crossing-parity point-in-polygon
//! test of Shimrat, "Algorithm 112: Position of point relative to polygon"
//! (CACM 5(8), 1962).

use crate::{Loop, Point2, Region, near};

/// Refusal reason: a loop of the cutter-centre area whose direction does
/// not agree with its nesting (an outer loop counter-clockwise, a hole
/// clockwise, an island in a hole counter-clockwise).
pub const NOT_ORIENTED: &str = "a loop of the cutter-centre area is not oriented by its nesting";
/// Refusal reason: part walls and stock edges that are not the boundary
/// loops of the cutter-centre area, each one exactly one time.
pub const NOT_A_PARTITION: &str =
    "the part walls and the stock edges must be the boundary loops of the cutter-centre area, each one time";

/// True when `p` is inside the polygon `pts` (crossing parity; a point on
/// the boundary can read either way, so callers count many vertices).
pub(crate) fn inside_loop(pts: &[Point2], p: Point2) -> bool {
    let n = pts.len();
    let mut inside = false;
    for i in 0..n {
        let (Some(a), Some(b)) = (pts.get(i), pts.get((i + 1) % n)) else { continue };
        if (a.y > p.y) != (b.y > p.y) {
            let t = (p.y - a.y) / (b.y - a.y);
            if p.x < a.x + t * (b.x - a.x) {
                inside = !inside;
            }
        }
    }
    inside
}

/// True when more than half of the vertices of `inner` are inside `outer`.
/// Loops of a normalized region do not cross, so this is their nesting;
/// the majority keeps a vertex that touches `outer` from deciding it.
fn nested_in(inner: &Loop, outer: &Loop) -> bool {
    let pts = inner.points();
    let count = pts.iter().filter(|p| inside_loop(outer.points(), **p)).count();
    count.saturating_mul(2) > pts.len()
}

/// The first vertex of the first loop of `r` whose direction disagrees with
/// its nesting depth, or `None` when every loop is oriented.
pub(crate) fn oriented(r: &Region) -> Option<Point2> {
    oriented_loops(r.loops())
}

/// [`oriented`] on a list of loops: the first vertex of the first loop
/// whose direction disagrees with its nesting depth.
fn oriented_loops(loops: &[Loop]) -> Option<Point2> {
    loops.iter().enumerate().find_map(|(i, l)| {
        let depth = loops.iter().enumerate().filter(|(j, o)| *j != i && nested_in(l, o)).count();
        let want_ccw = depth % 2 == 0;
        (l.is_ccw() != want_ccw).then(|| l.points().first().copied()).flatten()
    })
}

/// True when `a` and `b` are the same cyclic sequence of points (the same
/// direction, any start) within the kernel tolerance.
pub(crate) fn same_cycle(a: &Loop, b: &Loop) -> bool {
    let (pa, pb) = (a.points(), b.points());
    let n = pa.len();
    if n != pb.len() || n == 0 {
        return false;
    }
    let Some(first) = pa.first() else { return false };
    pb.iter().enumerate().filter(|(_, q)| near(*first, **q)).any(|(shift, _)| {
        (0..n).all(|i| match (pa.get(i), pb.get((i + shift) % n)) {
            (Some(p), Some(q)) => near(*p, *q),
            _ => false,
        })
    })
}

/// The first point of the first edge loop that breaks the partition, or of
/// the first boundary loop that no edge names; `None` when each loop of
/// `center` is named by exactly one part wall or stock edge.
pub(crate) fn partition(center: &Region, walls: &[Loop], edge: &[Loop]) -> Option<Point2> {
    let loops = center.loops();
    let mut used = vec![false; loops.len()];
    for e in walls.iter().chain(edge) {
        let hit = loops.iter().enumerate().find(|(i, l)| !used.get(*i).copied().unwrap_or(true) && same_cycle(e, l));
        match hit.and_then(|(i, _)| used.get_mut(i)) {
            Some(slot) => *slot = true,
            None => return e.points().first().copied().or(Some(Point2::new(0.0, 0.0))),
        }
    }
    used.iter().position(|u| !u).and_then(|i| loops.get(i)).and_then(|l| l.points().first().copied())
}

#[cfg(test)]
mod tests {
    //! #4407: the orientation guard of `PlanarRegion::new` on loops that
    //! `camcraft-geometry` cannot give (it gives only normal regions), so
    //! the guard is pinned here on the loops themselves.
    use super::*;

    fn sq(x0: f64, s: f64, ccw: bool) -> Result<Loop, camcraft_geometry::geo2d::GeoError> {
        let mut pts = vec![Point2::new(x0, x0), Point2::new(x0 + s, x0), Point2::new(x0 + s, x0 + s), Point2::new(x0, x0 + s)];
        if !ccw {
            pts.reverse();
        }
        Loop::new(pts)
    }

    #[test]
    fn a_loop_against_its_nesting_is_found() -> Result<(), camcraft_geometry::geo2d::GeoError> {
        assert_eq!(oriented_loops(&[sq(0.0, 10.0, true)?, sq(2.0, 4.0, false)?]), None, "outer ccw, hole cw");
        assert_eq!(oriented_loops(&[sq(0.0, 10.0, false)?]), Some(Point2::new(0.0, 10.0)), "an outer loop clockwise");
        assert_eq!(oriented_loops(&[sq(0.0, 10.0, true)?, sq(2.0, 4.0, true)?]), Some(Point2::new(2.0, 2.0)), "a hole counter-clockwise");
        Ok(())
    }
}
