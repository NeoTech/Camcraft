//! #3012: pattern folders - the undoable commands (create, edit, set
//! members, turn a copy off, delete, reorder, rename, undo, redo), the
//! refusals made when a command is created, and the project round trip.
//! Provenance: our own document model; linear and grid part patterns are
//! general CAM practice.
//!
//! Gate map: covers app::pattern_folder, app::commands::pattern_folder,
//! app::commands::operation (ReorderOperation, RenameOperation,
//! DeleteOperation), app::commands (DeleteEntity), app::project
//! (pattern_dto), app::program_entry (sequence).

use flecs_ecs::prelude::World;

use camcraft_app::commands::{
    AddTool, CreateOperation, CreatePatternFolder, DeleteEntity, DeleteOperation, DeletePatternFolder,
    EditPatternFolder, GenerateOperation, History, PatternEdit, RenameOperation, ReorderOperation, SetPatternInstanceEnabled,
    SetPatternMembers, SetStock,
};
use camcraft_app::model;
use camcraft_app::operation::{Height, OpParams, SrcRef};
use camcraft_app::pattern_folder::{enabled_offsets, pattern_of};
use camcraft_app::program_entry;
use camcraft_app::schema::{PatternDef, PatternKind, PatternOrder};
use camcraft_app::stock::StockShape;
use camcraft_app::tools::catalog;
use camcraft_app::world::new_world;

pub(crate) fn face_params(cut_z: f64) -> OpParams {
    OpParams::Face {
        holes: Default::default(),
        blend_turns: false,
        blend_radius: None,
        tilt: None,
        stepover: 12.0,
        clearance_z: Height::fixed(25.0),
        cut_z: Height::fixed(cut_z),
        top_z: None,
        stepdown: None,
        feed: 1200.0,
        plunge_feed: 300.0,
        climb: true,
        picks: Default::default(),
    }
}

/// Three generated face ops on a 30 x 30 x 20 box: Face1 and Face3 on
/// one tool, Face2 on another.
pub(crate) fn three_faces() -> (World, History) {
    let world = new_world();
    let mut h = History::default();
    let stock = SetStock::new(&world, StockShape::Box { dx: 30.0, dy: 30.0, dz: 20.0 }, Default::default());
    h.execute(&world, stock.expect("box stock"));
    for t in ["EM-0600-4FL", "EM-1000-4FL"] {
        let tool = catalog().into_iter().find(|(n, _)| *n == t).map(|(_, t)| t).expect("catalog entry");
        h.execute(&world, AddTool::new(&world, t, tool).expect("catalog tool"));
    }
    for (name, tool, z) in [("Face1", "EM-1000-4FL", 19.5), ("Face2", "EM-0600-4FL", 19.0), ("Face3", "EM-1000-4FL", 18.5)] {
        let cmd = CreateOperation::new(&world, name, tool, SrcRef::Stock, face_params(z));
        h.execute(&world, cmd.expect("create"));
        h.execute(&world, GenerateOperation::new(&world, name).expect("generate"));
    }
    (world, h)
}

pub(crate) fn linear(count: u32) -> PatternKind {
    PatternKind::Linear { dir: [1.0, 0.0, 0.0], count, spacing: 40.0 }
}

pub(crate) fn grid() -> PatternKind {
    PatternKind::Grid { dir1: [1.0, 0.0, 0.0], count1: 2, spacing1: 40.0, dir2: [0.0, 2.0, 0.0], count2: 2, spacing2: 50.0 }
}

pub(crate) fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_owned()).collect()
}

fn list(world: &World) -> Vec<String> {
    program_entry::sequence(world).into_iter().map(|(n, _)| n).collect()
}

fn def(world: &World, key: &str) -> Option<PatternDef> {
    model::find_by_name(world, key).and_then(|e| pattern_of(world, &e))
}

fn create(world: &World, h: &mut History, kind: &PatternKind, mode: PatternOrder, members: &[&str]) -> String {
    let cmd = CreatePatternFolder::new(world, kind, mode, &names(members)).expect("create folder");
    let name = cmd.name().to_owned();
    h.execute(world, cmd);
    name
}

#[test]
fn a_folder_is_created_reordered_renamed_and_undone() {
    let (world, mut h) = three_faces();
    let start = list(&world);
    assert_eq!(start, ["Face1", "Face2", "Face3"]);
    let folder = create(&world, &mut h, &linear(3), PatternOrder::ByTool, &["Face3", "Face2"]);
    assert_eq!(folder, "Pattern1");
    // The folder takes its first member's row; the members follow it in folder order.
    assert_eq!(list(&world), ["Face1", "Pattern1", "Face3", "Face2"]);
    let made = def(&world, "Pattern1").expect("a folder");
    assert_eq!(made.members, names(&["Face3", "Face2"]));
    assert_eq!(made.kind, linear(3));
    let ops: Vec<String> = camcraft_app::setup::operation_order(&world).into_iter().map(|(n, _)| n).collect();
    assert_eq!(ops, ["Face1", "Face3", "Face2"], "the op walk never sees a folder, and runs in list order");

    // A member moves inside its folder: the folder order follows.
    h.execute(&world, ReorderOperation::new(&world, "Face2", "Face3", false).expect("inside the folder"));
    assert_eq!(list(&world), ["Face1", "Pattern1", "Face2", "Face3"]);
    assert_eq!(def(&world, "Pattern1").expect("folder").members, names(&["Face2", "Face3"]));
    // The folder moves with its members.
    h.execute(&world, ReorderOperation::new(&world, "Pattern1", "Face1", false).expect("folder moves"));
    assert_eq!(list(&world), ["Pattern1", "Face2", "Face3", "Face1"]);
    // A row dropped after the folder lands after its members.
    h.execute(&world, ReorderOperation::new(&world, "Face1", "Pattern1", false).expect("before the folder"));
    assert_eq!(list(&world), ["Face1", "Pattern1", "Face2", "Face3"]);
    h.execute(&world, ReorderOperation::new(&world, "Face1", "Pattern1", true).expect("after the folder"));
    assert_eq!(list(&world), ["Pattern1", "Face2", "Face3", "Face1"]);
    let into = ReorderOperation::new(&world, "Face1", "Face2", false).err().expect("into a folder");
    assert!(into.to_string().contains("Pattern1"), "{into}");
    let out = ReorderOperation::new(&world, "Face2", "Face1", true).err().expect("out of a folder");
    assert!(out.to_string().contains("Pattern1"), "{out}");

    h.execute(&world, RenameOperation::new(&world, "Pattern1", "Vise row").expect("a folder takes a label"));
    let e = model::find_by_name(&world, "Pattern1").expect("folder");
    assert_eq!(model::operation_label(&world, "Pattern1", &e), "Vise row");
    assert!(RenameOperation::new(&world, "Face1", "Pattern1").is_err(), "no label takes a folder's key");

    // Undo it all, then redo it all.
    let after = (list(&world), def(&world, "Pattern1"));
    for _ in 0..6 {
        h.undo(&world);
    }
    assert_eq!(list(&world), start);
    assert!(def(&world, "Pattern1").is_none(), "undo removes the folder");
    for _ in 0..6 {
        h.redo(&world);
    }
    assert_eq!((list(&world), def(&world, "Pattern1")), after);
}

#[test]
fn edits_members_copies_and_delete_are_undoable() {
    let (world, mut h) = three_faces();
    create(&world, &mut h, &linear(4), PatternOrder::ByTool, &["Face1"]);
    let edit = PatternEdit { kind: Some(grid()), order_mode: Some(PatternOrder::ByPart), suppressed: None };
    h.execute(&world, EditPatternFolder::new(&world, "Pattern1", &edit).expect("to a grid"));
    let g = def(&world, "Pattern1").expect("folder");
    assert_eq!(g.order_mode, PatternOrder::ByPart);
    // Directions are stored as unit vectors.
    assert_eq!(g.kind, PatternKind::Grid { dir1: [1.0, 0.0, 0.0], count1: 2, spacing1: 40.0, dir2: [0.0, 1.0, 0.0], count2: 2, spacing2: 50.0 });
    h.execute(&world, SetPatternInstanceEnabled::new(&world, "Pattern1", 4, false).expect("copy 4 off"));
    assert_eq!(
        enabled_offsets(&def(&world, "Pattern1").expect("folder")),
        vec![(1, [0.0, 0.0, 0.0]), (2, [40.0, 0.0, 0.0]), (3, [0.0, 50.0, 0.0])]
    );
    // A smaller pattern drops the off-switch of a copy it no longer has.
    let small = PatternEdit { kind: Some(linear(2)), ..Default::default() };
    h.execute(&world, EditPatternFolder::new(&world, "Pattern1", &small).expect("smaller"));
    assert!(def(&world, "Pattern1").expect("folder").disabled_instances.is_empty());
    h.undo(&world);
    assert_eq!(def(&world, "Pattern1").expect("folder").disabled_instances, vec![4]);

    h.execute(&world, SetPatternMembers::new(&world, "Pattern1", &names(&["Face3", "Face1"])).expect("members"));
    assert_eq!(list(&world), ["Pattern1", "Face3", "Face1", "Face2"]);
    // Deleting a member takes it out of the folder; undo puts it back.
    h.execute(&world, DeleteOperation::new(&world, "Face3").expect("delete op"));
    assert_eq!(def(&world, "Pattern1").expect("folder").members, names(&["Face1"]));
    h.undo(&world);
    assert_eq!(def(&world, "Pattern1").expect("folder").members, names(&["Face3", "Face1"]));
    h.execute(&world, DeleteEntity::new(&world, "Face1").expect("delete entity"));
    assert_eq!(def(&world, "Pattern1").expect("folder").members, names(&["Face3"]));
    h.undo(&world);

    // Delete ungroups: the members keep their rows.
    let before = def(&world, "Pattern1");
    h.execute(&world, DeletePatternFolder::new(&world, "Pattern1").expect("delete folder"));
    assert!(def(&world, "Pattern1").is_none());
    assert_eq!(list(&world), ["Face3", "Face1", "Face2"]);
    h.undo(&world);
    assert_eq!(def(&world, "Pattern1"), before);
    h.execute(&world, DeleteEntity::new(&world, "Pattern1").expect("delete as an entity"));
    h.undo(&world);
    assert_eq!(def(&world, "Pattern1"), before, "DeleteEntity restores the folder too");
}

#[test]
fn bad_patterns_and_members_are_refused_when_the_command_is_made() {
    let (world, mut h) = three_faces();
    let refuse = |kind: PatternKind, members: &[&str], want: &str| {
        let err = CreatePatternFolder::new(&world, &kind, PatternOrder::ByTool, &names(members)).err().expect("refused");
        assert!(err.to_string().contains(want), "{want}: {err}");
    };
    refuse(linear(0), &["Face1"], "count must be 1 to 1000");
    refuse(PatternKind::Linear { dir: [1.0, 0.0, 0.0], count: 2, spacing: -1.0 }, &["Face1"], "spacing");
    refuse(PatternKind::Linear { dir: [1.0, 0.0, 0.0], count: 2, spacing: f64::NAN }, &["Face1"], "spacing");
    refuse(PatternKind::Linear { dir: [0.0, 0.0, 0.0], count: 2, spacing: 5.0 }, &["Face1"], "must not be zero");
    refuse(PatternKind::Linear { dir: [1.0, 0.0, 1.0], count: 2, spacing: 5.0 }, &["Face1"], "XY plane");
    refuse(PatternKind::Linear { dir: [f64::INFINITY, 0.0, 0.0], count: 2, spacing: 5.0 }, &["Face1"], "finite");
    let parallel = PatternKind::Grid { dir1: [1.0, 0.0, 0.0], count1: 2, spacing1: 5.0, dir2: [-2.0, 0.0, 0.0], count2: 2, spacing2: 5.0 };
    refuse(parallel, &["Face1"], "parallel");
    let huge = PatternKind::Grid { dir1: [1.0, 0.0, 0.0], count1: 1000, spacing1: 5.0, dir2: [0.0, 1.0, 0.0], count2: 11, spacing2: 5.0 };
    refuse(huge, &["Face1"], "at most 10000 copies");
    refuse(linear(2), &["Face1", "Face1"], "named twice");
    refuse(linear(2), &["Nope"], "Nope");
    create(&world, &mut h, &linear(2), PatternOrder::ByTool, &["Face1"]);
    refuse(linear(2), &["Face2", "Face1"], "already in pattern folder 'Pattern1'");
    let none_left = SetPatternInstanceEnabled::new(&world, "Pattern1", 1, false)
        .and_then(|c| {
            h.execute(&world, c);
            SetPatternInstanceEnabled::new(&world, "Pattern1", 2, false)
        })
        .err()
        .expect("the last copy");
    assert!(none_left.to_string().contains("at least one copy"), "{none_left}");
    for bad in [0, 3] {
        let err = SetPatternInstanceEnabled::new(&world, "Pattern1", bad, false).err().expect("no such copy");
        assert!(err.to_string().contains("copies 1 to 2"), "{err}");
    }
    let same = EditPatternFolder::new(&world, "Pattern1", &PatternEdit::default()).err().expect("no change");
    assert!(same.to_string().contains("already reads that way"), "{same}");
}

#[test]
fn folders_round_trip_and_documents_without_them_write_no_key() {
    let (world, mut h) = three_faces();
    let path = std::env::temp_dir().join(format!("camcraft-pattern-{}.json", std::process::id()));
    camcraft_app::project::save(&world, &path).expect("save");
    let plain = std::fs::read_to_string(&path).expect("written");
    assert!(!plain.contains("\"pattern\""), "no folders, no key - byte shape unchanged");

    create(&world, &mut h, &linear(3), PatternOrder::ByTool, &["Face2"]);
    create(&world, &mut h, &grid(), PatternOrder::ByPart, &["Face3"]);
    h.execute(&world, SetPatternInstanceEnabled::new(&world, "Pattern2", 3, false).expect("off"));
    h.execute(&world, RenameOperation::new(&world, "Pattern2", "Grid of four").expect("label"));
    camcraft_app::project::save(&world, &path).expect("save");
    let text = std::fs::read_to_string(&path).expect("written");
    assert!(text.contains("\"kind\": \"pattern\"") && text.contains("\"order_mode\": \"by-part\""), "{text}");
    let back = new_world();
    camcraft_app::project::load(&back, &path).expect("load");
    assert_eq!(list(&back), list(&world));
    for name in ["Pattern1", "Pattern2"] {
        assert_eq!(def(&back, name), def(&world, name), "{name}");
    }
    let e = model::find_by_name(&back, "Pattern2").expect("folder");
    assert_eq!(model::operation_label(&back, "Pattern2", &e), "Grid of four");
    camcraft_app::project::save(&back, &path).expect("save again");
    assert_eq!(std::fs::read_to_string(&path).expect("written"), text, "save-load-save is byte-stable");

    // A corrupt folder row refuses the file by name.
    let bad = text.replace("\"count\": 3", "\"count\": 0");
    std::fs::write(&path, bad).expect("rewrite");
    let err = camcraft_app::project::load(&new_world(), &path).expect_err("no copies");
    assert!(err.to_string().contains("Pattern1"), "{err}");
    // An older reader meets a kind word it does not know: the same kind
    // word that serde refuses here when it is not a known kind.
    let unknown = text.replace("\"kind\": \"pattern\"", "\"kind\": \"patternx\"");
    std::fs::write(&path, unknown).expect("rewrite");
    let err = camcraft_app::project::load(&new_world(), &path).expect_err("unknown kind");
    let _ = std::fs::remove_file(&path);
    assert!(err.to_string().contains("patternx"), "{err}");
}
