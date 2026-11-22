//! #3466: an upright operation after a tilted operation states the
//! return of the rotaries. The tilted section ends with the tool still
//! tilted (its exit keeps the pose), and the upright section's moves
//! state no angles. The post must command the rotaries back to zero at
//! a safe height before the upright section's first X/Y traverse, or the
//! upright cut runs under the old lean.
//!
//! Provenance: RS-274 modal rotary words; FANUC tool centre point
//! control (G43.4 / G49) as the post scripts already implement it; our
//! own design notes only.
//!
//! Gate map: covers assets/posts/fanuc.cps, assets/posts/doosan-dnm.cps,
//! assets/posts/tormach.cps (the section opening after a tilted
//! section), post::dump.

use camcraft_post::cl::{Move, MoveKind, Toolpath};
use camcraft_post::geometry::Point3;
use camcraft_post::post::{self, cnc, engine, Dump};
use serde_json::json;
use std::path::Path;

fn read(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../").join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

fn tilthead() -> serde_json::Value {
    post::machine::mirror(&camcraft_post::machine::parse_mch(&read("assets/machines/tilthead.mch")).expect("tilthead"))
}

fn p(x: f64, y: f64, z: f64) -> Point3 {
    Point3 { x, y, z }
}

const UP: [f64; 3] = [0.0, 0.0, 1.0];

/// A 5-axis section: upright down to the part, one feed leaned 20
/// degrees about Y, then the exit rapid that keeps the lean.
fn tilted_path() -> Toolpath {
    let (s, c) = (20f64.to_radians().sin(), 20f64.to_radians().cos());
    let lean = [s, 0.0, c];
    Toolpath::new(vec![
        Move::new(MoveKind::Rapid, p(0.0, 0.0, 50.0), p(0.0, 0.0, 20.0), None).with_axis(UP),
        Move::new(MoveKind::Rapid, p(0.0, 0.0, 20.0), p(0.0, 0.0, 5.0), None).with_axis(UP),
        Move::new(MoveKind::Feed, p(0.0, 0.0, 5.0), p(0.0, 0.0, 0.0), Some(300.0)).with_axis(UP),
        Move::new(MoveKind::Feed, p(0.0, 0.0, 0.0), p(10.0, 0.0, 0.0), Some(500.0)).with_axis(lean),
        Move::new(MoveKind::Rapid, p(10.0, 0.0, 0.0), p(10.0, 0.0, 20.0), None).with_axis(lean),
    ])
}

/// A 3-axis section: no move states an axis.
fn upright_path() -> Toolpath {
    Toolpath::new(vec![
        Move::new(MoveKind::Rapid, p(10.0, 0.0, 20.0), p(30.0, 30.0, 20.0), None),
        Move::new(MoveKind::Rapid, p(30.0, 30.0, 20.0), p(30.0, 30.0, 5.0), None),
        Move::new(MoveKind::Feed, p(30.0, 30.0, 5.0), p(30.0, 30.0, -1.0), Some(300.0)),
        Move::new(MoveKind::Feed, p(30.0, 30.0, -1.0), p(40.0, 30.0, -1.0), Some(500.0)),
        Move::new(MoveKind::Rapid, p(40.0, 30.0, -1.0), p(40.0, 30.0, 20.0), None),
    ])
}

fn dump_of(path: &Toolpath, op: &str, tool: u32) -> Vec<cnc::Record> {
    let d = Dump {
        op_name: op,
        tool_number: tool,
        tool_name: if tool == 1 { "BN-0600" } else { "EM-0800" },
        tool_diameter: 6.0,
        rpm: 8000.0,
        clearance_z: 20.0,
        retract_z: 10.0,
        top_z: 0.0,
        wcs: 1,
        origin: [0.0; 3],
        stock: [-50.0, 50.0, -50.0, 50.0],
        stock_top_z: 0.0,
        ..Default::default()
    };
    post::dump(path, &d)
}

fn program(stem: &str, second_tool: u32) -> engine::PostOutput {
    let mut r = dump_of(&tilted_path(), "Lean1", 1);
    r.extend(dump_of(&upright_path(), "Flat2", second_tool));
    run(stem, &r)
}

/// The Doosan post asks for the rotary clamp words on a 5-axis machine.
fn run(stem: &str, r: &[cnc::Record]) -> engine::PostOutput {
    engine::run_with(&read(&format!("assets/posts/{stem}.cps")), &tilthead(), &cnc::serialize(r), &json!({ "rotaryBrake": true }))
        .unwrap_or_else(|e| panic!("{stem}: the stream posts or refuses by warning: {e}"))
}

fn word(line: &str, letter: char) -> Option<f64> {
    line.split_whitespace().find_map(|w| w.strip_prefix(letter).and_then(|v| v.parse::<f64>().ok()))
}

fn is_motion(l: &str) -> bool {
    l.split_whitespace().any(|w| matches!(w, "G00" | "G0" | "G01" | "G1" | "G02" | "G03"))
}

/// The upright section, from the last tilted block on: the rotaries are
/// commanded to zero, after a retract, before the first X/Y traverse.
fn assert_upright_return(stem: &str, second_tool: u32) {
    assert_returns(stem, &program(stem, second_tool), 'B', &format!("tool {second_tool}"));
}

/// `letter` is the rotary that the tilted section turns; `case` names the
/// program in the messages.
fn assert_returns(stem: &str, out: &engine::PostOutput, letter: char, case: &str) {
    let prog = &out.program;
    assert!(out.warnings.is_empty(), "{stem}: no refusal: {:?}\n{prog}", out.warnings);
    let lines: Vec<&str> = prog.lines().collect();
    let start = lines.iter().position(|l| l.contains("Flat2")).unwrap_or_else(|| panic!("{stem}: the upright section opens:\n{prog}"));
    assert!(
        lines[..start].iter().any(|l| word(l, letter).is_some_and(|v| v.abs() > 1.0)),
        "{stem}: the lean posts in the first section:\n{prog}"
    );
    let rest = &lines[start..];
    let cut = rest
        .iter()
        .position(|l| l.split_whitespace().any(|w| w == "G01" || w == "G1"))
        .unwrap_or_else(|| panic!("{stem}: the upright section cuts:\n{prog}"));
    let zero = rest[..cut]
        .iter()
        .position(|l| word(l, letter).is_some_and(|v| v.abs() < 1e-9))
        .unwrap_or_else(|| panic!("{stem} ({case}): the upright section commands {letter}0 before its first cut:\n{prog}"));
    let traverse = rest[..cut]
        .iter()
        .position(|l| is_motion(l) && (word(l, 'X').is_some() || word(l, 'Y').is_some()))
        .unwrap_or_else(|| panic!("{stem}: the upright section traverses X/Y:\n{prog}"));
    assert!(zero < traverse, "{stem} ({case}): the rotaries return before the first X/Y traverse:\n{prog}");
    let zl = rest[zero];
    assert!(word(zl, 'X').is_none() && word(zl, 'Y').is_none(), "{stem}: the return moves no X/Y: {zl}\n{prog}");
    assert!(
        rest[..zero].iter().any(|l| l.contains("G28") || l.contains("G53")),
        "{stem} ({case}): the tool retracts to a safe height before the rotaries turn:\n{prog}"
    );
}

#[test]
fn fanuc_states_upright_after_a_tilted_section_with_the_same_tool() {
    assert_upright_return("fanuc", 1);
}

#[test]
fn fanuc_states_upright_after_a_tilted_section_with_a_tool_change() {
    assert_upright_return("fanuc", 2);
}

#[test]
fn doosan_states_upright_after_a_tilted_section_with_the_same_tool() {
    assert_upright_return("doosan-dnm", 1);
}

#[test]
fn doosan_states_upright_after_a_tilted_section_with_a_tool_change() {
    assert_upright_return("doosan-dnm", 2);
}

/// Two tilted sections in a row. The second opens while `rotAt` still
/// holds the lean of the first. The return is kept on purpose (the
/// review decision of #3466): each section that follows a tilted
/// section starts upright at the reference height, even when it then
/// leans again. Skipping it would save one move and add a look-ahead
/// case to the scripts.
fn two_tilted_program(stem: &str) -> engine::PostOutput {
    let mut r = dump_of(&tilted_path(), "Lean1", 1);
    r.extend(dump_of(&tilted_path(), "Lean2", 1));
    run(stem, &r)
}

/// From the second section on: a retract, then the zero word with no
/// X/Y, then the first X/Y traverse; the zero precedes the section's
/// own lean word, and the section states its lean again before it cuts.
fn assert_return_between_two_tilted(stem: &str) {
    let out = two_tilted_program(stem);
    assert!(out.warnings.is_empty(), "{stem}: no refusal: {:?}\n{}", out.warnings, out.program);
    let prog = &out.program;
    let lines: Vec<&str> = prog.lines().collect();
    let start = lines.iter().position(|l| l.contains("Lean2")).unwrap_or_else(|| panic!("{stem}: the second section opens:\n{prog}"));
    let rest = &lines[start..];
    let cut = rest.iter().position(|l| l.split_whitespace().any(|w| w == "G01" || w == "G1")).unwrap_or_else(|| panic!("{stem}: the second section cuts:\n{prog}"));
    let zero = rest[..cut]
        .iter()
        .position(|l| word(l, 'B').is_some_and(|v| v.abs() < 1e-9))
        .unwrap_or_else(|| panic!("{stem}: the second section commands B0 before its first cut:\n{prog}"));
    let traverse = rest[..cut]
        .iter()
        .position(|l| is_motion(l) && (word(l, 'X').is_some() || word(l, 'Y').is_some()))
        .unwrap_or_else(|| panic!("{stem}: the second section traverses X/Y:\n{prog}"));
    assert!(zero < traverse, "{stem}: the rotaries return before the first X/Y traverse:\n{prog}");
    let zl = rest[zero];
    assert!(word(zl, 'X').is_none() && word(zl, 'Y').is_none(), "{stem}: the return moves no X/Y: {zl}\n{prog}");
    assert!(
        rest[..zero].iter().any(|l| l.contains("G28") || l.contains("G53")),
        "{stem}: the tool retracts to a safe height before the rotaries turn:\n{prog}"
    );
    assert!(
        rest.iter().any(|l| is_motion(l) && word(l, 'B').is_some_and(|v| (v - 20.0).abs() < 1e-9)),
        "{stem}: the second tilted section states its lean again:\n{prog}"
    );
}

#[test]
fn fanuc_returns_to_zero_between_two_tilted_sections() {
    assert_return_between_two_tilted("fanuc");
}

#[test]
fn doosan_returns_to_zero_between_two_tilted_sections() {
    assert_return_between_two_tilted("doosan-dnm");
}

/// Tormach PathPilot indexes an A about X (3+2): the whole first section
/// stands at one A angle. By the read in #3466 it already writes
/// `G53 G0 Z` and `G0 A0` at the next upright section; this pins it.
#[test]
fn tormach_states_upright_after_an_indexed_section() {
    let (s, c) = (30f64.to_radians().sin(), 30f64.to_radians().cos());
    let lean = [0.0, -s, c];
    let tilted = Toolpath::new(
        upright_path().moves.iter().map(|m| m.with_axis(lean)).collect(),
    );
    let m = post::machine::mirror(
        &camcraft_post::machine::parse_mch(&read("assets/machines/tormach-770mx-microarc4.mch")).expect("770mx A"),
    );
    for tool in [1, 2] {
        let mut r = dump_of(&tilted, "Lean1", 1);
        r.extend(dump_of(&upright_path(), "Flat2", tool));
        let out = engine::run(&read("assets/posts/tormach.cps"), &m, &cnc::serialize(&r)).expect("tormach posts");
        assert_returns("tormach", &out, 'A', &format!("tool {tool}"));
    }
}

#[test]
fn an_upright_program_after_an_upright_section_writes_no_return() {
    // Byte for byte as before: no lean, no return block.
    for stem in ["fanuc", "doosan-dnm"] {
        let mut r = dump_of(&upright_path(), "Flat1", 1);
        r.extend(dump_of(&upright_path(), "Flat2", 2));
        let out = run(stem, &r);
        assert!(out.warnings.is_empty(), "{stem}: {:?}", out.warnings);
        assert!(!out.program.lines().any(|l| word(l, 'A').is_some() || word(l, 'B').is_some()), "{stem}:\n{}", out.program);
    }
}
