//! Unit pins of `level_seam` (#3329): the exact column law, which the
//! ladder's islands law already keeps for every ring point (so no
//! ladder fixture reaches it), and the seat of one level's passes.

use super::super::level_columns::column_hit;
use super::{LevelSeam, columns, seat_level};
use crate::geo2d::Point2;
use crate::geometry::{Point3, Triangle};

fn p(x: f64, y: f64) -> Point2 {
    Point2::new(x, y)
}

/// A horizontal square plate at height `z`, `lo..hi` in X and Y.
fn plate(lo: f64, hi: f64, z: f64) -> Vec<Triangle> {
    let v = |x, y| Point3::new(x, y, z);
    vec![Triangle::new(v(lo, lo), v(hi, lo), v(hi, hi)), Triangle::new(v(lo, lo), v(hi, hi), v(lo, hi))]
}

/// A vertical wall in the plane x = `x`, y -20..20, z `z0..z1`.
fn wall(x: f64, z0: f64, z1: f64) -> Vec<Triangle> {
    let v = |y, z| Point3::new(x, y, z);
    vec![Triangle::new(v(-20.0, z0), v(20.0, z0), v(20.0, z1)), Triangle::new(v(-20.0, z0), v(20.0, z1), v(-20.0, z1))]
}

/// The column at the origin from Z 0 up to Z 30 with a 3 mm tool: the
/// floor it stands on and a wall a full radius away are clear; a wall
/// inside the radius and an overhang above are hits, at their heights.
/// Mutation M1 (no column law) and M7 (only the column's two ends
/// tested, no segment distance) turn this red.
#[test]
fn the_column_law_is_exact_above_the_foot() {
    let floor = plate(-20.0, 20.0, 0.0);
    assert_eq!(column_hit(p(0.0, 0.0), 0.0, 3.0, 30.0, &floor), None, "the floor under the tool is not a hit");
    let tangent = wall(3.0, -5.0, 25.0);
    assert_eq!(column_hit(p(0.0, 0.0), 0.0, 3.0, 30.0, &tangent), None, "a wall a full radius away is clear");
    let close = wall(2.5, -5.0, 25.0);
    let hit = column_hit(p(0.0, 0.0), 0.0, 3.0, 30.0, &close);
    assert!(hit.is_some_and(|z| z.abs() < 1e-5), "a wall inside the radius is a hit from the foot: {hit:?}");
    // An overhang at Z 12, between the column's ends: no end of the
    // column is near it, only the segment between them.
    let roof = plate(-1.0, 1.0, 12.0);
    assert_eq!(column_hit(p(0.0, 0.0), 0.0, 3.0, 30.0, &roof), Some(12.0), "an overhang above the foot is a hit");
    let e = columns(&[p(0.0, 0.0)], |_| 0.0, 3.0, 30.0, &roof).err().map(|e| e.to_string());
    assert!(e.as_deref().is_some_and(|m| m.contains("column at (0.000, 0.000)") && m.contains("at Z 12.000")), "{e:?}");
    assert!(columns(&[p(0.0, 0.0)], |_| 0.0, 3.0, 30.0, &tangent).is_ok());
}

fn square_ring(c: f64, h: f64) -> Vec<Point2> {
    vec![p(c - h, c - h), p(c + h, c - h), p(c + h, c + h), p(c - h, c + h), p(c - h, c - h)]
}

/// One level of one closed ring: the start re-seats it at its point
/// nearest the start pick, the exit carries it on to the exit point;
/// both moved points are named. Mutation M6 (no run-on) turns red.
#[test]
fn a_level_of_one_ring_laps_from_the_start_and_runs_on_to_the_exit() {
    let sm = LevelSeam { start: Some(p(10.0, -1.0)), exit: Some(p(21.0, 10.0)), z_unread: false };
    let mut passes = vec![square_ring(10.0, 5.0)];
    let moved = seat_level(&sm, &mut passes, false, 0.0, 6.0).expect("seated");
    let ring = &passes[0];
    assert_eq!(ring.first().copied(), Some(p(10.0, 5.0)), "starts at the point nearest the start");
    assert_eq!(ring.last().copied(), Some(p(15.0, 10.0)), "ends at the point nearest the exit");
    let want = vec![p(10.0, 5.0), p(15.0, 5.0), p(15.0, 15.0), p(5.0, 15.0), p(5.0, 5.0), p(10.0, 5.0), p(15.0, 5.0), p(15.0, 10.0)];
    assert_eq!(ring, &want, "the whole lap from the start, then the run-on to the exit");
    assert_eq!(moved, vec![p(10.0, 5.0), p(15.0, 10.0)], "both moved points are named");
}

/// A start pick beyond the reach of the first pass is refused by name
/// with the level's Z. Mutation M3 (no reach law) turns red.
#[test]
fn a_start_beyond_the_first_pass_is_refused_with_its_level() {
    let sm = LevelSeam { start: Some(p(40.0, 40.0)), exit: None, z_unread: false };
    let mut passes = vec![square_ring(10.0, 5.0), square_ring(10.0, 8.0)];
    let e = seat_level(&sm, &mut passes, false, 4.5, 6.0).err().map(|e| e.to_string());
    assert!(e.as_deref().is_some_and(|m| m.contains("start point (40.000, 40.000)") && m.contains("Z 4.500") && m.contains("farther than the 9.000 mm")), "{e:?}");
}

/// A descent that the entry rule puts far from the start point is
/// refused by name; one beside it, or on the pass start, rides.
/// Mutation M5 (no descent law) turns red.
#[test]
fn a_descent_far_from_the_start_point_is_refused() {
    let sm = LevelSeam { start: Some(p(0.0, 0.0)), exit: None, z_unread: false };
    assert!(sm.descent(None, 2.0, 6.0).is_ok());
    assert!(sm.descent(Some(p(1.0, 1.0)), 2.0, 6.0).is_ok());
    let e = sm.descent(Some(p(20.0, 0.0)), 2.0, 6.0).err().map(|e| e.to_string());
    assert!(e.as_deref().is_some_and(|m| m.contains("start point (0.000, 0.000)") && m.contains("descends at (20.000, 0.000)")), "{e:?}");
}

/// A chain of two rings: the start re-seats the first ring and runs on
/// to its old start, so the second ring and its link follow unchanged;
/// the exit runs the last ring on to its point nearest the exit. Only
/// the new start and the new end are moved points.
#[test]
fn a_chain_keeps_its_links_and_moves_only_its_ends() {
    let inner = square_ring(10.0, 5.0);
    let outer = square_ring(10.0, 8.0);
    let chain: Vec<Point2> = inner.iter().chain(outer.iter()).copied().collect();
    let sm = LevelSeam { start: Some(p(15.5, 10.0)), exit: Some(p(10.0, 19.0)), z_unread: false };
    let mut passes = vec![chain.clone()];
    let moved = seat_level(&sm, &mut passes, false, 0.0, 6.0).expect("seated");
    let got = &passes[0];
    assert_eq!(got.first().copied(), Some(p(15.0, 10.0)), "the first ring starts at the start point");
    let mut tail: Vec<Point2> = chain[4..].to_vec();
    tail.extend([p(18.0, 2.0), p(18.0, 18.0), p(10.0, 18.0)]);
    assert!(got.ends_with(&tail), "the old start, the link, the second ring, then the run-on to the exit: {got:?}");
    let head = vec![p(15.0, 10.0), p(15.0, 15.0), p(5.0, 15.0), p(5.0, 5.0), p(15.0, 5.0), p(15.0, 10.0), p(15.0, 15.0), p(5.0, 15.0)];
    assert!(got.starts_with(&head) && got.len() == head.len() + tail.len(), "the lap from the start, the run back: {got:?}");
    assert_eq!(moved, vec![p(10.0, 18.0), p(15.0, 10.0)], "the new end and the new start are the moved points");
}
