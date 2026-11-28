//! #4513: the `doosan-dnm` post's non-RTCP indexing path writes the A
//! word with the sign the machine travel check judged. The check signs a
//! lone rotary's angle by its stated direction - a rotary stated along
//! -X commands the negated angle (`judge.rs` `x_rotary_problem`) - so on
//! such a table the stream's pose A20 is machine word A-20. The indexing
//! arm used to write the stream's letter as it came: the check passed
//! (the judged -20 sat inside the stated travel) and the program
//! commanded A20, outside the approved travel, with no warning. The
//! G43.4 arm cannot re-express its RTCP stream and refuses such a table;
//! the indexing arm owns the word and writes the judged sign.
//!
//! The four pins pair each direction (+1 and -1) with each sign
//! convention (`tool_rh` and `table_rh`). The convention changes no
//! sign on this path - the check and the post both read the stated
//! direction - so every case must post the judged word, inside the
//! stated travel, with no refusal. A future "fix" that negates by
//! `signConvention` instead of the direction fails the -1 cases; one
//! that drops the negation fails them again.
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
/// `conv` the machine's rotary sign convention. The travel holds only
/// the judged word of a +20 degree tilt (+20 for a +X table, -20 for a
/// -X table), so a posted word of the wrong sign fails the assertions
/// below.
fn machine(direction: f64, conv: &str) -> (camcraft_post::machine::Machine, serde_json::Value, (f64, f64)) {
    let mut raw: serde_json::Value = serde_json::from_str(&read(MCH)).expect("doosan machine json");
    // The one rotary part of the shipped tree (table > X > Y > A).
    let rot = rotary_part(&mut raw["kinematics"]["default"]["parts"]).expect("the shipped file's rotary moved");
    assert_eq!(rot["name"], json!("A"));
    rot["axis"] = json!([direction, 0.0, 0.0]);
    let (lo, hi) = if direction < 0.0 { (-25.0, -15.0) } else { (15.0, 25.0) };
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

/// The checked machine, the posted program and the judged A word.
fn post_case(direction: f64, conv: &str) -> (engine::PostOutput, f64, (f64, f64)) {
    let (machine, mirror, travel) = machine(direction, conv);
    let path = tilted_path();

    // The travel check judges the pose: the machine word is the tool
    // angle signed by the stated direction, and it sits inside travel.
    let judged = direction * 20.0;
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
    (out, judged, travel)
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

fn assert_judged_sign_is_posted(direction: f64, conv: &str) {
    let (out, judged, travel) = post_case(direction, conv);
    let prog = &out.program;
    assert!(out.warnings.is_empty(), "{conv} direction {direction}: no refusal: {:?}\n{prog}", out.warnings);
    let tilt_words: Vec<f64> = a_words(prog).into_iter().filter(|a| a.abs() > 1e-9).collect();
    assert_eq!(tilt_words.len(), 1, "one tilt word:\n{prog}");
    let a = tilt_words[0];
    assert!(
        (a - judged).abs() < 1e-9,
        "{conv} direction {direction}: the posted A{a} must be the judged A{judged}:\n{prog}"
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
    assert_judged_sign_is_posted(1.0, "tool_rh");
}

#[test]
fn lone_x_along_plus_x_table_rh_posts_the_judged_sign() {
    assert_judged_sign_is_posted(1.0, "table_rh");
}

#[test]
fn lone_x_along_minus_x_tool_rh_posts_the_judged_sign() {
    assert_judged_sign_is_posted(-1.0, "tool_rh");
}

#[test]
fn lone_x_along_minus_x_table_rh_posts_the_judged_sign() {
    assert_judged_sign_is_posted(-1.0, "table_rh");
}
