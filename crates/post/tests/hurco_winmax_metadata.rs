//! Pin the original Hurco post metadata required by the post naming law, and
//! the Hurco post's opt-in declared options (#2573 tester feedback).
//! Provenance: project assets/posts/AGENTS.md naming requirements; Hurco
//! WinMax Mill NC Programming online help (Tool and Radius Offsets (G40-G49),
//! Program Functions M126/M127/M140) and the 2010 NC Programming manual
//! 704-0116-310 pages 5-13 and 5-19.
//! Gate map: covers assets/posts/hurco-winmax.cps, camcraft_post::post::engine.

use camcraft_post::machine::parse_mch;
use camcraft_post::post::{engine, machine};
use serde_json::{json, Value};

const SCRIPT: &str = include_str!("../../../assets/posts/hurco-winmax.cps");

fn vm10i() -> Value {
    let m = parse_mch(include_str!("../../../assets/machines/hurco-vm10i.mch")).expect("hurco-vm10i.mch parses");
    machine::mirror(&m)
}

/// Two operations, tool `tool` then tool `second`, the tester's DRILL1 shape.
fn two_ops(tool: u32, second: u32) -> String {
    format!(
        "; HSMW-CNC v1
PARAM ClearancePlane 6
PARAM RetractPlane 2
TOOL {tool} 6mm Drill
PARAM SpindleSpeed 1500
SECTION
Rapid X 20000 Y 10000 Z 6000
Feed X 20000 Y 10000 Z -2000 F 60000
Rapid X 20000 Y 10000 Z 6000
ENDSECTION
PARAM ClearancePlane 6
TOOL {second} 6mm Endmill
PARAM SpindleSpeed 6000
SECTION
Rapid X 0 Y 0 Z 6000
Feed X 0 Y 0 Z -1000 F 100000
ENDSECTION
"
    )
}

#[test]
fn hurco_post_vendor_is_camcraft() {
    assert!(SCRIPT.lines().any(|line| line.trim() == r#"vendor = "Camcraft";"#));
}

#[test]
fn hurco_default_writes_no_g43_h_and_no_opt_in_m_codes() {
    let out = engine::run(SCRIPT, &vm10i(), &two_ops(562, 4)).expect("default posts");
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    let p = &out.program;
    assert!(p.contains("T562 M6\n") && p.contains("T4 M6\n"), "tool changes:\n{p}");
    assert!(!p.contains("G43") && !p.contains(" H"), "default must write no G43 H:\n{p}");
    for m in ["M126", "M127", "M140"] {
        assert!(!p.contains(m), "default must write no {m}:\n{p}");
    }
}

#[test]
fn hurco_tool_length_offset_option_writes_g43_h_and_refuses_out_of_table() {
    let opt = json!({ "toolLengthOffset": true });
    let out = engine::run_with(SCRIPT, &vm10i(), &two_ops(109, 4), &opt).expect("option posts");
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    assert!(out.program.contains("T109 M6\nG43 H109\n"), "G43 H109:\n{}", out.program);
    assert!(out.program.contains("T4 M6\nG43 H4\n"), "G43 H4:\n{}", out.program);

    // The WinMax offset table holds H00..H200: H562 is refused, never written.
    let res = engine::run_with(SCRIPT, &vm10i(), &two_ops(562, 4), &opt);
    match res {
        Err(e) => assert!(e.to_string().contains("H01..H200"), "{e}"),
        Ok(o) => {
            assert!(o.warnings.iter().any(|w| w.contains("H01..H200")), "{:?}", o.warnings);
            assert!(!o.program.contains("H562"), "H562 written:\n{}", o.program);
        }
    }
}

#[test]
fn hurco_m140_and_m126_options_write_their_blocks() {
    let opt = json!({ "retractAlongToolVector": true, "shortestRotaryPath": true });
    let out = engine::run_with(SCRIPT, &vm10i(), &two_ops(1, 1), &opt).expect("options post");
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    let p = &out.program;
    assert_eq!(p.matches("G00 M140\n").count(), 2, "one M140 per operation:\n{p}");
    assert!(p.contains("\nM126\n") && p.contains("M127\nM30\n"), "M126 at start, M127 before M30:\n{p}");
    // Same tool, no tool change: after M140 the next operation states its Z before X/Y.
    assert_eq!(p.matches("T1 M6").count(), 1, "one tool change:\n{p}");
    let after = p.split("G00 M140\n").nth(1).expect("first M140");
    let z = after.find("G00 Z6.").expect("Z first");
    let xy = after.find("G00 X0. Y0.").expect("then X/Y");
    assert!(z < xy, "Z before X/Y after M140:\n{p}");
}
