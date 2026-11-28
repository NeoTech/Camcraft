//! #4389: holder files. Pinned: an export -> import into a new document
//! round-trips the user holders and is byte-stable; the import is one
//! undo step; an id the document or the library holds takes a new
//! `user:holder:<name>` id; a file that is not correct (bad JSON, a
//! truncated file, a wrong envelope, a row that is not a holder, no
//! row, a holder outline with a negative radius or fewer than 3 points)
//! is refused by name with nothing applied; a built-in row is not
//! exported; the MCP tools do the same through the same command, and
//! an unreadable component library refuses the MCP import.
//! Provenance: our own tests over our own format and command.

use camcraft_app::commands::holder_file::is_holder;
use camcraft_app::commands::{History, ImportHolders, UpsertDocumentComponent, export_holders};
use camcraft_app::component_library::{self, LibraryComponent};
use camcraft_app::mcp::Server;
use camcraft_app::tool_assembly::{component, components};
use camcraft_app::world::new_world;
use camcraft_tools::component::ToolComponent;
use camcraft_tools::component_geometry::ComponentGeometry;
use flecs_ecs::prelude::World;
use serde_json::{Value, json};

use super::assembly_cmds::{fly, holder};
use super::call;

fn add(w: &World, h: &mut History, (c, g): (ToolComponent, ComponentGeometry)) {
    h.execute(w, UpsertDocumentComponent::new(w, c, g).expect("upsert"));
}

/// A document with one user holder and one cutter body (not a holder).
fn shop() -> (World, History) {
    let (w, mut h) = (new_world(), History::default());
    add(&w, &mut h, holder());
    add(&w, &mut h, fly("fly-25", 12.5));
    (w, h)
}

fn import(w: &World, h: &mut History, text: &str, reserved: &[String]) -> Result<String, String> {
    let cmd = ImportHolders::from_text(w, text, reserved).map_err(|e| e.to_string())?;
    let s = cmd.summary();
    h.execute(w, cmd);
    Ok(s)
}

/// `text` with every body outline replaced by `points` ([z, r, arc] rows).
fn with_outline(text: &str, points: &Value) -> String {
    fn walk(v: &mut Value, points: &Value) {
        match v {
            Value::Object(o) => o.iter_mut().for_each(|(k, x)| if k == "points" { *x = points.clone() } else { walk(x, points) }),
            Value::Array(a) => a.iter_mut().for_each(|x| walk(x, points)),
            _ => {}
        }
    }
    let mut v: Value = serde_json::from_str(text).expect("a holder file");
    walk(&mut v, points);
    let out = serde_json::to_string_pretty(&v).expect("text");
    assert_ne!(out, serde_json::to_string_pretty(&serde_json::from_str::<Value>(text).expect("json")).expect("text"));
    out
}

#[test]
fn an_export_imports_into_a_new_document_byte_stable_in_one_undo_step() {
    let (src, _) = shop();
    let text = export_holders(&src).expect("export");
    let entries = component_library::parse(&text).expect("a component library file");
    assert_eq!(entries.len(), 1, "the holder only, not the cutter body");
    assert_eq!(entries[0].component.id(), "bt40-cyl20");

    let (dst, mut h) = (new_world(), History::default());
    let s = import(&dst, &mut h, &text, &[]).expect("import");
    assert!(s.starts_with("1 holder imported"), "{s}");
    assert_eq!(component(&dst, "bt40-cyl20"), component(&src, "bt40-cyl20"), "the holder arrives whole");
    assert_eq!(export_holders(&dst).expect("re-export"), text, "export -> import -> export is byte-stable");
    assert!(h.undo(&dst), "one undo step");
    assert!(components(&dst).is_empty(), "undo removes the import");
    assert!(!h.undo(&dst), "the import was one step");
    assert!(h.redo(&dst));
    assert_eq!(components(&dst).len(), 1);
}

#[test]
fn a_held_id_takes_a_new_user_holder_id() {
    let (w, mut h) = shop();
    let text = export_holders(&w).expect("export");
    let before = components(&w).len();
    let s = import(&w, &mut h, &text, &[]).expect("import twice");
    assert!(s.contains("'bt40-cyl20' as 'user:holder:BT40 CYL 20'"), "{s}");
    let copy = component(&w, "user:holder:BT40 CYL 20").expect("the copy");
    assert_eq!(copy.component.name(), "BT40 CYL 20");
    assert_eq!(copy.library_ref, None, "a user holder is not linked");
    // A library id is held too: the next copy steps on.
    let reserved = vec!["user:holder:BT40 CYL 20 (2)".to_owned()];
    let s = import(&w, &mut h, &text, &reserved).expect("import three times");
    assert!(s.contains("'user:holder:BT40 CYL 20 (3)'"), "{s}");
    assert_eq!(components(&w).len(), before + 2);
}

#[test]
fn a_file_that_is_not_correct_is_refused_by_name_with_nothing_applied() {
    let (src, _) = shop();
    let text = export_holders(&src).expect("export");
    let body = LibraryComponent::from_row(component(&src, "fly-25").expect("body")).expect("library row");
    let with_body = component_library::to_text(&[body]).expect("text");
    let empty = component_library::to_text(&[]).expect("text");
    let negative = with_outline(&text, &json!([[0.0, 0.0, 0.0], [0.0, 25.0, 0.0], [50.0, -25.0, 0.0], [50.0, 0.0, 0.0]]));
    let too_few = with_outline(&text, &json!([[0.0, 0.0, 0.0], [50.0, 0.0, 0.0]]));
    let cases: [(&str, String, &str); 8] = [
        ("bad JSON", "{ not json".to_owned(), "bad JSON"),
        ("truncated", text.chars().take(text.chars().count() / 2).collect(), "bad JSON"),
        ("wrong format", text.replace("camcraft-components", "camcraft-tools"), "format is 'camcraft-tools'"),
        ("binary", "\u{0}\u{1}\u{2}".to_owned(), "not a holder file"),
        ("not a holder", with_body, "'fly-25' in the holder file is not a holder"),
        ("no row", empty, "holds no holder"),
        ("outline: a negative radius", negative, "point 3 has a negative radius"),
        ("outline: fewer than 3 points", too_few, "at least 3 points"),
    ];
    for (what, bad, says) in cases {
        let (w, mut h) = shop();
        let before = components(&w);
        let err = import(&w, &mut h, &bad, &[]).expect_err(what);
        assert!(err.contains(says), "{what}: {err}");
        if what.starts_with("outline") {
            assert!(err.contains("'bt40-cyl20'"), "{what}: the refusal names the holder: {err}");
        }
        assert_eq!(components(&w), before, "{what}: the document does not change");
        assert!(h.undo(&w) && h.undo(&w) && !h.undo(&w), "{what}: no history entry");
    }
}

#[test]
fn a_document_with_no_user_holder_is_refused_and_a_builtin_row_is_not_exported() {
    let w = new_world();
    let err = export_holders(&w).expect_err("no holder");
    assert!(err.contains("no user holder"), "{err}");
    let mut h = History::default();
    let builtin = component_library::builtins().into_iter().find(|c| is_holder(&c.component));
    let builtin = builtin.expect("a built-in holder");
    add(&w, &mut h, (builtin.component, builtin.geometry));
    let err = export_holders(&w).expect_err("a built-in only");
    assert!(err.contains("no user holder"), "{err}");
}

#[test]
fn mcp_export_and_import_holders_use_the_one_command() {
    let dir = std::env::temp_dir().join(format!("camcraft-holder-files-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("dir");
    let file = dir.join("holders.camcomponents").display().to_string();
    let library = dir.join("lib.camcomponents").display().to_string();
    let mut s = Server::new();
    s.handle(&json!({ "jsonrpc": "2.0", "id": 0, "method": "initialize", "params": { "protocolVersion": "2025-06-18" } }));
    let (err, msg) = call(&mut s, 1, "export_holders", json!({ "path": file }));
    assert!(err && msg.as_str().is_some_and(|m| m.contains("no user holder")), "{msg}");
    let (c, g) = holder();
    let dto = serde_json::to_value(camcraft_app::project::component_to_dto(&camcraft_app::schema::DocumentComponent::new(c, g)))
        .expect("dto");
    let (err, body) = call(&mut s, 2, "upsert_tool_component", json!({ "component": dto }));
    assert!(!err, "{body}");
    let (err, body) = call(&mut s, 3, "export_holders", json!({ "path": file }));
    assert!(!err && body["holders"] == json!(1), "{body}");
    let (err, body) = call(&mut s, 4, "import_holders", json!({ "path": file, "library": library }));
    assert!(!err, "{body}");
    assert_eq!(body["imported"][0]["id"], json!("user:holder:BT40 CYL 20"), "{body}");
    let (err, _) = call(&mut s, 5, "undo", json!({}));
    assert!(!err);
    let (_, list) = call(&mut s, 6, "list_tool_components", json!({ "library": library }));
    assert!(!list.to_string().contains("user:holder:"), "one undo removes the import: {list}");
    std::fs::write(dir.join("bad.camcomponents"), "{ not json").expect("write");
    let bad = dir.join("bad.camcomponents").display().to_string();
    let (err, msg) = call(&mut s, 7, "import_holders", json!({ "path": bad, "library": library }));
    assert!(err && msg.as_str().is_some_and(|m| m.contains("not a holder file")), "{msg}");
    // An unreadable library is refused by name: its ids cannot be kept clear.
    let (err, msg) = call(&mut s, 8, "import_holders", json!({ "path": file, "library": bad }));
    assert!(err && msg.as_str().is_some_and(|m| m.contains("component library is not read")), "{msg}");
    let (_, list) = call(&mut s, 9, "list_tool_components", json!({ "library": library }));
    assert!(!list.to_string().contains("user:holder:"), "nothing is applied: {list}");
    let _ = std::fs::remove_dir_all(&dir);
}
