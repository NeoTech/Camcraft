//! #3012, review of PR #4423: the laws of a pattern folder that a file
//! or a second folder could break - a folder is not a model; an
//! operation is a member of one folder only (at load and at the post);
//! a by-part stop is written once, after the last copy; and each copy
//! is judged against the fixtures and the machine travel.
//! Provenance: our own document and post law (see `posting_pattern`).
//!
//! Gate map: covers app::model (is_model), app::project (pattern_dto),
//! app::posting (posting_pattern: plan, pattern_problems), app::mcp::job.

use super::pattern_folder::{face_params, linear, names, three_faces};
use super::pattern_folder_post::{folder, has_word, post_fanuc, tool_changes};
use super::Client;
use camcraft_app::commands::{
    AddTool, CreateOperation, CreateSetup, EditOperationParams, GenerateOperation, History, LoadMachine,
    SetPatternInstanceEnabled, SetStock, TagFixture,
};
use camcraft_app::geometry::{Point3, Triangle};
use camcraft_app::machine::{AxisLimits, Machine};
use camcraft_app::operation::{AfterOperation, SrcRef};
use camcraft_app::pattern_folder::pattern_of;
use camcraft_app::schema::PatternOrder;
use camcraft_app::stock::StockShape;
use camcraft_app::tools::catalog;
use camcraft_app::world::new_world;
use camcraft_app::{model, posting};
use flecs_ecs::prelude::World;
use serde_json::{json, Value};

fn model_names(world: &World) -> Vec<String> {
    model::model_entities(world).into_iter().map(|(n, _)| n).collect()
}

#[test]
fn a_folder_is_not_a_model() {
    let (world, mut h) = three_faces();
    let before = model_names(&world);
    folder(&world, &mut h, &linear(2), PatternOrder::ByTool, &["Face1"]);
    let e = model::find_by_name(&world, "Pattern1").expect("folder");
    assert!(!model::is_model(&world, &e), "a folder is a list row, not a part");
    assert_eq!(model_names(&world), before, "the model list does not change");

    let mut c = Client::new();
    c.ok("set_stock", json!({ "box": { "dx": 30.0, "dy": 30.0, "dz": 20.0 } }));
    c.ok("add_tool", json!({ "catalog": "EM-1000-4FL" }));
    c.ok("add_operation", json!({ "kind": "Face", "tool": "EM-1000-4FL", "name": "F1" }));
    let models = c.ok("get_job", json!({}))["models"].clone();
    let pattern = json!({ "type": "linear", "dir": [1, 0], "count": 2, "spacing": 40 });
    c.ok("create_pattern_folder", json!({ "pattern": pattern, "members": ["F1"] }));
    let job = c.ok("get_job", json!({}));
    assert_eq!(job["pattern_folders"][0]["key"], json!("Pattern1"));
    assert_eq!(job["models"], models, "get_job lists no folder as a model");
    // A direction entry that is not a number is refused, not dropped.
    let bad = json!({ "type": "linear", "dir": [1, "x", 0], "count": 2, "spacing": 40 });
    let msg = c.refused("edit_pattern_folder", json!({ "name": "Pattern1", "pattern": bad }));
    assert!(msg.contains("\"dir\" must be [x, y] or [x, y, z] numbers"), "{msg}");
}

/// The saved file of two folders, with the members of `Pattern2` set to
/// `members` by hand, loaded into a new document.
fn load_with_members(tag: &str, members: &[&str]) -> Result<(), String> {
    let (world, mut h) = three_faces();
    folder(&world, &mut h, &linear(2), PatternOrder::ByTool, &["Face1", "Face2"]);
    folder(&world, &mut h, &linear(2), PatternOrder::ByTool, &["Face3"]);
    let path = std::env::temp_dir().join(format!("camcraft-pattern-law-{tag}-{}.json", std::process::id()));
    camcraft_app::project::save(&world, &path).expect("save");
    let mut file: Value = serde_json::from_str(&std::fs::read_to_string(&path).expect("written")).expect("json");
    let rows = file["entities"].as_array_mut().expect("entities");
    let row = rows.iter_mut().find(|r| r["name"] == json!("Pattern2")).expect("the folder row");
    row["pattern"]["members"] = json!(members);
    std::fs::write(&path, serde_json::to_string_pretty(&file).expect("text")).expect("rewrite");
    let out = camcraft_app::project::load(&new_world(), &path).map(drop).map_err(|e| e.to_string());
    let _ = std::fs::remove_file(&path);
    out
}

#[test]
fn a_file_breaking_the_membership_law_is_refused_at_load() {
    load_with_members("same", &["Face3"]).expect("the file as saved loads");
    // One operation in two folders: the post would have two shifts for it.
    let err = load_with_members("two", &["Face2", "Face3"]).expect_err("Face2 is in Pattern1");
    assert!(err.contains("'Face2'") && err.contains("'Pattern1'") && err.contains("'Pattern2'"), "{err}");
    assert!(err.contains("one folder only"), "{err}");
    // A member that is no row of the file, and two that are not operations.
    for bad in ["Nope", "EM-1000-4FL", "Pattern1"] {
        let err = load_with_members("not-op", &[bad]).expect_err("not an operation");
        assert!(err.contains("Pattern2") && err.contains(bad) && err.contains("not an operation"), "{bad}: {err}");
    }
}

#[test]
fn two_folders_that_name_one_operation_refuse_the_post_by_name() {
    let (world, mut h) = three_faces();
    folder(&world, &mut h, &linear(2), PatternOrder::ByTool, &["Face1", "Face2"]);
    folder(&world, &mut h, &linear(3), PatternOrder::ByTool, &["Face3"]);
    assert!(posting::stream_records(&world).is_ok(), "two folders post");
    // No command makes this state; a world edit names Face2 in the two folders.
    let e = model::find_by_name(&world, "Pattern2").expect("folder");
    let mut def = pattern_of(&world, &e).expect("def");
    def.members = names(&["Face2", "Face3"]);
    e.entity_view(&world).set(def);
    let err = posting::stream_records(&world).expect_err("Face2 has two shifts");
    assert!(err.contains("Pattern1") && err.contains("Pattern2") && err.contains("Face2"), "{err}");
    assert!(err.contains("one folder only"), "{err}");
    let gate = posting::verification_gate(&world);
    assert!(gate.iter().any(|p| p.contains("Face2") && p.contains("one folder only")), "the gate says it too: {gate:?}");
}

fn lines_with(program: &str, codes: &[&str]) -> Vec<usize> {
    program.lines().enumerate().filter(|(_, l)| codes.iter().any(|c| has_word(l, c))).map(|(i, _)| i).collect()
}

#[test]
fn a_by_part_stop_is_written_once_after_the_last_copy() {
    let (world, mut h) = three_faces();
    h.execute(&world, EditOperationParams::with_after(&world, "Face2", AfterOperation::ProgramStop).expect("stop after Face2"));
    let plain = post_fanuc(&world, "pattern-stop-plain");
    assert_eq!(lines_with(&plain.program, &["M0"]).len(), 1, "{}", plain.program);
    folder(&world, &mut h, &linear(3), PatternOrder::ByPart, &["Face2", "Face3"]);
    let posted = post_fanuc(&world, "pattern-stop");
    // Face1, then (Face2, Face3) at copies 1, 2 and 3: Face2 is cut three times.
    assert_eq!(posted.ops, 7);
    let stops = lines_with(&posted.program, &["M0"]);
    assert_eq!(stops.len(), 1, "one M0, not one for each copy:\n{}", posted.program);
    let changes = lines_with(&posted.program, &["M6", "M06"]);
    assert_eq!(changes.len(), tool_changes(&posted.program).len());
    assert_eq!(changes.len(), 7, "{}", posted.program);
    // Face2 at copy 3 is section 6; Face3 at copy 3 is section 7.
    assert!(changes[5] < stops[0] && stops[0] < changes[6], "the stop follows Face2's last copy:\n{}", posted.program);
}

/// Two setups: Face1 in Setup1 and Face2 in Setup2, on one tool.
fn two_setups() -> (World, History) {
    let world = new_world();
    let mut h = History::default();
    let stock = || StockShape::Box { dx: 30.0, dy: 30.0, dz: 20.0 };
    h.execute(&world, SetStock::new(&world, stock(), Default::default()).expect("stock"));
    let tool = catalog().into_iter().find(|(n, _)| *n == "EM-1000-4FL").map(|(_, t)| t).expect("catalog entry");
    h.execute(&world, AddTool::new(&world, "EM-1000-4FL", tool).expect("tool"));
    for i in 1..=2 {
        h.execute(&world, CreateSetup::new(&world, None, None).expect("setup"));
        if i > 1 {
            h.execute(&world, SetStock::new(&world, stock(), Default::default()).expect("setup stock"));
        }
        let name = format!("Face{i}");
        h.execute(&world, CreateOperation::new(&world, &name, "EM-1000-4FL", SrcRef::Stock, face_params(19.5)).expect("create"));
        h.execute(&world, GenerateOperation::new(&world, &name).expect("generate"));
    }
    (world, h)
}

#[test]
fn a_by_part_setup_change_is_written_once_after_the_last_copy() {
    let (world, mut h) = two_setups();
    let plain = post_fanuc(&world, "pattern-setups-plain");
    assert_eq!(plain.program.matches("SETUP CHANGE").count(), 1, "{}", plain.program);
    folder(&world, &mut h, &linear(3), PatternOrder::ByPart, &["Face1"]);
    let posted = post_fanuc(&world, "pattern-setups");
    assert_eq!(posted.ops, 4, "Face1 at three copies, then Face2");
    assert_eq!(posted.program.matches("SETUP CHANGE - NEXT: Setup2").count(), 1, "{}", posted.program);
    let stops = lines_with(&posted.program, &["M0"]);
    assert_eq!(stops.len(), 1, "one stop for the re-clamp:\n{}", posted.program);
    // The stream asks for the setup change one time (the post reads the
    // ask together with the stop of the same section).
    let stream = camcraft_app::post::cnc::serialize(&posting::stream_records(&world).expect("streams"));
    assert_eq!(stream.matches("SetupChange").count(), 1, "{stream}");
    // The stop stands between the last copy of Face1 and Face2.
    let starts = lines_with(&posted.program, &["M3", "M03"]);
    assert_eq!(starts.len(), 4, "one spindle start for each section:\n{}", posted.program);
    assert!(starts[2] < stops[0] && stops[0] < starts[3], "the stop follows the last copy of Face1:\n{}", posted.program);
}

/// The gate lines that name a copy of a member of `Pattern1`.
fn copy_problems(world: &World) -> Vec<String> {
    posting::verification_gate(world).into_iter().filter(|p| p.contains("(Pattern1)")).collect()
}

fn post_refusal(world: &World, tag: &str) -> String {
    let dir = super::tmp_dir(tag);
    let cps = posting::builtin_post_text("Fanuc").expect("a built-in");
    let out = posting::post_script(world, &dir.join("out.nc"), cps).map(|p| p.program);
    let _ = std::fs::remove_dir_all(&dir);
    out.expect_err("the gate blocks the post")
}

#[test]
fn a_copy_that_strikes_a_fixture_blocks_the_post_until_it_is_off() {
    let (world, mut h) = three_faces();
    folder(&world, &mut h, &linear(3), PatternOrder::ByTool, &["Face1"]);
    assert!(posting::verification_gate(&world).is_empty(), "no fixture: the copies post");
    // Steel in the middle of copy 2 (40 mm along X), 10 mm wide: above
    // the cut at Z 19.5, below the clearance plane at Z 25.
    let stock = model::stock_world_aabb(&world).expect("stock box");
    let (lo, hi) = (Point3::new(stock.min.x + 50.0, stock.min.y, 0.0), Point3::new(stock.min.x + 60.0, stock.max.y, 22.0));
    let block = model::spawn(&world, "Block");
    let tri = Triangle::new(lo, hi, Point3::new(lo.x, hi.y, hi.z));
    model::set_facets(&world, &block, vec![tri].into());
    model::set_rev(&world, &block, 1);
    h.execute(&world, TagFixture::new(&world, "Block", None).expect("tag"));
    let problems = copy_problems(&world);
    assert!(!problems.is_empty(), "copy 2 runs into the block: {:?}", posting::verification_gate(&world));
    assert!(problems.iter().all(|p| p.starts_with("Face1 copy 2 (Pattern1):") && p.contains("strikes a fixture")), "{problems:?}");
    let err = post_refusal(&world, "pattern-fixture");
    assert!(err.contains("Face1 copy 2 (Pattern1)") && err.contains("strikes a fixture"), "{err}");
    h.execute(&world, SetPatternInstanceEnabled::new(&world, "Pattern1", 2, false).expect("copy 2 off"));
    let gate = posting::verification_gate(&world);
    assert!(gate.is_empty(), "copy 2 is off: the block is not cut: {gate:?}");
}

#[test]
fn a_copy_outside_the_machine_travel_blocks_the_post_until_it_is_off() {
    let (world, mut h) = three_faces();
    folder(&world, &mut h, &linear(3), PatternOrder::ByTool, &["Face1"]);
    // X ends 70 mm past the stock: copies 1 and 2 (40 mm) are inside,
    // copy 3 (80 mm) is not.
    let stock = model::stock_world_aabb(&world).expect("stock box");
    let axis = |name: &str, travel| AxisLimits { name: name.into(), travel: Some(travel), ..Default::default() };
    let machine = Machine {
        model: "Short X".into(),
        axes: vec![axis("X", (-500.0, stock.max.x + 70.0)), axis("Y", (-500.0, 500.0)), axis("Z", (-500.0, 500.0))],
        ..Default::default()
    };
    h.execute(&world, LoadMachine::new(&world, "short-x.mch".into(), Some(machine)).expect("machine"));
    let problems = copy_problems(&world);
    assert!(!problems.is_empty(), "copy 3 leaves the travel: {:?}", posting::verification_gate(&world));
    assert!(problems.iter().all(|p| p.contains("Face1 copy 3 (Pattern1)") && p.contains('X')), "{problems:?}");
    let err = post_refusal(&world, "pattern-travel");
    assert!(err.contains("Face1 copy 3 (Pattern1)"), "{err}");
    h.execute(&world, SetPatternInstanceEnabled::new(&world, "Pattern1", 3, false).expect("copy 3 off"));
    let gate = posting::verification_gate(&world);
    assert!(gate.is_empty(), "copy 3 is off: the program is inside the travel: {gate:?}");
}
