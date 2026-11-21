//! Pins of #1882, the generic ISO abrasive-waterjet post
//! (`assets/posts/waterjet-iso.cps`): its program for a recorded plate
//! (one hole, one outer contour) as goldens, in mm with G2/G3 and in
//! inch with G1 chords and CR LF, and its refusals by name - spindle
//! words, a section that is not a waterjet cut, a machine that is not a
//! waterjet, a cut outside the stated travel, a Z cut, a canned cycle.
//!
//! Regenerate after an intentional change, then read the diff:
//! `WRITE_GOLDEN=1 cargo test -p camcraft-post --test waterjet_iso_post`
//!
//! Gate map (M4.4): covers assets/posts/waterjet-iso.cps,
//! camcraft_post::post::engine, camcraft_post::post::cnc.

use camcraft_post::post::engine;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn here() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/waterjet")
}

fn cps() -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/posts/waterjet-iso.cps")).expect("the waterjet post")
}

fn plate() -> String {
    std::fs::read_to_string(here().join("plate.cnc")).expect("the plate stream").replace("\r\n", "\n")
}

fn jet_machine() -> Value {
    json!({ "name": "Test waterjet", "kind": "waterjet" })
}

fn post(mch: &Value, stream: &str, options: &Value) -> engine::PostOutput {
    engine::run_with(&cps(), mch, stream, options).expect("the post runs")
}

fn golden(name: &str, text: &str) {
    let path = here().join(name);
    if std::env::var_os("WRITE_GOLDEN").is_some() {
        std::fs::write(&path, text).expect("write the golden");
        return;
    }
    let want = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read golden {}: {e}; create it with WRITE_GOLDEN=1", path.display()));
    assert_eq!(want.replace("\r\n", "\n"), text, "golden {} is stale; regenerate with WRITE_GOLDEN=1", path.display());
}

#[test]
fn waterjet_plate_posts_the_golden() {
    let out = post(&jet_machine(), &plate(), &json!({}));
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    assert!(!out.program.contains('\r'));
    golden("plate.nc", &out.program);
}

#[test]
fn waterjet_plate_inch_chords_crlf_posts_the_golden() {
    let options = json!({
        "Units": "inch", "arcs": "linear", "lineEnding": "crlf", "dwellWord": "p-milliseconds",
        "waterOn": 64, "waterOff": 65, "abrasiveOn": 66, "abrasiveOff": 67,
    });
    let out = post(&jet_machine(), &plate(), &options);
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    assert!(out.program.ends_with("\r\n") && out.program.matches('\n').count() == out.program.matches("\r\n").count());
    let lf = out.program.replace("\r\n", "\n");
    assert!(!lf.lines().any(|l| l.starts_with("G2 ") || l.starts_with("G3 ")), "no circular word under arcs = linear:\n{lf}");
    golden("plate_inch_chords.nc", &lf);
}

/// The counted warnings of one post.
fn refusals(mch: &Value, stream: &str) -> Vec<String> {
    post(mch, stream, &json!({})).warnings
}

fn assert_refused(w: &[String], why: &str) {
    assert!(w.iter().any(|w| w.contains(why)), "expected a refusal with {why:?}, got {w:?}");
}

#[test]
fn waterjet_post_refuses_spindle_words_by_name() {
    let rpm = plate().replace("PARAM OriginZ", "PARAM SpindleSpeed 1200.000000\nPARAM OriginZ");
    assert_refused(&refusals(&jet_machine(), &rpm), "operation Plate 100x60: states 1200 rpm - a waterjet turns no spindle (no S word, no G95");
    let css = plate().replace("PARAM OriginZ", "PARAM SurfaceSpeed 200.000000\nPARAM OriginZ");
    assert_refused(&refusals(&jet_machine(), &css), "constant surface speed - a waterjet turns no spindle (no G96)");
}

#[test]
fn waterjet_post_refuses_a_section_that_is_not_a_waterjet_cut() {
    let mill = plate()
        .replace("WATERJET ABRASIVE DWELL800 LAG300\n", "")
        .replace("JET ON C0 INNER\n", "")
        .replace("JET OFF C0\n", "")
        .replace("JET ON C1 OUTER\n", "")
        .replace("JET OFF C1\n", "")
        .replace("Waterjet Profile", "2D Contour");
    assert_refused(&refusals(&jet_machine(), &mill), "a 2D Contour operation is not waterjet cutting");
    let torch = plate().replace("WATERJET ABRASIVE DWELL800 LAG300", "PLASMA KERF1000");
    let w = refusals(&jet_machine(), &torch);
    assert_refused(&w, "does not define onTorchOn / onTorchOff");
    assert_refused(&w, "no WATERJET header");
}

#[test]
fn waterjet_post_refuses_a_machine_that_is_not_a_waterjet() {
    assert_refused(&refusals(&Value::Null, &plate()), "no machine is stated, so none is declared a waterjet machine");
    let mill = json!({ "name": "VMC", "kind": "mill" });
    assert_refused(&refusals(&mill, &plate()), "VMC is not declared a waterjet machine");
}

#[test]
fn waterjet_post_refuses_a_cut_outside_the_stated_travel() {
    let small = json!({
        "name": "Small jet", "kind": "waterjet",
        "kinematics": { "static": { "axes": [
            { "letter": "X", "direction": [1, 0, 0], "travel": [-10.0, 90.0] },
            { "letter": "Y", "direction": [0, 1, 0], "travel": [-10.0, 500.0] }
        ], "rotaries": [] } }
    });
    let w = refusals(&small, &plate());
    assert_refused(&w, "the cut runs X -5.5 to 100.5 - outside the machine's stated X travel -10 to 90");
    assert!(!w.iter().any(|w| w.contains("Y travel")), "{w:?}");
    let big = json!({
        "name": "Big jet", "kind": "waterjet",
        "kinematics": { "static": { "axes": [
            { "letter": "X", "direction": [1, 0, 0], "travel": [-10.0, 3000.0] },
            { "letter": "Y", "direction": [0, 1, 0], "travel": [-10.0, 1500.0] }
        ], "rotaries": [] } }
    });
    assert!(refusals(&big, &plate()).is_empty());
}

#[test]
fn waterjet_post_refuses_a_z_cut_and_a_cycle() {
    let z = plate().replace("Feed X 100500 Y 60500 Z 0", "Feed X 100500 Y 60500 Z -1000");
    assert_refused(&refusals(&jet_machine(), &z), "a cutting move changes Z - a waterjet program cuts in X and Y only");
    let dry = plate().replace("JET ON C1 OUTER\n", "").replace("JET OFF C1\n", "");
    assert_refused(&refusals(&jet_machine(), &dry), "a cutting move with the jet off");
    let cycle = plate().replace("Rapid X 0 Y 0 Z 0\n", "Rapid X 0 Y 0 Z 0\nCYCLE DRILL X 0 Y 0 Z -5000 R 2000\n");
    assert_refused(&refusals(&jet_machine(), &cycle), "a canned cycle - a waterjet has no drilling cycle");
}

#[test]
fn waterjet_post_refuses_a_helical_arc_in_both_arc_modes() {
    let machine = json!({
        "name": "Large jet", "kind": "waterjet",
        "kinematics": { "static": { "axes": [
            { "letter": "X", "direction": [1, 0, 0], "travel": [-2000.0, 2000.0] },
            { "letter": "Y", "direction": [0, 1, 0], "travel": [-2000.0, 2000.0] }
        ], "rotaries": [] } }
    });
    let stream = "PARAM StartX 1000\nWATERJET ABRASIVE\nSECTION\nRapid X 1000000 Y 0 Z 0\nJET ON C0 OUTER\nArc CCW X -1000000 Y 0 Z 100 F300000 I 0 J 0\nJET OFF C0\nENDSECTION\n";
    for mode in ["circular", "linear"] {
        let out = post(&machine, stream, &json!({ "arcs": mode }));
        assert_refused(&out.warnings, "a cutting move changes Z - a waterjet program cuts in X and Y only");
        assert!(
            !out.program.lines().any(|line| ["G1 ", "G2 ", "G3 "].iter().any(|word| line.starts_with(word))),
            "{mode}: the refused arc must not write a cut:\n{}",
            out.program
        );
    }
}

/// A positive feed keeps its value after formatting, or the post refuses it.
/// The stream states feed in um/min: F100 is 0.1 mm/min.
#[test]
fn waterjet_post_keeps_a_small_positive_feed_or_refuses_it() {
    let stream = |f: u32| format!("WATERJET\nSECTION\nRapid X 0 Y 0 Z 0\nJET ON C0 OUTER\nFeed X 1000 Y 0 Z 0 F{f}\nJET OFF C0\nENDSECTION\n");
    let mm = post(&jet_machine(), &stream(100), &json!({}));
    assert!(mm.warnings.is_empty(), "{:?}", mm.warnings);
    assert!(mm.program.contains("G1 X1 Y0 F0.1\n"), "metric keeps 0.1 mm/min:\n{}", mm.program);
    assert!(!mm.program.contains("F0\n") && !mm.program.contains("F0 "), "{}", mm.program);
    let tiny = post(&jet_machine(), &stream(0), &json!({}));
    assert_refused(&tiny.warnings, "states no feed");
    let thou = post(&jet_machine(), &stream(1), &json!({}));
    assert!(thou.program.contains("F0.001\n"), "metric keeps 0.001 mm/min:\n{}", thou.program);
    // 0.1 mm/min is 0.0039 in/min: below the inch format, so refused.
    let inch = post(&jet_machine(), &stream(100), &json!({ "Units": "inch" }));
    assert_refused(&inch.warnings, "below the program's feed resolution (F0)");
    assert!(!inch.program.contains("F0\n") && !inch.program.contains("F0 "), "{}", inch.program);
    // 254 mm/min is 10 in/min and posts as such.
    let ten = post(&jet_machine(), &stream(254_000), &json!({ "Units": "inch" }));
    assert!(ten.warnings.is_empty(), "{:?}", ten.warnings);
    assert!(ten.program.contains("F10\n"), "{}", ten.program);
}

#[test]
fn waterjet_rapid_cache_resets_when_the_coordinate_frame_changes() {
    // Each new frame needs a physical traverse, even when its words match
    // the last rapid. Exercise offset and each origin independently.
    for (parameter, value, x, y, word) in [
        ("WorkOffset", 2, 0, 0, "G55"),
        ("OriginX", 100, 100000, 0, "G54"),
        ("OriginY", 100, 0, 100000, "G54"),
    ] {
        let stream = format!(
            "PARAM WorkOffset 1\nWATERJET\nSECTION\nRapid X 0 Y 0 Z 0\nENDSECTION\nPARAM {parameter} {value}\nWATERJET\nSECTION\nRapid X {x} Y {y} Z 0\nJET ON C0 OUTER\nFeed X {} Y {y} Z 0 F300000\nJET OFF C0\nENDSECTION\n",
            x + 10000
        );
        let out = post(&jet_machine(), &stream, &json!({}));
        assert!(out.warnings.is_empty(), "{parameter}: {:?}", out.warnings);
        let frame = out
            .program
            .rfind(&format!("{word}\n"))
            .expect("second frame");
        let tail = &out.program[frame..];
        let rapid = tail
            .find("G0 X0 Y0\n")
            .expect("the new frame must rapid before opening the jet");
        let jet = tail.find("M10\n").expect("water on");
        assert!(rapid < jet, "{parameter}: {tail}");
        assert_eq!(
            out.program.matches("G0 X0 Y0\n").count(),
            2,
            "{parameter}: {}",
            out.program
        );
    }
}
