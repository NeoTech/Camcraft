//! #4370: `RenameTool` gives a document tool a new name and moves each
//! operation that uses it, in one undo step; `rename_tool` is its MCP
//! door. Pinned: the tool keeps its definition, number, body, stack and
//! cutter under the new name; its operation follows and stays Ok; undo
//! restores every reference and redo lands the same state; an empty
//! name, a clash, an unknown name, a non-tool, the same name, a setup
//! name and a name that a dangling operation holds are refused by name
//! and change nothing.
//! Provenance: our own tests over our own commands.

use camcraft_app::commands::{DeleteTool, RenameTool};
use camcraft_app::mcp::Server;
use camcraft_app::model;
use camcraft_app::operation::{OpStatus, status_of};
use camcraft_app::tool_assembly::assembly_of;
use camcraft_app::tool_body::body_of;
use camcraft_app::tool_cutter::cutter_of;
use flecs_ecs::prelude::World;
use serde_json::json;

use super::assembly_cmds::{A, B, shop};
use super::call;

const NEW: &str = "Fly25 roughing";

/// The tool's records and its number, read under `name`.
fn records(w: &World, name: &str) -> (camcraft_app::tools::Tool, String, String, String, Option<u32>) {
    let e = model::find_by_name(w, name).expect("tool");
    (
        model::tool_of(w, &e).expect("a tool"),
        format!("{:?}", body_of(w, &e)),
        format!("{:?}", assembly_of(w, &e)),
        format!("{:?}", cutter_of(w, &e)),
        model::tool_number_of(w, &e),
    )
}

/// The tool name the operation holds, and its status.
fn op(w: &World, name: &str) -> (String, OpStatus) {
    let e = model::find_by_name(w, name).expect("operation");
    (model::operation_of(w, &e).expect("an operation").tool, status_of(w, &e))
}

#[test]
fn rename_moves_the_tool_and_its_operation_in_one_undo_step() {
    let (w, mut h) = shop();
    let src = records(&w, A);
    let face_a = format!("Face{A}");
    let face_b = format!("Face{B}");
    assert_eq!(op(&w, &face_a), (A.to_string(), OpStatus::Ok), "the operation is generated and up to date");
    let other = op(&w, &face_b);
    let done = h.done_len();

    let cmd = RenameTool::new(&w, A, &format!("  {NEW} ")).expect("rename");
    assert_eq!((cmd.from(), cmd.to()), (A, NEW), "the new name is trimmed");
    assert_eq!(cmd.operations(), [face_a.clone()]);
    h.execute(&w, cmd);
    assert_eq!(h.done_len(), done + 1, "one undo step");
    assert!(model::find_by_name(&w, A).is_none(), "the old name is free");
    assert_eq!(records(&w, NEW), src, "the tool keeps every record and its number");
    assert_eq!(op(&w, &face_a), (NEW.to_string(), OpStatus::Ok), "the operation follows and stays up to date");
    assert_eq!(op(&w, &face_b), other, "another tool's operation does not change");

    assert!(h.undo(&w));
    assert!(model::find_by_name(&w, NEW).is_none(), "undo frees the new name");
    assert_eq!(records(&w, A), src);
    assert_eq!(op(&w, &face_a), (A.to_string(), OpStatus::Ok), "undo restores the reference");
    assert!(h.redo(&w));
    assert_eq!(records(&w, NEW), src, "redo lands the same rename");
    assert_eq!(op(&w, &face_a), (NEW.to_string(), OpStatus::Ok));
}

#[test]
fn a_stale_operation_stays_stale_through_a_rename() {
    let (w, mut h) = shop();
    let face_a = format!("Face{A}");
    let mut t = model::tool_of(&w, &model::find_by_name(&w, A).expect("tool")).expect("a tool");
    t.flute_len += 1.0;
    h.execute(&w, camcraft_app::commands::ChangeTool::new(&w, A, t).expect("edit"));
    assert_eq!(op(&w, &face_a).1, OpStatus::Stale, "the edit makes the operation stale");
    h.execute(&w, RenameTool::new(&w, A, NEW).expect("rename"));
    assert_eq!(op(&w, &face_a), (NEW.to_string(), OpStatus::Stale), "a rename does not make a stale path fresh");
    assert!(h.undo(&w));
    assert_eq!(op(&w, &face_a), (A.to_string(), OpStatus::Stale));
}

#[test]
fn rename_refuses_by_name_and_changes_nothing() {
    let (w, mut h) = shop();
    let refused = |w: &World, from: &str, to: &str| RenameTool::new(w, from, to).err().expect("refused").to_string();
    let done = h.done_len();
    assert!(refused(&w, A, "   ").contains("must not be empty"), "an empty name");
    assert!(refused(&w, A, B).contains(&format!("'{B}' already exists")), "a clash");
    assert!(refused(&w, "Nope", NEW).contains("'Nope'"), "an unknown name");
    assert!(refused(&w, &format!("Face{A}"), NEW).contains(&format!("'Face{A}' is not a tool")), "a non-tool");
    assert!(refused(&w, A, A).contains("already has that name"), "the same name");
    assert!(refused(&w, A, "Stock").contains("reserved for a setup"), "a setup name");
    assert!(refused(&w, A, "T\u{7}").contains("control character"), "a control character");
    assert_eq!(h.done_len(), done, "a refusal changes nothing");
    // A deleted tool's operation still names it: the rename must not hand
    // it this tool.
    h.execute(&w, DeleteTool::new(&w, B).expect("delete"));
    let dangling = refused(&w, A, B);
    assert!(dangling.contains(&format!("operation 'Face{B}' still names a tool '{B}'")), "{dangling}");
    assert!(model::find_by_name(&w, A).is_some());
}

#[test]
fn mcp_rename_tool_runs_the_command_and_undo_restores_it() {
    let mut s = Server::new();
    s.handle(&json!({ "jsonrpc": "2.0", "id": 0, "method": "initialize", "params": { "protocolVersion": "2025-06-18" } }));
    let (err, r) = call(&mut s, 1, "add_tool", json!({ "name": "T6", "catalog": "EM-0600-4FL", "holder": "BT30 ER20" }));
    assert!(!err, "{r}");
    let (_, job) = call(&mut s, 2, "get_job", json!({}));
    let row = |job: &serde_json::Value, n: &str| {
        let tools = job["tools"].as_array().expect("tools");
        let mut r = tools.iter().find(|t| t["name"] == json!(n)).cloned().expect("tool row");
        if let Some(o) = r.as_object_mut() {
            o.remove("name");
        }
        r
    };
    let before = row(&job, "T6");
    let (err, r) = call(&mut s, 3, "rename_tool", json!({ "name": "T6", "new_name": "Finisher" }));
    assert!(!err, "{r}");
    assert_eq!((r["from"].clone(), r["tool"].clone(), r["operations"].clone()), (json!("T6"), json!("Finisher"), json!([])));
    let (_, job) = call(&mut s, 4, "get_job", json!({}));
    assert_eq!(row(&job, "Finisher"), before, "the tool keeps its number and holder");
    assert!(!job["tools"].as_array().expect("tools").iter().any(|t| t["name"] == json!("T6")));
    let (err, _) = call(&mut s, 5, "undo", json!({}));
    assert!(!err);
    let (_, job) = call(&mut s, 6, "get_job", json!({}));
    assert_eq!(row(&job, "T6"), before, "one undo restores the old name");
    let (err, why) = call(&mut s, 7, "rename_tool", json!({ "name": "T6", "new_name": "" }));
    assert!(err && why.as_str().unwrap_or_default().contains("new_name"), "an empty argument is refused by name: {why}");
    let (err, why) = call(&mut s, 8, "rename_tool", json!({ "name": "T6", "new_name": "  " }));
    assert!(err && why.as_str().unwrap_or_default().contains("must not be empty"), "a blank name is refused by the command: {why}");
}
