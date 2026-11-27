//! Issue #4152: LinuxCNC active comment guard and probe tolerance preview check.
//!
//! Provenance: public LinuxCNC G-code reference (linuxcnc.org/docs)
//! for active comment keywords and predefined named parameters (#<_task>);
//! no vendor materials consulted. Original clean-room tests.
//!
//! Gate map: covers assets/posts/linuxcnc.cps, camcraft_post::post::engine.

mod recorded;

use camcraft_post::cl::{Move, MoveKind, Toolpath};
use camcraft_post::geometry::Point3;
use camcraft_post::post::{cnc, dump, engine, Dump};
use serde_json::{json, Value};

const LINUXCNC: &str = include_str!("../../../assets/posts/linuxcnc.cps");
const MACHINE_JSON: &str = include_str!("../../../assets/machines/3axis-vmc.mch");

fn post_stream(stream: &str) -> String {
    let out = engine::run_with(LINUXCNC, &Value::Null, stream, &json!({ "retractMode": "g28" }))
        .expect("post");
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    out.program
}

#[test]
fn probe_measure_check_guarded_by_task_parameter() {
    let prog = post_stream(&recorded::input("probe_t99_corner_g54.cnc"));
    assert!(
        prog.contains("o901 if [[#<_task> EQ 1] AND [ABS[#190] GT 0.5]]\n(MSG, PROBE X OUT OF TOLERANCE)\nM2\no901 endif"),
        "expected X measure check guarded by task in:\n{prog}"
    );
    assert!(
        prog.contains("o902 if [[#<_task> EQ 1] AND [ABS[#191] GT 0.5]]\n(MSG, PROBE Y OUT OF TOLERANCE)\nM2\no902 endif"),
        "expected Y measure check guarded by task in:\n{prog}"
    );
}

#[test]
fn active_comment_guard_catches_spaced_keywords_and_pyreload() {
    let p = |x, y, z| Point3 { x, y, z };
    let path = Toolpath::new(vec![
        Move::new(MoveKind::Rapid, p(0.0, 0.0, 10.0), p(5.0, 5.0, 10.0), None),
        Move::new(MoveKind::Plunge, p(5.0, 5.0, 10.0), p(5.0, 5.0, -1.0), Some(200.0)),
        Move::new(MoveKind::Feed, p(5.0, 5.0, -1.0), p(25.0, 5.0, -1.0), Some(800.0)),
        Move::new(MoveKind::Rapid, p(25.0, 5.0, -1.0), p(25.0, 5.0, 10.0), None),
    ]);

    for (name, expected_note) in [
        ("M S G", "(- M S G)"),
        ("m s g", "(- m s g)"),
        ("M  S  G, Mill", "(- M S G, Mill)"),
        ("MSG, Mill", "(- MSG, Mill)"),
        ("PYRELOAD", "(- PYRELOAD)"),
        ("P Y R E L O A D", "(- P Y R E L O A D)"),
        ("p y r e l o a d", "(- p y r e l o a d)"),
        ("D E B U G, test", "(- D E B U G, test)"),
        ("PROBEOPEN log.txt", "(- PROBEOPEN log.txt)"),
        ("P R O B E O P E N log.txt", "(- P R O B E O P E N log.txt)"),
        ("Message", "(Message)"),
        ("Python", "(Python)"),
    ] {
        let mut m: Value = serde_json::from_str(MACHINE_JSON).expect("machine json");
        m["name"] = json!(name);

        let d = Dump {
            setup_name: "Setup1",
            op_name: "Contour",
            op_kind: "2D Contour",
            tool_number: 1,
            tool_name: "EM",
            tool_kind: "flat",
            tool_diameter: 6.0,
            rpm: 5000.0,
            clearance_z: 10.0,
            retract_z: 5.0,
            ..Default::default()
        };
        let stream = cnc::serialize(&dump(&path, &d));
        let out = engine::run_with(LINUXCNC, &m, &stream, &json!({ "retractMode": "g28" })).expect("post");
        assert!(
            out.program.lines().any(|l| l == expected_note),
            "expected line {expected_note:?} in program for name {name:?}:\n{}",
            out.program
        );
    }
}
