//! Pins of the one ring planner (#3750): the completeness guard, the
//! circle rings of a disc, and the Auto rings of a pocket with an island
//! against `waterline_passes`.

use super::{LEFT_TOL, RingRef, RingSet, unswept};
use crate::geo2d::{Loop, Point2, Region};
use crate::machining2d::pass_start::{PassStart, StartSide};
use crate::machining2d::pocket_strategy::PocketStrategy;
use crate::machining2d::waterscan::waterline_passes;

fn circle(r: f64) -> Vec<Point2> {
    (0..128)
        .map(|k| {
            let a = f64::from(k) * std::f64::consts::TAU / 128.0;
            Point2::new(r * a.cos(), r * a.sin())
        })
        .collect()
}

fn region(loops: Vec<Vec<Point2>>) -> Region {
    let loops = loops.into_iter().map(|l| Loop::new(l).unwrap_or_else(|e| panic!("loop: {e:?}"))).collect();
    Region::new(loops).unwrap_or_else(|e| panic!("region: {e:?}"))
}

fn side(start: PassStart) -> StartSide {
    StartSide { start, stock: None, around_part: false }
}

/// The completeness guard reads a standing column, never a sliver: a
/// lone ring on a disc of radius 10 leaves the core (radius 6) unswept;
/// rings 4 mm apart down to the middle leave nothing.
#[test]
fn the_guard_reads_a_standing_core_and_passes_a_whole_plan() {
    let Ok(l) = Loop::new(circle(10.0)) else { panic!("loop") };
    let Ok(disc) = Region::new(vec![l]) else { panic!("disc") };
    let ring = |r: f64| {
        let mut c = circle(r);
        c.push(c[0]);
        c
    };
    let Ok(lone) = unswept(&disc, &[ring(10.0)], 4.0) else { panic!("unswept") };
    assert!(lone > 100.0 && lone < 114.0, "the core of radius 6 stands: {lone}");
    let Ok(whole) = unswept(&disc, &[ring(10.0), ring(6.0), ring(2.0)], 4.0) else { panic!("unswept") };
    assert!(whole <= LEFT_TOL, "a whole plan: {whole}");
    assert!(unswept(&disc, &[vec![Point2::new(f64::NAN, 0.0)]], 4.0).is_err(), "NaN is refused");
}

/// A disc gives circle rings: every pass point of every start side stands
/// on a circle of radius `rc - k * step` (to the 10 um of the polygon and
/// Clipper's arc flats), and the rings run from `rc` down to the core.
#[test]
fn a_disc_gives_circle_rings_one_step_apart() {
    let (rc, step, tool_r) = (20.0, 3.0, 4.0);
    let center = region(vec![circle(rc)]);
    for start in [PassStart::Auto, PassStart::Inside, PassStart::Outside] {
        let set = RingSet { region: &center, center: &center, reference: RingRef::Boundary, first: 0.0, tool_r, step, climb: true };
        let (passes, notes) = set.plan(PocketStrategy::OffsetRings, &side(start)).unwrap_or_else(|e| panic!("{start:?}: {e:?}"));
        assert!(notes.is_empty(), "{start:?}: {notes:?}");
        let mut ks = std::collections::BTreeSet::new();
        for q in passes.iter().flatten() {
            let r = q.x.hypot(q.y);
            let k = ((rc - r) / step).round();
            assert!((r - (rc - k * step)).abs() < 0.01, "{start:?}: a point at radius {r:.4} is on no ring");
            ks.insert(k as i64);
        }
        let want: Vec<i64> = (0..=((rc / step).floor() as i64 - 1)).collect();
        assert_eq!(ks.into_iter().collect::<Vec<_>>(), want, "{start:?}: the rings from rc to the core");
    }
}

/// A square with an island: the Auto rings of [`RingRef::Boundary`] are the
/// rings of `waterline_passes` (all walls of the area) within 1 um.
#[test]
fn a_square_with_an_island_gives_the_waterline_rings() {
    let square = vec![Point2::new(-30.0, -30.0), Point2::new(30.0, -30.0), Point2::new(30.0, 30.0), Point2::new(-30.0, 30.0)];
    let island: Vec<Point2> = circle(8.0).into_iter().rev().collect();
    let center = region(vec![square, island]);
    let set = RingSet { region: &center, center: &center, reference: RingRef::Boundary, first: 0.0, tool_r: 4.0, step: 3.0, climb: true };
    let (passes, _) = set.plan(PocketStrategy::Waterline, &side(PassStart::Auto)).unwrap_or_else(|e| panic!("{e:?}"));
    let want = waterline_passes(&center, 3.0, true).unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(passes.len(), want.len(), "the same pass count");
    for (a, b) in passes.iter().zip(&want) {
        assert_eq!(a.len(), b.len(), "the same point count");
        for (p, q) in a.iter().zip(b) {
            assert!((p.x - q.x).hypot(p.y - q.y) < 1e-3, "{p:?} vs {q:?}");
        }
    }
}

/// The reference of a start side: an area round a part is measured from
/// the part, a pocket from its own walls.
#[test]
fn the_reference_follows_the_area() {
    assert_eq!(RingRef::of(&StartSide { start: PassStart::Inside, stock: None, around_part: true }), RingRef::Part);
    assert_eq!(RingRef::of(&side(PassStart::Outside)), RingRef::Boundary);
}
