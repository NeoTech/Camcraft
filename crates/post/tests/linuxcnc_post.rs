//! Issue #4151: LinuxCNC post header clears G92 offsets and states G97.
//! Issue #4152: LinuxCNC active comment guard and probe tolerance preview check.
//!
//! Provenance: public LinuxCNC G-code reference (linuxcnc.org/docs)
//! for G92.1 (reset G92 offsets), G97 (constant spindle RPM mode),
//! active comment keywords and predefined named parameters (#<_task>);
//! no vendor materials consulted. Original clean-room tests.
//!
//! Gate map: covers assets/posts/linuxcnc.cps, camcraft_post::post::engine.

mod recorded;

use camcraft_post::cl::{Move, MoveKind, Toolpath};
use camcraft_post::geometry::Point3;
use camcraft_post::post::cnc::{self, MoveRec, Record};
use camcraft_post::post::{dump, engine, Dump};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const LINUXCNC: &str = include_str!("../../../assets/posts/linuxcnc.cps");
const MACHINE_JSON: &str = include_str!("../../../assets/machines/3axis-vmc.mch");

fn repo(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../").join(rel)
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

fn simple_stream() -> String {
    cnc::serialize(&[
        Record::Param {
            name: "OperationName".into(),
            value: json!("Face1"),
        },
        Record::Param {
            name: "WorkOffset".into(),
            value: json!(1),
        },
        Record::Param {
            name: "ClearancePlane".into(),
            value: json!(25.0),
        },
        Record::Param {
            name: "RetractPlane".into(),
            value: json!(10.0),
        },
        Record::Param {
            name: "SpindleSpeed".into(),
            value: json!(3000.0),
        },
        Record::Tool {
            number: 1,
            name: "10mm Endmill".into(),
        },
        Record::Section { begin: true },
        Record::Move(MoveRec {
            kind: MoveKind::Rapid,
            x: 0.0,
            y: 0.0,
            z: 10.0,
            feed: None,
            center: None,
            ccw: false,
            plane: Default::default(),
            abc: None,
        }),
        Record::Move(MoveRec {
            kind: MoveKind::Feed,
            x: 10.0,
            y: 0.0,
            z: -1.0,
            feed: Some(500.0),
            center: None,
            ccw: false,
            plane: Default::default(),
            abc: None,
        }),
        Record::Section { begin: false },
    ])
}

fn post_stream(stream: &str) -> String {
    let out = engine::run_with(LINUXCNC, &Value::Null, stream, &json!({ "retractMode": "g28" }))
        .expect("post");
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    out.program
}

#[test]
fn header_clears_g92_and_states_g97_by_default() {
    let cps = read("assets/posts/linuxcnc.cps");
    let mch = json!({ "name": "LinuxCNC Mill", "safeRetractZ": 50.0 });
    let stream = simple_stream();

    let out = engine::run(&cps, &mch, &stream).expect("post run");
    let nc = &out.program;

    // Must emit G92.1 and G97 in the header modal safety block
    let header_line = nc
        .lines()
        .find(|l| l.contains("G21") && l.contains("G17"))
        .expect("modal header block");

    assert!(
        header_line.contains("G92.1"),
        "header must clear G92 offsets: {header_line}"
    );
    assert!(
        header_line.contains("G97"),
        "header must state G97 constant RPM mode: {header_line}"
    );

    // G92.1 and G97 must appear before the first spindle start S3000 M3
    let s_pos = nc.find("S3000 M3").expect("spindle start block");
    let g92_pos = nc.find("G92.1").expect("G92.1 in nc");
    let g97_pos = nc.find("G97").expect("G97 in nc");

    assert!(g92_pos < s_pos, "G92.1 must precede spindle start");
    assert!(g97_pos < s_pos, "G97 must precede spindle start");
}

#[test]
fn header_omits_g92_1_when_clearg92_option_is_false() {
    let cps = read("assets/posts/linuxcnc.cps");
    let mch = json!({ "name": "LinuxCNC Mill", "safeRetractZ": 50.0 });
    let stream = simple_stream();
    let options = json!({ "clearG92": false });

    let out = engine::run_with(&cps, &mch, &stream, &options).expect("post run");
    let nc = &out.program;

    let header_line = nc
        .lines()
        .find(|l| l.contains("G21") && l.contains("G17"))
        .expect("modal header block");

    assert!(
        !header_line.contains("G92.1"),
        "G92.1 must be omitted when clearG92 is false: {header_line}"
    );
    assert!(
        header_line.contains("G97"),
        "G97 must still be emitted: {header_line}"
    );
}

#[test]
fn header_includes_g92_1_when_clearg92_option_is_true() {
    let cps = read("assets/posts/linuxcnc.cps");
    let mch = json!({ "name": "LinuxCNC Mill", "safeRetractZ": 50.0 });
    let stream = simple_stream();
    let options = json!({ "clearG92": true });

    let out = engine::run_with(&cps, &mch, &stream, &options).expect("post run");
    let nc = &out.program;

    let header_line = nc
        .lines()
        .find(|l| l.contains("G21") && l.contains("G17"))
        .expect("modal header block");

    assert!(
        header_line.contains("G92.1"),
        "G92.1 must be emitted when clearG92 is true: {header_line}"
    );
    assert!(
        header_line.contains("G97"),
        "G97 must be emitted: {header_line}"
    );
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
