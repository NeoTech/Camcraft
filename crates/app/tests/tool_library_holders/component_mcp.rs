//! #3647: the tool component, assembly and component library MCP tools,
//! driven only through the MCP door over the #3644 shop (one shared
//! insert in two indexable cutters on one BT40 holder). Pinned: the
//! compatibility query lists the cutter bodies that take the insert; an
//! assembly built by `set_tool_assembly` reads back through
//! `get_tool_assembly`; an edit of the shared insert updates both tools
//! in one undo step; a save to a library file, the sync and its undo;
//! the named refusals, which change nothing; an upsert carries an
//! insert's running data (#3646) and a library save refuses it; an
//! upsert carries a lathe body (#3643) the same way.
//! Provenance: our own tests over our own tools.

use camcraft_app::mcp::Server;
use camcraft_app::project::{component_to_dto, tool_to_json};
use camcraft_app::schema::DocumentComponent;
use camcraft_app::tools::{Tool, ToolKind};
use camcraft_tools::component::ToolComponent;
use camcraft_tools::component_geometry::ComponentGeometry;
use serde_json::{Value, json};

use super::assembly_cmds::{A, B, INSERT, fly, holder, insert};
use super::{call, cat};

struct Mcp {
    s: Server,
    id: i64,
}

impl Mcp {
    fn call(&mut self, name: &str, args: Value) -> (bool, Value) {
        self.id += 1;
        call(&mut self.s, self.id, name, args)
    }

    fn ok(&mut self, name: &str, args: Value) -> Value {
        let (err, body) = self.call(name, args);
        assert!(!err, "{name}: {body}");
        body
    }

    fn refused(&mut self, name: &str, args: Value) -> String {
        let (err, body) = self.call(name, args);
        assert!(err, "{name} must be refused: {body}");
        body.as_str().unwrap_or_default().to_owned()
    }
}

fn dto((c, g): (ToolComponent, ComponentGeometry)) -> Value {
    serde_json::to_value(component_to_dto(&DocumentComponent::new(c, g))).expect("dto")
}

fn library(tag: &str) -> String {
    let p = std::env::temp_dir().join(format!("camcraft-mcp-components-{}-{tag}.camcomponents", std::process::id()));
    let _ = std::fs::remove_file(&p);
    p.display().to_string()
}

fn links(body: &str) -> Value {
    json!([
        { "component": "bt40-cyl20" },
        { "component": body, "mount": { "parent": 0 } },
        { "component": INSERT, "mount": { "parent": 1, "port": "CSW1" } },
    ])
}

/// The four components and both tools on their stacks, all by MCP.
fn shop() -> Mcp {
    let mut m = Mcp { s: Server::new(), id: 0 };
    m.s.handle(&json!({ "jsonrpc": "2.0", "id": 0, "method": "initialize", "params": { "protocolVersion": "2025-06-18" } }));
    for c in [holder(), fly("fly-25", 12.5), fly("fly-32", 16.0), insert("CCMT09T304")] {
        let r = m.ok("upsert_tool_component", json!({ "component": dto(c) }));
        assert_eq!(r["added"], json!(true), "{r}");
    }
    let cutter = Tool { kind: ToolKind::BullNose, diameter: 20.0, corner_r: 0.2, flute_len: 5.0, shank_d: 20.0, ..cat("EM-1200-2FL") };
    for (tool, body) in [(A, "fly-25"), (B, "fly-32")] {
        m.ok("add_tool", json!({ "name": tool, "definition": tool_to_json(&cutter), "holder": "none" }));
        m.ok("set_tool_assembly", json!({ "tool": tool, "links": links(body) }));
    }
    m
}

fn corner_r(m: &mut Mcp, tool: &str) -> f64 {
    m.ok("get_tool_assembly", json!({ "tool": tool }))["cutting"]["corner_r"].as_f64().expect("corner_r")
}

fn ids(rows: &Value) -> Vec<String> {
    rows["components"].as_array().expect("rows").iter().map(|r| r["component"]["id"].as_str().unwrap_or("").to_owned()).collect()
}

#[test]
fn mcp_lists_compatible_components_and_reads_a_built_assembly() {
    let mut m = shop();
    let fits = m.ok("list_tool_components", json!({ "compatible_with": INSERT, "scope": "document" }));
    assert_eq!(ids(&fits), vec!["fly-25", "fly-32"], "the cutter bodies take the insert: {fits}");
    assert_eq!(fits["components"][0]["fits"], json!([{ "as": "parent", "port": "CSW1" }]));
    assert_eq!(fits["compatible_with"], json!({ "id": INSERT, "scope": "document" }));
    let seats = m.ok("list_tool_components", json!({ "compatible_with": "fly-25", "scope": "document", "kind": "insert" }));
    assert_eq!(ids(&seats), vec![INSERT]);
    assert_eq!(seats["components"][0]["fits"], json!([{ "as": "child", "port": "CSW1" }]));
    let builtin = m.ok("list_tool_components", json!({ "scope": "builtin", "kind": "adaptive" }));
    let rows = builtin["components"].as_array().expect("rows");
    assert!(!rows.is_empty() && rows.iter().all(|r| r["scope"] == json!("builtin") && r["kind"] == json!("adaptive")), "{builtin}");

    let a = m.ok("get_tool_assembly", json!({ "tool": A }));
    assert!((a["cutting"]["diameter"].as_f64().expect("d") - 25.0).abs() < 1e-9, "{a}");
    assert!((a["cutting"]["corner_r"].as_f64().expect("r") - 0.4).abs() < 1e-12, "the insert's nose radius: {a}");
    assert!(a["gauge_len"].as_f64().expect("gauge") > 0.0, "{a}");
    assert_eq!(a["nodes"].as_array().map(Vec::len), Some(3));
    assert_eq!(a["nodes"][2]["mount"]["port"], json!("CSW1"));
    assert_eq!(a["nodes"][0]["mount"], Value::Null, "the root sits on the spindle");
    assert_eq!(a["nodes"][2]["sync_status"]["status"], json!("not_linked"));
    assert_eq!(a["links"], links("fly-25"), "links is the form set_tool_assembly takes");

    m.ok("undo", json!({}));
    assert!(m.refused("get_tool_assembly", json!({ "tool": B })).contains("has no tool assembly"));
    m.ok("redo", json!({}));
    assert_eq!(m.ok("get_tool_assembly", json!({ "tool": B }))["links"], links("fly-32"));
    m.ok("set_tool_assembly", json!({ "tool": B, "links": null }));
    assert!(m.refused("get_tool_assembly", json!({ "tool": B })).contains("has no tool assembly"));
}

#[test]
fn mcp_edits_a_shared_insert_and_syncs_it_from_the_library() {
    let mut m = shop();
    let lib = library("sync");
    let r = m.ok("upsert_tool_component", json!({ "component": dto(insert("CCMT09T308")) }));
    assert_eq!(r["added"], json!(false));
    assert_eq!(r["updated_tools"], json!([A, B]), "one edit updates both tools");
    assert!((corner_r(&mut m, A) - 0.8).abs() < 1e-12 && (corner_r(&mut m, B) - 0.8).abs() < 1e-12);
    m.ok("undo", json!({}));
    assert!((corner_r(&mut m, A) - 0.4).abs() < 1e-12 && (corner_r(&mut m, B) - 0.4).abs() < 1e-12, "one undo step");

    // Save the document's insert, then link it: the same content, no tool derived again.
    m.ok("save_library_component", json!({ "id": INSERT, "library": lib }));
    assert_eq!(ids(&m.ok("list_tool_components", json!({ "scope": "user", "library": lib }))), vec![INSERT]);
    let r = m.ok("sync_tool_component", json!({ "id": INSERT, "library": lib }));
    assert_eq!(r["updated_tools"], json!([]), "{r}");
    assert_eq!(r["component"]["sync_status"]["status"], json!("up_to_date"), "{r}");

    // A newer library copy is reported, then synced into both tools.
    let saved = m.ok("save_library_component", json!({ "component": dto(insert("CCMT09T308")), "library": lib }));
    assert_eq!(saved["saved"], json!(INSERT));
    let doc = m.ok("list_tool_components", json!({ "scope": "document", "kind": "insert", "library": lib }));
    assert_eq!(doc["components"][0]["sync_status"]["status"], json!("update_available"), "{doc}");
    let r = m.ok("sync_tool_component", json!({ "id": INSERT, "library": lib }));
    assert_eq!(r["updated_tools"], json!([A, B]));
    assert!((corner_r(&mut m, A) - 0.8).abs() < 1e-12);
    m.ok("undo", json!({}));
    assert!((corner_r(&mut m, A) - 0.4).abs() < 1e-12, "undo restores the earlier copy");
    m.ok("redo", json!({}));
    assert!((corner_r(&mut m, B) - 0.8).abs() < 1e-12, "redo lands the sync again");

    m.ok("delete_library_component", json!({ "id": INSERT, "library": lib }));
    let a = m.ok("get_tool_assembly", json!({ "tool": A, "library": lib }));
    assert_eq!(a["nodes"][2]["sync_status"]["status"], json!("missing_from_library"), "{a}");
    let _ = std::fs::remove_file(&lib);
}

#[test]
fn mcp_component_tools_refuse_by_name_and_change_nothing() {
    let mut m = shop();
    let lib = library("refuse");
    let before = m.ok("get_tool_assembly", json!({ "tool": A }));
    let e = m.refused("upsert_tool_component", json!({ "component": dto(insert("DCMT11T304")) }));
    assert!(e.contains(INSERT) && e.contains(A) && e.contains("does not fit"), "{e}");
    let mut bogus = dto(insert("CCMT09T308"));
    bogus["bogus"] = json!(1);
    assert!(m.refused("upsert_tool_component", json!({ "component": bogus })).contains("bogus"));
    let mut linked = dto(insert("CCMT09T308"));
    linked["library_ref"] = json!({ "id": INSERT, "content_hash": "fnv1a64:0000000000000000" });
    assert!(m.refused("upsert_tool_component", json!({ "component": linked })).contains("library_ref"));
    assert!(m.refused("upsert_tool_component", json!({ "component": dto(insert("CCMT09T304")) })).contains("already so"));
    let e = m.refused("delete_tool_component", json!({ "id": INSERT }));
    assert!(e.contains(INSERT) && e.contains(A) && e.contains(B), "{e}");
    assert!(m.refused("delete_tool_component", json!({ "id": "nope" })).contains("'nope'"));

    let e = m.refused("set_tool_assembly", json!({ "tool": A, "links": [{ "component": "nope" }] }));
    assert!(e.contains(A) && e.contains("'nope'"), "{e}");
    assert!(m.refused("set_tool_assembly", json!({ "tool": A, "links": [{ "component": "bt40-cyl20", "mnt": 0 }] })).contains("mnt"));
    assert!(m.refused("set_tool_assembly", json!({ "tool": A, "links": [] })).contains("empty"));
    let e = m.refused("set_tool_assembly", json!({ "tool": A, "links": [{ "component": "bt40-cyl20" }, { "component": INSERT, "mount": { "parent": 0 } }] }));
    assert!(e.contains(A), "an insert on a coupling port: {e}");
    assert!(m.refused("set_tool_assembly", json!({ "tool": "Nobody", "links": null })).contains("Nobody"));
    assert!(m.refused("get_tool_assembly", json!({ "tool": "Nobody" })).contains("no tool named 'Nobody'"));

    assert!(m.refused("list_tool_components", json!({ "kind": "drill" })).contains("unknown kind 'drill'"));
    assert!(m.refused("list_tool_components", json!({ "scope": "shop" })).contains("unknown scope 'shop'"));
    assert!(m.refused("list_tool_components", json!({ "compatible_with": "nope", "library": lib })).contains("'nope'"));
    assert!(m.refused("list_tool_components", json!({ "colour": "red" })).contains("unknown argument \"colour\""));
    assert!(m.refused("sync_tool_component", json!({ "id": "fly-25", "library": lib })).contains("library holds no tool component 'fly-25'"));

    let mut built = dto(insert("CCMT09T304"));
    built["id"] = json!("builtin:mine");
    assert!(m.refused("save_library_component", json!({ "component": built, "library": lib })).contains("built in"));
    assert!(m.refused("delete_library_component", json!({ "id": "builtin:holder:BT40 ER32", "library": lib })).contains("built in"));
    assert!(m.refused("delete_library_component", json!({ "id": "nope", "library": lib })).contains("'nope'"));
    assert!(m.refused("save_library_component", json!({ "id": INSERT, "component": dto(insert("CCMT09T304")), "library": lib })).contains("not both"));
    assert!(m.refused("save_library_component", json!({ "id": "nope", "library": lib })).contains("'nope'"));
    assert!(!std::path::Path::new(&lib).exists(), "no refused save writes the library");

    assert_eq!(m.ok("get_tool_assembly", json!({ "tool": A })), before, "nothing changed");
}

#[test]
fn mcp_upsert_carries_insert_running_data_and_a_library_save_refuses_it() {
    let mut m = shop();
    let mut row = dto(insert("CCMT09T304"));
    row["running"] = json!({ "tool_material": "carbide" });
    let r = m.ok("upsert_tool_component", json!({ "component": row }));
    assert_eq!(r["component"]["component"]["running"]["tool_material"], json!("carbide"), "the stated data lands: {r}");
    // A later upsert that does not state the key keeps the held data.
    let r = m.ok("upsert_tool_component", json!({ "component": dto(insert("CCMT09T308")) }));
    assert_eq!(r["component"]["component"]["running"]["tool_material"], json!("carbide"), "the held data stays: {r}");
    let lib = library("running");
    let e = m.refused("save_library_component", json!({ "id": INSERT, "library": lib }));
    assert!(e.contains(INSERT) && e.contains("running data"), "{e}");
    let mut facts = dto(insert("CCMT09T304"));
    facts["milling"] = json!({ "dc": 25.0, "kapr_deg": 90.0, "apmx": 5.0, "center_cutting": false });
    assert!(m.refused("upsert_tool_component", json!({ "component": facts })).contains("not a cutter body"));
}

#[test]
fn mcp_upsert_carries_a_lathe_body_and_a_library_save_refuses_it() {
    let mut m = shop();
    let facets = json!({ "facets": [[[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 10.0, 0.0]]] });
    let mut row = dto(fly("fly-40", 20.0));
    row["lathe_body"] = facets.clone();
    let r = m.ok("upsert_tool_component", json!({ "component": row }));
    assert_eq!(r["component"]["component"]["lathe_body"], facets, "the stated body lands: {r}");
    // A later upsert that does not state the key keeps the held body.
    let r = m.ok("upsert_tool_component", json!({ "component": dto(fly("fly-40", 21.0)) }));
    assert_eq!(r["component"]["component"]["lathe_body"], facets, "the held body stays: {r}");
    let lib = library("lathe");
    let e = m.refused("save_library_component", json!({ "id": "fly-40", "library": lib }));
    assert!(e.contains("fly-40") && e.contains("lathe body"), "{e}");
    let mut bad = dto(fly("fly-40", 20.0));
    bad["lathe_body"] = json!({ "facets": [] });
    let e = m.refused("upsert_tool_component", json!({ "component": bad }));
    assert!(e.contains("fly-40") && e.contains("no facets"), "{e}");
}
