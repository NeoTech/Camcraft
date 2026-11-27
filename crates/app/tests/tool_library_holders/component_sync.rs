//! #3645: the user component library (`camcraft-components/1`) and the
//! undoable sync. Pinned: the store round-trips, refuses a bad file
//! whole and names the entry, saves atomically, and resolves its folder
//! from `CAMCRAFT_COMPONENTS` first; the built-ins are read-only; a sync
//! pulls a newer library copy into both assemblies in one undo step and
//! undo restores the exact state; a library copy that differs from the
//! document's after an open is reported, never applied; a hand edit
//! keeps the link and the status reports the difference. Provenance:
//! our own tests over our own format and commands.

use std::ffi::OsString;
use std::path::PathBuf;

use camcraft_app::commands::{EditDocumentComponent, History, SyncDocumentComponent};
use camcraft_app::component_library::store::{self, ENV_VAR};
use camcraft_app::component_library::{self as lib, LibraryComponent, SyncStatus, sync_status};
use camcraft_app::operation::OpStatus;
use camcraft_app::project;
use camcraft_app::tool_assembly::{component, components};
use flecs_ecs::prelude::World;
use serde_json::{Value, json};

use super::assembly_cmds::{fly, holder, insert, op_status, shop, snapshot, A, B, INSERT};

fn entry((component, geometry): (camcraft_tools::component::ToolComponent, camcraft_tools::component_geometry::ComponentGeometry)) -> LibraryComponent {
    LibraryComponent { component, geometry }
}

fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<OsString> + use<> {
    let owned: Vec<(String, String)> = pairs.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_owned())).collect();
    move |k| owned.iter().find(|(n, _)| n == k).map(|(_, v)| OsString::from(v))
}

fn temp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("camcraft-3645-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

#[test]
fn the_store_round_trips_and_refuses_a_bad_file_whole_by_name() {
    let dir = temp_dir("store");
    let path = store::library_path(&dir);
    assert_eq!(store::load(&path).expect("missing file"), Vec::new(), "a missing file is the empty library");
    let entries = vec![entry(holder()), entry(fly("fly-25", 12.5)), entry(insert("CCMT09T304"))];
    for e in entries.clone() {
        store::put(&path, e).expect("put");
    }
    assert_eq!(store::load(&path).expect("load"), entries);
    let text = std::fs::read_to_string(&path).expect("read");
    assert_eq!(lib::to_text(&lib::parse(&text).expect("parse")).expect("text"), text, "byte-stable");

    let with = |f: &dyn Fn(&mut Value)| {
        let mut v: Value = serde_json::from_str(&text).expect("json");
        f(&mut v);
        lib::parse(&v.to_string()).err().unwrap_or_default()
    };
    let e = with(&|v| v["components"][1]["item"]["tool_item"]["seats"][0]["shape"] = json!("Q"));
    assert!(e.contains("tool component 'fly-25'") && e.contains("'Q'"), "{e}");
    let e = with(&|v| v["components"][2]["library_ref"] = json!({ "id": INSERT, "content_hash": "x" }));
    assert!(e.contains(INSERT) && e.contains("library link"), "{e}");
    let e = with(&|v| v["components"][2]["running"] = json!({ "tool_material": "carbide" }));
    assert!(e.contains(INSERT) && e.contains("no running data"), "#3646 data stays in the document: {e}");
    let (c, g) = insert("CCMT09T304");
    let running = camcraft_app::schema::InsertRunning::default();
    let row = camcraft_app::schema::DocumentComponent { running: Some(running), ..camcraft_app::schema::DocumentComponent::new(c, g) };
    let e = LibraryComponent::from_row(row).err().unwrap_or_default();
    assert!(e.contains(INSERT) && e.contains("running data"), "a save never drops the data silently: {e}");
    let e = with(&|v| v["components"][0]["id"] = json!("builtin:holder:mine"));
    assert!(e.contains("built-in"), "{e}");
    let e = with(&|v| v["version"] = json!(2));
    assert!(e.contains("version 2"), "{e}");
    let e = with(&|v| v["units"] = json!("in"));
    assert!(e.contains("units"), "{e}");
    let e = with(&|v| {
        let first = v["components"][0].clone();
        v["components"].as_array_mut().expect("rows").push(first);
    });
    assert!(e.contains("twice"), "{e}");

    // A refused file is an error on load, never the empty library.
    std::fs::write(&path, "{ not json").expect("write");
    assert!(store::load(&path).is_err());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_save_is_atomic_and_the_env_folder_comes_first() {
    let dir = temp_dir("atomic");
    let path = store::library_path(&dir);
    store::save(&path, &[entry(holder())]).expect("save");
    let before = std::fs::read_to_string(&path).expect("read");
    // A target that is a folder: the rename fails, the temp file goes.
    let blocked = dir.join("blocked.camcomponents");
    std::fs::create_dir_all(blocked.join("inside")).expect("folder");
    assert!(store::save(&blocked, &[entry(holder())]).is_err());
    let left: Vec<String> = std::fs::read_dir(&dir).expect("dir").filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    assert!(left.iter().all(|n| !n.ends_with(".tmp")), "no temp file is left: {left:?}");
    assert_eq!(std::fs::read_to_string(&path).expect("read"), before, "the library stays as it was");
    // A built-in id is never written to the user file.
    let builtin = lib::builtins().into_iter().next().expect("a built-in");
    assert!(store::save(&path, std::slice::from_ref(&builtin)).is_err() && store::delete(&path, builtin.component.id()).is_err());
    let _ = std::fs::remove_dir_all(&dir);

    let abs = if cfg!(windows) { "C:\\lib" } else { "/lib" };
    let over = env(&[(ENV_VAR, abs), ("HOME", "/home/u"), ("APPDATA", "C:\\Users\\u")]);
    assert_eq!(store::user_dir_for("linux", over), Some(PathBuf::from(abs)), "the override first");
    if !cfg!(windows) {
        let rel = [(ENV_VAR, "relative"), ("HOME", "/home/u")];
        assert_eq!(store::user_dir_for("linux", env(&rel)), Some(PathBuf::from("/home/u/.local/share/camcraft/components")), "a relative override is ignored");
        assert_eq!(store::user_dir_for("macos", env(&rel)), Some(PathBuf::from("/home/u/Library/Application Support/camcraft/components")));
    }
    assert_eq!(store::user_dir_for("linux", |_| None), None, "no home: no folder");
}

#[test]
fn the_built_ins_are_the_holders_and_the_starter_tools_read_only() {
    let all = lib::builtins();
    assert!(all.iter().all(LibraryComponent::is_builtin));
    assert!(all.iter().any(|c| c.component.id() == "builtin:holder:BT40 ER32"), "{:?}", all.iter().map(|c| c.component.id()).collect::<Vec<_>>());
    assert!(all.iter().any(|c| c.component.id() == "builtin:tool:EM-1200-2FL"));
    let ids: std::collections::HashSet<&str> = all.iter().map(|c| c.component.id()).collect();
    assert_eq!(ids.len(), all.len(), "each built-in id once");
}

/// The shop, its insert linked to a library copy (the first sync).
fn linked() -> (World, History, Vec<LibraryComponent>) {
    let (w, mut h) = shop();
    let library = lib::with_builtins(vec![entry(insert("CCMT09T304"))]);
    let row = component(&w, INSERT).expect("row");
    assert_eq!(sync_status(&row, &library), SyncStatus::NotLinked);
    let cmd = SyncDocumentComponent::new(&w, INSERT, &library).expect("link");
    assert!(cmd.updated_tools().is_empty(), "the same content: the link alone, no tool derived again");
    h.execute(&w, cmd);
    assert_eq!(sync_status(&component(&w, INSERT).expect("row"), &library), SyncStatus::UpToDate);
    let Err(e) = SyncDocumentComponent::new(&w, INSERT, &library) else { panic!("a second sync") };
    assert!(e.to_string().contains("already up to date"), "{e}");
    (w, h, library)
}

#[test]
fn a_sync_pulls_a_newer_library_copy_into_both_assemblies_and_undo_restores() {
    let (w, mut h, _) = linked();
    let before = snapshot(&w);
    let newer = lib::with_builtins(vec![entry(insert("CCMT09T308"))]);
    assert!(matches!(sync_status(&component(&w, INSERT).expect("row"), &newer), SyncStatus::UpdateAvailable { .. }));
    let cmd = SyncDocumentComponent::new(&w, INSERT, &newer).expect("sync");
    assert_eq!(cmd.updated_tools(), vec![A.to_owned(), B.to_owned()]);
    h.execute(&w, cmd);
    let after = snapshot(&w);
    assert!(after.1.iter().all(|(t, _, _)| (t.corner_r - 0.8).abs() < 1e-12), "both tools take the 0.8 mm nose");
    assert_eq!(op_status(&w, A), OpStatus::Stale);
    assert_eq!(op_status(&w, B), OpStatus::Stale);
    assert_eq!(sync_status(&component(&w, INSERT).expect("row"), &newer), SyncStatus::UpToDate);
    assert!(h.undo(&w));
    assert_eq!(snapshot(&w), before, "one undo step restores the rows, the records and the bodies");
    assert!(h.redo(&w));
    assert_eq!(snapshot(&w), after);
    let Err(e) = SyncDocumentComponent::new(&w, "fly-99", &newer) else { panic!("no such row") };
    assert!(e.to_string().contains("fly-99"), "{e}");
    let Err(e) = SyncDocumentComponent::new(&w, "fly-25", &newer) else { panic!("no library copy") };
    assert!(e.to_string().contains("library holds no tool component 'fly-25'"), "{e}");
}

#[test]
fn a_library_copy_that_differs_after_an_open_is_reported_never_applied() {
    let (w, _h, _) = linked();
    let text = project::save_text(&w).expect("save");
    let file: Value = serde_json::from_str(&text).expect("json");
    let row = file["components"].as_array().and_then(|r| r.iter().find(|r| r["id"] == json!(INSERT))).expect("row");
    assert_eq!(row["library_ref"]["id"], json!(INSERT), "the link is on file: {row}");
    // The other machine's library holds a newer copy.
    let other = lib::with_builtins(vec![entry(insert("CCMT09T308"))]);
    let w2 = camcraft_app::world::new_world();
    project::open(&w2, &text).expect("open");
    assert_eq!(components(&w2), components(&w), "the document copy, not the library's");
    assert_eq!(snapshot(&w2), snapshot(&w), "the records that posted stay");
    let row = component(&w2, INSERT).expect("row");
    let SyncStatus::UpdateAvailable { library_hash } = sync_status(&row, &other) else { panic!("the change is reported") };
    assert_ne!(Some(library_hash), row.library_ref.map(|r| r.content_hash));
    assert_eq!(sync_status(&component(&w2, INSERT).expect("row"), &lib::builtins()), SyncStatus::MissingFromLibrary);
}

#[test]
fn a_hand_edit_keeps_the_link_and_the_status_reports_the_difference() {
    let (w, mut h, library) = linked();
    let link = component(&w, INSERT).expect("row").library_ref;
    assert!(link.is_some(), "linked");
    let (c, g) = insert("CCMT09T308");
    h.execute(&w, EditDocumentComponent::new(&w, c, g).expect("a hand edit"));
    let row = component(&w, INSERT).expect("row");
    assert_eq!(row.library_ref, link, "the hand edit keeps the library link");
    assert!(matches!(sync_status(&row, &library), SyncStatus::UpdateAvailable { .. }), "the document copy now differs");
}
