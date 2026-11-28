//! #4339: standalone tools converted to component stacks in one step.
//! A document of standalone end mills: two on a BT30 ER20 holder with
//! 10 mm of shank in the holder, one with a longer gauge length than the
//! built-in holder (a narrower stack outline: no match), one with the old default stick-out (no shank in the
//! holder), one stuck out 5 mm past its shank's end, one with no holder,
//! and one with a 6.35 mm shank that no whole-millimetre collet takes. Pinned: the preview names each verdict
//! (same, different with its differences, no match with its reason); the
//! "same" rows convert in one undo step, and undo restores every tool and
//! the component table exactly; a "different" row is not converted by
//! default and converts only when named; a "no match" row refuses the
//! whole command by name; the MCP dry run returns the preview and changes
//! nothing. The posted program and the holder outline of each converted
//! row kind are pinned in `convert_outline`.
//! Provenance: our own tests over our own commands.

use camcraft_app::commands::{
    AddTool, ConvertToComponents, CreateOperation, GenerateOperation, History, SetStock, SetToolBody,
};
use camcraft_app::mcp::Server;
use camcraft_app::model;
use camcraft_app::operation::{Height, OpParams, SrcRef};
use camcraft_app::schema::{DocumentComponent, ToolAssembly, ToolBody};
use camcraft_app::tool_assembly::{assembly_of, components};
use camcraft_app::tool_body::body_of;
use camcraft_app::tool_convert::{plan, Verdict};
use camcraft_app::tools::Tool;
use flecs_ecs::prelude::World;
use serde_json::json;

use super::{call, cat};

pub(super) const SIX: &str = "Six";
pub(super) const SIX_B: &str = "Six B";
pub(super) const LONG: &str = "Long";
pub(super) const BARE: &str = "Bare";
pub(super) const INCH: &str = "Inch";
pub(super) const LEGACY: &str = "Legacy";
pub(super) const DEEP: &str = "Deep";
pub(super) const ALL: [&str; 7] = [SIX, SIX_B, LONG, BARE, INCH, LEGACY, DEEP];
const HOLDER: &str = "BT30 ER20";
/// How far past its shank's end `Deep` sticks out (mm).
pub(super) const DEEP_GAP: f64 = 5.0;

pub(super) fn face() -> OpParams {
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

/// The seven standalone tools, stock, and a generated Face operation on
/// each of `Six`, `Legacy` and `Deep` (one "same" row, the two kinds of
/// "different" shank row).
pub(super) fn shop() -> (World, History) {
    let w = camcraft_app::world::new_world();
    let mut h = History::default();
    let stock = camcraft_app::stock::StockShape::Box { dx: 60.0, dy: 60.0, dz: 20.0 };
    h.execute(&w, SetStock::new(&w, stock, Default::default()).expect("stock"));
    let inch = Tool { shank_d: 6.35, ..cat("EM-0600-4FL") };
    let six = cat("EM-0600-4FL");
    let tools = [
        (SIX, six.clone()),
        (SIX_B, six.clone()),
        (LONG, six.clone()),
        (LEGACY, six.clone()),
        (DEEP, six.clone()),
        (BARE, cat("EM-0800-4FL")),
        (INCH, inch),
    ];
    for (name, tool) in tools {
        h.execute(&w, AddTool::new(&w, name, tool).expect("tool"));
    }
    let held = Some(six.flute_len + six.shank_len - 10.0);
    for name in [SIX, SIX_B] {
        h.execute(&w, SetToolBody::library(&w, name, HOLDER, None, held).expect("holder"));
    }
    for name in [INCH, LEGACY] {
        h.execute(&w, SetToolBody::library(&w, name, HOLDER, None, None).expect("default holder"));
    }
    let gauge = camcraft_app::holder_library::find(HOLDER).expect("entry").gauge_len;
    h.execute(&w, SetToolBody::library(&w, LONG, HOLDER, Some(gauge + 5.0), held).expect("long holder"));
    let deep = Some(six.flute_len + six.shank_len + DEEP_GAP);
    h.execute(&w, SetToolBody::library(&w, DEEP, HOLDER, None, deep).expect("deep holder"));
    for (op, tool) in OPS {
        h.execute(&w, CreateOperation::new(&w, op, tool, SrcRef::Stock, face()).expect("op"));
        h.execute(&w, GenerateOperation::new(&w, op).expect("generate"));
    }
    (w, h)
}

/// The operations of `shop`, with their tools.
pub(super) const OPS: [(&str, &str); 3] = [("FaceSix", SIX), ("FaceLegacy", LEGACY), ("FaceDeep", DEEP)];

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
    assert_eq!(tools, vec![BARE, DEEP, INCH, LEGACY, LONG, SIX, SIX_B], "every standalone tool, sorted");
    let six = rows.iter().find(|r| r.tool == SIX).expect("six");
    assert_eq!(six.verdict, Verdict::Same, "{six:?}");
    assert_eq!(six.names, vec!["BT30 ER20".to_owned(), "ER20 collet 8 mm".to_owned(), SIX.to_owned()]);
    // A longer gauge length than the built-in holder: the stack's holder
    // is shorter, so its outline is narrower where the old flange was.
    let Verdict::NoMatch(why) = verdict(&w, LONG) else { panic!("Long has a narrower holder outline") };
    assert!(why.contains("narrower than the library holder"), "{why}");
    // The old default stick-out holds no shank: one shank diameter more, the same gauge.
    let Verdict::Different(diffs) = verdict(&w, LEGACY) else { panic!("Legacy is different") };
    let six = cat("EM-0600-4FL");
    let want = format!("shank length {:.3} -> {:.3} mm", six.shank_len, six.shank_len + six.shank_d);
    assert_eq!(diffs, vec![want], "only the shank, never the gauge length");
    // Stuck out past the shank's end: the shank fills the gap and one shank diameter more.
    let Verdict::Different(diffs) = verdict(&w, DEEP) else { panic!("Deep is different") };
    let want = format!("shank length {:.3} -> {:.3} mm", six.shank_len, six.shank_len + DEEP_GAP + six.shank_d);
    assert_eq!(diffs, vec![want], "only the shank, never the gauge length");
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
    assert_eq!(ids, vec!["builtin:holder:BT30 ER20", "builtin:collet:ER20 8", "user:tool:Six", "user:tool:Six B"]);
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
    assert!(stack_of(&w, DEEP).is_none() && stack_of(&w, LEGACY).is_none(), "the default selection leaves the different rows");
    let cmd = ConvertToComponents::new(&w, &[DEEP.to_owned(), LEGACY.to_owned()]).expect("named");
    h.execute(&w, cmd);
    assert!(stack_of(&w, DEEP).is_some() && stack_of(&w, LEGACY).is_some(), "named different rows convert");
    let e = model::find_by_name(&w, LEGACY).expect("tool");
    let six = cat("EM-0600-4FL");
    let shank = model::tool_of(&w, &e).expect("tool").shank_len;
    assert!((shank - (six.shank_len + six.shank_d)).abs() < 1e-9, "the shown difference lands: {shank}");
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

#[test]
fn mcp_dry_run_returns_the_preview_and_apply_converts_the_named_rows() {
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
    assert_eq!(results, vec![(BARE.to_owned(), "no match".to_owned()), (SIX.to_owned(), "different".to_owned())], "{body}");
    let (err, body) = ok(&mut s, "get_tool_assembly", json!({ "tool": SIX }));
    assert!(err, "the dry run changes nothing: {body}");

    let (err, body) = ok(&mut s, "convert_tools_to_components", json!({ "tools": [BARE] }));
    assert!(err && body.as_str().unwrap_or("").contains("no holder"), "{body}");
    let (err, body) = ok(&mut s, "convert_tools_to_components", json!({}));
    assert!(err, "no 'same' row: the default converts nothing: {body}");
    let (err, body) = ok(&mut s, "convert_tools_to_components", json!({ "tools": [SIX] }));
    assert!(!err, "{body}");
    assert_eq!(body["converted"], json!([SIX]), "{body}");
    let (err, body) = ok(&mut s, "get_tool_assembly", json!({ "tool": SIX }));
    assert!(!err && body["nodes"].as_array().map(Vec::len) == Some(3), "{body}");
}
