//! #531 (part of #290): the EDM condition table, user data in the
//! `.mch` `edm.conditions[]`. CamCraft reads only the geometry a path
//! needs (spark gap, wire diameter, undersize) and names each condition
//! by the control's own id; nothing is shipped (D4). The lookup an EDM
//! op calls at command creation refuses by name a missing condition or
//! a missing field; a garbled table refuses the file; the table rides
//! the project file.
//!
//! Gate map (#531): covers post::machine::edm (Condition, parse, lookup,
//! is_sane), app::project (Machine round trip).
//! Provenance: our own SYNTHETIC conditions below - invented numbers of
//! a plausible size, from no machine's manual.

use camcraft_app::machine::{parse_mch, ConditionField, Edm};

const WIRE: &str = include_str!("edm/wire-edm.mch");

fn with_conditions(conds: &str) -> String {
    let mut v: serde_json::Value = serde_json::from_str(WIRE).expect("template json");
    v["edm"]["conditions"] = serde_json::from_str(conds).expect("conditions json");
    v.to_string()
}

/// Synthetic: a rough, a skim and a finish, the finish stating no gap.
const TABLE: &str = r#"[
    { "id": "C110", "name": "rough", "gap": 0.04, "wire_diameter": 0.25, "role": "rough" },
    { "id": "C210", "gap": 0.02, "wire_diameter": 0.25, "role": "skim", "notes": "synthetic" },
    { "id": 330, "wire_diameter": 0.25, "role": "finish" }
]"#;

fn edm(text: &str) -> Edm {
    parse_mch(text).expect("parses").edm.expect("edm section")
}

#[test]
fn a_stated_table_reads_and_the_template_states_none() {
    assert!(edm(WIRE).conditions.is_empty(), "no vendor conditions ship");
    let e = edm(&with_conditions(TABLE));
    assert_eq!(e.conditions.len(), 3);
    let ids: Vec<&str> = e.conditions.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(ids, ["C110", "C210", "330"], "a numeric id reads as its word");
    assert_eq!(e.conditions[1].notes, "synthetic");
    assert!(e.is_sane());
}

#[test]
fn an_inch_file_states_its_gap_in_inches() {
    let mut v: serde_json::Value = serde_json::from_str(&with_conditions(
        r#"[ { "id": "R1", "gap": 0.001, "wire_diameter": 0.01 } ]"#,
    ))
    .expect("json");
    v["kinematics"]["default"]["units"]["length"] = "inch".into();
    let e = edm(&v.to_string());
    let c = &e.conditions[0];
    assert!((c.gap.expect("gap") - 0.0254).abs() < 1e-12 && (c.wire_diameter.expect("wire") - 0.254).abs() < 1e-12, "{c:?}");
}

#[test]
fn the_lookup_refuses_a_missing_condition_or_field_by_name() {
    let e = edm(&with_conditions(TABLE));
    let rough = e.condition("Wire 1", "C110", &[ConditionField::Gap, ConditionField::WireDiameter]).expect("complete");
    assert_eq!((rough.gap, rough.wire_diameter), (Some(0.04), Some(0.25)));
    let none = e.condition("Wire 1", "C999", &[ConditionField::Gap]).expect_err("missing");
    assert!(none.contains("Wire 1") && none.contains("C999"), "{none}");
    let gapless = e.condition("Wire 1", "330", &[ConditionField::Gap]).expect_err("no gap");
    assert!(gapless.contains("330") && gapless.contains("gap"), "{gapless}");
    let sinker = e.condition("Wire 1", "C110", &[ConditionField::Undersize]).expect_err("no undersize");
    assert!(sinker.contains("undersize"), "{sinker}");
    assert!(Edm::default().condition("Bare", "C110", &[]).is_err(), "an empty table states nothing");
}

#[test]
fn a_garbled_table_refuses_the_file() {
    for (conds, says) in [
        (r#"{ "id": "C1" }"#, "must be a list"),
        (r#"[ { "gap": 0.02 } ]"#, "states no id"),
        (r#"[ { "id": "C1" }, { "id": "C1" } ]"#, "stated twice"),
        (r#"[ { "id": "C1", "gap": -0.01 } ]"#, "gap must be positive"),
        (r#"[ { "id": "C1", "wire_diameter": 0 } ]"#, "wire_diameter must be positive"),
        (r#"[ { "id": "C1", "gap": "wide" } ]"#, "gap must be a number"),
        (r#"[ { "id": "C1", "role": "polish" } ]"#, "polish"),
        (r#"[ { "id": "C1", "name": 7 } ]"#, "name must be a word"),
    ] {
        let err = parse_mch(&with_conditions(conds)).expect_err(conds);
        assert!(err.contains(says), "{conds}: {err}");
    }
}

#[test]
fn a_hand_edited_project_machine_with_a_bad_table_is_not_sane() {
    let mut m = parse_mch(&with_conditions(TABLE)).expect("parses");
    assert!(m.is_sane());
    if let Some(e) = m.edm.as_mut() {
        e.conditions[1].id = "C110".into(); // a duplicate id
    }
    assert!(!m.is_sane(), "two conditions answering one id");
    let mut m = parse_mch(&with_conditions(TABLE)).expect("parses");
    if let Some(e) = m.edm.as_mut() {
        e.conditions[0].gap = Some(f64::NAN);
    }
    assert!(!m.is_sane(), "a NaN gap");
}

#[test]
fn the_table_rides_the_project_file() {
    use camcraft_app::commands::{History, LoadMachine};
    use camcraft_app::world::new_world;
    use camcraft_app::{machine, project};
    let world = new_world();
    let mut h = History::default();
    let m = parse_mch(&with_conditions(TABLE)).expect("parses");
    h.execute(&world, LoadMachine::new(&world, "wire.mch".into(), Some(m)).expect("mount"));
    let path = std::env::temp_dir().join(format!("camcraft-531-{}.ccam", std::process::id()));
    project::save(&world, &path).expect("save");
    let back = new_world();
    project::load(&back, &path).expect("load");
    let _ = std::fs::remove_file(&path);
    let (_, m) = machine::of(&back);
    let e = m.and_then(|m| m.edm).expect("edm survives");
    assert_eq!(e.conditions.len(), 3);
    assert_eq!(e.conditions[1].gap, Some(0.02));
    assert!(e.condition("back", "C210", &[ConditionField::Gap]).is_ok());
}
