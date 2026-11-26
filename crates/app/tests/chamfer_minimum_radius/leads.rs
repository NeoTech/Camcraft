//! #3237 operation, project and stamp pins for 2D Chamfer leads.

use camcraft_app::machining2d::Lead;

use super::*;

#[test]
fn explicit_none_is_a_persisted_seed_but_not_the_legacy_marker() {
    let seed = camcraft_app::operation::seed_params(
        camcraft_app::operation::OpType::Chamfer2D.ordinal(),
    );
    assert!(matches!(
        &seed,
        OpParams::Chamfer2D {
            lead: Some(Lead::None),
            lead_out: None,
            ..
        }
    ));
    let explicit_json = project::params_to_json(&seed);
    assert_eq!(explicit_json["chamfer2_d"]["lead"], serde_json::json!("none"));
    assert!(explicit_json["chamfer2_d"]["lead_out"].is_null());

    let legacy = params(0.0);
    let legacy_json = project::params_to_json(&legacy);
    assert!(
        legacy_json["chamfer2_d"].get("lead").is_none(),
        "absence is the v82 compatibility marker: {legacy_json}"
    );
    assert_eq!(
        project::params_from_json(&explicit_json).expect("explicit lead DTO"),
        seed
    );
    assert_eq!(
        project::params_from_json(&legacy_json).expect("legacy lead DTO"),
        legacy
    );
}

#[test]
fn lead_markers_change_the_stamp_while_absent_fields_add_no_bytes() {
    let (world, _) = world(taper(6.0, 3.0));
    let legacy = params(0.0);
    let legacy_stamp = camcraft_app::operation::current_stamp(
        &world,
        TOOL,
        &SrcRef::Stock,
        &legacy,
        "LEAD",
        None,
    )
    .expect("legacy stamp");
    let legacy_round_trip = project::params_from_json(&project::params_to_json(&legacy))
        .expect("legacy DTO round trip");
    let round_trip_stamp = camcraft_app::operation::current_stamp(
        &world,
        TOOL,
        &SrcRef::Stock,
        &legacy_round_trip,
        "LEAD",
        None,
    )
    .expect("round-trip stamp");
    assert_eq!(
        legacy_stamp, round_trip_stamp,
        "the v82 absent marker writes no new digest byte"
    );
    let mut explicit = legacy.clone();
    if let OpParams::Chamfer2D { lead, .. } = &mut explicit {
        *lead = Some(Lead::None);
    }
    let explicit_stamp = camcraft_app::operation::current_stamp(
        &world,
        TOOL,
        &SrcRef::Stock,
        &explicit,
        "LEAD",
        None,
    )
    .expect("explicit stamp");
    assert_ne!(
        legacy_stamp, explicit_stamp,
        "explicit None changes motion semantics for open runs"
    );
    if let OpParams::Chamfer2D { lead_out, .. } = &mut explicit {
        *lead_out = Some(Lead::None);
    }
    let explicit_out_stamp = camcraft_app::operation::current_stamp(
        &world,
        TOOL,
        &SrcRef::Stock,
        &explicit,
        "LEAD",
        None,
    )
    .expect("explicit lead-out stamp");
    assert_ne!(explicit_stamp, explicit_out_stamp);
}

#[test]
fn v82_absent_lead_fields_keep_the_old_open_path_marker() {
    assert_eq!(project::PROJECT_VERSION, 83);
    let (source, mut history) = world(taper(6.0, 3.0));
    generate(&source, &mut history, "LEGACY", params(0.0));
    let file = std::env::temp_dir().join(format!(
        "camcraft-3237-v82-{}.camproj",
        std::process::id()
    ));
    project::save(&source, &file).expect("save legacy");
    let mut json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&file).expect("read")).expect("JSON");
    json["version"] = serde_json::json!(82);
    std::fs::write(&file, serde_json::to_vec_pretty(&json).expect("write"))
        .expect("write v82");

    let loaded = new_world();
    project::load(&loaded, &file).expect("migrate v82");
    let operation = model::operation_of(
        &loaded,
        &model::find_by_name(&loaded, "LEGACY").expect("legacy op"),
    )
    .expect("operation");
    assert!(matches!(
        operation.params,
        OpParams::Chamfer2D {
            lead: None,
            lead_out: None,
            ..
        }
    ));
    let _ = std::fs::remove_file(file);
}
