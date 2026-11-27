//! Tests for Tormach PathPilot mill post (issue #4228).
//!
//! Verifies:
//! - G54.1 P<n> for extended work offsets (10..500) and standard G54..G59.3 (1..9).
//! - Reverse spindle (PARAM MillSpindle reverse) emits M4.
//! - Left-hand tap is refused by name (G74 unsupported).
//! - Right-hand programs remain unchanged (M3 and G84).
//! - Control cutter compensation (G41/G42 with D word and G40 cancel).
//!
//! Gate map (M4.4): covers assets/posts/tormach.cps, camcraft_post::post::engine.

use camcraft_post::machine::parse_mch;
use camcraft_post::post::{engine, machine};
use std::path::{Path, PathBuf};

fn repo(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../").join(rel)
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

fn mx770() -> serde_json::Value {
    let rel = "assets/machines/tormach-770mx.mch";
    machine::mirror(&parse_mch(&read(rel)).unwrap_or_else(|e| panic!("{rel}: {e}")))
}

fn post(stream: &str) -> engine::PostOutput {
    engine::run(&read("assets/posts/tormach.cps"), &mx770(), stream).expect("runs")
}

#[test]
fn tormach_extended_work_offsets_emit_g54_1_p() {
    let base_stream = |wcs: u32| -> String {
        format!(
            "PARAM WorkOffset {wcs}\n\
             PARAM OperationName SetupOffsetTest\n\
             PARAM ClearancePlane 25.0\n\
             PARAM RetractPlane 5.0\n\
             PARAM SpindleSpeed 1200\n\
             TOOL 1 \"10MM ENDMILL\"\n\
             SECTION\n\
             Rapid X 0 Y 0 Z 5000\n\
             Feed X 10000 Y 0 Z -2000 F 600000\n\
             ENDSECTION\n"
        )
    };

    // Standard offsets 1-6 -> G54..G59
    assert!(post(&base_stream(1)).program.contains("G54"));
    assert!(post(&base_stream(6)).program.contains("G59"));

    // Offsets 7-9 -> G59.1..G59.3
    assert!(post(&base_stream(7)).program.contains("G59.1"));
    assert!(post(&base_stream(8)).program.contains("G59.2"));
    assert!(post(&base_stream(9)).program.contains("G59.3"));

    // Extended offsets 10..500 -> G54.1 P<n>
    let out10 = post(&base_stream(10));
    assert!(out10.warnings.is_empty(), "{:?}", out10.warnings);
    assert!(out10.program.contains("G54.1 P10"), "expected G54.1 P10:\n{}", out10.program);

    let out100 = post(&base_stream(100));
    assert!(out100.warnings.is_empty(), "{:?}", out100.warnings);
    assert!(out100.program.contains("G54.1 P100"), "expected G54.1 P100:\n{}", out100.program);

    let out500 = post(&base_stream(500));
    assert!(out500.warnings.is_empty(), "{:?}", out500.warnings);
    assert!(out500.program.contains("G54.1 P500"), "expected G54.1 P500:\n{}", out500.program);

    // Offset 501 is above maximum -> refused
    let out501 = post(&base_stream(501));
    assert!(
        out501.warnings.iter().any(|w| w.contains("work offsets 1-500 only")),
        "offset 501 refused: {:?}",
        out501.warnings
    );
}

#[test]
fn tormach_reverse_mill_spindle_emits_m4() {
    let stream = "\
PARAM ClearancePlane 25.0
PARAM RetractPlane 5.0
PARAM OperationName OpReverse
PARAM SpindleSpeed 1500
PARAM MillSpindle reverse
TOOL 2 \"LEFT CUTTER\"
SECTION
Rapid X 0 Y 0 Z 5000
Feed X 10000 Y 0 Z -1000 F 300000
ENDSECTION
PARAM OperationName OpForward
PARAM ClearancePlane 25.0
PARAM RetractPlane 5.0
PARAM SpindleSpeed 3000
TOOL 3 \"RIGHT CUTTER\"
SECTION
Rapid X 0 Y 0 Z 5000
Feed X 10000 Y 0 Z -1000 F 500000
ENDSECTION
";
    let out = post(stream);
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    let spindle_lines: Vec<&str> = out.program.lines().filter(|l| l.contains('S') && (l.contains("M3") || l.contains("M4"))).collect();
    assert_eq!(spindle_lines, ["S1500 M4", "S3000 M3"], "program:\n{}", out.program);
}

#[test]
fn tormach_left_hand_tap_is_refused() {
    let stream = "\
PARAM ClearancePlane 25.0
PARAM RetractPlane 5.0
PARAM OperationName LeftTapOp
PARAM SpindleSpeed 600
PARAM MillSpindle reverse
TOOL 4 \"M6X1 LH TAP\"
SECTION
Rapid X 10000 Y 20000 Z 5000
CYCLE TAP X 10000 Y 20000 R 2000 Z -15000 F 600000
Rapid X 10000 Y 20000 Z 2000
Feed X 10000 Y 20000 Z -15000 F 600000
Feed X 10000 Y 20000 Z 2000 F 600000
ENDSECTION
";
    let out = post(stream);
    assert!(
        out.warnings.iter().any(|w| w.contains("left-hand rigid tapping (G74) is not supported")),
        "expected refusal warning:\n{:?}",
        out.warnings
    );
    assert!(!out.program.contains("G74"), "must not emit G74:\n{}", out.program);
    assert!(!out.program.contains("G84"), "must not emit G84:\n{}", out.program);
}

#[test]
fn tormach_right_hand_programs_remain_unchanged() {
    let stream = "\
PARAM ClearancePlane 25.0
PARAM RetractPlane 5.0
PARAM OperationName RightTapOp
PARAM SpindleSpeed 600
TOOL 5 \"M6X1 RH TAP\"
SECTION
Rapid X 10000 Y 20000 Z 5000
CYCLE TAP X 10000 Y 20000 R 2000 Z -15000 F 600000
Rapid X 10000 Y 20000 Z 2000
Feed X 10000 Y 20000 Z -15000 F 600000
Feed X 10000 Y 20000 Z 2000 F 600000
ENDSECTION
";
    let out = post(stream);
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    assert!(out.program.contains("G84"), "expected G84 right-hand tap cycle:\n{}", out.program);
    assert!(!out.program.contains("G74"), "must not contain G74:\n{}", out.program);
    assert!(out.program.contains("S600 M3"), "expected S600 M3:\n{}", out.program);
}

#[test]
fn tormach_control_cutter_compensation_g41_g42_and_g40() {
    let stream = "\
PARAM ClearancePlane 25.0
PARAM RetractPlane 5.0
PARAM OperationName CompOp
PARAM SpindleSpeed 5000
TOOL 6 \"1/4 EM\"
SECTION
Rapid X 0 Y 0 Z 5000
Plunge X 0 Y 0 Z -2000 F 300000
COMP LEFT 3175
Feed X 10000 Y 0 Z -2000 F 800000
Feed X 10000 Y 10000 Z -2000 F 800000
COMP OFF
Feed X 0 Y 10000 Z -2000 F 800000
COMP RIGHT 3175
Feed X 0 Y 0 Z -2000 F 800000
COMP OFF
Feed X -5000 Y 0 Z -2000 F 800000
ENDSECTION
";
    let out = post(stream);
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    assert!(out.program.contains("G41"), "expected G41 left comp:\n{}", out.program);
    assert!(out.program.contains("D6"), "expected D6 tool radius word:\n{}", out.program);
    assert!(out.program.contains("G42"), "expected G42 right comp:\n{}", out.program);
    assert!(out.program.contains("G40"), "expected G40 cancel:\n{}", out.program);
}
