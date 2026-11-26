//! #4076: the control-side wire offset of the Sodick LN2W wire post
//! (`sodick-ln2w.cps`) on synthetic streams. Pins the owner's form
//! (`G41` / `G42 Hnnn` on its own line before the lead-in, H from the pass
//! number, `C001` with `H001`, a later pass on the condition that the
//! operation states, `G40` on the lead-out move) and each named refusal of
//! the shared wire-offset law (`wire_offset.rs`), and the header and section
//! notes of an offset program (`programOffsets`).
//! Provenance: synthetic streams from our CNC intermediate contract; the
//! expected words are the statements of the owner of a Sodick AQ600L (LN2W)
//! on #4076, 2026-11-25.
//! Gate map: covers assets/posts/sodick-ln2w.cps, camcraft_post::post::wire_offset,
//! camcraft_post::post::engine, camcraft_post::post::cnc.

use camcraft_post::cl::{ArcPlane, CompSide, MoveKind};
use camcraft_post::post::cnc::{self, MoveRec, Record};
use camcraft_post::post::engine;
use serde_json::{json, Value};

const SCRIPT: &str = include_str!("../../../../assets/posts/sodick-ln2w.cps");

fn mv(kind: MoveKind, x: f64, y: f64) -> Record {
    Record::Move(MoveRec { kind, x, y, z: 0.0, feed: (kind == MoveKind::Feed).then_some(2.0), center: None, ccw: false, plane: ArcPlane::Xy, abc: None })
}

fn feed(x: f64, y: f64) -> Record {
    mv(MoveKind::Feed, x, y)
}

/// A clockwise arc to `(x, y)` about `(cx, cy)`.
fn arc(x: f64, y: f64, cx: f64, cy: f64) -> Record {
    Record::Move(MoveRec { kind: MoveKind::Feed, x, y, z: 0.0, feed: Some(2.0), center: Some((cx, cy)), ccw: false, plane: ArcPlane::Xy, abc: None })
}

fn param(name: &str, value: Value) -> Record {
    Record::Param { name: name.into(), value }
}

fn comp(side: Option<CompSide>, radius_mm: f64, wear: bool) -> Record {
    Record::Comp { side, radius_mm, wear }
}

fn head(condition: &str) -> Vec<Record> {
    vec![
        param("OperationName", json!("Punch1")),
        param("WorkOffset", json!("G54")),
        param("EdmCondition", json!(condition)),
        Record::Section { begin: true },
        mv(MoveKind::Rapid, -22.0, 10.0),
    ]
}

/// One pass on the control's offset: a 3 mm straight lead-in, one side
/// of the contour, and a straight lead-out with the offset off.
fn pass(side: CompSide, radius: f64, wear: bool) -> Vec<Record> {
    vec![comp(Some(side), radius, wear), feed(-25.0, 10.0), feed(-25.0, -10.0), comp(None, 0.0, wear), feed(-22.0, 10.0)]
}

fn machine(extra: Value) -> Value {
    let mut m = json!({"kind": "wire_edm", "name": "Sodick AQ600L", "edm": {"awt": true, "tank": "submerged", "flushing": "high", "skim_flushing": "low"}});
    if let (Some(e), Some(x)) = (m["edm"].as_object_mut(), extra.as_object()) {
        e.extend(x.clone());
    }
    m
}

fn run(records: &[Record]) -> engine::PostOutput {
    run_on(json!({}), records)
}

fn run_on(extra: Value, records: &[Record]) -> engine::PostOutput {
    engine::run(SCRIPT, &machine(extra), &cnc::serialize(records)).expect("the synthetic stream runs")
}

/// A section named `name` on `C001` that states `WireSide left`.
fn sided(name: &str, body: Vec<Record>) -> Vec<Record> {
    let mut r = vec![param("OperationName", json!(name)), param("WireSide", json!("left"))];
    r.extend(head("C001").into_iter().skip(1));
    r.extend(body);
    r.push(Record::Section { begin: false });
    r
}

fn words(out: &engine::PostOutput) -> Vec<&str> {
    out.program.lines().filter(|l| !l.starts_with('(')).collect()
}

fn offset_word(out: &engine::PostOutput) -> bool {
    words(out).iter().any(|l| l.split(' ').any(|w| w == "G41" || w == "G42" || w == "G40" || w.starts_with('H')))
}

fn refused(out: &engine::PostOutput, needles: &[&str]) {
    assert!(
        out.warnings.iter().any(|w| w.contains("Punch1") && needles.iter().all(|n| w.contains(n))),
        "{needles:?} not in {:?}\n{}",
        out.warnings,
        out.program
    );
}

/// The owner's example on #4076, rough cut plus one skim: `C001` with
/// `H001`, the skim on the condition the operation states (`C007`, not a
/// computed `C002`) with `H002`, each `G42 Hnnn` on its own line before the
/// lead-in, and `G40` on each lead-out move. Control and wear compensation
/// write the same words.
#[test]
fn a_control_side_wire_offset_posts_in_the_owners_form() {
    for wear in [false, true] {
        let mut r = head("C001");
        r.extend(pass(CompSide::Right, 0.165, wear));
        r.push(param("EdmCondition", json!("C007")));
        r.extend(pass(CompSide::Right, 0.155, wear));
        r.push(Record::Section { begin: false });
        let out = run(&r);
        assert!(out.warnings.is_empty(), "wear {wear}: {:?}", out.warnings);
        assert_eq!(
            words(&out),
            [
                "G90", "G54", "G00 X-22. Y10. U0. V0.", "Z0.", "T91", "T84", "T94",
                "C001", "G42 H001", "G01 X-25. Y10.", "G01 X-25. Y-10.", "G40 G01 X-22. Y10.",
                "T85", "C007", "G42 H002", "G01 X-25. Y10.", "G01 X-25. Y-10.", "G40 G01 X-22. Y10.",
                "T90", "M02",
            ],
            "wear {wear}:\n{}",
            out.program
        );
        // The operator's note names the register, its condition and the stated value.
        assert!(out.program.contains("H001") && out.program.contains("0.165 MM"), "{}", out.program);
    }
    // The left side is G41.
    let mut r = head("C001");
    r.extend(pass(CompSide::Left, 0.165, false));
    r.push(Record::Section { begin: false });
    let out = run(&r);
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    assert!(words(&out).contains(&"G41 H001"), "{}", out.program);
}

/// The lead-in must be at least the offset radius long (the owner's rule,
/// as for a mill cutter): a shorter one is refused by name, and no offset
/// word or cut is written.
#[test]
fn a_lead_in_shorter_than_the_wire_radius_is_refused_by_name() {
    let mut r = head("C001");
    r.extend([comp(Some(CompSide::Right), 0.165, false), feed(-22.1, 10.0), feed(-22.1, -10.0), comp(None, 0.0, false), feed(-22.0, 10.0)]);
    r.push(Record::Section { begin: false });
    let out = run(&r);
    refused(&out, &["lead-in", "0.1 mm", "0.165 mm"]);
    assert!(!offset_word(&out) && !words(&out).iter().any(|l| l.starts_with("G01")), "{}", out.program);
    // Exactly the radius is long enough.
    let mut r = head("C001");
    r.extend([comp(Some(CompSide::Right), 0.165, false), feed(-22.165, 10.0), feed(-22.165, -10.0), comp(None, 0.0, false), feed(-22.0, 10.0)]);
    r.push(Record::Section { begin: false });
    let out = run(&r);
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
}

/// The lead-in and the lead-out are straight moves: an arc that would
/// engage or cancel the offset is refused by name.
#[test]
fn an_arc_lead_in_or_lead_out_is_refused_by_name() {
    let mut r = head("C001");
    r.extend([comp(Some(CompSide::Right), 0.165, false), feed(-25.0, 10.0), arc(-25.0, -10.0, -25.0, 0.0)]);
    r.extend([comp(None, 0.0, false), arc(-22.0, 10.0, -22.0, 0.0), Record::Section { begin: false }]);
    let out = run(&r);
    refused(&out, &["lead-out", "straight"]);
    assert!(!words(&out).iter().any(|l| l.contains("G40")), "{}", out.program);

    let mut r = head("C001");
    r.extend([feed(-23.0, 10.0), comp(Some(CompSide::Right), 0.165, false), arc(-25.0, 10.0, -24.0, 10.0)]);
    r.extend([comp(None, 0.0, false), feed(-22.0, 10.0), Record::Section { begin: false }]);
    let out = run(&r);
    refused(&out, &["lead-in", "straight"]);
    assert!(!offset_word(&out), "{}", out.program);
}

/// H001 always goes with C001 on this control: a first pass on another
/// condition is refused by name, never posted with a register of a
/// different condition.
#[test]
fn a_first_offset_pass_on_a_condition_other_than_c001_is_refused_by_name() {
    let mut r = head("C110");
    r.extend(pass(CompSide::Right, 0.165, false));
    r.push(Record::Section { begin: false });
    let out = run(&r);
    refused(&out, &["H001", "C001", "C110"]);
    assert!(!offset_word(&out), "{}", out.program);
}

/// A section that ends with the offset on would leave G41 / G42 modal for
/// the next cut: refused by name.
#[test]
fn a_section_that_ends_with_the_offset_on_is_refused_by_name() {
    let mut r = head("C001");
    r.extend([comp(Some(CompSide::Right), 0.165, false), feed(-25.0, 10.0), feed(-25.0, -10.0), Record::Section { begin: false }]);
    let out = run(&r);
    refused(&out, &["ends with the wire offset on", "G40"]);
}

/// The C000 approach option cuts the lead-in on C000, but H001 goes with
/// C001: the two together are refused by name.
#[test]
fn the_approach_condition_with_a_wire_offset_is_refused_by_name() {
    let mut r = head("C001");
    r.extend(pass(CompSide::Right, 0.165, false));
    r.push(Record::Section { begin: false });
    let out = engine::run_with(SCRIPT, &machine(json!({})), &cnc::serialize(&r), &json!({"approachCondition": true}));
    let out = out.expect("the synthetic stream runs");
    refused(&out, &["C000", "wire offset"]);
    assert!(!offset_word(&out), "{}", out.program);
}

/// The lead-out that cancels the offset gets the rule of the lead-in (PR
/// #4139 review, finding 1): on a move of no length, or one shorter than
/// the radius, the control puts the wire centre on the contour inside the
/// part. Each is refused by name and no `G40` block is written. A lead-out
/// of exactly the radius posts.
#[test]
fn a_lead_out_shorter_than_the_wire_radius_is_refused_by_name() {
    for (end, needles) in [((-25.0, -10.0), ["lead-out", "no length", ""]), ((-25.0, -10.01), ["lead-out", "0.01 mm", "0.165 mm"])] {
        let mut r = head("C001");
        r.extend([comp(Some(CompSide::Right), 0.165, false), feed(-25.0, 10.0), feed(-25.0, -10.0), comp(None, 0.0, false), feed(end.0, end.1)]);
        r.push(Record::Section { begin: false });
        let out = run(&r);
        refused(&out, &needles);
        assert!(!words(&out).iter().any(|l| l.contains("G40")), "{}", out.program);
        assert_eq!(words(&out).iter().filter(|l| l.starts_with("G01")).count(), 2, "{}", out.program);
    }
    let mut r = head("C001");
    r.extend([comp(Some(CompSide::Right), 0.165, false), feed(-25.0, 10.0), feed(-25.0, -10.0), comp(None, 0.0, false), feed(-25.0, -10.165)]);
    r.push(Record::Section { begin: false });
    let out = run(&r);
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    assert!(words(&out).contains(&"G40 G01 X-25. Y-10.165"), "{}", out.program);
}

/// The notes are true for an offset program (finding 2). The header does
/// not say "NO WIRE OFFSET", and a section on the control's offset does
/// not say that the kerf is in the path or that the kerf uses the row's
/// gap. A section without a `COMP` record keeps its wire-centre notes, and
/// a program without one keeps the old header line.
#[test]
fn the_notes_of_an_offset_program_state_the_offset() {
    let table = json!({"conditions": [{"id": "C001", "gap": 0.04, "notes": ""}]});
    let centre = vec![feed(-25.0, 10.0), feed(-22.0, 10.0)];
    const OLD: [&str; 3] = [
        "(WIRE-CENTRE PATH - NO WIRE OFFSET G41 G42 IS USED)",
        "(SIDE LEFT - KERF IN THE PATH)",
        "(CONDITION C001 KERF GAP 0.040 MM - COMPARE WITH THE CONTROL ROW)",
    ];
    // No COMP record: the three old lines, and no word of an H register.
    let out = run_on(table.clone(), &sided("Punch1", centre.clone()));
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    assert!(OLD.iter().all(|l| out.program.lines().any(|p| p == *l)), "{}", out.program);
    assert!(!out.program.contains("H REGISTER"), "{}", out.program);

    // A wire-centre section, then a section on the control's offset.
    let mut r = sided("Punch1", centre);
    r.extend(sided("Punch2", pass(CompSide::Right, 0.165, false)));
    let out = run_on(table.clone(), &r);
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    let (first, second) = out.program.split_once("(Punch2").expect("the second section's note");
    assert!(!first.contains("NO WIRE OFFSET") && first.contains("(WIRE OFFSET G41 G42 IS USED IN EACH SECTION THAT NAMES AN H REGISTER"), "{first}");
    assert!(OLD[1..].iter().all(|l| first.contains(l)), "{first}");
    assert!(!second.contains("KERF IN THE PATH") && !second.contains("COMPARE WITH THE CONTROL ROW"), "{second}");
    assert!(second.contains("(WIRE OFFSET G41 G42 IN THIS SECTION - THE PATH IS THE PART CONTOUR WITHOUT THE KERF)"), "{second}");
    assert!(second.contains("(SIDE LEFT - THE CONTROL WIRE OFFSET SETS THE KERF)"), "{second}");
    assert!(second.contains("(CONDITION C001 - THE MACHINE FILE GAP 0.040 MM IS NOT IN THE PATH - THE H REGISTER AT THE CONTROL SETS THE KERF)"), "{second}");

    // Wear: the path is the wire-centre path, so the kerf notes stay.
    let out = run_on(table, &sided("Punch1", pass(CompSide::Right, 0.01, true)));
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    assert!(!out.program.contains("NO WIRE OFFSET") && OLD[1..].iter().all(|l| out.program.contains(l)), "{}", out.program);
    assert!(out.program.contains("(WIRE OFFSET G41 G42 IN THIS SECTION - THE H REGISTER HOLDS THE WEAR DELTA ONLY)"), "{}", out.program);
}

/// A `COMP` record that changes the side while the offset is on is refused
/// by name (finding 4), and no word is written after it.
#[test]
fn a_change_of_side_while_the_offset_is_on_is_refused_by_name() {
    let mut r = head("C001");
    r.extend([comp(Some(CompSide::Right), 0.165, false), feed(-25.0, 10.0), comp(Some(CompSide::Left), 0.165, false)]);
    r.extend([feed(-25.0, -10.0), comp(None, 0.0, false), feed(-22.0, 10.0), Record::Section { begin: false }]);
    let out = run(&r);
    refused(&out, &["changes to left while it is on", "G40"]);
    let w = words(&out);
    assert_eq!(w.iter().filter(|l| l.starts_with("G01")).count(), 1, "{}", out.program);
    assert!(!w.iter().any(|l| l.contains("G41") || l.contains("G40")), "{}", out.program);
}

/// The H register is H001 to H999 (finding 4): pass 999 posts `H999`, and
/// pass 1000 is refused by name with no register word.
#[test]
fn a_pass_above_h999_is_refused_by_name() {
    let mut r = head("C001");
    for n in 0..1000 {
        if n > 0 {
            r.push(param("EdmCondition", json!("C007")));
        }
        r.extend(pass(CompSide::Right, 0.165, false));
    }
    r.push(Record::Section { begin: false });
    let out = run(&r);
    refused(&out, &["pass 1000", "H001 to H999"]);
    let w = words(&out);
    assert!(w.contains(&"G42 H999"), "no H999");
    assert_eq!(w.iter().filter(|l| l.starts_with("G42")).count(), 999);
    assert!(!w.iter().any(|l| l.contains("H000") || l.contains("H1000")));
}
