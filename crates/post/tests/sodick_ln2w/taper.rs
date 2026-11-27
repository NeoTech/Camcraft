//! #4075: a constant taper on the Sodick LN2W post, as a 4-axis program
//! (the owner of an AQ600L, 2026-11-25): TP (the part's bottom above the
//! worktable) and TN (its top above the worktable), G74, X Y on the part's
//! bottom and U V = top minus bottom on every move, G75 at the end. The
//! G51 / G52 angle form stays refused.
//! Provenance: synthetic streams from our CNC intermediate contract
//! (docs/CNC-INTERMEDIATE.md: WireU / WireV at the guide planes); the
//! expected words are the owner's statements recorded on #4075.
//! Gate map: covers assets/posts/sodick-ln2w.cps, camcraft_post::post::engine.

use camcraft_post::cl::{ArcPlane, CompSide, MoveKind};
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

fn machine() -> Value {
    json!({"kind": "wire_edm", "name": "Sodick AQ600L", "edm": {"awt": true, "tank": "submerged", "flushing": "high", "skim_flushing": "low"}})
}

/// The section's opening params for `setup`: a 3 degree taper whose
/// guides stand at Z-10 and Z90 (a 100 mm span) and whose part runs from
/// Z0 to Z20 (work Z); `drop` leaves out the named params.
fn head(setup: &str, drop: &[&str]) -> Vec<Record> {
    head_at("Punch1", setup, drop, 0.0)
}

/// The same opening for operation `op`, with the cut's bottom at work Z
/// `bottom` (two Wire Contour operations of one setup can state two).
fn head_at(op: &str, setup: &str, drop: &[&str], bottom: f64) -> Vec<Record> {
    let mut r = vec![param("OperationName", json!(op)), param("SetupName", json!(setup)), param("WorkOffset", json!("G54")), param("EdmCondition", json!("C001"))];
    for (n, v) in [("WireTaper", 3.0), ("WireGuideLower", -10.0), ("WireGuideUpper", 90.0), ("WireBottom", bottom), ("WireTop", 20.0)] {
        if !drop.contains(&n) {
            r.push(param(n, json!(v)));
        }
    }
    r.extend([Record::Section { begin: true }, mv(MoveKind::Rapid, -22.0, 10.0)]);
    r
}

/// The lead-in leans (U5 at the upper guide, over the 100 mm span), one
/// side, and the lead-out stands upright again.
fn body() -> Vec<Record> {
    vec![
        param("WireU", json!(5.0)),
        param("WireV", json!(0.0)),
        mv(MoveKind::Feed, -25.0, 10.0),
        param("WireU", json!(5.0)),
        param("WireV", json!(0.0)),
        mv(MoveKind::Feed, -25.0, -10.0),
        mv(MoveKind::Feed, -22.0, 10.0),
        Record::Section { begin: false },
    ]
}

fn tapered(setup: &str, drop: &[&str]) -> Vec<Record> {
    let mut r = head(setup, drop);
    r.extend(body());
    r
}

fn run(records: &[Record], options: &Value) -> engine::PostOutput {
    engine::run_with(SCRIPT, &machine(), &cnc::serialize(records), options).expect("the synthetic stream runs")
}

fn words(out: &engine::PostOutput) -> Vec<&str> {
    out.program.lines().filter(|l| !l.starts_with('(')).collect()
}

/// A program that writes no taper word and no cut.
fn nothing_cut(out: &engine::PostOutput) -> bool {
    !words(out).iter().any(|l| l.starts_with("TP") || l.starts_with("TN") || l.starts_with("G74") || l.starts_with("G01") || *l == "T91")
}

fn height(h: f64) -> Value {
    json!({ "partBottomAboveTable": h })
}

/// The owner's 4-axis form: the start order of #4065, then TP TN G74,
/// each move with X Y on the part's bottom and U V over its height, and
/// G75 after the last cut. Slope 5 / 100: the bottom (Z0) is 10 mm above
/// the lower guide, so X moves 0.5 toward the lean; U is 0.05 x 20 = 1.
#[test]
fn a_taper_posts_as_a_four_axis_program_with_tp_tn_and_uv_on_every_move() {
    let out = run(&tapered("Setup1", &[]), &height(5.0));
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    assert_eq!(
        words(&out),
        [
            "G90", "G54", "G00 X-22. Y10. U0. V0.", "Z0.", "T91", "T84", "T94", "C001", "TP5.", "TN25.", "G74",
            "G01 X-24.5 Y10. U1. V0.", "G01 X-24.5 Y-10. U1. V0.", "G01 X-22. Y10. U0. V0.", "G75", "T90", "M02",
        ],
        "{}",
        out.program
    );
    for banned in ["G51", "G52", "G50"] {
        assert!(!words(&out).iter().any(|l| l.split(' ').any(|w| w == banned)), "{banned} in\n{}", out.program);
    }
}

/// The lean side is the sign of U V: a top that leans to -X and +Y.
#[test]
fn the_lean_side_is_the_sign_of_u_and_v() {
    let mut r = head("Setup1", &[]);
    r.extend([param("WireU", json!(-5.0)), param("WireV", json!(2.5)), mv(MoveKind::Feed, -25.0, 10.0), mv(MoveKind::Feed, -22.0, 10.0), Record::Section { begin: false }]);
    let out = run(&r, &height(0.0));
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    assert!(words(&out).contains(&"G01 X-25.5 Y10.25 U-1. V0.5"), "{}", out.program);
    assert!(words(&out).contains(&"TP0.") && words(&out).contains(&"TN20."), "{}", out.program);
}

/// TP and TN are heights above the worktable; CamCraft does not know the
/// fixture. Unstated, the taper is refused by name and nothing is cut.
#[test]
fn a_taper_without_the_table_height_is_refused_by_name() {
    for options in [Value::Null, height(-1.0)] {
        let out = run(&tapered("Setup1", &[]), &options);
        assert!(
            out.warnings.iter().any(|w| w.contains("Punch1") && w.contains("is tapered") && w.contains("partBottomAboveTable") && w.contains("#4075")),
            "{options}: {:?}",
            out.warnings
        );
        assert!(nothing_cut(&out), "{options}:\n{}", out.program);
    }
}

/// A stream that states no guide planes or no part planes cannot be
/// converted: refused by name, never written from the guide planes.
#[test]
fn a_taper_stream_without_its_planes_is_refused_by_name() {
    for drop in ["WireGuideLower", "WireGuideUpper", "WireBottom", "WireTop"] {
        let out = run(&tapered("Setup1", &[drop]), &height(5.0));
        assert!(out.warnings.iter().any(|w| w.contains("is tapered") && w.contains(drop)), "{drop}: {:?}", out.warnings);
        assert!(nothing_cut(&out), "{drop}:\n{}", out.program);
    }
}

/// One option states one table height: tapers from two setups are refused.
#[test]
fn tapers_from_two_setups_are_refused_by_name() {
    let mut r = tapered("Setup1", &[]);
    r.extend(tapered("Setup2", &[]));
    let out = run(&r, &height(5.0));
    assert!(out.warnings.iter().any(|w| w.contains("Setup2") && w.contains("Setup1") && w.contains("one table height")), "{:?}", out.warnings);
}

/// Two tapered cuts of `Setup1`, operations `First` then `Second`, with
/// their bottoms at the two given work Z; the top of each is Z20.
fn two_bottoms(first: f64, second: f64) -> Vec<Record> {
    let mut r = head_at("First", "Setup1", &[], first);
    r.extend(body());
    r.extend(head_at("Second", "Setup1", &[], second));
    r.extend(body());
    r
}

/// The lines of operation `op`'s section: from its name note to its T90.
fn section<'a>(out: &'a engine::PostOutput, op: &str) -> Vec<&'a str> {
    let all: Vec<&str> = out.program.lines().collect();
    let from = all.iter().position(|l| *l == format!("({op})")).unwrap_or_else(|| panic!("no section {op} in\n{}", out.program));
    let len = all[from..].iter().position(|l| *l == "T90").unwrap_or_else(|| panic!("section {op} has no T90 in\n{}", out.program));
    all[from..from + len].to_vec()
}

/// A section's words from its TP word on.
fn cut<'a>(lines: &[&'a str]) -> Vec<&'a str> {
    lines.iter().filter(|l| !l.starts_with('(')).skip_while(|l| !l.starts_with("TP")).copied().collect()
}

/// X Y are on each section's own bottom, so TP follows it (the review of
/// PR #4171): the option states the first taper's bottom above the
/// worktable, and a cut that starts 8 mm higher gets TP + 8. TN is the
/// same top for the two (5 + 20). The second cut's bottom is 18 mm above
/// the lower guide: X moves 0.05 x 18 = 0.9, and U is 0.05 x 12 = 0.6.
/// Each section note names the work Z that its TP stands for.
#[test]
fn tp_follows_each_tapered_sections_own_bottom() {
    let out = run(&two_bottoms(0.0, 8.0), &height(5.0));
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    let (first, second) = (section(&out, "First"), section(&out, "Second"));
    assert_eq!(cut(&first), ["TP5.", "TN25.", "G74", "G01 X-24.5 Y10. U1. V0.", "G01 X-24.5 Y-10. U1. V0.", "G01 X-22. Y10. U0. V0.", "G75"], "{}", out.program);
    assert_eq!(cut(&second), ["TP13.", "TN25.", "G74", "G01 X-24.1 Y10. U0.6 V0.", "G01 X-24.1 Y-10. U0.6 V0.", "G01 X-22. Y10. U0. V0.", "G75"], "{}", out.program);
    assert!(first.iter().any(|l| l.contains("TP 5. IS WORK Z0. ABOVE THE WORKTABLE") && l.contains("THE POST OPTION STATES WORK Z0. AT 5.")), "{}", out.program);
    assert!(second.iter().any(|l| l.contains("TP 13. IS WORK Z8. ABOVE THE WORKTABLE") && l.contains("THE POST OPTION STATES WORK Z0. AT 5.")), "{}", out.program);
    // The reverse order: the option names the first cut's bottom (work Z8),
    // and the later, lower cut gets TP - 8.
    let out = run(&two_bottoms(8.0, 0.0), &height(8.0));
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    assert_eq!(cut(&section(&out, "First"))[..2], ["TP8.", "TN20."], "{}", out.program);
    assert_eq!(cut(&section(&out, "Second"))[..2], ["TP0.", "TN20."], "{}", out.program);
    assert!(section(&out, "Second").iter().any(|l| l.contains("TP 0. IS WORK Z0. ABOVE THE WORKTABLE") && l.contains("THE POST OPTION STATES WORK Z8. AT 8.")), "{}", out.program);
}

/// A later cut whose bottom is below the worktable by the stated option is
/// refused by name, and no word of it is written.
#[test]
fn a_tapered_section_below_the_worktable_is_refused_by_name() {
    let out = run(&two_bottoms(8.0, 0.0), &height(5.0));
    assert!(
        out.warnings.iter().any(|w| w.contains("Second") && w.contains("is tapered") && w.contains("below the worktable") && w.contains("work Z8.") && w.contains("partBottomAboveTable")),
        "{:?}",
        out.warnings
    );
    assert_eq!(words(&out).iter().filter(|l| l.starts_with("TP")).copied().collect::<Vec<_>>(), ["TP5."], "{}", out.program);
    assert_eq!(words(&out).iter().filter(|l| **l == "G74" || **l == "T91").count(), 2, "{}", out.program);
}

/// What the 4-axis form does not write is refused by name: an arc, and
/// the control's wire offset under a taper.
#[test]
fn an_arc_or_a_wire_offset_in_a_taper_is_refused_by_name() {
    let mut arc = head("Setup1", &[]);
    arc.extend([
        mv(MoveKind::Feed, -25.0, 10.0),
        Record::Move(MoveRec { kind: MoveKind::Feed, x: -25.0, y: -10.0, z: 0.0, feed: Some(2.0), center: Some((-25.0, 0.0)), ccw: false, plane: ArcPlane::Xy, abc: None }),
        mv(MoveKind::Feed, -22.0, 10.0),
        Record::Section { begin: false },
    ]);
    let out = run(&arc, &height(5.0));
    assert!(out.warnings.iter().any(|w| w.contains("Punch1") && w.contains("an arc in a tapered cut")), "{:?}", out.warnings);
    assert!(!words(&out).iter().any(|l| l.starts_with("G02") || l.starts_with("G03")), "{}", out.program);
    let mut comp = head("Setup1", &[]);
    comp.extend([
        Record::Comp { side: Some(CompSide::Right), radius_mm: 0.165, wear: false },
        mv(MoveKind::Feed, -25.0, 10.0),
        mv(MoveKind::Feed, -25.0, -10.0),
        Record::Comp { side: None, radius_mm: 0.0, wear: false },
        mv(MoveKind::Feed, -22.0, 10.0),
        Record::Section { begin: false },
    ]);
    let out = run(&comp, &height(5.0));
    assert!(out.warnings.iter().any(|w| w.contains("Punch1") && w.contains("wire offset") && w.contains("taper")), "{:?}", out.warnings);
    assert!(!words(&out).iter().any(|l| l.starts_with("G41") || l.starts_with("G42") || l.starts_with("G74")), "{}", out.program);
}

/// The section must stand the wire upright before G75: a cut that ends
/// leaning is refused by name.
#[test]
fn a_taper_that_ends_leaning_is_refused_by_name() {
    let mut r = head("Setup1", &[]);
    r.extend([param("WireU", json!(5.0)), param("WireV", json!(0.0)), mv(MoveKind::Feed, -25.0, 10.0), Record::Section { begin: false }]);
    let out = run(&r, &height(5.0));
    assert!(out.warnings.iter().any(|w| w.contains("Punch1") && w.contains("ends with the wire leaning")), "{:?}", out.warnings);
}
