//! #2201: the displayed operation status (`operation::status_shown`) is
//! computed once per world revision, and every writer of a status input
//! moves that revision - so the cache always equals a fresh
//! `operation::status_of`.
//!
//! Gate map: covers app::operation, app::world_rev, app::commands, app::project, app::schema.

use camcraft_app::commands::{
    AddTool, ChangeTool, ClearToolpath, CreateOperation, CreateSetup, DeleteEntity, DeleteTool,
    EditOperationParams, GenerateOperation, History, ImportModel, LoadMachine, MoveModel, SetSetupPostScriptOptions,
    SetStock, UpdateModel,
};
use camcraft_app::geometry::{cube, Point3};
use camcraft_app::model;
use camcraft_app::operation::{self, Height, OpParams, OpStatus, SrcRef};
use camcraft_app::schema::{Location, PostScriptOptions};
use camcraft_app::stock::StockShape;
use camcraft_app::tools::catalog;
use camcraft_app::world::new_world;
use camcraft_app::{project, world_rev};
use flecs_ecs::prelude::World;

const TOOL: &str = "EM-0600-4FL";

fn face(cut_z: f64) -> OpParams {
    OpParams::Face {
        holes: Default::default(),
        tilt: None,
        blend_turns: false,
        stepover: 3.0,
        clearance_z: Height::fixed(25.0),
        cut_z: Height::fixed(cut_z),
        top_z: None,
        stepdown: None,
        feed: 1200.0,
        plunge_feed: 300.0,
        climb: true,
    }
}

/// A part, a stock, a tool and two generated Face operations G1, G2 on
/// the part.
fn fixture() -> (World, History) {
    let world = new_world();
    let mut h = History::default();
    let tool = catalog().into_iter().find(|(n, _)| *n == TOOL).map(|(_, t)| t).expect("tool");
    h.execute(&world, AddTool::new(&world, TOOL, tool).expect("tool"));
    h.execute(&world, ImportModel::from_mesh("part", cube(Point3::ZERO, Point3::new(10.0, 10.0, 5.0))));
    h.execute(
        &world,
        SetStock::new(&world, StockShape::Box { dx: 40.0, dy: 40.0, dz: 20.0 }, Location::default()).expect("stock"),
    );
    for op in ["G1", "G2"] {
        let src = SrcRef::Entity("part".into());
        h.execute(&world, CreateOperation::new(&world, op, TOOL, src, face(4.5)).expect("create"));
        h.execute(&world, GenerateOperation::new(&world, op).expect("generate"));
    }
    (world, h)
}

fn ent(world: &World, name: &str) -> flecs_ecs::prelude::Entity {
    model::find_by_name(world, name).expect("entity exists")
}

/// The shown status of `op`, asserted equal to the uncached truth.
fn shown(world: &World, op: &str) -> OpStatus {
    let e = ent(world, op);
    let cached = operation::status_shown(world, &e);
    assert_eq!(cached, operation::status_of(world, &e), "the cache of '{op}' disagrees with a fresh status");
    assert_eq!(
        operation::stale_reason_shown(world, &e),
        operation::stale_reason(world, &e),
        "the cached reason of '{op}' disagrees with a fresh one"
    );
    cached
}

/// Read both statuses so the cache holds them, then return the build
/// count, so a later read can prove it computed again.
fn warm(world: &World) -> u64 {
    let _ = (shown(world, "G1"), shown(world, "G2"));
    operation::status_builds()
}

#[test]
fn still_frames_compute_each_status_once() {
    let (world, _h) = fixture();
    let before = operation::status_builds();
    for _ in 0..10 {
        assert_eq!(operation::status_shown(&world, &ent(&world, "G1")), OpStatus::Ok);
        assert_eq!(operation::status_shown(&world, &ent(&world, "G2")), OpStatus::Ok);
        let _ = operation::stale_reason_shown(&world, &ent(&world, "G1"));
    }
    assert_eq!(operation::status_builds() - before, 2, "ten still frames, one compute per operation");
}

#[test]
fn reads_never_move_the_revision() {
    let (world, _h) = fixture();
    let rev = world_rev::of(&world).expect("a new_world has a revision");
    for op in ["G1", "G2"] {
        let e = ent(&world, op);
        let _ = operation::status_of(&world, &e);
        let _ = operation::stale_reason(&world, &e);
        let _ = model::toolpath_nonempty(&world, &e);
    }
    let _ = model::named_entities(&world);
    let _ = project::save_text(&world).expect("save");
    assert_eq!(world_rev::of(&world), Some(rev), "a read wrote a component");
}

#[test]
fn an_operation_param_edit_rebuilds_and_undo_redo_follow() {
    let (world, mut h) = fixture();
    let n = warm(&world);
    h.execute(&world, EditOperationParams::new(&world, "G1", face(4.0)).expect("edit"));
    assert_eq!(shown(&world, "G1"), OpStatus::Stale);
    assert!(operation::status_builds() > n, "the edit rebuilt the status");
    let n = warm(&world);
    assert!(h.undo(&world));
    assert_eq!(shown(&world, "G1"), OpStatus::Ok, "undo shows the fresh status again");
    assert!(operation::status_builds() > n);
    let n = warm(&world);
    assert!(h.redo(&world));
    assert_eq!(shown(&world, "G1"), OpStatus::Stale, "redo shows the stale status again");
    assert!(operation::status_builds() > n);
}

#[test]
fn a_tool_edit_and_a_tool_delete_rebuild() {
    let (world, mut h) = fixture();
    warm(&world);
    let mut tool = model::tool_of(&world, &ent(&world, TOOL)).expect("tool");
    tool.diameter *= 0.5;
    h.execute(&world, ChangeTool::new(&world, TOOL, tool).expect("change tool"));
    assert_eq!(shown(&world, "G1"), OpStatus::Stale, "a new diameter is a new stamp");
    assert!(h.undo(&world));
    assert_eq!(shown(&world, "G1"), OpStatus::Ok);
    if let Ok(cmd) = DeleteTool::new(&world, TOOL) {
        h.execute(&world, cmd);
        let _ = shown(&world, "G1");
        assert!(h.undo(&world));
        assert_eq!(shown(&world, "G1"), OpStatus::Ok);
    }
}

#[test]
fn generate_and_clear_rebuild() {
    let (world, mut h) = fixture();
    warm(&world);
    h.execute(&world, ClearToolpath::new(&world, "G1").expect("clear"));
    assert_eq!(shown(&world, "G1"), OpStatus::Never);
    h.execute(&world, EditOperationParams::new(&world, "G1", face(4.0)).expect("edit"));
    h.execute(&world, GenerateOperation::new(&world, "G1").expect("generate"));
    assert_eq!(shown(&world, "G1"), OpStatus::Ok, "a new generation shows Ok");
    assert!(h.undo(&world));
    assert_eq!(shown(&world, "G1"), OpStatus::Never, "undo of the generation");
}

#[test]
fn stock_model_and_placement_edits_rebuild() {
    let (world, mut h) = fixture();
    warm(&world);
    h.execute(
        &world,
        SetStock::new(&world, StockShape::Box { dx: 50.0, dy: 50.0, dz: 30.0 }, Location::default()).expect("stock"),
    );
    let _ = shown(&world, "G1");
    assert!(h.undo(&world));
    assert_eq!(shown(&world, "G1"), OpStatus::Ok);
    warm(&world);
    let slide = Location { pos: Point3::new(4.0, 0.0, 0.0), ..Location::default() };
    h.execute(&world, MoveModel::new(&world, "part", slide).expect("move"));
    assert_eq!(shown(&world, "G1"), OpStatus::Stale, "a moved part is a new stamp");
    assert!(h.undo(&world));
    assert_eq!(shown(&world, "G1"), OpStatus::Ok);
    warm(&world);
    let taller = cube(Point3::ZERO, Point3::new(10.0, 10.0, 6.0));
    h.execute(&world, UpdateModel::new(&world, "part", taller).expect("update"));
    assert_eq!(shown(&world, "G1"), OpStatus::Stale, "new geometry is a new stamp");
    assert!(h.undo(&world));
    assert_eq!(shown(&world, "G1"), OpStatus::Ok);
}

#[test]
fn a_prior_operation_edit_rebuilds_the_next_one() {
    let (world, mut h) = fixture();
    warm(&world);
    h.execute(&world, EditOperationParams::new(&world, "G1", face(3.0)).expect("edit"));
    h.execute(&world, GenerateOperation::new(&world, "G1").expect("regenerate"));
    assert_eq!(shown(&world, "G1"), OpStatus::Ok);
    // G2's stamp reads the program ahead (#109): equal to the truth
    // whatever that truth is.
    let _ = shown(&world, "G2");
    assert!(h.undo(&world));
    let _ = shown(&world, "G2");
}

#[test]
fn setup_machine_and_delete_rebuild() {
    let (world, mut h) = fixture();
    let rev = world_rev::of(&world);
    h.execute(&world, CreateSetup::new(&world, None, None).expect("setup"));
    assert_ne!(world_rev::of(&world), rev);
    let _ = shown(&world, "G1");
    let rev = world_rev::of(&world);
    // The one write that was in place (#2201): it must move the revision.
    let cmd = SetSetupPostScriptOptions::for_setup(&world, "Setup1", Some(PostScriptOptions::default()));
    h.execute(&world, cmd.expect("setup script options"));
    assert_ne!(world_rev::of(&world), rev, "a setup's script options moved the revision");
    let rev = world_rev::of(&world);
    h.execute(&world, LoadMachine::builtin(&world, "Sienci AltMill 4x4").expect("machine"));
    assert_ne!(world_rev::of(&world), rev, "a machine load moved the revision");
    let _ = shown(&world, "G1");
    warm(&world);
    h.execute(&world, DeleteEntity::new(&world, "part").expect("delete"));
    assert_eq!(shown(&world, "G1"), OpStatus::Stale, "a deleted source is a new stamp");
    assert!(h.undo(&world));
    let _ = shown(&world, "G1");
}

#[test]
fn a_project_load_rebuilds() {
    let (world, mut h) = fixture();
    let ok_text = project::save_text(&world).expect("save");
    h.execute(&world, EditOperationParams::new(&world, "G1", face(4.0)).expect("edit"));
    assert_eq!(shown(&world, "G1"), OpStatus::Stale);
    // Open the saved document into the same world: the cache must not
    // keep the status of the document it replaced.
    project::open(&world, &ok_text).expect("open");
    assert_eq!(shown(&world, "G1"), OpStatus::Ok, "the loaded document's status");
    // And into a new world: its revision never matches the old one.
    let other = new_world();
    assert_ne!(world_rev::of(&other), world_rev::of(&world));
    project::open(&other, &ok_text).expect("open");
    assert_eq!(shown(&other, "G1"), OpStatus::Ok);
    let _ = shown(&world, "G1");
}

#[test]
fn no_component_is_written_in_place() {
    // A write through `get::<&mut T>` sends no event, so the revision
    // would not move and a cached status could show a stale "Ok".
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut bad = Vec::new();
    let mut stack = vec![root.join("app/src"), root.join("ui/src")];
    while let Some(dir) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&dir) else { continue };
        for entry in rd.flatten() {
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "rs") {
                let text = std::fs::read_to_string(&p).unwrap_or_default();
                for (i, line) in text.lines().enumerate() {
                    let code = line.split("//").next().unwrap_or("");
                    let compact: String = code.chars().filter(|c| !c.is_whitespace()).collect();
                    let typed_mut = ["get::<", "each::<", "each_entity::<", "each_iter::<", "query::<", "system::<"]
                        .iter()
                        .any(|door| {
                            compact.match_indices(door).any(|(at, _)| {
                                let rest = &compact[at + door.len()..];
                                rest.split('>').next().is_some_and(|args| args.contains("&mut"))
                            })
                        });
                    if typed_mut || compact.contains("ensure_mut") || compact.contains("get_mut::<") {
                        bad.push(format!("{}:{}", p.display(), i + 1));
                    }
                }
            }
        }
    }
    assert!(bad.is_empty(), "components written in place (use clone + set): {bad:?}");
}
