//! #4339: standalone tools converted to component stacks in one step.
//! A document of standalone end mills: two on a BT30 ER20 holder as the
//! library states it, one with a longer gauge length than the built-in
//! holder, one with no holder, and one with a 6.35 mm shank that no
//! whole-millimetre collet takes. Pinned: the preview names each verdict
//! (same, different with its differences, no match with its reason); the
//! "same" rows convert in one undo step, and undo restores every tool and
//! the component table exactly; a "different" row is not converted by
//! default and converts only when named; a "no match" row refuses the
//! whole command by name; the posted program of a converted tool does not
//! change; the MCP dry run returns the preview and changes nothing.
//! Provenance: our own tests over our own commands.

use camcraft_app::commands::{
    AddTool, ConvertToComponents, CreateOperation, GenerateOperation, History, SetStock, SetToolBody,
};
use camcraft_app::mcp::Server;
use camcraft_app::model;
use camcraft_app::operation::{Height, OpParams, OpStatus, SrcRef, status_of};
use camcraft_app::posting;
use camcraft_app::schema::{DocumentComponent, ToolAssembly, ToolBody};
use camcraft_app::tool_assembly::{assembly_of, components};
use camcraft_app::tool_body::body_of;
use camcraft_app::tool_convert::{plan, Verdict};
use camcraft_app::tools::Tool;
use flecs_ecs::prelude::World;
use serde_json::json;

use super::{call, cat};

const SIX: &str = "Six";
const SIX_B: &str = "Six B";
const LONG: &str = "Long";
const BARE: &str = "Bare";
const INCH: &str = "Inch";
const ALL: [&str; 5] = [SIX, SIX_B, LONG, BARE, INCH];
const HOLDER: &str = "BT30 ER20";

fn face() -> OpParams {
    OpParams::Face {
        holes: Default::default(),
        tilt: None,
        blend_turns: false,
        blend_radius: None,
        stepover: 3.0,
        clearance_z: Height::fixed(25.0),
        cut_z: Height::fixed(19.5),
        top_z: None,
        stepdown: None,
        feed: 1200.0,
        plunge_feed: 300.0,
        climb: true,
        picks: Default::default(),
    }
}

/// The five standalone tools, stock, and a generated Face operation on
/// `Six`.
fn shop() -> (World, History) {
    let w = camcraft_app::world::new_world();
    let mut h = History::default();
    let stock = camcraft_app::stock::StockShape::Box { dx: 60.0, dy: 60.0, dz: 20.0 };
    h.execute(&w, SetStock::new(&w, stock, Default::default()).expect("stock"));
    let inch = Tool { shank_d: 6.35, ..cat("EM-0600-4FL") };
    for (name, tool) in [(SIX, cat("EM-0600-4FL")), (SIX_B, cat("EM-0600-4FL")), (LONG, cat("EM-0600-4FL")), (BARE, cat("EM-0800-4FL")), (INCH, inch)] {
        h.execute(&w, AddTool::new(&w, name, tool).expect("tool"));
    }
    for name in [SIX, SIX_B, INCH] {
        h.execute(&w, SetToolBody::library(&w, name, HOLDER, None, None).expect("holder"));
    }
    let gauge = camcraft_app::holder_library::find(HOLDER).expect("entry").gauge_len;
    h.execute(&w, SetToolBody::library(&w, LONG, HOLDER, Some(gauge + 5.0), None).expect("long holder"));
    h.execute(&w, CreateOperation::new(&w, "FaceSix", SIX, SrcRef::Stock, face()).expect("op"));
    h.execute(&w, GenerateOperation::new(&w, "FaceSix").expect("generate"));
    (w, h)
}

type Snapshot = (Vec<DocumentComponent>, Vec<(Tool, Option<ToolBody>, Option<ToolAssembly>)>);

fn snapshot(w: &World) -> Snapshot {
    let tool = |n: &str| {
        let e = model::find_by_name(w, n).expect("tool");
        (model::tool_of(w, &e).expect("a tool"), body_of(w, &e), assembly_of(w, &e))
    };
    (components(w), ALL.iter().map(|n| tool(n)).collect())
}

fn stack_of(w: &World, name: &str) -> Option<ToolAssembly> {
    assembly_of(w, &model::find_by_name(w, name).expect("tool"))
}

fn verdict(w: &World, name: &str) -> Verdict {
    plan(w, &[name.to_owned()]).expect("plan").remove(0).verdict
}

#[test]
fn the_preview_names_each_verdict() {
    let (w, _) = shop();
    let rows = plan(&w, &[]).expect("plan");
    let tools: Vec<&str> = rows.iter().map(|r| r.tool.as_str()).collect();
    assert_eq!(tools, vec![BARE, INCH, LONG, SIX, SIX_B], "every standalone tool, sorted");
    let six = rows.iter().find(|r| r.tool == SIX).expect("six");
    assert_eq!(six.verdict, Verdict::Same, "{six:?}");
    assert_eq!(six.names, vec!["BT30 ER20".to_owned(), "ER20 collet 6 mm".to_owned(), SIX.to_owned()]);
    let Verdict::Different(diffs) = verdict(&w, LONG) else { panic!("Long is different") };
    assert_eq!(diffs.len(), 1, "{diffs:?}");
    assert!(diffs[0].starts_with("gauge length"), "{diffs:?}");
    let Verdict::NoMatch(why) = verdict(&w, BARE) else { panic!("Bare has no holder") };
    assert!(why.contains("no holder"), "{why}");
    let Verdict::NoMatch(why) = verdict(&w, INCH) else { panic!("Inch has no collet") };
    assert!(why.contains("no collet for the shank 6.35 mm"), "{why}");
    assert!(plan(&w, &["Nope".to_owned()]).is_err(), "an unknown tool is refused");
}

#[test]
fn the_same_rows_convert_in_one_undo_step_and_undo_restores_every_tool() {
    let (w, mut h) = shop();
    let before = snapshot(&w);
    let cmd = ConvertToComponents::same_rows(&w).expect("same rows");
    assert_eq!(cmd.tools(), vec![SIX.to_owned(), SIX_B.to_owned()]);
    h.execute(&w, cmd);
    let after = snapshot(&w);
    for (i, name) in ALL.iter().enumerate() {
        let converted = [SIX, SIX_B].contains(name);
        assert_eq!(after.1[i].2.is_some(), converted, "{name}: stack");
        assert_eq!(after.1[i].0, before.1[i].0, "{name}: the tool record stays the same");
    }
    // The shared holder and collet are one row each; each tool has its own cutting item.
    let ids: Vec<&str> = after.0.iter().map(|r| r.component.id()).collect();
    assert_eq!(ids, vec!["builtin:holder:BT30 ER20", "builtin:collet:ER20 6", "user:tool:Six", "user:tool:Six B"]);
    assert!(after.0[0].library_ref.is_some() && after.0[1].library_ref.is_some(), "built-in copies keep their link");

    assert!(h.undo(&w), "one undo step");
    assert_eq!(snapshot(&w), before, "undo restores every tool and the table exactly");
    assert!(h.redo(&w));
    assert_eq!(snapshot(&w), after, "redo lands the same state");
}

#[test]
fn a_different_row_converts_only_when_it_is_named() {
    let (w, mut h) = shop();
    h.execute(&w, ConvertToComponents::same_rows(&w).expect("same rows"));
    assert!(stack_of(&w, LONG).is_none(), "the default selection leaves a different row");
    let cmd = ConvertToComponents::new(&w, &[LONG.to_owned()]).expect("named");
    h.execute(&w, cmd);
    assert!(stack_of(&w, LONG).is_some(), "a named different row converts");
}

#[test]
fn a_no_match_row_refuses_the_whole_command_by_name() {
    let (w, _) = shop();
    let before = snapshot(&w);
    let Err(e) = ConvertToComponents::new(&w, &[SIX.to_owned(), BARE.to_owned()]) else { panic!("must refuse") };
    let msg = e.to_string();
    assert!(msg.contains("'Bare' is not converted") && msg.contains("no holder"), "{msg}");
    assert!(ConvertToComponents::new(&w, &[]).is_err(), "an empty selection");
    assert!(ConvertToComponents::new(&w, &[SIX.to_owned(), SIX.to_owned()]).is_err(), "a name twice");
    assert_eq!(snapshot(&w), before, "nothing changed");
}

fn post(w: &World, tag: &str) -> String {
    let out = std::env::temp_dir().join(format!("camcraft-4339-{tag}-{}.nc", std::process::id()));
    let _ = std::fs::remove_file(&out);
    posting::post_setup(w, &out).expect("posted");
    let text = std::fs::read_to_string(&out).expect("program");
    let _ = std::fs::remove_file(&out);
    text
}

#[test]
fn the_posted_program_of_a_converted_tool_does_not_change() {
    let (w, mut h) = shop();
    let before = post(&w, "before");
    h.execute(&w, ConvertToComponents::new(&w, &[SIX.to_owned()]).expect("convert"));
    let op = model::find_by_name(&w, "FaceSix").expect("op");
    if status_of(&w, &op) == OpStatus::Stale {
        h.execute(&w, GenerateOperation::new(&w, "FaceSix").expect("generate again"));
    }
    assert_eq!(post(&w, "before"), before, "the same program after the conversion");
}

#[test]
fn mcp_dry_run_returns_the_preview_and_apply_converts_the_same_rows() {
    let mut s = Server::new();
    s.handle(&json!({ "jsonrpc": "2.0", "id": 0, "method": "initialize", "params": { "protocolVersion": "2025-06-18" } }));
    let mut id = 0;
    let mut ok = |s: &mut Server, name: &str, args: serde_json::Value| {
        id += 1;
        let (err, body) = call(s, id, name, args);
        (err, body)
    };
    let def = camcraft_app::project::tool_to_json(&cat("EM-0600-4FL"));
    for (name, holder) in [(SIX, HOLDER), (BARE, "none")] {
        let (err, body) = ok(&mut s, "add_tool", json!({ "name": name, "definition": def, "holder": holder }));
        assert!(!err, "{body}");
    }
    let (err, body) = ok(&mut s, "convert_tools_to_components", json!({ "dry_run": true }));
    assert!(!err, "{body}");
    let results: Vec<(String, String)> = body["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .map(|r| (r["tool"].as_str().unwrap_or("").to_owned(), r["result"].as_str().unwrap_or("").to_owned()))
        .collect();
    assert_eq!(results, vec![(BARE.to_owned(), "no match".to_owned()), (SIX.to_owned(), "same".to_owned())], "{body}");
    let (err, body) = ok(&mut s, "get_tool_assembly", json!({ "tool": SIX }));
    assert!(err, "the dry run changes nothing: {body}");

    let (err, body) = ok(&mut s, "convert_tools_to_components", json!({ "tools": [BARE] }));
    assert!(err && body.as_str().unwrap_or("").contains("no holder"), "{body}");
    let (err, body) = ok(&mut s, "convert_tools_to_components", json!({}));
    assert!(!err, "{body}");
    assert_eq!(body["converted"], json!([SIX]), "{body}");
    let (err, body) = ok(&mut s, "get_tool_assembly", json!({ "tool": SIX }));
    assert!(!err && body["nodes"].as_array().map(Vec::len) == Some(3), "{body}");
}
