//! Issue #4148: Mach3 sequence numbers and G64 path mode.
//!
//! Provenance: public Mach3 G-code and programming manual (machsupport.com);
//! no vendor materials consulted. Original clean-room tests.
//!
//! Gate map: covers assets/posts/mach3.cps, camcraft_post::post::engine.

use camcraft_post::post::engine;
use serde_json::json;
use std::path::{Path, PathBuf};

fn repo(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .join(rel)
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

fn mach3_post() -> String {
    read("assets/posts/mach3.cps")
}

fn default_vmc() -> serde_json::Value {
    serde_json::from_str(&read("assets/machines/3axis-vmc.mch")).expect("vmc machine json")
}

const SIMPLE_STREAM: &str = "\
PARAM WorkOffset 1
PARAM ClearancePlane 25
PARAM RetractPlane 5
TOOL 1 \"FLAT ENDMILL\"
PARAM ToolKind Flat Endmill
PARAM ToolDiameter 10
PARAM SpindleSpeed 8000
SECTION Simple
Rapid X 0 Y 0 Z 10
Feed X 10 Y 0 Z 0 F 500
Feed X 20 Y 0 Z 0 F 500
ENDSECTION
";

#[test]
fn mach3_header_writes_g64_path_mode() {
    let cps = mach3_post();
    let mch = default_vmc();
    let out = engine::run(&cps, &mch, SIMPLE_STREAM).expect("mach3 runs");
    assert!(out.warnings.is_empty(), "warnings: {:?}", out.warnings);
    let header_line = out
        .program
        .lines()
        .find(|l| l.contains("G90") && l.contains("G91.1"))
        .expect("modal header line");
    assert!(
        header_line.split_whitespace().any(|w| w == "G64"),
        "Mach3 header must include G64 path mode: {header_line}"
    );
}

#[test]
fn mach3_sequence_numbers_cap_and_refuse_over_99999() {
    let cps = mach3_post();
    let mch = default_vmc();

    // Within limit: N10, N20
    let ok_opts = json!({
        "showSequenceNumbers": "every-block",
        "sequenceNumberStart": 10,
        "sequenceNumberIncrement": 10
    });
    let out_ok = engine::run_with(&cps, &mch, SIMPLE_STREAM, &ok_opts).expect("posts");
    assert!(out_ok.warnings.is_empty(), "{:?}", out_ok.warnings);
    assert!(out_ok.program.contains("N10"), "{}", out_ok.program);
    assert!(out_ok.program.contains("N20"), "{}", out_ok.program);

    // Exceeds 99999 limit: refuses by name and does not emit 6-digit N word
    let over_opts = json!({
        "showSequenceNumbers": "every-block",
        "sequenceNumberStart": 99995,
        "sequenceNumberIncrement": 10
    });
    let out_over = engine::run_with(&cps, &mch, SIMPLE_STREAM, &over_opts).expect("posts");
    assert!(
        out_over.warnings.iter().any(|w| w.contains("exceeds Mach3 5-digit N99999 limit")),
        "must refuse sequence numbers exceeding 99999: {:?}",
        out_over.warnings
    );
    assert!(
        !out_over.program.lines().any(|l| l.starts_with("N100005")),
        "must not emit 6-digit sequence number: {}",
        out_over.program
    );
}
