//! #885: a feed block that turns a rotary is posted at an F the machine's
//! stated rates allow - no rotary past its stated deg/min, no linear part
//! past the fastest stated feed - read the way each control reads F on a
//! combined block. One prelude rule (`post::rotary_cap`), every dialect
//! that writes a rotary word on a feed: fanuc.cps and doosan-dnm.cps under
//! G43.4 (F is the tip's speed, the rotaries carried along), doosan-dnm.cps
//! without it (one composite length, degrees counted as mm), fanuc-lathe.cps
//! (composite, pinned by `c_axis_unwind`). tormach.cps writes A only on a
//! G0 index and refuses an A that changes inside an operation
//! (`tormach_fourth_axis`), so no feed of it turns a rotary.
//! Provenance: our own tests over our own posts; the feed readings from
//! the FANUC 0i-F / 30i-B operator's manuals (feedrate for rotation axes,
//! tool center point control), behaviour only.
//!
//! Gate map (M4.4): covers post::rotary_cap (the prelude's cap), app::post
//! (fanuc.cps, doosan-dnm.cps feed words on tilted blocks).
//! Lane shape (M4.4): all tests here run in well under a second.

use camcraft_app::post::engine;
use serde_json::{Value, json};
use std::path::Path;

fn read(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../").join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

const FEED_MAX: f64 = 5000.0;
const A_RATE: f64 = 1000.0;

/// A trunnion stating an A about X (TCP-capable) at `rate` deg/min, or no
/// rate when `None`.
fn trunnion(rate: Option<f64>) -> Value {
    let mut a = json!({ "letter": "A", "type": "rotary", "direction": [1.0, 0.0, 0.0], "travel": [-120.0, 120.0] });
    if let Some(r) = rate {
        a["speeds"] = json!({ "normal": r });
    }
    json!({ "name": "TEST TRUNNION", "feedRateMax": FEED_MAX, "kinematics": { "static": { "rotaries": [a] } } })
}

/// A 4th axis stating no direction (no TCP: `machine.rtcp` false), rated.
fn plain_fourth(rate: f64) -> Value {
    json!({ "name": "TEST 4TH", "feedRateMax": FEED_MAX, "kinematics": { "static": { "rotaries": [
        { "letter": "A", "type": "rotary", "speeds": { "normal": rate } } ] } } })
}

/// Short tip moves with large turns, a turn with the tip still, a long
/// move with a small turn, and a tilted arc that turns A.
const STREAM: &str = "\
TOOL 1 10MM BALL
SECTION
Rapid X 10000 Y 0 Z 5000 A 0 B 0 C 0
Feed X 10000 Y 0 Z 0 F 2000000 A 0 B 0 C 0
Feed X 10100 Y 0 Z 0 F 2000000 A 30000 B 0 C 0
Feed X 10100 Y 0 Z 0 F 2000000 A 60000 B 0 C 0
Feed X 40000 Y 0 Z 0 F 2000000 A 61000 B 0 C 0
Arc CCW X 40000 Y 1000 Z 0 F 2000000 I 40000 J 500 A 70000 B 0 C 0
Feed X 40000 Y 1000 Z 1000 F 2000000 A 69000 B 0 C 0
ENDSECTION
";

#[derive(Clone, Copy)]
enum Reading {
    /// F is the speed along the linear path; rotaries carried along.
    Path,
    /// F over sqrt(lin^2 + sum d^2), degrees counted as mm.
    Composite,
}

fn word(l: &str, k: char) -> Option<f64> {
    l.split(' ').find_map(|w| w.strip_prefix(k).and_then(|v| v.parse::<f64>().ok()))
}

/// Reads every posted motion block back, the axes modal, and checks each
/// feed block's rates under `reading` against what the machine states.
/// Returns how many blocks turned A under a feed.
fn assert_within(program: &str, reading: Reading, a_rate: f64) -> usize {
    let mut at = [0.0_f64; 4]; // X Y Z A
    let mut turned = 0;
    for l in program.lines() {
        let g = l.split(' ').find(|w| matches!(*w, "G00" | "G01" | "G02" | "G03"));
        let Some(g) = g else { continue };
        let next = [
            word(l, 'X').unwrap_or(at[0]),
            word(l, 'Y').unwrap_or(at[1]),
            word(l, 'Z').unwrap_or(at[2]),
            word(l, 'A').unwrap_or(at[3]),
        ];
        let d = [next[0] - at[0], next[1] - at[1], next[2] - at[2], next[3] - at[3]];
        if g != "G00" && d[3].abs() > 1e-9 {
            let f = word(l, 'F').unwrap_or_else(|| panic!("feed block without F: {l}"));
            // An arc is judged over its chord in the plane (XY here): at
            // most the length any reading of its F is measured along.
            let lin = if g == "G01" { (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() } else { (d[0] * d[0] + d[1] * d[1]).sqrt() };
            let (lin_rate, a) = match (reading, g) {
                (Reading::Composite, "G01") => {
                    let lcmd = (lin * lin + d[3] * d[3]).sqrt();
                    (f * lin / lcmd, f * d[3].abs() / lcmd)
                }
                _ if lin > 1e-9 => (f, f * d[3].abs() / lin),
                _ => (0.0, f),
            };
            assert!(lin_rate <= FEED_MAX * 1.0005, "linear past {FEED_MAX}: {l}");
            assert!(a <= a_rate * 1.0005, "A at {a:.1} deg/min past {a_rate}: {l}");
            turned += 1;
        }
        at = next;
    }
    turned
}

/// The 4th-axis brake words a Doosan post asks for (test-bench values).
fn brake() -> Value {
    json!({ "rotaryClamp": "M10", "rotaryUnclamp": "M11" })
}

/// Post `cps` (the Doosan one with the brake words stated).
fn post(cps: &str, mch: &Value, stream: &str) -> engine::PostOutput {
    let opts = if cps.contains("doosan") { brake() } else { json!({}) };
    engine::run_with(&read(cps), mch, stream, &opts).expect("post")
}

fn feeds(program: &str) -> Vec<f64> {
    program.lines().filter(|l| l.contains("G01") || l.contains("G02") || l.contains("G03")).filter_map(|l| word(l, 'F')).collect()
}

#[test]
fn fanuc_tcp_feed_holds_the_stated_a_rate() {
    let out = post("assets/posts/fanuc.cps", &trunnion(Some(A_RATE)), STREAM);
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    assert!(out.program.contains("G43.4"), "{}", out.program);
    assert_eq!(assert_within(&out.program, Reading::Path, A_RATE), 5, "{}", out.program);
    // 0.1 mm of tip for 30 degrees: 1000 * 0.1 / 30 = 3.33 -> F3.3, floored.
    assert!(out.program.contains("G01 X10.1 Y0 Z0 A30 F3.3"), "{}", out.program);
    // The tip still while A turns: F is the A rate itself.
    assert!(out.program.contains("G01 X10.1 Y0 Z0 A60 F1000"), "{}", out.program);
    // A long move with a small turn is under the rate: the op's F stands.
    assert!(out.program.contains("G01 X40 Y0 Z0 A61 F2000"), "{}", out.program);
}

#[test]
fn doosan_tcp_feed_holds_the_stated_a_rate() {
    let out = post("assets/posts/doosan-dnm.cps", &trunnion(Some(A_RATE)), STREAM);
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    assert!(out.program.contains("G43.4"), "{}", out.program);
    assert_eq!(assert_within(&out.program, Reading::Path, A_RATE), 5, "{}", out.program);
    assert!(out.program.contains("A30 F3.3"), "{}", out.program);
}

#[test]
fn doosan_fourth_axis_composite_feed_holds_the_stated_a_rate() {
    let rate = 600.0;
    // #950: the opening rapid writes its A0 (the post never assumes the
    // table stands at A0), so the control's A is placed and every turn
    // after it is held at the stated rate.
    let placed = post("assets/posts/doosan-dnm.cps", &plain_fourth(rate), STREAM);
    assert!(placed.warnings.is_empty(), "{:?}", placed.warnings);
    assert!(placed.program.contains("G00 X10 Y0 Z5 A0"), "{}", placed.program);
    assert_eq!(assert_within(&placed.program, Reading::Composite, rate), 5, "{}", placed.program);
    // A program whose rapid writes an A word places the table first.
    let stream = STREAM.replace("Rapid X 10000 Y 0 Z 5000 A 0 ", "Rapid X 10000 Y 0 Z 5000 A 1000 ");
    let out = post("assets/posts/doosan-dnm.cps", &plain_fourth(rate), &stream);
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    assert!(!out.program.contains("G43.4"), "{}", out.program);
    // Six: the opening feed also turns A1 back to A0.
    assert_eq!(assert_within(&out.program, Reading::Composite, rate), 6, "{}", out.program);
    // Composite: lcmd = sqrt(0.1^2 + 30^2); A runs at F * 30 / lcmd <= 600.
    let lcmd = (0.1_f64 * 0.1 + 900.0).sqrt();
    let want = (rate * lcmd / 30.0 * 10.0).floor() / 10.0;
    assert!(out.program.contains(&format!("A30 F{want}")), "want F{want}: {}", out.program);
}

#[test]
fn an_unstated_rate_caps_nothing() {
    for cps in ["assets/posts/fanuc.cps", "assets/posts/doosan-dnm.cps"] {
        let out = post(cps, &trunnion(None), STREAM);
        assert!(out.warnings.is_empty(), "{cps}: {:?}", out.warnings);
        assert!(feeds(&out.program).iter().all(|&f| (f - 2000.0).abs() < 1e-9), "{cps}: {}", out.program);
    }
}

#[test]
fn a_feed_turning_from_an_unknown_angle_is_refused_by_name() {
    // The first tilted record is a feed: the post never placed A, so the
    // turn it commands - and its rate - is unknown.
    let stream = "\
TOOL 1 T1
SECTION
Feed X 10000 Y 0 Z 0 F 2000000 A 30000 B 0 C 0
ENDSECTION
";
    for cps in ["assets/posts/fanuc.cps", "assets/posts/doosan-dnm.cps"] {
        let out = post(cps, &trunnion(Some(A_RATE)), stream);
        assert_eq!(
            out.warnings,
            vec!["a feed that turns A from an angle the post does not know - the stated A rate cannot be held"],
            "{cps}"
        );
    }
}
