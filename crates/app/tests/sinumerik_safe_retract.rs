//! The Siemens 840D post's safe Z retract (#503, #495's rule): `G53 Z0`
//! is "straight up" only where the machine builder put machine Z0 at the
//! top of travel - the control family's public documentation leaves that
//! to the builder. The machine file states its own height (the Z part's
//! `safe_retract`): a stated Z0 (every shipped mill file, the default VMC)
//! and no machine at all post byte-identical to before, a machine stating
//! another height posts that height, and a machine stating none is
//! refused by name - never a guess that could rapid the spindle down.
//! #4144: each G53 retract is a rapid (`G0`) and deselects the tool offset
//! (`D0`) in the same block, as the 840D sl Fundamentals program example
//! does; a later section with the same tool selects its D again.
//! #4143: the tool call selects the first cutting edge (`T12 D1 M6`), not
//! D = T, unless the machine file states `tooling.d_number: "tool-number"`.
//! Provenance: our own tests over our own post and machine files.
//!
//! Gate map (M4.4): covers post::post (sinumerik.cps safe retract),
//! post::machine (safeRetractZ, model, dNumber).

use camcraft_app::machine::parse_mch;
use camcraft_app::post::{engine, machine};
use serde_json::{json, Value};
use std::path::Path;

/// Two sections, the first asking for a home move after it: the post
/// writes its G53 retract in both places (the home ask and the end).
const STREAM: &str = "PARAM SpindleSpeed 2000\nPARAM AfterOperation home\nPARAM ClearancePlane 5\nTOOL 1 T1\nSECTION\nRapid X 0 Y 0 Z 5000\nFeed X 1000 Y 0 Z -1000 F 500000\nENDSECTION\n\
                      PARAM SpindleSpeed 2000\nPARAM ClearancePlane 5\nTOOL 2 T2\nSECTION\nRapid X 0 Y 0 Z 5000\nFeed X 1000 Y 0 Z -1000 F 500000\nENDSECTION\n";

fn read(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../").join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

fn cps() -> String {
    read("assets/posts/sinumerik.cps")
}

/// A machine file in the file form, its Z part carrying `z_extra` keys.
fn file_form(model: &str, z_extra: Value) -> String {
    let mut z = json!({ "id": "col", "name": "Z", "type": "linear", "travel": { "min": -300.0, "max": 0.0 } });
    if let (Some(o), Some(e)) = (z.as_object_mut(), z_extra.as_object()) {
        for (k, v) in e {
            o.insert(k.clone(), v.clone());
        }
    }
    json!({
        "general": { "model": model, "vendor": "Camcraft", "kind": "mill" },
        "kinematics": { "default": {
            "units": { "length": "mm", "angle": "degrees" },
            "parts": [{ "id": "base", "name": "table", "type": "table", "parts": [
                { "id": "x", "name": "X", "type": "linear", "travel": { "min": -300.0, "max": 300.0 } },
                { "id": "y", "name": "Y", "type": "linear", "travel": { "min": -200.0, "max": 200.0 } },
                z
            ] }]
        } }
    })
    .to_string()
}

fn mounted(text: &str) -> Value {
    machine::mirror(&parse_mch(text).unwrap_or_else(|e| panic!("parses: {e}")))
}

/// The pre-#503 retract lines: the home ask's pair and the end's lift.
fn assert_z0(p: &str) {
    assert!(p.contains(" G0 G53 Z0 D0\n") && p.contains(" G0 G53 X0 Y0\n"), "home move\n{p}");
    assert_eq!(p.lines().filter(|l| l.ends_with(" G0 G53 Z0 D0")).count(), 2, "home + end lift\n{p}");
    assert!(!p.contains("SAFE RETRACT"), "no header on Z0\n{p}");
}

/// Every shipped mill file stating Z0 posts exactly what the pre-#503
/// script wrote for it: the same mirror with neither `safeRetractZ` nor
/// a name (the no-machine path, which keeps the old Z0) is the same
/// program past the name line. The default VMC (the conformance goldens'
/// machine, which hold the full bytes) and no machine keep `G53 Z0`.
#[test]
fn a_stated_z0_and_no_machine_post_byte_identical() {
    let null = engine::run(&cps(), &Value::Null, STREAM).expect("posts");
    assert!(null.warnings.is_empty(), "{:?}", null.warnings);
    assert_z0(&null.program);
    let vmc: Value = serde_json::from_str(&read("assets/machines/3axis-vmc.mch")).expect("json");
    let out = engine::run(&cps(), &vmc, STREAM).expect("posts");
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    assert_z0(&out.program);
    let strip = |p: &str| p.lines().skip(1).collect::<Vec<_>>().join("\n");
    let mut checked = 0;
    for (name, text) in camcraft_app::posting::shipped_machines().expect("the shipped machines read") {
        let m = parse_mch(text).unwrap_or_else(|e| panic!("{name}: {e}"));
        if m.safe_retract_z != Some(0.0) {
            continue;
        }
        checked += 1;
        let with = mounted(text);
        let mut bare = with.clone();
        if let Some(o) = bare.as_object_mut() {
            assert!(o.remove("safeRetractZ").is_some(), "{name}: the mirror carries it");
            o.remove("name");
            o.remove("model");
        }
        let a = engine::run(&cps(), &with, STREAM).expect("posts");
        let b = engine::run(&cps(), &bare, STREAM).expect("posts");
        assert!(a.warnings.is_empty() && b.warnings.is_empty(), "{name}: {:?} {:?}", a.warnings, b.warnings);
        assert_z0(&a.program);
        assert_eq!(strip(&a.program), strip(&b.program), "{name}: byte-identical past the name");
    }
    assert!(checked >= 3, "the shipped Z0 machines were found ({checked})");
}

/// A machine stating another height posts that height in both places,
/// through the app's mirror and the CLI's raw file alike, and the header
/// says where the retract goes. An inch file's height arrives in mm (this
/// dialect writes mm).
#[test]
fn a_stated_safe_z_posts_that_z() {
    let text = file_form("BUILDER 840D VMC", json!({ "safe_retract": 290.0 }));
    for m in [mounted(&text), serde_json::from_str(&text).expect("raw door")] {
        let out = engine::run(&cps(), &m, STREAM).expect("posts");
        assert!(out.warnings.is_empty(), "{:?}", out.warnings);
        let p = &out.program;
        assert!(p.contains(" G0 G53 Z290 D0\n") && p.contains(" G0 G53 X0 Y0\n"), "home move\n{p}");
        assert_eq!(p.lines().filter(|l| l.ends_with(" G0 G53 Z290 D0")).count(), 2, "home + end\n{p}");
        assert!(!p.contains("Z0\n"), "no Z0 retract left\n{p}");
        assert!(p.contains("; SAFE RETRACT G53 Z290 - STATED BY THE MACHINE\n"), "{p}");
    }
    let inch = file_form("BUILDER 840D VMC", json!({ "safe_retract": 11.5 }))
        .replace("\"length\":\"mm\"", "\"length\":\"inch\"");
    let out = engine::run(&cps(), &mounted(&inch), STREAM).expect("posts");
    assert_eq!(out.program.lines().filter(|l| l.ends_with(" G0 G53 Z292.1 D0")).count(), 2, "{}", out.program);
}

/// A machine that states no safe Z is refused once, by name - the post
/// cannot know where the builder put machine Z0 - and no retract (nor
/// the XY half of the home move) is guessed. No name is exempt: unlike
/// Tormach's PathPilot, the family documents no "Z0 is the top" rule.
#[test]
fn a_machine_without_a_safe_z_is_refused_by_name() {
    for model in ["BUILDER 840D VMC", "Siemens 840D mill"] {
        let text = file_form(model, json!({}));
        for m in [mounted(&text), serde_json::from_str(&text).expect("raw door")] {
            let out = engine::run(&cps(), &m, STREAM).expect("runs");
            assert_eq!(out.warnings.len(), 1, "refused once: {:?}", out.warnings);
            assert!(
                out.warnings[0].contains(&format!("machine {model} does not state its safe Z retract"))
                    && out.warnings[0].contains("safe_retract"),
                "{:?}",
                out.warnings
            );
            assert!(!out.program.contains("G53"), "no guessed retract\n{}", out.program);
        }
    }
}

/// #4144: the program-end retract is `G0 G53 Z.. D0` right before `M2`,
/// even after a feed move (G53 is no motion word), and no G53 Z line in
/// the program lacks the `G0` or the `D0`.
#[test]
fn every_g53_retract_is_a_rapid_that_cancels_the_tool_offset() {
    let vmc: Value = serde_json::from_str(&read("assets/machines/3axis-vmc.mch")).expect("json");
    let out = engine::run(&cps(), &vmc, STREAM).expect("posts");
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    let lines: Vec<&str> = out.program.lines().collect();
    let end = lines.iter().position(|l| l.ends_with(" M2")).expect("M2");
    assert!(lines[end - 1].ends_with(" G0 G53 Z0 D0"), "program end retract\n{}", out.program);
    let retracts: Vec<&&str> = lines.iter().filter(|l| l.contains(" G53 Z")).collect();
    assert_eq!(retracts.len(), 2, "home + end\n{}", out.program);
    for l in retracts {
        let words: Vec<&str> = l.split_whitespace().collect();
        assert_eq!(words.get(1), Some(&"G0"), "a rapid: {l}");
        assert_eq!(words.last(), Some(&"D0"), "the tool offset off: {l}");
    }
}

/// #4144: after a home retract wrote D0, a section that keeps the same
/// tool (no tool change) selects its D again before its first move.
#[test]
fn a_section_after_a_retract_with_the_same_tool_selects_the_d_again() {
    let stream = "PARAM SpindleSpeed 2000\nPARAM AfterOperation home\nPARAM ClearancePlane 5\nTOOL 12 T12\nSECTION\nRapid X 0 Y 0 Z 5000\nFeed X 1000 Y 0 Z -1000 F 500000\nENDSECTION\n\
                  PARAM SpindleSpeed 2000\nPARAM ClearancePlane 5\nSECTION\nRapid X 0 Y 0 Z 5000\nFeed X 1000 Y 0 Z -1000 F 500000\nENDSECTION\n";
    let out = engine::run(&cps(), &Value::Null, stream).expect("posts");
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    let lines: Vec<&str> = out.program.lines().collect();
    let home = lines.iter().position(|l| l.ends_with(" G0 G53 X0 Y0")).expect("home move");
    let again = lines[home..].iter().position(|l| l.split_whitespace().nth(1) == Some("D1")).map(|i| i + home);
    let first_move = lines[home + 1..].iter().position(|l| l.contains(" G0 ") || l.contains(" G1 ")).map(|i| i + home + 1);
    match (again, first_move) {
        (Some(d), Some(m)) => assert!(d < m, "D1 before the first move\n{}", out.program),
        _ => panic!("no D1 again after the home retract\n{}", out.program),
    }
}

/// #4143: the tool call selects the first cutting edge of the active tool.
/// Tool 12 is `T12 D1 M6` - never `D12` - with no machine, the default VMC
/// and a machine file that states `first-edge`.
#[test]
fn the_tool_call_selects_the_first_cutting_edge_by_default() {
    let stream = STREAM.replace("TOOL 1 T1", "TOOL 12 T12");
    let vmc: Value = serde_json::from_str(&read("assets/machines/3axis-vmc.mch")).expect("json");
    let first = file_form("BUILDER 840D VMC", json!({ "safe_retract": 0.0 }))
        .replace("\"general\":", "\"tooling\":{\"d_number\":\"first-edge\"},\"general\":");
    for m in [Value::Null, vmc, mounted(&first)] {
        let out = engine::run(&cps(), &m, &stream).expect("posts");
        assert!(out.warnings.is_empty(), "{:?}", out.warnings);
        let p = &out.program;
        assert!(p.contains(" T12 D1 M6\n") && p.contains(" T2 D1 M6\n"), "first edge\n{p}");
        assert!(!p.contains("D12") && !p.contains(" D2 "), "no D = T\n{p}");
    }
}

/// #4143: a machine file stating `tooling.d_number: "tool-number"` (a control
/// set to free D numbers, D equal to T) writes `T12 D12 M6`, through the
/// app's mirror and the CLI's raw file alike; any other value is refused
/// by name when the file is read.
#[test]
fn a_machine_stating_tool_number_d_writes_d_equal_to_t() {
    let stream = STREAM.replace("TOOL 1 T1", "TOOL 12 T12");
    let text = file_form("BUILDER 840D VMC", json!({ "safe_retract": 0.0 }))
        .replace("\"general\":", "\"tooling\":{\"d_number\":\"tool-number\"},\"general\":");
    for m in [mounted(&text), serde_json::from_str(&text).expect("raw door")] {
        let out = engine::run(&cps(), &m, &stream).expect("posts");
        assert!(out.warnings.is_empty(), "{:?}", out.warnings);
        assert!(out.program.contains(" T12 D12 M6\n") && out.program.contains(" T2 D2 M6\n"), "{}", out.program);
    }
    let bad = text.replace("tool-number", "edge-2");
    let err = parse_mch(&bad).expect_err("refused");
    assert!(err.contains("tooling.d_number"), "{err}");
}
