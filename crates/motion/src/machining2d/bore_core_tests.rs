//! #1556 pins: the boring ride clears every point of its hole's floor,
//! or refuses by name - never a core left that no pass touches.
//!
//! Provenance: our own law tests over the circular-interpolation bore
//! (`bore.rs`); the coverage check is plain point-to-segment distance.

use super::*;
use crate::cl::{MoveKind, Toolpath};
use crate::geo2d::Point2;

fn hole(radius: f64, through: bool) -> BoreHole {
    BoreHole {
        xy: Point2::new(10.0, 20.0),
        radius,
        top_z: 0.0,
        floor_z: -4.0,
        through,
        stepped: false,
    }
}

fn params(h: BoreHole, dia: f64, leave: f64) -> BoreParams {
    params_c(h, dia, 0.0, leave)
}

fn params_c(h: BoreHole, dia: f64, corner: f64, leave: f64) -> BoreParams {
    BoreParams {
        links: crate::links::Links::PLANAR,
        holes: vec![h],
        tool_diameter: dia,
        corner_radius: corner,
        stock_to_leave: leave,
        stepdown: 2.0,
        through_allowance: 1.0,
        clearance_z: 5.0,
        retract_z: None,
        feed: 800.0,
        plunge_feed: 200.0,
        climb: true,
        tolerance: 0.01,
        entry: crate::entry::Entry3D::Plunge,
        helix_radius: 0.0,
    }
}

fn seg_dist(p: Point2, a: Point2, b: Point2) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len2 = dx * dx + dy * dy;
    let t = if len2 > 0.0 { (((p.x - a.x) * dx + (p.y - a.y) * dy) / len2).clamp(0.0, 1.0) } else { 0.0 };
    let (qx, qy) = (a.x + t * dx, a.y + t * dy);
    ((p.x - qx).powi(2) + (p.y - qy).powi(2)).sqrt()
}

/// The farthest floor point (inside the finished wall, short by `inset`)
/// from every cut move at the deepest level, minus the tool radius:
/// positive means a point no pass touches.
fn worst_uncut(path: &Toolpath, h: &BoreHole, r: f64, leave: f64, inset: f64) -> f64 {
    worst_uncut_within(path, h, r, h.radius - leave - inset)
}

/// [`worst_uncut`] over the floor disc of radius `reach` about the hole's
/// centre (a corner-radiused cutter's own wall fillet stands outside the
/// ring's floor-plane band and is joined to the wall, so it is no core).
fn worst_uncut_within(path: &Toolpath, h: &BoreHole, r: f64, reach: f64) -> f64 {
    let floor = path
        .moves
        .iter()
        .filter(|m| m.kind != MoveKind::Rapid)
        .map(|m| m.to.z)
        .fold(f64::INFINITY, f64::min);
    let segs: Vec<(Point2, Point2)> = path
        .moves
        .iter()
        .filter(|m| m.kind != MoveKind::Rapid && (m.from.z - floor).abs() < 1e-9 && (m.to.z - floor).abs() < 1e-9)
        // Arcs walked as fine chords (inscribed: a chord stands inside
        // its arc, so the check can only under-credit the ride).
        .flat_map(|m| m.chords(1e-4))
        .collect();
    assert!(!segs.is_empty(), "the floor level cuts");
    let mut worst = f64::NEG_INFINITY;
    let n = 60;
    for i in 0..=n {
        for j in 0..=n {
            let x = -reach + 2.0 * reach * i as f64 / n as f64;
            let y = -reach + 2.0 * reach * j as f64 / n as f64;
            if x * x + y * y > reach * reach {
                continue;
            }
            let p = Point2::new(h.xy.x + x, h.xy.y + y);
            let d = segs.iter().map(|(a, b)| seg_dist(p, *a, *b)).fold(f64::INFINITY, f64::min);
            worst = worst.max(d - r);
        }
    }
    worst
}

#[test]
fn bore_refuses_a_bore_that_would_leave_a_free_core() {
    // The issue's bore (74.305 mm there): a 74.3 mm through hole, a 4 mm
    // end mill. The dive and one ring at R35.15 would free a 66.3 mm core.
    let err = bore(&params(hole(37.15, true), 4.0, 0.0)).expect_err("a core is refused, never posted");
    let msg = format!("{err:?}");
    for want in ["74.300 mm hole", "4.000 mm cutter", "R35.150", "core from 4.000 to 66.300 mm", "up to 12.000 mm", "pocket"] {
        assert!(msg.contains(want), "refusal names `{want}`: {msg}");
    }
}

#[test]
fn bore_refuses_just_past_the_limit_and_takes_the_limit() {
    // 4 mm tool, 0.5 mm leave: the limit is 3 x 4 + 2 x 0.5 = 13 mm.
    let at = bore(&params(hole(6.5, false), 4.0, 0.5)).expect("the 13 mm limit hole bores");
    assert!(worst_uncut(&at, &hole(6.5, false), 2.0, 0.5, 1e-3) <= 1e-3, "the limit hole is cleared");
    let err = bore(&params(hole(6.51, false), 4.0, 0.5)).expect_err("13.02 mm is past the limit");
    assert!(format!("{err:?}").contains("too wide to bore"), "{err:?}");
    assert_eq!(max_bore_diameter(4.0, 0.0, 0.5, None), 13.0);
    assert_eq!(max_bore_diameter(4.0, 0.0, 0.5, Some(1.0)), 13.0, "a square end reaches its radius at any floor");
}

#[test]
fn every_bore_the_ride_takes_clears_its_whole_floor() {
    // Below the limit, no floor point is left: the dive's disc and the
    // ring's band meet or overlap, whatever the ratio.
    for (radius, dia) in [(4.0, 6.0), (5.0, 4.0), (6.0, 4.0), (8.9, 6.0), (2.6, 4.0)] {
        let h = hole(radius, false);
        let path = bore(&params(h, dia, 0.0)).expect("a hole within 3 tool diameters bores");
        let gap = worst_uncut(&path, &h, dia * 0.5, 0.0, 1e-3);
        assert!(gap <= 1e-3, "{}-mm hole, {dia} mm tool: a floor point {gap} mm past every pass", radius * 2.0);
    }
}


#[test]
fn the_cutting_radius_follows_the_corner_torus() {
    // r = 6, corner 2: the flat bottom is R4; at 1 mm up the corner adds
    // sqrt(2^2 - 1^2) = sqrt(3); from 2 mm up it is the full R6.
    assert!((cutting_radius_at(6.0, 2.0, 1.0) - (4.0 + 3f64.sqrt())).abs() < 1e-12);
    assert_eq!(cutting_radius_at(6.0, 2.0, 0.0), 4.0);
    assert_eq!(cutting_radius_at(6.0, 2.0, 2.0), 6.0);
    assert_eq!(cutting_radius_at(6.0, 2.0, 7.0), 6.0);
    // A ball (corner = r): no flat; 1 mm up a R3 ball cuts sqrt(1 x 5).
    assert!((cutting_radius_at(3.0, 3.0, 1.0) - 5f64.sqrt()).abs() < 1e-12);
    assert_eq!(cutting_radius_at(3.0, 3.0, 0.0), 0.0);
    // A ball smaller than the drop reaches its full radius.
    assert_eq!(cutting_radius_at(0.5, 0.5, 1.0), 0.5);
    // A square end mill is its radius at every height.
    assert_eq!(cutting_radius_at(2.0, 0.0, 0.0), 2.0);
    // Through: judged at the floor reach; blind: the full radius.
    assert!((bore_floor_reach(12.0, 2.0, Some(1.0)) - (4.0 + 3f64.sqrt())).abs() < 1e-12);
    assert_eq!(bore_floor_reach(12.0, 2.0, None), 6.0);
}

/// The review's example: a Ø12 bull nose with r2 corners on a through
/// hole, its tip 1 mm under the floor, reaches only R5.732 there, so the
/// ride clears through holes up to 12 + 4 x 5.732 = 34.928 mm (not 36).
#[test]
fn a_bull_nose_through_hole_is_judged_at_its_floor_reach() {
    let reach = 4.0 + 3f64.sqrt();
    let limit = 12.0 + 4.0 * reach;
    assert!((max_bore_diameter(12.0, 2.0, 0.0, Some(1.0)) - limit).abs() < 1e-12);
    // At the limit: bores, and every floor point is within the corner's
    // floor-plane reach of a cut at the deepest level.
    let at = hole(limit * 0.5, true);
    let path = bore(&params_c(at, 12.0, 2.0, 0.0)).expect("the 34.928 mm limit hole bores");
    // Inside the ring's floor-plane band (`rc + reach`): the wall's own
    // corner fillet outside it is joined to the wall, no core.
    let band_out = at.radius - 6.0 + reach - 1e-3;
    assert!(worst_uncut_within(&path, &at, reach, band_out) <= 1e-3, "the limit hole's floor is cleared");
    // Just past it: refused by name, with the corner's numbers - the
    // full-radius limit (36 mm) would have posted a free C-shaped ridge.
    let err = bore(&params_c(hole(limit * 0.5 + 0.01, true), 12.0, 2.0, 0.0)).expect_err("34.948 mm frees a ridge");
    let msg = format!("{err:?}");
    for want in ["34.948 mm hole", "up to 34.928 mm", "R5.732 its 2.000 mm corner reaches"] {
        assert!(msg.contains(want), "refusal names `{want}`: {msg}");
    }
    // A blind hole keeps its floor under the corner: the ridge stands on
    // it, never free, so the square-end 36 mm limit holds there. Its
    // ride is the one a through 36 mm hole would have posted under the
    // full-radius limit: at a through floor's reach it leaves the ridge
    // (rc 12 vs 2 x 5.732: a 0.536 mm band, ~0.27 mm from every pass).
    let blind = hole(18.0, false);
    let ride = bore(&params_c(blind, 12.0, 2.0, 0.0)).expect("a blind 36 mm hole bores");
    let sliver = worst_uncut_within(&ride, &blind, reach, 12.0 + reach - 1e-3);
    assert!(sliver > 0.2, "the coverage check sees the through ridge: {sliver}");
    assert!(bore(&params_c(hole(18.01, false), 12.0, 2.0, 0.0)).is_err(), "36.02 mm is past the blind limit");
}

#[test]
fn a_ball_through_hole_and_a_corner_that_does_not_fit_are_judged() {
    // Ø6 ball (corner 3): at the floor, 1 mm above its tip, it cuts
    // sqrt(5) = R2.236, so through holes stop at 6 + 4 x 2.236 = 14.944.
    let limit = 6.0 + 4.0 * 5f64.sqrt();
    bore(&params_c(hole(limit * 0.5, true), 6.0, 3.0, 0.0)).expect("the ball's limit hole bores");
    assert!(bore(&params_c(hole(limit * 0.5 + 0.01, true), 6.0, 3.0, 0.0)).is_err(), "past the ball's limit");
    // A corner wider than the cutter's radius is refused at the door.
    let err = bore(&params_c(hole(5.0, false), 6.0, 3.5, 0.0)).expect_err("a 3.5 corner on a 6 mm cutter");
    assert!(format!("{err:?}").contains("corner radius does not fit"), "{err:?}");
}
