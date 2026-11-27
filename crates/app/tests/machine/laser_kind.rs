//! #4081 slice 1: the laser machine kind and the laser head at the app
//! doors - the editor's validator, the blank draft, the tool door, the
//! project round trip, the family gate and the program check. No laser
//! operation exists yet, so a laser machine refuses each operation by
//! name and no built-in post claims it.
//! Provenance: our own synthetic fixtures from the issue's rules.

use camcraft_app::commands::{AddTool, CreateOperation, History, SetStock};
use camcraft_app::geometry::{IDENTITY_Q, Point3};
use camcraft_app::machine::{self, Kind, Machine};
use camcraft_app::model;
use camcraft_app::operation::{Family, Height, OpParams, SrcRef};
use camcraft_app::project;
use camcraft_app::schema::Location;
use camcraft_app::stock::StockShape;
use camcraft_app::tools::{Tool, ToolKind};
use camcraft_app::world::new_world;
use camcraft_tools::laser::laser_tool;
use serde_json::{json, Value};

fn file() -> Value {
    json!({
        "general": {"model": "Laser test", "kind": "laser"},
        "kinematics": {"default": {"parts": [
            {"name": "X", "type": "linear", "travel": {"min": 0, "max": 400}},
            {"name": "Y", "type": "linear", "travel": {"min": 0, "max": 300}},
            {"name": "laser head", "type": "head"}
        ]}}
    })
}

fn laser() -> Machine {
    machine::parse_mch(&file().to_string()).expect("a valid laser")
}

fn face() -> OpParams {
    OpParams::Face { holes: Default::default(), tilt: None, blend_turns: false, blend_radius: None,
        stepover: 3.0, clearance_z: Height::fixed(25.0), cut_z: Height::fixed(18.0), top_z: None,
        stepdown: None, feed: 1200.0, plunge_feed: 300.0, climb: true, picks: Default::default() }
}

#[test]
fn the_laser_validator_names_the_spindle_the_coolant_and_a_missing_axis() {
    let keys = |v: &Value| -> Vec<String> {
        machine::validate::validate_text(&v.to_string()).into_iter().map(|r| r.key).collect()
    };
    assert!(keys(&file()).is_empty(), "{:?}", keys(&file()));
    let mut v = file();
    v["kinematics"]["default"]["parts"][2]["spindle"] = json!({"max_speed": 20000});
    v["kinematics"]["default"]["parts"][2]["tool_station"] = json!({"coolants": [{"name": "Flood", "mCode": "M8"}]});
    let k = keys(&v);
    assert!(k.iter().any(|k| k == "spindle.max_speed") && k.iter().any(|k| k == "coolants"), "{k:?}");
    let mut v = file();
    v["kinematics"]["default"]["parts"].as_array_mut().expect("parts").remove(1);
    assert!(keys(&v).iter().any(|k| k == "Y axis"), "{:?}", keys(&v));
}

#[test]
fn the_blank_laser_draft_is_valid_and_no_built_in_post_claims_it() {
    let text = machine::draft::MachineDraft::blank(Kind::Laser).text();
    assert!(machine::validate::validate_text(&text).is_empty(), "{text}");
    let m = machine::parse_mch(&text).expect("the blank draft parses");
    assert!(m.is_sane());
    assert_eq!(machine::kind(Some(&m)), Kind::Laser);
    assert!(m.axis(0).is_some() && m.axis(1).is_some() && m.axis(2).is_none());
    assert_eq!((m.spindle_max, m.coolants.len()), (None, 0));
    for (name, _) in camcraft_app::posting::BUILTINS {
        assert!(!camcraft_app::posting::post_serves(name, Kind::Laser), "{name} claims a laser");
    }
}

#[test]
fn the_laser_head_passes_the_tool_door_and_round_trips_the_project() {
    let w = new_world();
    let mut h = History::default();
    h.execute(&w, AddTool::new(&w, "Beam", laser_tool(0.15)).expect("a good laser head"));
    for (bad, word) in [
        (Tool { rpm: 1000.0, ..laser_tool(0.15) }, "rpm"),
        (Tool { diameter: 0.0, ..laser_tool(0.15) }, "kerf"),
        (Tool { corner_r: 0.05, ..laser_tool(0.15) }, "corner_r"),
    ] {
        let error = AddTool::new(&w, "Beam 2", bad).err().expect("refused").to_string();
        assert!(error.contains(word), "{word}: {error}");
    }
    let read = |w: &flecs_ecs::prelude::World| {
        model::find_by_name(w, "Beam").and_then(|e| model::tool_of(w, &e)).expect("the laser head")
    };
    let original = read(&w);
    assert_eq!(original.kind, ToolKind::Laser);
    let data = project::tool_to_json(&original);
    assert_eq!(data["kind"], "laser", "{data}");
    assert_eq!(project::tool_from_json(&data).expect("reads back"), original);
    let again = new_world();
    project::open(&again, &project::save_text(&w).expect("saves")).expect("opens");
    assert_eq!(read(&again), original);
}

#[test]
fn a_laser_machine_admits_no_operation_in_this_slice() {
    for family in [Family::Mill, Family::Turn, Family::Plasma, Family::WireEdm, Family::Sinker, Family::Waterjet] {
        assert!(!family.admits(Kind::Laser), "{family:?} admits a laser");
    }
    assert!(camcraft_app::operation::shown_for(Kind::Laser).is_empty(), "no ribbon section for a laser yet");

    // The door: a milling op on a laser setup never exists, refused by name.
    let w = new_world();
    let mut h = History::default();
    let mill = camcraft_app::tools::catalog().into_iter().find(|(n, _)| *n == "EM-0400-2FL").map(|(_, t)| t).expect("a catalog tool");
    h.execute(&w, AddTool::new(&w, "EM-0400-2FL", mill).expect("tool"));
    let loc = Location { pos: Point3::new(0.0, 0.0, 0.0), rot: IDENTITY_Q };
    h.execute(&w, SetStock::new(&w, StockShape::Box { dx: 60.0, dy: 60.0, dz: 20.0 }, loc).expect("stock"));
    h.execute(&w, CreateOperation::new(&w, "Face1", "EM-0400-2FL", SrcRef::Stock, face()).expect("a face on the mill"));
    machine::set(&w, "synthetic-laser.mch", Some(&laser()));
    assert_eq!(machine::kind_of(&w), Kind::Laser);
    let err = CreateOperation::new(&w, "Face2", "EM-0400-2FL", SrcRef::Stock, face()).err().expect("refused");
    assert!(format!("{err:?}").contains("laser cutter, which cuts only laser operations"), "{err:?}");

    // The program check: an op made before the laser was mounted is refused too.
    let face1 = model::find_by_name(&w, "Face1").expect("the face op");
    let check = machine::check_ops(&w, &laser(), &[("Face1".to_owned(), face1)]).expect("the check runs");
    assert!(check.problems.iter().any(|p| p == "Face1: a laser machine admits only laser operations"), "{:?}", check.problems);
}
