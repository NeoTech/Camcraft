//! #4062: stock columns cannot decide whether a vertical rapid passes
//! through a canopy. Synthetic closed box and explicit cutter profiles.
//! Gate map: covers camcraft_sim::verify (engine, solid, rapids, solid_index).

use camcraft_sim::cl::{Move, MoveKind, Toolpath};
use camcraft_sim::geo2d::{Loop, Point2, Region};
use camcraft_sim::geometry::{Point3, cube};
use camcraft_sim::verify::{Cutter, Mode, ToolGeom, VerifyContext, Warning, verify};
use camcraft_tools::undercut::{Head, Undercut};

fn point(x: f64, z: f64) -> Point3 {
    Point3::new(x, 0.0, z)
}

fn motion(kind: MoveKind, x0: f64, z0: f64, x1: f64, z1: f64) -> Move {
    Move::new(kind, point(x0, z0), point(x1, z1), None)
}

fn judge(moves: Vec<Move>, head: Head) -> Vec<Warning> {
    let region = Region::new(vec![Loop::new(vec![
        Point2::new(-5.0, -5.0), Point2::new(0.0, -5.0),
        Point2::new(0.0, 5.0), Point2::new(-5.0, 5.0),
    ]).expect("box outline")]).expect("stock");
    let roof = cube(Point3::new(-5.0, -5.0, 10.0), Point3::new(0.0, 5.0, 12.0));
    let tool = ToolGeom {
        radius: 3.0, nose_radius: 0.0, flute_len: 1.0,
        shank_radius: 0.5, shank_len: 20.0, holder_radius: 0.5, holder_len: 5.0,
    };
    let cutter = Cutter {
        undercut: Some(Undercut { r: 3.0, height: 1.0, neck_r: 0.5, head }),
        ..Cutter::flat(&tool)
    };
    verify(&Toolpath::new(moves), &VerifyContext {
        region: &region, stock: Some(&region), model: Some(&roof.facets),
        mode: Mode::Envelope, tool: &tool, cutter: Some(cutter),
        stock_top_z: 12.0, gouge_tol: 0.01, z_tol: 0.001, finish_tol: Some(0.1),
        ..Default::default()
    })
}

fn assert_canopy_hit(head: Head, from_z: f64, to_z: f64) {
    // X=2: the narrow shank clears the roof edge, the head overlaps it.
    // The feed and both endpoints of the rapid clear the roof.
    let approach = motion(MoveKind::Feed, 4.0, from_z, 2.0, from_z);
    let rapid = motion(MoveKind::Rapid, 2.0, from_z, 2.0, to_z);
    let warnings = judge(vec![approach, rapid], head);
    let rapid_hits: Vec<_> = warnings.iter()
        .filter(|w| matches!(w, Warning::RapidInPart { .. }))
        .collect();
    assert_eq!(rapid_hits, [&Warning::RapidInPart { at: 1 }], "{warnings:?}");
    assert!(rapid_hits[0].blocks_post());
}

#[test]
fn thin_disc_retract_hits_canopy() {
    assert_canopy_hit(Head::Disc, 4.0, 14.0);
}

#[test]
fn thin_disc_descent_hits_canopy() {
    assert_canopy_hit(Head::Disc, 14.0, 4.0);
}

#[test]
fn dovetail_retract_hits_canopy() {
    assert_canopy_hit(Head::Dove { cot: 0.5 }, 4.0, 14.0);
}

#[test]
fn dovetail_descent_hits_canopy() {
    assert_canopy_hit(Head::Dove { cot: 0.5 }, 14.0, 4.0);
}

#[test]
fn lateral_exit_then_retract_stays_clear() {
    for head in [Head::Disc, Head::Dove { cot: 0.5 }] {
        let exit = motion(MoveKind::Feed, 2.0, 4.0, 4.0, 4.0);
        let up = motion(MoveKind::Rapid, 4.0, 4.0, 4.0, 14.0);
        let warnings = judge(vec![exit, up], head);
        assert!(warnings.is_empty(), "{head:?}: {warnings:?}");
    }
}

#[test]
fn an_ordinary_peck_retract_in_its_own_stock_kerf_stays_clear() {
    let region = Region::new(vec![Loop::new(vec![
        Point2::new(-10.0, -10.0), Point2::new(10.0, -10.0),
        Point2::new(10.0, 10.0), Point2::new(-10.0, 10.0),
    ]).expect("blank outline")]).expect("stock");
    let tool = ToolGeom {
        radius: 1.0, nose_radius: 0.0, flute_len: 8.0,
        shank_radius: 1.0, shank_len: 20.0, holder_radius: 2.0, holder_len: 5.0,
    };
    let cutter = Cutter { tip_angle: 118.0, ..Cutter::flat(&tool) };
    let path = Toolpath::new(vec![
        motion(MoveKind::Plunge, 0.0, 12.0, 0.0, 5.0),
        motion(MoveKind::Rapid, 0.0, 5.0, 0.0, 7.0),
    ]);
    let w = verify(&path, &VerifyContext {
        region: &region, stock: Some(&region), mode: Mode::Area,
        tool: &tool, cutter: Some(cutter), stock_top_z: 10.0,
        gouge_tol: 0.01, z_tol: 0.001, ..Default::default()
    });
    assert!(!w.iter().any(|w| matches!(w, Warning::RapidInPart { at: 1 } | Warning::RapidInStock { at: 1 })), "{w:?}");
}

/// #4211: a finish pass ends with the edge on the wall, inside the finish
/// tolerance (0.05 in, finish tolerance 0.1). The retract up its own axis
/// out of that pose enters no part. The cut's pose is the finish law's;
/// the lift is judged by the shank, the holder and the edge's rise column.
#[test]
fn a_retract_from_a_finish_touch_on_the_wall_stays_clear() {
    let region = Region::new(vec![Loop::new(vec![
        Point2::new(-10.0, -10.0), Point2::new(10.0, -10.0),
        Point2::new(10.0, 10.0), Point2::new(-10.0, 10.0),
    ]).expect("blank outline")]).expect("stock");
    let wall = cube(Point3::new(-5.0, -5.0, 0.0), Point3::new(0.0, 5.0, 4.0));
    let tool = ToolGeom {
        radius: 1.0, nose_radius: 0.0, flute_len: 8.0,
        shank_radius: 1.0, shank_len: 20.0, holder_radius: 2.0, holder_len: 5.0,
    };
    let path = Toolpath::new(vec![
        Move::new(MoveKind::Feed, Point3::new(0.95, -3.0, 2.0), Point3::new(0.95, 3.0, 2.0), None),
        Move::new(MoveKind::Rapid, Point3::new(0.95, 3.0, 2.0), Point3::new(0.95, 3.0, 25.0), None),
    ]);
    let w = verify(&path, &VerifyContext {
        region: &region, stock: Some(&region), model: Some(&wall.facets),
        mode: Mode::Envelope, tool: &tool, cutter: Some(Cutter::flat(&tool)),
        stock_top_z: 4.0, clearance_z: 25.0, gouge_tol: 0.01, z_tol: 0.001,
        finish_tol: Some(0.1), ..Default::default()
    });
    assert!(!w.iter().any(|w| matches!(w, Warning::RapidInPart { at: 1 })), "{w:?}");
}
