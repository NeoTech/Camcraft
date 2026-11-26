//! #4045: the Sodick LN2W wire post (`sodick-ln2w.cps`) on synthetic
//! streams. Pins the start sequence that the owner of an AQ600L gave, the
//! absence of a unit word (the control alarms on G21), the refusal of each
//! taper (#4075), the skim flushing and each named refusal.
//! Provenance: synthetic streams from our CNC intermediate contract; the
//! expected words are the owner's statements and the LN manual facts that
//! docs/decisions/4045-sodick-ln2w-dialect.md records.
//! Gate map: covers assets/posts/sodick-ln2w.cps, camcraft_post::post::engine,
//! camcraft_post::post::cnc.

use camcraft_post::cl::{ArcPlane, MoveKind};
use camcraft_post::post::cnc::{self, MoveRec, Record};
use camcraft_post::post::engine;
use serde_json::{json, Value};

const SCRIPT: &str = include_str!("../../../../assets/posts/sodick-ln2w.cps");

fn mv(kind: MoveKind, x: f64, y: f64) -> Record {
    Record::Move(MoveRec { kind, x, y, z: 0.0, feed: (kind == MoveKind::Feed).then_some(2.0), center: None, ccw: false, plane: ArcPlane::Xy, abc: None })
}

fn param(name: &str, value: Value) -> Record {
    Record::Param { name: name.into(), value }
}

/// The section's opening params; `wire` holds the taper params.
fn head(wire: &[(&str, Value)]) -> Vec<Record> {
    let mut r = vec![param("OperationName", json!("Punch1")), param("WorkOffset", json!("G54")), param("EdmCondition", json!("C001"))];
    r.extend(wire.iter().map(|(n, v)| param(n, v.clone())));
    r.extend([Record::Section { begin: true }, mv(MoveKind::Rapid, -22.0, 10.0)]);
    r
}

fn machine(extra: Value) -> Value {
    let mut m = json!({"kind": "wire_edm", "name": "Sodick AQ600L", "edm": {"awt": true, "tank": "submerged", "flushing": "high", "skim_flushing": "low"}});
    if let (Some(e), Some(x)) = (m["edm"].as_object_mut(), extra.as_object()) {
        e.extend(x.clone());
    }
    m
}

fn run(machine: &Value, records: &[Record]) -> engine::PostOutput {
    engine::run(SCRIPT, machine, &cnc::serialize(records)).expect("the synthetic stream runs")
}

/// The program's executable lines (no comment lines).
fn words(out: &engine::PostOutput) -> Vec<&str> {
    out.program.lines().filter(|l| !l.starts_with('(')).collect()
}

/// A taper of `angle` degrees: the lead-in leans, one side, the lead-out
/// stands upright.
fn tapered(angle: f64) -> Vec<Record> {
    let mut r = head(&[
        ("WireTaper", json!(angle.abs())),
        ("WireTaperDialect", json!("angle")),
        ("WireCorner", json!("mitre")),
        ("WirePlane", json!(12.0)),
    ]);
    r.extend([
        param("WireAngle", json!(angle)),
        mv(MoveKind::Feed, -25.0, 10.0),
        param("WireAngle", json!(angle)),
        mv(MoveKind::Feed, -25.0, -10.0),
        param("WireAngle", json!(0.0)),
        mv(MoveKind::Feed, -22.0, 10.0),
        Record::Section { begin: false },
    ]);
    r
}

#[test]
fn the_start_sequence_is_the_owners_and_no_unit_word_is_written() {
    let mut r = head(&[]);
    r.extend([mv(MoveKind::Feed, -25.838, 10.0), mv(MoveKind::Feed, -22.0, 10.0), Record::Section { begin: false }]);
    let out = run(&machine(json!({})), &r);
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    assert_eq!(
        words(&out),
        ["G90", "G54", "G00 X-22. Y10. U0. V0.", "Z0.", "T91", "T84", "T94", "C001", "G01 X-25.838 Y10.", "G01 X-22. Y10.", "T90", "M02"],
        "{}",
        out.program
    );
    for banned in ["G21", "G20", "G40", "G92", "M30"] {
        assert!(!words(&out).iter().any(|l| l.split(' ').any(|w| w == banned)), "{banned} in\n{}", out.program);
    }
    assert!(out.program.contains("(SODICK LN2W WORDS - NOT VERIFIED ON A CONTROL - CHECK BEFORE CUTTING)"), "{}", out.program);
    assert!(SCRIPT.contains("programUnits = \"mm\";"), "an inch document must be refused: no unit word can be written");
}

/// Review of PR #4065 (findings 3, 4, 6): no taper word is written. A
/// positive or negative stream angle (a top_left or a top_right machine
/// signs it either way), a machine that states a table height, and a
/// 0.0004 degree angle that would round to `A0.` are each refused by name,
/// with no TP / TN, G51 / G52, A, G50 or cut written (#4075).
#[test]
fn every_taper_is_refused_by_name_and_writes_no_taper_word() {
    let taper_word = |out: &engine::PostOutput| {
        words(out).iter().any(|l| l.starts_with("TP") || l.starts_with("TN") || l.starts_with("G5") || l.contains(" A") || l.starts_with("G01") || *l == "T91")
    };
    let side = |s: &str| json!({"table_to_part_bottom": 5.0, "taper_angle": {"positive": s, "corner": "mitre"}});
    for (m, angle) in [(machine(json!({})), 3.0), (machine(side("top_right")), 3.0), (machine(side("top_right")), -3.0), (machine(json!({})), 0.0004)] {
        let out = run(&m, &tapered(angle));
        assert!(out.warnings.iter().any(|w| w.contains("Punch1") && w.contains("is tapered") && w.contains("#4075")), "{angle}: {:?}", out.warnings);
        assert!(!taper_word(&out), "{angle}:\n{}", out.program);
    }
}

#[test]
fn a_lean_without_a_taper_section_is_refused_by_name() {
    let cut = |out: &engine::PostOutput| words(out).iter().any(|l| l.starts_with("G01") || *l == "T91");
    // A guide-offset (U V) stream.
    let mut uv = head(&[("WireTaper", json!(3.0))]);
    uv.extend([param("WireU", json!(0.6)), param("WireV", json!(0.0)), mv(MoveKind::Feed, -25.0, 10.0), Record::Section { begin: false }]);
    let out = run(&machine(json!({})), &uv);
    assert!(out.warnings.iter().any(|w| w.contains("is tapered")), "{:?}", out.warnings);
    assert!(!cut(&out), "{}", out.program);
    // A 4-axis cut.
    let mut four = head(&[("WireFourAxis", json!("arc length"))]);
    four.extend([param("WireU", json!(0.6)), param("WireV", json!(0.0)), mv(MoveKind::Feed, -25.0, 10.0), Record::Section { begin: false }]);
    let out = run(&machine(json!({})), &four);
    assert!(out.warnings.iter().any(|w| w.contains("is a 4-axis cut")), "{:?}", out.warnings);
    assert!(!cut(&out), "{}", out.program);
    // An angle word in a vertical section.
    let mut stray = head(&[]);
    stray.extend([param("WireAngle", json!(3.0)), mv(MoveKind::Feed, -25.0, 10.0), Record::Section { begin: false }]);
    let out = run(&machine(json!({})), &stray);
    assert!(out.warnings.iter().any(|w| w.contains("an angle word") && w.contains("#4075")), "{:?}", out.warnings);
    assert!(!words(&out).iter().any(|l| l.starts_with("G01")), "{}", out.program);
    // A leaning (U V) move in a vertical section.
    let mut lean = head(&[]);
    lean.extend([param("WireU", json!(0.6)), param("WireV", json!(0.0)), mv(MoveKind::Feed, -25.0, 10.0), Record::Section { begin: false }]);
    let out = run(&machine(json!({})), &lean);
    assert!(out.warnings.iter().any(|w| w.contains("writes no U V lean")), "{:?}", out.warnings);
    assert!(!words(&out).iter().any(|l| l.starts_with("G01")), "{}", out.program);
}

#[test]
fn a_skim_pass_takes_the_low_flushing_word_and_its_own_condition() {
    let mut r = head(&[]);
    r.extend([
        mv(MoveKind::Feed, -25.0, 10.0),
        mv(MoveKind::Feed, -22.0, 10.0),
        param("EdmCondition", json!("C002")),
        mv(MoveKind::Feed, -25.02, 10.0),
        mv(MoveKind::Feed, -22.0, 10.0),
        Record::Section { begin: false },
    ]);
    let out = run(&machine(json!({})), &r);
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    let w = words(&out);
    let at = w.iter().position(|l| *l == "T85").expect("the skim's flushing word");
    assert_eq!(&w[at..at + 3], ["T85", "C002", "G01 X-25.02 Y10."], "{}", out.program);
    assert_eq!(w.iter().filter(|l| **l == "T91").count(), 1, "one threading for the section: {}", out.program);
}
