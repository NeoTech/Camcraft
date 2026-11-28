//! #3645: the component table and the tool stacks in the `.camproj`
//! (v84). Pinned: a project with a table and two assemblies saves and
//! opens again with the same rows, bodies, stacks and records, byte for
//! byte; a file without a table (a v83 file, and a #2016 holder project)
//! opens unchanged; a reader of an older version refuses the new file;
//! a bad row or a stack link to a missing row refuses the file by name
//! and changes nothing. Provenance: our own tests over our own format.

use camcraft_app::commands::{AddTool, History, SetToolAssembly, SetToolBody, UpsertDocumentComponent};
use camcraft_app::model;
use camcraft_app::project::{self, PROJECT_VERSION};
use camcraft_app::tool_assembly::{assembly_of, components};
use camcraft_app::tool_body::body_of;
use camcraft_tools::component::{ComponentKind, CuttingItem, ToolComponent};
use camcraft_tools::component_geometry::{ComponentBody, ComponentGeometry};
use camcraft_tools::frame::Frame;
use camcraft_tools::spun::SpunPoint;
use flecs_ecs::prelude::World;
use serde_json::{Value, json};

use super::assembly_cmds::{code, link, on, shop, snapshot, A, INSERT};
use super::cat;

/// A turned port frame, a body, a cutting point and a bore stop: each
/// geometry key the file writes.
fn adapter() -> (ToolComponent, ComponentGeometry) {
    let p = SpunPoint::line;
    let body = ComponentBody::new(vec![p(0.0, 0.0), p(0.0, 15.0), p(30.0, 15.0), p(30.0, 0.0)], -30.0).expect("body");
    let port = Frame::new([0.0, 0.0, -30.0], [0.0, 0.0, 1.0], [0.6, 0.8, 0.0]).expect("frame");
    let cip = Frame::new([1.0, 2.0, -31.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]).expect("cip");
    let g = ComponentGeometry::new(vec![("CSW".into(), port)], Some(body), Some(cip))
        .and_then(|g| g.with_stop("CSW", 20.0))
        .expect("geometry");
    let kind = ComponentKind::AdaptiveItem { mcs: code("CYL 20"), csw: code("CYL 12") };
    (ToolComponent::new("adapter-12", "Adapter CYL 12", kind).expect("adapter"), g)
}

fn solid() -> (ToolComponent, ComponentGeometry) {
    let tool = cat("EM-1200-2FL");
    let kind = ComponentKind::CuttingItem(CuttingItem::Solid { mcs: code(&format!("CYL {}", tool.shank_d)), tool: Box::new(tool) });
    (ToolComponent::new("em-12", "EM 12", kind).expect("solid"), ComponentGeometry::default())
}

/// A third tool: a solid end mill home in an adapter in the holder -
/// two bodies, united band by band.
const EM: &str = "Em12";

/// The shop of `assembly_cmds` with the adapter, the end mill and [`EM`].
fn rich_shop() -> (World, History) {
    let (w, mut h) = shop();
    for (c, g) in [adapter(), solid()] {
        h.execute(&w, UpsertDocumentComponent::new(&w, c, g).expect("upsert"));
    }
    h.execute(&w, AddTool::new(&w, EM, cat("EM-1200-2FL")).expect("tool"));
    let stack = vec![link("bt40-cyl20", None), link("adapter-12", on(0, None)), link("em-12", on(1, None))];
    h.execute(&w, SetToolAssembly::new(&w, EM, stack).expect("assembly"));
    (w, h)
}

fn em(w: &World) -> (Option<camcraft_app::tools::Tool>, Option<camcraft_app::schema::ToolBody>, Option<camcraft_app::schema::ToolAssembly>) {
    let e = model::find_by_name(w, EM).expect("tool");
    (model::tool_of(w, &e), body_of(w, &e), assembly_of(w, &e))
}

fn reopen(text: &str) -> World {
    let w = camcraft_app::world::new_world();
    project::open(&w, text).expect("open");
    w
}

#[test]
fn a_project_with_a_component_table_and_its_assemblies_reopens_identically() {
    let (w, _h) = rich_shop();
    let text = project::save_text(&w).expect("save");
    let file: Value = serde_json::from_str(&text).expect("json");
    assert_eq!(file["version"], json!(PROJECT_VERSION));
    assert_eq!(file["components"].as_array().map(Vec::len), Some(6), "{}", file["components"]);
    let w2 = reopen(&text);
    assert_eq!(components(&w2), components(&w), "the rows, their geometry and their codes");
    assert_eq!(snapshot(&w2), snapshot(&w), "the records, the bodies and the stacks");
    let united = em(&w);
    assert_eq!(united.1.as_ref().map(|b| b.source.as_str()), Some(camcraft_app::tool_assembly::ASSEMBLY_SOURCE));
    assert_eq!(em(&w2), united, "the united body keeps the judged bands of the resolver");
    assert_eq!(project::save_text(&w2).expect("save"), text, "save -> open -> save is byte-stable");
}

#[test]
fn a_file_without_a_table_and_a_holder_project_open_unchanged() {
    let w = camcraft_app::world::new_world();
    let mut h = History::default();
    h.execute(&w, AddTool::new(&w, "T1", cat("EM-0600-4FL")).expect("tool"));
    h.execute(&w, AddTool::new(&w, "T2", cat("EM-0600-4FL")).expect("tool"));
    h.execute(&w, SetToolBody::library(&w, "T2", "BT30 ER20", None, None).expect("holder"));
    let text = project::save_text(&w).expect("save");
    assert!(!text.contains("\"components\"") && !text.contains("\"assembly\""), "no table, no key");
    // The same document as a v83 file: the key set of a v83 build.
    let old = text.replacen(&format!("\"version\": {PROJECT_VERSION}"), "\"version\": 83", 1);
    assert_ne!(old, text);
    let w2 = reopen(&old);
    assert!(components(&w2).is_empty());
    for n in ["T1", "T2"] {
        let (a, b) = (model::find_by_name(&w, n).expect("tool"), model::find_by_name(&w2, n).expect("tool"));
        assert_eq!(model::tool_of(&w2, &b), model::tool_of(&w, &a), "{n}: the record");
        assert_eq!(body_of(&w2, &b), body_of(&w, &a), "{n}: the body (the #2016 holder reference)");
        assert!(assembly_of(&w2, &b).is_none(), "{n}: no stack");
    }
    assert_eq!(project::save_text(&w2).expect("save"), text, "it saves as a v84 file of the same content");
}

#[test]
fn a_reader_of_an_older_version_refuses_the_new_file() {
    const { assert!(PROJECT_VERSION >= 84, "v83 and earlier readers must refuse a file with a component table") };
    let (w, _h) = rich_shop();
    let text = project::save_text(&w).expect("save");
    // A v83 reader meets a file newer than it writes: the same refusal
    // this build gives a file one version past its own.
    let newer = text.replacen(&format!("\"version\": {PROJECT_VERSION}"), &format!("\"version\": {}", PROJECT_VERSION + 1), 1);
    let w2 = camcraft_app::world::new_world();
    let Err(e) = project::open(&w2, &newer) else { panic!("a newer file opened") };
    assert!(e.to_string().contains("unsupported project version"), "{e}");
}

/// Open `text` changed by `f` over a document that holds a tool `Keep`:
/// the refusal, and the proof that nothing of the file landed.
fn refused(text: &str, f: impl Fn(&mut Value)) -> String {
    let mut v: Value = serde_json::from_str(text).expect("json");
    f(&mut v);
    let w = camcraft_app::world::new_world();
    let mut h = History::default();
    h.execute(&w, AddTool::new(&w, "Keep", cat("EM-0600-4FL")).expect("tool"));
    let Err(e) = project::open(&w, &v.to_string()) else { panic!("a corrupt file opened") };
    assert!(model::find_by_name(&w, "Keep").is_some() && model::find_by_name(&w, A).is_none(), "nothing landed");
    assert!(components(&w).is_empty(), "no row landed");
    e.to_string()
}

fn tool_mut<'a>(v: &'a mut Value, name: &str) -> &'a mut Value {
    let entities = v["entities"].as_array_mut().expect("entities");
    let row = entities.iter_mut().find(|e| e["name"] == json!(name)).expect("tool row");
    &mut row["tool"]
}

fn row_mut<'a>(v: &'a mut Value, id: &str) -> &'a mut Value {
    let rows = v["components"].as_array_mut().expect("components");
    rows.iter_mut().find(|r| r["id"] == json!(id)).expect("component row")
}

#[test]
fn a_corrupt_row_or_link_refuses_the_file_by_name() {
    let (w, _h) = rich_shop();
    let text = project::save_text(&w).expect("save");
    let e = refused(&text, |v| tool_mut(v, A)["assembly"][2]["component"] = json!("no-such"));
    assert!(e.contains(&format!("tool '{A}'")) && e.contains("'no-such'"), "{e}");
    let e = refused(&text, |v| row_mut(v, INSERT)["item"]["insert"]["code"] = json!("CCMT09"));
    assert!(e.contains(&format!("tool component '{INSERT}'")) && e.contains("insert code"), "{e}");
    let e = refused(&text, |v| row_mut(v, "adapter-12")["geometry"]["stops"][0]["port"] = json!("CSW9"));
    assert!(e.contains("tool component 'adapter-12'") && e.contains("CSW9"), "{e}");
    let e = refused(&text, |v| row_mut(v, "adapter-12")["colour"] = json!("red"));
    assert!(e.contains("colour"), "an unknown key is refused: {e}");
    let e = refused(&text, |v| {
        let rows = v["components"].as_array_mut().expect("components");
        let first = rows[0].clone();
        rows.push(first);
    });
    assert!(e.contains("twice"), "{e}");
    // A stack that names held rows but does not resolve: the insert as root.
    let e = refused(&text, |v| tool_mut(v, A)["assembly"] = json!([{ "component": INSERT }]));
    assert!(e.contains(&format!("tool '{A}'")) && e.contains("assembly is refused"), "{e}");
    let e = refused(&text, |v| row_mut(v, "em-12")["item"]["solid"]["tool"]["number"] = json!(3));
    assert!(e.contains("tool component 'em-12'") && e.contains("\"number\""), "{e}");
}
