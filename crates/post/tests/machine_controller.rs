//! Machine-file and script views of optional installation data (#3181), from Camcraft's .mch grammar.
//! Gate map (M4.4): covers post::machine, post::post.

use camcraft_post::{
    machine::parse_mch,
    post::{engine, machine},
};
use serde_json::{Value, json};

const PICOMAX: &str = include_str!("../../../assets/machines/fehlmann-picomax55.mch");
const FACTS: &str = r#"
manualToolChange = ["manual", "manual-retract-top"];
function onOpen() {
    writeln(JSON.stringify({
        control: machine.controllerIdentity, change: machine.toolChangeMode,
        axes: machine.linearAxes, count: machine.spindles,
        low: machine.spindleMin, high: machine.spindleMax, rtcp: machine.rtcp
    }));
}"#;

fn facts(input: &Value) -> Value {
    let output = engine::run(FACTS, input, "").expect("valid machine");
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    serde_json::from_str(&output.program).unwrap()
}

#[test]
fn installed_options_and_unrecognized_releases_survive_both_post_entry_points() {
    for options in [
        None,
        Some(json!([])),
        Some(json!(["feature-1", "feature-2"])),
    ] {
        let mut file: Value = serde_json::from_str(PICOMAX).unwrap();
        let mut identity = json!({"model":"iTNC 530","nc_software":"340490-04"});
        if let Some(options) = options {
            identity["installed_options"] = options;
        }
        file["controller"]["identity"] = identity.clone();
        file["controller"]["rtcp"] = true.into();
        let parsed = parse_mch(&file.to_string()).unwrap();
        let expected = json!({"control":identity,"change":"manual-retract-top",
            "axes":["X","Y","Z"],"count":1,"low":50,"high":9000,"rtcp":false});
        assert_eq!(facts(&file), expected);
        let mirrored = machine::mirror(&parsed);
        assert_eq!(mirrored["controller"]["rtcp"], true);
        assert_eq!(facts(&mirrored), expected);
    }
}

#[test]
fn additional_or_missing_linear_axes_remain_visible_to_scripts() {
    let mut file: Value = serde_json::from_str(PICOMAX).unwrap();
    let parts = file["kinematics"]["default"]["parts"]
        .as_array_mut()
        .unwrap();
    parts.push(json!({"id":"w","name":"W","type":"linear"}));
    for expected in [json!(["X", "Y", "Z", "W"]), json!(["X", "Y", "W"])] {
        let parsed = parse_mch(&file.to_string()).unwrap();
        assert_eq!(facts(&file)["axes"], expected);
        assert_eq!(facts(&machine::mirror(&parsed))["axes"], expected);
        file["kinematics"]["default"]["parts"]
            .as_array_mut()
            .unwrap()
            .retain(|part| part["id"] != "z");
    }
}

#[test]
fn unstated_installation_facts_stay_absent_and_tool_procedure_is_independent() {
    let mut file: Value = serde_json::from_str(PICOMAX).unwrap();
    file["controller"]
        .as_object_mut()
        .unwrap()
        .remove("identity");
    file.as_object_mut().unwrap().remove("tooling");
    for mode in [None, Some("manual"), Some("changer")] {
        if let Some(mode) = mode {
            file["tooling"]["tool_change"] = mode.into();
        }
        let parsed = parse_mch(&file.to_string()).unwrap();
        let machine_json = serde_json::to_value(&parsed).unwrap();
        assert!(machine_json.get("controller_identity").is_none());
        if mode.is_none() {
            assert!(machine_json.get("tool_change").is_none());
        }
        for input in [&file, &machine::mirror(&parsed)] {
            let props = facts(input);
            assert!(props.get("control").is_none());
            assert_eq!(props.get("change").and_then(Value::as_str), mode);
        }
    }
}

#[test]
fn malformed_declared_facts_refuse_at_the_machine_reader_and_raw_engine() {
    let bad = [
        ("identity", json!({"model":"iTNC 530"}), "nc_software"),
        ("identity", json!({"nc_software":"340490-03"}), "model"),
        (
            "identity",
            json!({"model":"iTNC 530","nc_software":" "}),
            "nc_software",
        ),
        ("installed_options", json!(42), "identity"),
        ("installed_options", json!(["a", "a"]), "installed_options"),
        (
            "installed_options",
            json!([" invalid "]),
            "installed_options",
        ),
        ("tool_change", json!("unknown-mode"), "tool_change"),
        ("max_tool_number", json!(0), "max_tool_number"),
        ("max_tool_number", json!(-1), "max_tool_number"),
        ("max_tool_number", json!(25.5), "max_tool_number"),
        ("max_tool_number", json!("254"), "max_tool_number"),
    ];
    for (field, value, cause) in bad {
        let mut file: Value = serde_json::from_str(PICOMAX).unwrap();
        match field {
            "identity" => file["controller"]["identity"] = value,
            "tool_change" | "max_tool_number" => file["tooling"][field] = value,
            _ => file["controller"]["identity"][field] = value,
        }
        let reader_error = parse_mch(&file.to_string()).unwrap_err();
        assert!(reader_error.contains(cause), "{reader_error}");
        let engine_error = engine::run(FACTS, &file, "").unwrap_err().to_string();
        assert!(engine_error.contains(cause), "{engine_error}");
    }
}

#[test]
fn a_static_post_override_cannot_disappear_when_the_engine_reads_a_machine_file() {
    let mut file: Value = serde_json::from_str(PICOMAX).unwrap();
    file["rigidTapping"] = false.into();
    let error = parse_mch(&file.to_string()).unwrap_err();
    assert!(error.contains("rigidTapping"), "{error}");
    let error = engine::run(FACTS, &file, "").unwrap_err().to_string();
    assert!(error.contains("rigidTapping"), "{error}");
}

/// #4087: the Picomax tool table has 254 lines (MP 7260 = 254). Both
/// Heidenhain posts refuse a higher tool number by name, through the raw
/// file and through the parsed mirror; tool 254 posts.
#[test]
fn heidenhain_posts_refuse_a_tool_number_above_the_machine_tool_table() {
    let file: Value = serde_json::from_str(PICOMAX).unwrap();
    assert_eq!(file["tooling"]["max_tool_number"], 254);
    let mirrored = machine::mirror(&parse_mch(PICOMAX).unwrap());
    let stream = include_str!("klartext/contour.cnc");
    for (script, tool_call) in [
        ("heidenhain.cps", "T254 G17 S3500"),
        ("heidenhain-klartext.cps", "TOOL CALL 254 Z S3500"),
    ] {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/posts/");
        let text = std::fs::read_to_string(format!("{path}{script}")).unwrap();
        for input in [&file, &mirrored] {
            let refusal = |tool: u32| match engine::run(
                &text,
                input,
                &stream.replace("TOOL 2 FLAT-6", &format!("TOOL {tool} FLAT-6")),
            ) {
                Ok(out) => out.warnings.join("\n"),
                Err(e) => e.to_string(),
            };
            let at_limit = engine::run(
                &text,
                input,
                &stream.replace("TOOL 2 FLAT-6", "TOOL 254 FLAT-6"),
            )
            .unwrap_or_else(|error| panic!("{script}: tool 254 must post: {error}"));
            assert!(
                at_limit.warnings.is_empty(),
                "{script}: {:?}",
                at_limit.warnings
            );
            assert!(
                at_limit.program.contains(tool_call),
                "{script}: missing {tool_call}: {}",
                at_limit.program
            );
            let over = refusal(255);
            assert!(
                over.contains("tool number 255") && over.contains("tool table"),
                "{script}: {over}"
            );
        }
    }
}
