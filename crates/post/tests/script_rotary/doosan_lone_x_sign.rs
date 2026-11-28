//! #4513: the `doosan-dnm` post's non-RTCP indexing path writes the A
//! word with the sign the machine travel check judges. The indexing
//! arm used to ignore a lone rotary's stated direction. On a -X
//! `tool_rh` rotary, the check approved A-20 but the program commanded
//! A20. The G43.4 arm still refuses a reversed lone-X rotary.
//!
//! The four pins pair each direction (+1 and -1) with each sign
//! convention (`tool_rh` and `table_rh`). The check and the post first
//! account for the stated axis direction. An inherited `table_rh`
//! convention reverses the sign again because the work turns relative
//! to the tool. Each fixture's travel includes only the expected sign.
//!
//! Provenance: RS-274 / ISO 841 rotary lettering and our own pose law
//! (`cl::pose`); our own synthetic streams and machine JSON.
//!
//! Gate map (M4.4): covers assets/posts/doosan-dnm.cps (the indexing
//! arm of `tilt`), camcraft_post::machine::judge (`x_rotary_problem`),
//! camcraft_post::post::engine.

use camcraft_post::cl::{Move, MoveKind, Toolpath};
use camcraft_post::geometry::Point3;
use camcraft_post::machine::{parse_mch, MachineCheck};
use camcraft_post::post::{self, cnc, engine};
use serde_json::json;
use std::path::Path;

const POST: &str = "assets/posts/doosan-dnm.cps";
const MCH: &str = "assets/machines/doosan-dnm.mch";
const UP: [f64; 3] = [0.0, 0.0, 1.0];

fn read(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../").join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

fn p(x: f64, y: f64, z: f64) -> Point3 {
    Point3 { x, y, z }
}

/// The first part of the tree with `"type": "rotary"`, for mutation.
fn rotary_part(v: &mut serde_json::Value) -> Option<&mut serde_json::Value> {
    if v.get("type").and_then(serde_json::Value::as_str) == Some("rotary") {
        return Some(v);
    }
    match v {
        serde_json::Value::Array(arr) => arr.iter_mut().find_map(rotary_part),
        serde_json::Value::Object(_) => v.get_mut("parts").and_then(rotary_part),
        _ => None,
    }
}

/// The parsed machine, the script's mirror of it, and the stated A
/// travel. `direction` states the rotary's direction sense along X,
/// `conv` the machine's rotary sign convention. Center the travel on
/// the fixture's expected controller A word for the +20 degree tilt.
fn machine(
    direction: f64,
    conv: &str,
    expected_a: f64,
) -> (camcraft_post::machine::Machine, serde_json::Value, (f64, f64)) {
    let mut raw: serde_json::Value = serde_json::from_str(&read(MCH)).expect("doosan machine json");
    // The one rotary part of the shipped tree (table > X > Y > A).
    let rot = rotary_part(&mut raw["kinematics"]["default"]["parts"]).expect("the shipped file's rotary moved");
    assert_eq!(rot["name"], json!("A"));
    rot["axis"] = json!([direction, 0.0, 0.0]);
    let (lo, hi) = (expected_a - 5.0, expected_a + 5.0);
    rot["travel"] = json!({ "min": lo, "max": hi });
    raw["kinematics"]["default"]["sign_convention"] = json!(conv);
    raw["controller"]["rtcp"] = json!(false);
    let m = parse_mch(&raw.to_string()).expect("the machine parses");
    let mirror = post::machine::mirror(&m);
    assert!(!mirror["rtcp"].as_bool().unwrap_or(true), "the machine posts non-RTCP");
    (m, mirror, (lo, hi))
}

/// A tilted indexing path: upright approach, one feed leaned +20 degrees
/// about X (the pose law's (0, -sin t, cos t)), then the exit rapid that
/// keeps the lean.
fn tilted_path() -> Toolpath {
    let (s, c) = (20f64.to_radians().sin(), 20f64.to_radians().cos());
    let lean = [0.0, -s, c];
    Toolpath::new(vec![
        Move::new(MoveKind::Rapid, p(0.0, 0.0, 50.0), p(0.0, 0.0, 20.0), None).with_axis(UP),
        Move::new(MoveKind::Rapid, p(0.0, 0.0, 20.0), p(0.0, 0.0, 5.0), None).with_axis(UP),
        Move::new(MoveKind::Feed, p(0.0, 0.0, 5.0), p(0.0, 0.0, 0.0), Some(300.0)).with_axis(UP),
        Move::new(MoveKind::Feed, p(0.0, 0.0, 0.0), p(10.0, 0.0, 0.0), Some(500.0)).with_axis(lean),
        Move::new(MoveKind::Rapid, p(10.0, 0.0, 0.0), p(10.0, 0.0, 20.0), None).with_axis(lean),
    ])
}

/// Check the fixture's travel and post its program.
fn post_case(direction: f64, conv: &str, expected_a: f64) -> (engine::PostOutput, (f64, f64)) {
    let (machine, mirror, travel) = machine(direction, conv, expected_a);
    let path = tilted_path();

    let mut check = MachineCheck::default();
    check.check_path(&machine, None, &path, [0.0; 3], [false; 3], true);
    assert!(check.problems.is_empty(), "the check passes: {:?}", check.problems);

    let dump = post::dump(
        &path,
        &post::Dump {
            op_name: "Tilt",
            tool_number: 1,
            tool_name: "EM-0800",
            tool_diameter: 8.0,
            rpm: 6000.0,
            clearance_z: 45.0,
            retract_z: 20.0,
            top_z: 0.0,
            wcs: 1,
            origin: [0.0; 3],
            stock: [-50.0, 50.0, -50.0, 50.0],
            stock_top_z: 0.0,
            ..Default::default()
        },
    );
    let out = engine::run_with(&read(POST), &mirror, &cnc::serialize(&dump), &json!({ "rotaryBrake": true }))
        .unwrap_or_else(|e| panic!("the stream posts or refuses by warning: {e}"));
    (out, travel)
}

/// The program's A words, comments dropped.
fn a_words(program: &str) -> Vec<f64> {
    let mut plain = String::new();
    let mut depth = 0usize;
    for c in program.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ if depth == 0 => plain.push(c),
            _ => {}
        }
    }
    plain
        .split_whitespace()
        .filter_map(|w| w.strip_prefix('A').and_then(|a| a.parse::<f64>().ok()))
        .collect()
}

fn assert_judged_sign_is_posted(direction: f64, conv: &str, expected_a: f64) {
    let (out, travel) = post_case(direction, conv, expected_a);
    let prog = &out.program;
    assert!(out.warnings.is_empty(), "{conv} direction {direction}: no refusal: {:?}\n{prog}", out.warnings);
    let tilt_words: Vec<f64> = a_words(prog).into_iter().filter(|a| a.abs() > 1e-9).collect();
    assert_eq!(tilt_words.len(), 1, "one tilt word:\n{prog}");
    let a = tilt_words[0];
    assert!(
        (a - expected_a).abs() < 1e-9,
        "{conv} direction {direction}: the posted A{a} must be the expected A{expected_a}:\n{prog}"
    );
    assert!(
        a >= travel.0 - 1e-9 && a <= travel.1 + 1e-9,
        "{conv} direction {direction}: A{a} leaves the stated travel {}..{}:\n{prog}",
        travel.0,
        travel.1
    );
}

#[test]
fn lone_x_along_plus_x_tool_rh_posts_the_judged_sign() {
    assert_judged_sign_is_posted(1.0, "tool_rh", 20.0);
}

#[test]
fn lone_x_along_plus_x_table_rh_posts_the_judged_sign() {
    assert_judged_sign_is_posted(1.0, "table_rh", -20.0);
}

#[test]
fn lone_x_along_minus_x_tool_rh_posts_the_judged_sign() {
    assert_judged_sign_is_posted(-1.0, "tool_rh", -20.0);
}

#[test]
fn lone_x_along_minus_x_table_rh_posts_the_judged_sign() {
    assert_judged_sign_is_posted(-1.0, "table_rh", 20.0);
}
