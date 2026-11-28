//! The pose-composition proofs for the machine-frame view (#4266):
//! the work turns about the line and the tool stands upright; the
//! fallback (no line, or a pose that will not stand upright) draws
//! exactly what the playback drew before. Provenance: our own gate
//! math mirrored (`verify::posed_steel`); no vendor material consulted.

use super::*;
use crate::sim::WorkTurn;

/// An A rotary about +X through the WCS origin, the reporter's mill-turn.
fn line() -> WorkTurn {
    WorkTurn { origin: [0.0, 0.0, 0.0], dir: [1.0, 0.0, 0.0] }
}

/// A pose pointing along -Y: the tipped state the issue reports.
fn tipped() -> [f64; 3] {
    [0.0, -1.0, 0.0]
}

#[test]
fn upright_turn_stands_a_tipped_pose_vertical() {
    let t = upright_turn(line().dir, tipped()).expect("a level line and a pose in its plane stand upright");
    // The gate's own sign: -Y stands upright under the minus quarter
    // turn about +X (`posed_steel::upright_turn` reads the same).
    assert!((t + std::f64::consts::FRAC_PI_2).abs() < 1e-12, "{t}");
    // And the turn does stand the pose onto machine +Z.
    let stood = turn_point([0.0; 3], line().dir, tipped(), t);
    assert!(stood[0].abs() < 1e-12 && stood[1].abs() < 1e-12 && (stood[2] - 1.0).abs() < 1e-12, "{stood:?}");
}

#[test]
fn upright_turn_refuses_what_the_rotary_cannot_stand() {
    // A vertical (C) line never tilts a pose.
    assert_eq!(upright_turn([0.0, 0.0, 1.0], tipped()), None);
    // A pose already along the line.
    assert_eq!(upright_turn(line().dir, [1.0, 0.0, 0.0]), None);
    // A degenerate direction.
    assert_eq!(upright_turn([0.0, 0.0, 0.0], tipped()), None);
    // A vertical pose is home: zero turn.
    let t = upright_turn(line().dir, [0.0, 0.0, 1.0]).expect("vertical stands upright");
    assert!(t.abs() < 1e-12, "{t}");
}

#[test]
fn the_work_turn_rotates_points_about_the_line() {
    let t = std::f64::consts::FRAC_PI_2;
    // A point on the line does not move.
    let p = turn_point(line().origin, line().dir, [3.0, 0.0, 0.0], t);
    assert!((p[0] - 3.0).abs() < 1e-12 && p[1].abs() < 1e-12 && p[2].abs() < 1e-12, "{p:?}");
    // +Y goes to +Z under the quarter turn about +X; the radius holds.
    let p = turn_point(line().origin, line().dir, [0.0, 5.0, 0.0], t);
    assert!(p[0].abs() < 1e-12 && p[1].abs() < 1e-12 && (p[2] - 5.0).abs() < 1e-12, "{p:?}");
    // An off-origin line carries the point rigidly with it.
    let moved = WorkTurn { origin: [0.0, 0.0, 10.0], dir: [1.0, 0.0, 0.0] };
    let p = turn_point(moved.origin, moved.dir, [0.0, 5.0, 10.0], t);
    assert!(p[0].abs() < 1e-12 && p[1].abs() < 1e-12 && (p[2] - 15.0).abs() < 1e-12, "{p:?}");
}

#[test]
fn the_tool_stands_upright_at_its_carried_station() {
    // The issue's picture: a tool tipped to -Y at a tip on the work.
    let tip = [0.0, 5.0, 0.0];
    let (tip2, axis2) = posed_tool(tip, tipped(), &line()).expect("the tipped pose stands upright");
    assert_eq!(axis2, [0.0, 0.0, 1.0], "the spindle is vertical - the work turned, not the head");
    let t = upright_turn(line().dir, tipped()).unwrap();
    let want = turn_point(line().origin, line().dir, tip, t);
    assert!((tip2[0] - want[0]).abs() < 1e-12 && (tip2[1] - want[1]).abs() < 1e-12 && (tip2[2] - want[2]).abs() < 1e-12);
    // The tip rides rigidly with the work: turning the tip's own pose
    // by the same turn about the same line is where the tool stands.
    assert!((tip2[0] - 0.0).abs() < 1e-12 && (tip2[1] - 0.0).abs() < 1e-12 && (tip2[2] + 5.0).abs() < 1e-12);
}

#[test]
fn the_fallback_draws_what_the_playback_drew_before() {
    let tip = [1.0, 2.0, 3.0];
    // No line, or a pose the rotary cannot stand: unchanged.
    assert_eq!(frame_turn(&WorkTurn { origin: [0.0; 3], dir: [0.0, 0.0, 1.0] }, tipped()), None);
    assert_eq!(posed_tool(tip, tipped(), &WorkTurn { origin: [0.0; 3], dir: [0.0, 0.0, 1.0] }), None);
    assert_eq!(posed_tool(tip, tipped(), &WorkTurn { origin: [0.0; 3], dir: [0.0; 3] }), None);
}

#[test]
fn work_vertices_turn_about_the_line_in_place() {
    let mut verts: Vec<[f32; 3]> = vec![[0.0, 5.0, 0.0], [3.0, 0.0, 0.0]];
    turn_verts(&mut verts, [0.0; 3], [1.0, 0.0, 0.0], std::f64::consts::FRAC_PI_2);
    let (a, b) = (verts[0], verts[1]);
    assert!(a[0].abs() < 1e-4 && a[1].abs() < 1e-4 && (a[2] - 5.0).abs() < 1e-4, "{a:?}");
    assert!((b[0] - 3.0).abs() < 1e-4 && b[1].abs() < 1e-4 && b[2].abs() < 1e-4, "{b:?}");
    // A degenerate direction turns nothing (the fallback draw).
    let mut same: Vec<[f32; 3]> = vec![[1.0, 2.0, 3.0]];
    turn_verts(&mut same, [0.0; 3], [0.0; 3], 1.0);
    assert_eq!(same[0], [1.0, 2.0, 3.0]);
}

#[test]
fn frame_turn_reads_the_playhead_angle_off_the_frame_axis() {
    let w = line();
    assert!((frame_turn(&w, tipped()).unwrap() + std::f64::consts::FRAC_PI_2).abs() < 1e-12);
    assert!(frame_turn(&w, [0.0, 0.0, 1.0]).unwrap().abs() < 1e-12);
}
