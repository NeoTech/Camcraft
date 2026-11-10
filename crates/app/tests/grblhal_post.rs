//! The grblHAL post (#710): the program shape a grbl-family controller
//! reads (no % fence, no O word, no N numbers, no G64, M30), no tool
//! length word at all (the controller's tool-change handling owns the
//! offset), `T<n> M6` alone on its line after a machine-coordinate Z
//! lift, dwell in seconds, the machine's stated safe Z (refused by name
//! when unstated), and the words grblHAL's core does not list - G84 -
//! refused by name. The generic machine file mounts and posts clean.
//! Provenance: our own tests over our own post and machine file.
//!
//! Gate map (#710): covers app::post (grblhal.cps), post::machine
//! (safeRetractZ, units), app::posting (registry, machine library).

use camcraft_app::machine::parse_mch;
use camcraft_app::post::{engine, machine};
use camcraft_app::posting::{BUILTINS, MACHINES};
use serde_json::{Value, json};
use std::path::Path;

const STREAM: &str = "PARAM SpindleSpeed 8000\nTOOL 3 EM6\nSECTION\nRapid X 0 Y 0 Z 5000\nFeed X 10000 Y 0 Z -1000 F 500000\nENDSECTION\n\
                      PARAM SpindleSpeed 6000\nTOOL 4 DRILL5\nSECTION\nCYCLE DRILL X 20000 Y 5000 Z -3000 R 2000 P 1500 F 100000\nENDSECTION\n";

fn read(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../").join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

fn cps() -> String {
    read("assets/posts/grblhal.cps")
}

/// The shipped generic machine, mounted the way the app hands it on.
fn generic() -> Value {
    let text = MACHINES
        .iter()
        .find(|(n, _)| *n == "grblHAL 3-axis")
        .map(|(_, t)| *t)
        .expect("the library ships the grblHAL machine");
    machine::mirror(&parse_mch(text).unwrap_or_else(|e| panic!("parses: {e}")))
}

fn post(m: &Value, stream: &str) -> String {
    let out = engine::run(&cps(), m, stream).unwrap_or_else(|e| panic!("posts: {e}"));
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    out.program
}

#[test]
fn the_registry_and_the_library_ship_it() {
    assert!(BUILTINS.iter().any(|(n, s)| *n == "grblHAL" && *s == cps()), "the post is built in");
    let m = generic();
    assert_eq!(m["safeRetractZ"], json!(0.0), "the generic file states grbl's Z0-at-top convention");
}

#[test]
fn the_program_has_grbl_shape_and_never_touches_the_length_offset() {
    let nc = post(&generic(), STREAM);
    let lines: Vec<&str> = nc.lines().collect();
    assert_eq!(lines.first(), Some(&"(grblHAL 3-axis)"), "{nc}");
    assert_eq!(lines.get(1), Some(&"G21 G17 G90 G91.1 G94 G40 G80"), "{nc}");
    assert_eq!(lines.last(), Some(&"M30"), "{nc}");
    for l in &lines {
        let first = l.split_whitespace().next().unwrap_or("");
        assert!(!first.starts_with('%') && !first.starts_with('O') && !first.starts_with('N'), "{l:?}");
        for w in l.split_whitespace() {
            assert!(!["G43", "G43.1", "G49", "G64", "G28", "G30", "G84"].contains(&w), "{w} in {l:?}\n{nc}");
            assert!(!w.starts_with('H'), "no length register: {l:?}");
        }
    }
}

#[test]
fn each_tool_change_lifts_z_then_stands_alone() {
    let nc = post(&generic(), STREAM);
    let lines: Vec<&str> = nc.lines().collect();
    for call in ["T3 M6", "T4 M6"] {
        let at = lines.iter().position(|l| *l == call).unwrap_or_else(|| panic!("no {call:?}\n{nc}"));
        assert_eq!(lines[at - 1], "G53 G0 Z0", "Z goes up first\n{nc}");
    }
    assert_eq!(nc.matches("G53 G0 Z0\n").count(), 3, "two changes and the end\n{nc}");
    // X Y move first after a change, at the safe height; Z comes down after.
    let t4 = lines.iter().position(|l| *l == "T4 M6").expect("T4");
    assert!(lines[t4 + 1..].iter().any(|l| *l == "G0 X20 Y5"), "{nc}");
}

#[test]
fn a_dwell_is_written_in_seconds() {
    let nc = post(&generic(), STREAM);
    assert!(nc.lines().any(|l| l == "G82 X20 Y5 Z-3 R2 P1.5 F100"), "1500 ms is P1.5\n{nc}");
}

#[test]
fn a_tap_is_refused_by_name() {
    let stream = read("crates/app/tests/golden/tap_plate.cl");
    let out = engine::run(&cps(), &generic(), &stream).expect("the script runs");
    let taps: Vec<&String> = out.warnings.iter().filter(|w| w.contains("G84")).collect();
    assert_eq!(taps.len(), 1, "one named refusal per program: {:?}", out.warnings);
}

#[test]
fn the_safe_z_is_the_machines_word_or_a_refusal() {
    // No machine at all keeps G53 Z0.
    assert!(post(&Value::Null, STREAM).contains("G53 G0 Z0\nT3 M6\n"));
    // A builder whose Z0 sits elsewhere states its height.
    let mut m = generic();
    m["safeRetractZ"] = json!(-5.0);
    let nc = post(&m, STREAM);
    assert_eq!(nc.matches("G53 G0 Z-5\n").count(), 3, "{nc}");
    assert!(!nc.contains("G53 G0 Z0"), "{nc}");
    // A named machine stating none is refused, never guessed.
    let mut m = generic();
    if let Some(o) = m.as_object_mut() {
        o.remove("safeRetractZ");
    }
    let out = engine::run(&cps(), &m, STREAM).expect("the script runs");
    assert!(
        out.warnings.iter().any(|w| w.contains("does not state its safe Z retract")),
        "{:?}",
        out.warnings
    );
    assert!(!out.program.contains("G53"), "nothing guessed:\n{}", out.program);
}

#[test]
fn an_inch_machine_posts_g20_and_inches() {
    let mut m = generic();
    m["units"] = json!("inch");
    let nc = post(&m, STREAM);
    assert!(nc.lines().nth(1).is_some_and(|l| l.starts_with("G20 ")), "{nc}");
    assert!(nc.contains("G1 X0.3937 Y0 Z-0.0394 F19.69"), "{nc}");
}
