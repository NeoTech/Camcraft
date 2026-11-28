//! Installation data across the machine catalogue and saved document (#3181), from Camcraft's .mch grammar.
//! Gate map (M4.4): covers app::machine, app::posting, app::project.

use camcraft_app::{
    commands::{History, LoadMachine},
    machine::{
        self,
        library::{MachineEntry, MachineSource},
    },
    project,
    world::new_world,
};
use serde_json::{Value, json};

const NAME: &str = "Fehlmann Picomax 55 (iTNC 530)";
const PICOMAX: &str = include_str!("../../../assets/machines/fehlmann-picomax55.mch");

#[test]
fn machine_sheet_reports_known_options_and_distinguishes_an_unstated_list() {
    for (options, expected) in [
        (None, None),
        (Some(json!([])), Some("None")),
        (Some(json!(["A", "B"])), Some("A, B")),
    ] {
        let mut file: Value = serde_json::from_str(PICOMAX).unwrap();
        if let Some(options) = options {
            file["controller"]["identity"]["installed_options"] = options;
        }
        let entry = MachineEntry::from_text(NAME, MachineSource::Builtin(NAME), &file.to_string());
        assert!(entry.is_valid());
        let general = entry.groups.iter().find(|g| g.title == "General").unwrap();
        let row = |name| {
            general
                .rows
                .iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.as_str())
        };
        assert_eq!(row("Control"), Some("iTNC 530"));
        assert_eq!(row("NC software"), Some("340490-03"));
        assert_eq!(row("Installed control options"), expected);
        assert_eq!(
            row("Tool change"),
            Some("Manual at the physical upper change position")
        );
    }
}

#[test]
fn mounting_saving_and_reopening_the_picomax_preserves_its_installation() {
    let world = new_world();
    History::default().execute(&world, LoadMachine::builtin(&world, NAME).unwrap());
    let loaded = machine::of(&world);
    let m = loaded.1.as_ref().unwrap();
    assert_eq!(m.axes.len(), 3);
    let travel = |n: &str| m.axes.iter().find(|a| a.name == n).and_then(|a| a.travel);
    assert_eq!(
        (travel("X"), travel("Y"), travel("Z")),
        (Some((-500.0, 500.0)), Some((-255.0, 255.0)), Some((-147.0, 147.0)))
    );
    assert_eq!(
        (m.spindle_min, m.spindle_max, m.spindles),
        (Some(50.0), Some(9000.0), Some(1))
    );
    let saved = serde_json::to_value(project::collect(&world)).unwrap();
    assert_eq!(saved["version"], project::PROJECT_VERSION);
    assert!(
        saved["version"].as_u64().unwrap() >= 75,
        "a v74 reader must refuse this save"
    );
    let record = &saved["machine"]["machine"];
    assert_eq!(
        record["controller_identity"],
        json!({"model":"iTNC 530","nc_software":"340490-03"})
    );
    assert_eq!(record["tool_change"], "manual-retract-top");
    let reopened = new_world();
    project::open_noted(&reopened, &saved.to_string()).unwrap();
    assert_eq!(machine::of(&reopened), loaded);
    let mut malformed = saved;
    malformed["machine"]["machine"]["controller_identity"]["installed_options"] = json!(["a", "a"]);
    assert!(project::open_noted(&reopened, &malformed.to_string()).is_err());
    assert_eq!(machine::of(&reopened), loaded);
}

#[test]
fn a_v74_machine_opens_with_unstated_installation_facts_and_a_future_file_preserves_it() {
    let legacy = json!({
        "version": 74, "entities": [], "machine": {
            "path": "legacy.mch", "machine": {
                "model": "Legacy mill", "spindle_max": 6000.0,
                "axes": [{"name": "X", "travel": [-40.0, 40.0]}]
            }
        }
    });
    let world = new_world();
    project::open_noted(&world, &legacy.to_string()).expect("existing v74 machine");
    let loaded = machine::of(&world);
    assert_eq!(loaded.0, "legacy.mch");
    let mill = loaded.1.as_ref().unwrap();
    assert_eq!(mill.model, "Legacy mill");
    assert_eq!(mill.spindle_max, Some(6000.0));
    assert_eq!(mill.axes[0].travel, Some((-40.0, 40.0)));
    assert!(mill.controller_identity.is_none());
    assert!(mill.tool_change.is_none());

    let mut saved = serde_json::to_value(project::collect(&world)).unwrap();
    assert_eq!(saved["version"], project::PROJECT_VERSION);
    let record = &saved["machine"]["machine"];
    assert!(record.get("controller_identity").is_none());
    assert!(record.get("tool_change").is_none());

    let future = project::NEWEST_PROJECT_VERSION + 1;
    saved["version"] = json!(future);
    let error =
        project::open_noted(&world, &saved.to_string()).expect_err("too new for this reader");
    assert!(
        matches!(error, project::ProjectError::UnsupportedVersion(v) if v == future),
        "{error}"
    );
    assert_eq!(machine::of(&world), loaded);
}

#[test]
fn the_picomax_states_travel_and_rigid_tapping_and_no_parameter_file_name() {
    let entry = MachineEntry::from_text(NAME, MachineSource::Builtin(NAME), PICOMAX);
    assert!(entry.is_valid());
    let file: Value = serde_json::from_str(PICOMAX).unwrap();
    assert!(!PICOMAX.contains(".MP"), "the description names no parameter file");
    let head = &file["kinematics"]["default"]["parts"][1]["parts"][0]["spindle"];
    assert_eq!(head["rigid_tapping"], json!(true));
    assert_eq!(head["max_speed"], json!(9000));
    // A stated mount must be one the holder library knows; SF32 is not, so none is stated.
    assert!(file["tooling"].get("spindle_mount").is_none());
}

#[test]
fn a_face_wider_than_the_picomax_x_stroke_is_refused_naming_the_op_and_the_stroke() {
    use camcraft_app::{
        commands::{AddTool, CreateOperation, GenerateOperation, SetStock},
        geometry::{IDENTITY_Q, Point3, cube},
        machine::check_document,
        operation::{Height, OpParams, SrcRef},
        schema::Location,
        stock::StockShape,
        tools::catalog,
    };
    let m = camcraft_app::machine::parse_mch(PICOMAX).expect("the picomax file reads");
    let world = new_world();
    let mut history = History::default();
    let loc = Location { pos: Point3::new(0.0, 0.0, 0.0), rot: IDENTITY_Q };
    // 1100 mm wide: the path swings past X +/- 500 (the file's full stroke each way).
    let stock = StockShape::Box { dx: 1100.0, dy: 60.0, dz: 20.0 };
    history.execute(&world, SetStock::new(&world, stock, loc).expect("stock"));
    let tool = catalog().into_iter().find(|(n, _)| *n == "EM-0400-2FL").map(|(_, t)| t).expect("tool");
    history.execute(&world, AddTool::new(&world, "EM-0400-2FL", tool).expect("tool"));
    history.execute(
        &world,
        camcraft_app::commands::ImportModel::from_mesh(
            "part",
            cube(Point3::new(-550.0, -30.0, 0.0), Point3::new(550.0, 30.0, 18.0)),
        ),
    );
    let op = OpParams::Face {
        holes: Default::default(), tilt: None, blend_turns: false, blend_radius: None,
        stepover: 3.0, clearance_z: Height::fixed(25.0), cut_z: Height::fixed(18.0),
        top_z: None, stepdown: None, feed: 1200.0, plunge_feed: 300.0, climb: true,
        picks: Default::default(),
    };
    let cmd = CreateOperation::new(&world, "Face1", "EM-0400-2FL", SrcRef::Stock, op);
    history.execute(&world, cmd.expect("create face"));
    history.execute(&world, GenerateOperation::new(&world, "Face1").expect("generate face"));
    let c = check_document(&world, &m).expect("checks");
    assert!(
        c.problems.iter().any(|p| p.starts_with("Face1:") && p.contains("in X") && p.contains("the stroke is -500.000..500.000")),
        "{:?}", c.problems
    );
}
