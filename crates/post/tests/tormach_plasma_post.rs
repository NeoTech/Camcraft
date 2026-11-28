//! Issue #4231: Tormach plasma post active-comment guard.
//!
//! Provenance: public LinuxCNC G-code reference (linuxcnc.org/docs)
//! for active comment keywords (MSG, DEBUG, PRINT, LOG, PROBEOPEN, ABORT).
//! Original clean-room tests.
//!
//! Gate map: covers assets/posts/tormach-plasma.cps, camcraft_post::post::engine.

use camcraft_post::cl::MoveKind;
use camcraft_post::post::cnc_torch::PlasmaRec;
use camcraft_post::post::{cnc, engine, ProgramId};
use serde_json::{json, Value};

const CPS: &str = include_str!("../../../assets/posts/tormach-plasma.cps");
const MACHINE_JSON: &str = include_str!("../../../assets/machines/tormach-1300pl.mch");

fn plasma_stream() -> String {
    let rapid = |x, y| {
        cnc::Record::Move(cnc::MoveRec {
            kind: MoveKind::Rapid,
            x,
            y,
            z: 0.0,
            feed: None,
            center: None,
            ccw: false,
            plane: Default::default(),
            abc: None,
        })
    };
    let plasma_header = cnc::Record::Plasma(PlasmaRec {
        kerf: 1.5,
        control_comp: false,
        pierce_height: None,
        cut_height: None,
        pierce_delay: None,
        auto_feed: true,
        reversed: false,
    });
    cnc::serialize(&[
        cnc::Record::Param { name: "OperationName".into(), value: json!("Cut1") },
        cnc::Record::Param { name: "SetupName".into(), value: json!("Table") },
        cnc::Record::Param { name: "WorkOffset".into(), value: json!("G54") },
        cnc::Record::Tool { number: 1, name: "Torch".into() },
        plasma_header,
        cnc::Record::Section { begin: true },
        rapid(0.0, 5.0),
        rapid(10.0, 5.0),
        cnc::Record::Section { begin: false },
    ])
}

#[test]
fn active_comment_guard_protects_machine_name_on_plasma() {
    for (name, expected) in [
        ("MSG, 1300PL Plasma", "(- MSG, 1300PL Plasma)"),
        ("msg, lowercase", "(- msg, lowercase)"),
        ("DEBUG, test message", "(- DEBUG, test message)"),
        ("PRINT, output log", "(- PRINT, output log)"),
        ("ABORT, emergency stop", "(- ABORT, emergency stop)"),
        ("LOGOPEN filename.txt", "(- LOGOPEN filename.txt)"),
        ("Tormach 1300PL", "(Tormach 1300PL)"),
        ("CAMCRAFT", "(CAMCRAFT)"),
    ] {
        let mut mch: Value = serde_json::from_str(MACHINE_JSON).expect("mch json");
        mch["general"]["model"] = json!(name);
        mch["name"] = json!(name);
        let out = engine::run(CPS, &mch, &plasma_stream()).expect("runs");
        assert!(out.warnings.is_empty(), "{:?}", out.warnings);
        assert!(
            out.program.lines().any(|l| l == expected),
            "expected line {expected:?} for machine name {name:?}:\n{}",
            out.program
        );
    }
}

#[test]
fn active_comment_guard_protects_user_comment_on_plasma() {
    let mch: Value = serde_json::from_str(MACHINE_JSON).expect("mch json");
    let id = ProgramId {
        notes: vec!["MSG, operator torch inspection".into(), "DEBUG, test plasma feed".into()],
        ..Default::default()
    };
    let out = engine::run_program_with(CPS, &mch, &plasma_stream(), &json!(null), &id).expect("runs");
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    assert!(
        out.program.lines().any(|l| l == "(- MSG, operator torch inspection)"),
        "expected guarded comment in:\n{}",
        out.program
    );
    assert!(
        out.program.lines().any(|l| l == "(- DEBUG, test plasma feed)"),
        "expected guarded comment in:\n{}",
        out.program
    );
}
