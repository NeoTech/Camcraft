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
    BoreParams {
        links: crate::links::Links::PLANAR,
        holes: vec![h],
        tool_diameter: dia,
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
    let reach = h.radius - leave - inset;
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
    assert_eq!(max_bore_diameter(4.0, 0.5), 13.0);
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

