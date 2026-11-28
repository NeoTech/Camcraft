//! The `fluidnc.cps` script suite (#3192): the post binary on every recorded
//! edge-case stream (`recorded/streams/*.cnc`), its program or its named
//! refusal compared with the golden in `recorded/golden/fluidnc/`, and the
//! FluidNC dialect pins on the 3040 router machine. Provenance: original
//! tests of our own stream and the FluidNC post contract (#3192).
//!
//! Regenerate after an intentional change, then read the diff:
//! `WRITE_GOLDEN=1 cargo test -p camcraft-post --test script_goldens script_fluidnc::`
//!
//! Gate map (M4.4): covers assets/posts/fluidnc.cps,
//! assets/machines/fluidnc-3040.mch, camcraft_post::post::engine,
//! camcraft_post::post::cnc, the post binary.

use crate::recorded;

post_suite!("fluidnc.cps");

use camcraft_post::machine::parse_mch;
use camcraft_post::post::{engine, machine};

fn read(rel: &str) -> String {
    std::fs::read_to_string(recorded::repo().join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

/// Two sections on the 3040: a plunge with tool 1, then a peck-drilled
/// hole with tool 2 (a dwell at the bottom), so the post writes a tool
/// change and a drill chain.
fn two_tools() -> String {
    let mut s = String::new();
    s.push_str("PARAM SpindleSpeed 12000\nTOOL 1 FLAT\nPARAM ClearancePlane 25\nPARAM RetractPlane 5\nSECTION\n");
    s.push_str("Rapid 20000 10000 2000\nPlunge 20000 10000 -1000 F300000\nRapid 20000 10000 5000\nENDSECTION\n");
    s.push_str("PARAM SpindleSpeed 10000\nTOOL 2 DRILL\nPARAM ClearancePlane 25\nPARAM RetractPlane 5\nSECTION\n");
    s.push_str("Rapid X 30000 Y 10000 Z 2000\n");
    s.push_str("CYCLE DRILL X 30000 Y 10000 Z -6000 R 2000 ENTRY 0 Q 0 P 500 F 120000 AT 4\n");
    s.push_str("Plunge X 30000 Y 10000 Z -6000 F 120000\nRapid X 30000 Y 10000 Z 5000\nENDSECTION\n");
    s
}

fn post_on_3040(stream: &str) -> engine::PostOutput {
    let mch = parse_mch(&read("assets/machines/fluidnc-3040.mch")).expect("the 3040 machine reads");
    engine::run(&read("assets/posts/fluidnc.cps"), &machine::mirror(&mch), stream).expect("the post runs")
}

/// The words of a line outside its comments, upper case.
fn words(line: &str) -> Vec<String> {
    let code: String = line.split('(').next().unwrap_or("").split(';').next().unwrap_or("").to_string();
    code.split_whitespace().map(|w| w.to_ascii_uppercase()).collect()
}

#[test]
fn fluidnc_header_says_not_verified_on_a_machine() {
    let out = post_on_3040(&two_tools());
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    let head: Vec<&str> = out.program.lines().take(3).collect();
    // The machine's model, its parentheses cleaned out of the comment.
    assert_eq!(head[0], "(3040 router FluidNC)", "{}", out.program);
    assert_eq!(head[1], "(FLUIDNC POST - NOT VERIFIED ON A MACHINE)", "{}", out.program);
    assert_eq!(head[2], "G21 G17 G90 G91.1 G94 G40 G80", "{}", out.program);
}

#[test]
fn fluidnc_tool_change_is_a_stop_without_m6() {
    let out = post_on_3040(&two_tools());
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    let p = &out.program;
    // The first tool is loaded before the start; the second is one M0 stop.
    assert!(p.contains("(LOAD T1 AND SET Z ZERO BEFORE THE START)"), "{p}");
    assert_eq!(p.lines().filter(|l| *l == "M0").count(), 1, "{p}");
    let lines: Vec<&str> = p.lines().collect();
    let note = lines.iter().position(|l| *l == "(TOOL CHANGE - LOAD T2 - SET Z ZERO - RESUME)").expect("the T2 note");
    assert_eq!(lines[note + 1], "M0", "{p}");
    // The spindle is stopped and Z is up before the stop.
    let m5 = lines[..note].iter().rposition(|l| *l == "M5").expect("M5 before the stop");
    assert!(lines[m5..note].iter().any(|l| l.starts_with("G0 Z")), "{p}");
    for l in &lines {
        let w = words(l);
        assert!(!w.iter().any(|w| w == "M6"), "no M6 on a machine with no tool changer: {l}");
        assert!(!w.iter().any(|w| w.starts_with('T')), "no T word: {l}");
        assert!(!w.iter().any(|w| w.starts_with('H') || w == "G43" || w == "G49"), "no tool length word: {l}");
    }
}

#[test]
fn fluidnc_drill_cycle_posts_as_plain_moves_and_a_g4_dwell() {
    let out = post_on_3040(&two_tools());
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    let p = &out.program;
    for l in p.lines() {
        let w = words(l);
        assert!(!w.iter().any(|w| w.starts_with("G8") && w != "G80"), "no canned cycle word: {l}");
        assert!(!w.iter().any(|w| w == "G98" || w == "G99"), "no cycle return word: {l}");
    }
    let lines: Vec<&str> = p.lines().collect();
    let bottom = lines.iter().position(|l| *l == "G1 X30 Y10 Z-6 F120").expect("the plunge to the bottom");
    assert_eq!(lines[bottom + 1], "G4 P0.5", "{p}");
}

#[test]
fn fluidnc_refuses_spindle_rpm_over_the_3040_maximum() {
    let out = post_on_3040(&two_tools().replace("SpindleSpeed 12000", "SpindleSpeed 30000"));
    assert!(out.warnings.iter().any(|w| w.contains("spindle rpm over machine maximum")), "{:?}", out.warnings);
}

#[test]
fn fluidnc_refuses_tapping_and_control_compensation_by_name() {
    let tap = "PARAM SpindleSpeed 500\nTOOL 3 TAP\nPARAM ClearancePlane 25\nPARAM RetractPlane 5\nSECTION\n\
        Rapid X 30000 Y 10000 Z 2000\nCYCLE TAP X 30000 Y 10000 Z -6000 R 2000 ENTRY 0 Q 0 P 0 F 750000 AT 1\n\
        Feed X 30000 Y 10000 Z -6000 F 750000\nFeed X 30000 Y 10000 Z 2000 F 750000\nENDSECTION\n";
    let out = post_on_3040(tap);
    assert!(out.warnings.iter().any(|w| w.contains("tapping") && w.contains("FluidNC")), "{:?}", out.warnings);
    let comp = "PARAM SpindleSpeed 12000\nTOOL 1 FLAT\nPARAM ClearancePlane 25\nPARAM RetractPlane 5\nSECTION\n\
        Rapid X 20000 Y 10000 Z 2000\nPlunge X 20000 Y 10000 Z -1000 F 300000\nCOMP LEFT 3000\n\
        Feed X 30000 Y 10000 Z -1000 F 300000\nENDSECTION\n";
    let out = post_on_3040(comp);
    assert!(out.warnings.iter().any(|w| w.contains("G41/G42") && w.contains("FluidNC")), "{:?}", out.warnings);
}

#[test]
fn fluidnc_lifts_in_the_work_offset_on_a_machine_with_no_safe_retract() {
    // The 3040 states no safe_retract: no homing is assumed, so the lift
    // is a work-offset G0 Z to the proven clearance, never G53 G0 Z0.
    let out = post_on_3040(&two_tools());
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    assert!(!out.program.contains("G53"), "{}", out.program);
    let lines: Vec<&str> = out.program.lines().collect();
    assert_eq!(lines[lines.len() - 2], "G0 Z25", "{}", out.program);
    assert_eq!(lines[lines.len() - 1], "M30", "{}", out.program);
}
