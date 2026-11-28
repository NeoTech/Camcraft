//! #4340: `DuplicateTool` copies a document tool as a new tool with its
//! holder body and its component stack, under a free name and the next
//! free tool number, in one undo step; `duplicate_tool` is its MCP door.
//! Pinned: the copy of a stacked cutter carries the source's definition,
//! body and stack; the copy of a library-held tool carries the holder;
//! one undo removes the copy and redo lands it again identically; the
//! names count up from the stem; an unknown name or a non-tool is
//! refused by name and changes nothing.
//! Provenance: our own tests over our own commands.

use camcraft_app::commands::DuplicateTool;
use camcraft_app::model;
use camcraft_app::tool_assembly::assembly_of;
use camcraft_app::tool_body::body_of;
use camcraft_app::tool_cutter::cutter_of;
use camcraft_app::mcp::Server;
use flecs_ecs::prelude::World;
use serde_json::json;

use super::assembly_cmds::{A, B, shop};
use super::{DRAWN, call, source};

/// Everything a copy must carry, with the number apart.
type Records = (camcraft_app::tools::Tool, String, String, String, Option<u32>);

fn records(w: &World, name: &str) -> Records {
    let e = model::find_by_name(w, name).expect("tool");
    (
        model::tool_of(w, &e).expect("a tool"),
        format!("{:?}", body_of(w, &e)),
        format!("{:?}", assembly_of(w, &e)),
        format!("{:?}", cutter_of(w, &e)),
        model::tool_number_of(w, &e),
    )
}

fn without_number(r: &Records) -> (&camcraft_app::tools::Tool, &str, &str, &str) {
    (&r.0, &r.1, &r.2, &r.3)
}

#[test]
fn duplicate_copies_the_stack_and_the_body_in_one_undo_step() {
    let (w, mut h) = shop();
    let src = records(&w, A);
    assert!(src.2 != "None", "the source has a stack");
    let cmd = DuplicateTool::new(&w, A).expect("duplicate");
    assert_eq!(cmd.name(), "Fly25 (2)");
    let number = cmd.number();
    let taken = [records(&w, A).4, records(&w, B).4];
    assert!(!taken.contains(&Some(number)), "the copy takes a free number, not {taken:?}");
    h.execute(&w, cmd);

    let copy = records(&w, "Fly25 (2)");
    assert_eq!(without_number(&copy), without_number(&src), "the copy carries the definition, body, stack and cutter");
    assert_eq!(copy.4, Some(number));
    assert_eq!(records(&w, A), src, "the source does not change");

    assert!(h.undo(&w));
    assert!(model::find_by_name(&w, "Fly25 (2)").is_none(), "one undo removes the copy");
    assert_eq!(records(&w, A), src);
    assert!(h.redo(&w));
    assert_eq!(records(&w, "Fly25 (2)"), copy, "redo lands the same copy");
}

#[test]
fn duplicate_keeps_a_library_holder() {
    let (w, mut h) = source();
    let src = records(&w, DRAWN);
    assert!(src.1.contains("BT30 ER20"), "the source has the library holder: {}", src.1);
    let cmd = DuplicateTool::new(&w, DRAWN).expect("duplicate");
    let name = cmd.name().to_string();
    h.execute(&w, cmd);
    assert_eq!(without_number(&records(&w, &name)), without_number(&src));
}

#[test]
fn copy_names_count_up_from_the_stem() {
    let (w, mut h) = shop();
    let first = DuplicateTool::new(&w, A).expect("first");
    h.execute(&w, first);
    let again = DuplicateTool::new(&w, A).expect("second");
    assert_eq!(again.name(), "Fly25 (3)");
    let of_copy = DuplicateTool::new(&w, "Fly25 (2)").expect("copy of the copy");
    assert_eq!(of_copy.name(), "Fly25 (3)", "the copy of a copy counts on, not 'Fly25 (2) (2)'");
    assert_eq!(of_copy.source(), "Fly25 (2)");
}

#[test]
fn duplicate_refuses_an_unknown_name_and_a_non_tool() {
    let (w, h) = shop();
    let before = h.done_len();
    let unknown = DuplicateTool::new(&w, "Nope").err().expect("unknown refused").to_string();
    assert!(unknown.contains("Nope"), "{unknown}");
    let op = DuplicateTool::new(&w, "FaceFly25").err().expect("operation refused").to_string();
    assert!(op.contains("'FaceFly25' is not a tool"), "{op}");
    assert_eq!(h.done_len(), before, "a refusal changes nothing");
}

#[test]
fn mcp_duplicate_tool_runs_the_command_and_undo_removes_it() {
    let mut s = Server::new();
    s.handle(&json!({ "jsonrpc": "2.0", "id": 0, "method": "initialize", "params": { "protocolVersion": "2025-06-18" } }));
    let (err, r) = call(&mut s, 1, "add_tool", json!({ "name": "T6", "catalog": "EM-0600-4FL", "holder": "BT30 ER20" }));
    assert!(!err, "{r}");
    let (err, r) = call(&mut s, 2, "duplicate_tool", json!({ "name": "T6" }));
    assert!(!err, "{r}");
    assert_eq!(r["source"], json!("T6"));
    assert_eq!(r["tool"], json!("T6 (2)"));
    assert_eq!(r["number"], json!(2), "the next free number: {r}");
    let (_, job) = call(&mut s, 3, "get_job", json!({}));
    let tools = job["tools"].as_array().expect("tools");
    let row = |n: &str| {
        let mut r = tools.iter().find(|t| t["name"] == json!(n)).cloned().expect("tool row");
        if let Some(o) = r.as_object_mut() {
            for k in ["name", "number", "posts_as"] {
                o.remove(k);
            }
        }
        r
    };
    assert_eq!(row("T6 (2)"), row("T6"), "the copy has the definition and the holder");
    assert!(job.to_string().contains("BT30 ER20"), "the holder shows: {job}");
    let (err, _) = call(&mut s, 4, "undo", json!({}));
    assert!(!err);
    let (_, job) = call(&mut s, 5, "get_job", json!({}));
    assert!(!job["tools"].as_array().expect("tools").iter().any(|t| t["name"] == json!("T6 (2)")), "one undo removes the copy");
    let (err, why) = call(&mut s, 6, "duplicate_tool", json!({ "name": "Nope" }));
    assert!(err && why.as_str().unwrap_or_default().contains("Nope"), "{why}");
}
