//! #4341: a later section's preamble cancels the length offset (G49) and
//! changes the tool (T M6). The Mazak EIA/ISO manual says that G49 moves
//! the axis to remove the offset, so Z must first be at its safe retract
//! height. Mazak, Fanuc and Doosan retract Z by the `safeRetract` method
//! before that G49, once, and never on the first section.
//! Provenance: our synthetic streams; the posts' own `safeRetract` option.

use super::{machine, read, words};
use camcraft_post::cl::{MoveKind, PathBuilder};
use camcraft_post::geometry::Point3;
use camcraft_post::post::{AfterOperation, Dump, cnc, dump, engine};
use serde_json::{Value, json};

const POSTS: &[&str] = &["mazak", "fanuc", "doosan-dnm"];

/// Two operations on two tools (or the same tool twice), ending at Z25.
fn two_tools(second_tool: u32, after: AfterOperation) -> Vec<cnc::Record> {
    let mut b = PathBuilder::new(Point3::new(0.0, 0.0, 25.0));
    b.push(MoveKind::Rapid, Point3::new(10.0, 10.0, 25.0), None);
    b.push(MoveKind::Feed, Point3::new(10.0, 10.0, 10.0), Some(300.0));
    b.push(MoveKind::Rapid, Point3::new(10.0, 10.0, 25.0), None);
    let path = b.finish();
    let mut all = Vec::new();
    for (op, tool) in [("Cut1", 1), ("Cut2", second_tool)] {
        let d = Dump {
            op_name: op, tool_number: tool, tool_name: "EM6", tool_kind: "flat",
            tool_diameter: 6.0, rpm: 3000.0, clearance_z: 25.0, retract_z: 25.0,
            stock_top_z: 20.0, origin: [0.0, 0.0, 5.0],
            after: if op == "Cut1" { after } else { AfterOperation::default() },
            ..Default::default()
        };
        all.extend(dump(&path, &d));
    }
    all
}

fn lines(stem: &str, options: &Value, r: &[cnc::Record]) -> Vec<Vec<String>> {
    let out = engine::run_with(&read(&format!("assets/posts/{stem}.cps")), &machine(), &cnc::serialize(r), options)
        .expect("post runs");
    assert!(out.warnings.is_empty(), "{stem}: {:?}", out.warnings);
    out.program.lines().map(|l| words(l).into_iter().map(str::to_string).collect())
        .filter(|w: &Vec<String>| !w.is_empty()).collect()
}

fn has(line: &[String], expected: &[&str]) -> bool {
    expected.iter().all(|e| line.iter().any(|w| w == e))
}

/// The lines from the first section's spindle stop to the second preamble's G49.
fn change_gap(stem: &str, all: &[Vec<String>]) -> Vec<Vec<String>> {
    let g49: Vec<usize> = all.iter().enumerate().filter(|(_, w)| has(w, &["G49", "G80"])).map(|(i, _)| i).collect();
    assert_eq!(g49.len(), 2, "{stem}: one preamble G49 per section: {all:?}");
    let stop = all[..g49[1]].iter().rposition(|w| has(w, &["M5"])).expect("first section's spindle stop");
    all[stop..=g49[1]].to_vec()
}

fn retract_words(g53: bool) -> &'static [&'static str] {
    if g53 { &["G53", "G00", "Z150."] } else { &["G91", "G28", "Z0"] }
}

fn pin(stem: &str, g53: bool, second_tool: u32) {
    let options = if g53 { json!({ "safeRetract": "G53" }) } else { Value::Null };
    let all = lines(stem, &options, &two_tools(second_tool, AfterOperation::None));
    let gap = change_gap(stem, &all);
    let want = retract_words(g53);
    let lifts: Vec<usize> = gap.iter().enumerate().filter(|(_, w)| has(w, want)).map(|(i, _)| i).collect();
    assert_eq!(lifts.len(), 1, "{stem}: one Z retract before the second G49 and T M6: {gap:?}");
    let last = gap.len() - 1;
    assert!(lifts[0] < last, "{stem}: retract before G49: {gap:?}");
    assert!(has(&gap[last], &["G90"]), "{stem}: the G49 block restores absolute mode: {gap:?}");
    for w in &gap {
        assert!(!w.iter().any(|v| v.starts_with('X') || v.starts_with('Y')), "{stem}: Z only: {gap:?}");
    }
    let t = format!("T{}", if stem == "mazak" { second_tool.to_string() } else { format!("{second_tool:02}") });
    let g49 = all.iter().enumerate().filter(|(_, w)| has(w, &["G49", "G80"])).nth(1).expect("second G49").0;
    let m6 = all.iter().enumerate().skip(g49).find(|(_, w)| has(w, &[t.as_str(), "M6"])).expect("tool change").0;
    assert!(m6 > g49, "{stem}: T M6 after the retract and G49: {all:?}");
    // The first section has no retract before its preamble.
    let first = all.iter().position(|w| has(w, &["G49", "G80"])).expect("first G49");
    assert!(!all[..first].iter().any(|w| has(w, want)), "{stem}: no retract before the first tool: {all:?}");
}

#[test]
fn mazak_fanuc_doosan_retract_z_before_a_second_tool_g28() {
    for stem in POSTS { pin(stem, false, 2); }
}

#[test]
fn mazak_fanuc_doosan_retract_z_before_a_second_tool_g53() {
    for stem in POSTS { pin(stem, true, 2); }
}

#[test]
fn mazak_fanuc_doosan_retract_z_before_the_same_tool_called_again() {
    for stem in POSTS { pin(stem, false, 1); }
}

/// A Go home (or a setup-change stop) already lifted Z: no second retract.
#[test]
fn a_trip_home_is_not_followed_by_a_second_retract() {
    for stem in POSTS {
        let all = lines(stem, &Value::Null, &two_tools(2, AfterOperation::GoHome));
        let gap = change_gap(stem, &all);
        let n = gap.iter().filter(|w| has(w, &["G91", "G28", "Z0"])).count();
        assert_eq!(n, 1, "{stem}: one Z retract only: {gap:?}");
        assert!(gap.iter().any(|w| has(w, &["G28", "X0", "Y0"])), "{stem}: home ask still homes X Y: {gap:?}");
    }
}

#[test]
fn a_setup_change_stop_is_not_followed_by_a_second_retract() {
    for stem in POSTS {
        let mut r = two_tools(2, AfterOperation::ProgramStop);
        r.insert(0, cnc::Record::Param { name: "SetupChange".into(), value: json!("Flip B") });
        let all = lines(stem, &Value::Null, &r);
        let gap = change_gap(stem, &all);
        let n = gap.iter().filter(|w| has(w, &["G91", "G28", "Z0"])).count();
        assert_eq!(n, 1, "{stem}: one Z retract only: {gap:?}");
        let lift = gap.iter().position(|w| has(w, &["G91", "G28", "Z0"])).expect("retract");
        let stop = gap.iter().position(|w| has(w, &["M0"])).expect("stop");
        assert!(lift < stop, "{stem}: Z up before the operator stop: {gap:?}");
    }
}
