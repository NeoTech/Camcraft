//! Issue #4231: Tormach lathe post active-comment guard.
//!
//! Provenance: public LinuxCNC G-code reference (linuxcnc.org/docs)
//! for active comment keywords (MSG, DEBUG, PRINT, LOG, PROBEOPEN, ABORT).
//! Original clean-room tests.
//!
//! Gate map: covers assets/posts/tormach-lathe.cps, camcraft_post::post::engine.

use camcraft_post::post::{engine, ProgramId};
use serde_json::{json, Value};

const CPS: &str = include_str!("../../../assets/posts/tormach-lathe.cps");
const MACHINE_JSON: &str = include_str!("../../../assets/machines/tormach-8l.mch");
const STREAM: &str = include_str!("recorded/streams/empty_section.cnc");

#[test]
fn active_comment_guard_protects_machine_name_on_lathe() {
    for (name, expected) in [
        ("MSG, 8L Lathe", "(- MSG, 8L Lathe)"),
        ("msg, lowercase", "(- msg, lowercase)"),
        ("DEBUG, test message", "(- DEBUG, test message)"),
        ("PRINT, output log", "(- PRINT, output log)"),
        ("ABORT, emergency stop", "(- ABORT, emergency stop)"),
        ("LOGOPEN filename.txt", "(- LOGOPEN filename.txt)"),
        ("Tormach 8L", "(Tormach 8L)"),
        ("CAMCRAFT", "(CAMCRAFT)"),
    ] {
        let mut mch: Value = serde_json::from_str(MACHINE_JSON).expect("mch json");
        mch["name"] = json!(name);
        let out = engine::run(CPS, &mch, STREAM).expect("runs");
        assert!(out.warnings.is_empty(), "{:?}", out.warnings);
        assert!(
            out.program.lines().any(|l| l == expected),
            "expected line {expected:?} for machine name {name:?}:\n{}",
            out.program
        );
    }
}

#[test]
fn active_comment_guard_protects_user_comment_on_lathe() {
    let mch: Value = serde_json::from_str(MACHINE_JSON).expect("mch json");
    let id = ProgramId {
        notes: vec!["MSG, operator pause".into(), "DEBUG, check tool offset".into()],
        ..Default::default()
    };
    let out = engine::run_program_with(CPS, &mch, STREAM, &json!(null), &id).expect("runs");
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    assert!(
        out.program.lines().any(|l| l == "(- MSG, operator pause)"),
        "expected guarded comment in:\n{}",
        out.program
    );
    assert!(
        out.program.lines().any(|l| l == "(- DEBUG, check tool offset)"),
        "expected guarded comment in:\n{}",
        out.program
    );
}
