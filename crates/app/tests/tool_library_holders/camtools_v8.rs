//! #3645: the tool components and the tool stacks in a `.camtools` file
//! (version 8) and their import in the one `ImportTools` undo step.
//! Pinned: an export of two assembled tools says version 8, holds the
//! rows the stacks name, and imports with the same rows, records, bodies
//! and stacks, byte for byte on export again; a file of plain tools keeps
//! its older number and imports as before; a newer version is refused; a
//! bad row, a bad link or a row no tool names refuses the file by name
//! and nothing lands; undo and redo restore the tools and the table; a
//! document row of the same id is reused when its content is the same and
//! refuses the import by the id when it differs. #3643: a turning stack's
//! rows keep their lathe bodies through an export and an import, byte for
//! byte. A same-id row that differs only in its cutter facts, its running
//! data or its lathe body refuses the import by the id; a skipped
//! identical tool whose stack names a changed row refuses the import too,
//! with an added plain tool in the same file. Provenance: our own tests
//! over our own format.

use camcraft_app::commands::{AddTool, History, ImportMode, UpsertDocumentComponent};
use camcraft_app::model;
use camcraft_app::tool_assembly::{assembly_of, components};
use camcraft_app::tool_library::{export_tool_library, parse_tool_library, ASSEMBLY_VERSION, CUTTER_HAND_VERSION};
use camcraft_app::world::new_world;
use flecs_ecs::prelude::World;
use serde_json::{Value, json};

use super::assembly_cmds::{fly, shop, snapshot, A, B, INSERT};
use super::{cat, import, import_as};

/// The `.camtools` text of the shop: two tools, each on a stack of
/// three rows, the holder and the insert shared.
fn v8_text() -> String {
    let (w, _h) = shop();
    export_tool_library(&w).expect("export")
}

fn edited(text: &str, f: impl Fn(&mut Value)) -> String {
    let mut v: Value = serde_json::from_str(text).expect("json");
    f(&mut v);
    v.to_string()
}

fn def_mut<'a>(v: &'a mut Value, name: &str) -> &'a mut Value {
    let tools = v["tools"].as_array_mut().expect("tools");
    let row = tools.iter_mut().find(|t| t["name"] == json!(name)).expect("tool row");
    &mut row["definition"]
}

fn row_mut<'a>(v: &'a mut Value, id: &str) -> &'a mut Value {
    let rows = v["components"].as_array_mut().expect("components");
    rows.iter_mut().find(|r| r["id"] == json!(id)).expect("component row")
}

#[test]
fn an_export_of_assembled_tools_is_version_8_and_imports_identically() {
    let (w, _h) = shop();
    let text = export_tool_library(&w).expect("export");
    let file: Value = serde_json::from_str(&text).expect("json");
    assert_eq!(file["version"], json!(ASSEMBLY_VERSION));
    assert_eq!(file["components"].as_array().map(Vec::len), Some(4), "{}", file["components"]);
    assert!(file["tools"][0]["definition"]["assembly"].is_array(), "each tool states its stack");
    let w2 = new_world();
    let mut h2 = History::default();
    let r = import(&w2, &mut h2, &text, true).expect("import");
    assert_eq!(r.components_added.len(), 4, "{r:?}");
    assert!(r.summary().contains("4 tool components added"), "{}", r.summary());
    assert_eq!(components(&w2), components(&w), "the rows, their geometry and their codes");
    assert_eq!(snapshot(&w2), snapshot(&w), "the records, the bodies and the stacks");
    assert_eq!(export_tool_library(&w2).expect("export"), text, "export -> import -> export is byte-stable");
}

#[test]
fn a_library_of_plain_tools_keeps_its_older_number_and_imports_as_before() {
    let w = new_world();
    let mut h = History::default();
    h.execute(&w, AddTool::new(&w, "T1", cat("EM-0600-4FL")).expect("tool"));
    let text = export_tool_library(&w).expect("export");
    let file: Value = serde_json::from_str(&text).expect("json");
    assert!(file["version"].as_u64().is_some_and(|v| v <= CUTTER_HAND_VERSION), "{}", file["version"]);
    assert!(!text.contains("\"components\"") && !text.contains("\"assembly\""), "no table, no key");
    // A version 7 file, as a version 7 build writes it.
    let v7 = edited(&text, |v| v["version"] = json!(CUTTER_HAND_VERSION));
    let lib = parse_tool_library(&v7).expect("a v7 file reads");
    assert!(lib.iter().all(|t| t.assembly.is_none()));
    let w2 = new_world();
    let mut h2 = History::default();
    let r = import(&w2, &mut h2, &v7, true).expect("import");
    assert!(r.components_added.is_empty() && components(&w2).is_empty());
    let (a, b) = (model::find_by_name(&w, "T1").expect("tool"), model::find_by_name(&w2, "T1").expect("tool"));
    assert_eq!(model::tool_of(&w2, &b), model::tool_of(&w, &a));
    assert!(assembly_of(&w2, &b).is_none());
}

#[test]
fn a_newer_version_and_an_older_file_with_components_are_refused() {
    let text = v8_text();
    let newer = edited(&text, |v| v["version"] = json!(ASSEMBLY_VERSION + 1));
    let e = parse_tool_library(&newer).expect_err("a newer file read");
    assert!(e.contains(&format!("version {} is not supported", ASSEMBLY_VERSION + 1)), "{e}");
    assert!(e.contains(&format!("reads 1 to {ASSEMBLY_VERSION}")), "{e}");
    let older = edited(&text, |v| v["version"] = json!(CUTTER_HAND_VERSION));
    let e = parse_tool_library(&older).expect_err("a v7 file with a table read");
    assert!(e.contains("cannot hold tool components"), "{e}");
    let older = edited(&text, |v| {
        v["version"] = json!(CUTTER_HAND_VERSION);
        v.as_object_mut().expect("object").remove("components");
    });
    let e = parse_tool_library(&older).expect_err("a v7 file with a stack read");
    assert!(e.contains("states an assembly") && e.contains(&format!("version {CUTTER_HAND_VERSION}")), "{e}");
}

/// Import `text` over a document that holds a tool `Keep`: the refusal,
/// and the proof that nothing of the file landed.
fn refused(text: &str) -> String {
    let w = new_world();
    let mut h = History::default();
    h.execute(&w, AddTool::new(&w, "Keep", cat("EM-0600-4FL")).expect("tool"));
    let Err(e) = import(&w, &mut h, text, true) else { panic!("a corrupt file imported") };
    assert!(model::find_by_name(&w, "Keep").is_some() && model::find_by_name(&w, A).is_none(), "nothing landed");
    assert!(components(&w).is_empty(), "no row landed");
    e
}

#[test]
fn a_bad_row_a_bad_link_or_an_unused_row_refuses_the_file_by_name() {
    let text = v8_text();
    let e = refused(&edited(&text, |v| def_mut(v, A)["assembly"][2]["component"] = json!("no-such")));
    assert!(e.contains(&format!("tool '{A}'")) && e.contains("'no-such'"), "{e}");
    let e = refused(&edited(&text, |v| row_mut(v, INSERT)["item"]["insert"]["code"] = json!("CCMT09")));
    assert!(e.contains(&format!("tool component '{INSERT}'")) && e.contains("insert code"), "{e}");
    let e = refused(&edited(&text, |v| row_mut(v, "fly-25")["colour"] = json!("red")));
    assert!(e.contains("colour"), "an unknown key is refused: {e}");
    let e = refused(&edited(&text, |v| def_mut(v, A)["assembly"] = json!([{ "component": INSERT }])));
    assert!(e.contains(&format!("tool '{A}'")) && e.contains("assembly is refused"), "{e}");
    // Only B is left on a stack: the row of A's cutter body is unused.
    let e = refused(&edited(&text, |v| {
        def_mut(v, A).as_object_mut().expect("definition").remove("assembly");
    }));
    assert!(e.contains("tool component 'fly-25'") && e.contains("no tool's assembly names it"), "{e}");
}

fn state(w: &World) -> (Vec<String>, usize) {
    let tools = model::named_entities(w).into_iter().filter(|(_, e)| model::is_tool(w, e)).map(|(n, _)| n).collect();
    (tools, components(w).len())
}

#[test]
fn undo_and_redo_of_an_import_with_components_restore_tools_and_table() {
    let (src, _h) = shop();
    let text = export_tool_library(&src).expect("export");
    let w = new_world();
    let mut h = History::default();
    h.execute(&w, AddTool::new(&w, "Keep", cat("EM-0600-4FL")).expect("tool"));
    let before = state(&w);
    import(&w, &mut h, &text, true).expect("import");
    let after = snapshot(&w);
    assert!(h.undo(&w), "one undo step");
    assert_eq!(state(&w), before, "the tools and the table are as before");
    assert!(h.redo(&w));
    assert_eq!(snapshot(&w), after, "redo lands the same rows and stacks");
    // Holders off leaves the stacks and their rows out with the bodies.
    let w2 = new_world();
    let mut h2 = History::default();
    let r = import(&w2, &mut h2, &text, false).expect("import");
    assert!(r.components_added.is_empty() && components(&w2).is_empty(), "{r:?}");
    let e = model::find_by_name(&w2, A).expect("tool");
    assert!(assembly_of(&w2, &e).is_none());
}

#[test]
fn a_document_row_of_the_same_id_is_reused_when_the_same_and_refuses_when_different() {
    let text = v8_text();
    // The same content: reused, not added again.
    let w = new_world();
    let mut h = History::default();
    let (c, g) = fly("fly-25", 12.5);
    h.execute(&w, UpsertDocumentComponent::new(&w, c, g).expect("upsert"));
    let r = import(&w, &mut h, &text, true).expect("import");
    assert_eq!(r.components_reused, vec!["fly-25".to_owned()], "{r:?}");
    assert_eq!(r.components_added.len(), 3, "{r:?}");
    assert_eq!(components(&w).len(), 4);
    // A different content under the same id: refused by the id.
    let w = new_world();
    let mut h = History::default();
    let (c, g) = fly("fly-25", 14.0);
    h.execute(&w, UpsertDocumentComponent::new(&w, c, g).expect("upsert"));
    let before = components(&w);
    let Err(e) = import_as(&w, &mut h, &text, ImportMode::Update, true) else { panic!("a clash imported") };
    assert!(e.contains("tool component 'fly-25'") && e.contains("nothing imported"), "{e}");
    assert_eq!(components(&w), before, "the document row is unchanged");
    assert!(model::find_by_name(&w, A).is_none() && model::find_by_name(&w, B).is_none(), "no tool landed");
}

#[test]
fn a_turning_stack_with_a_lathe_body_exports_and_imports_identically() {
    use super::lathe_stack::{HOLDER_M, lathe_shop, snapshot as lathe_snapshot};
    let (w, _h) = lathe_shop();
    let text = export_tool_library(&w).expect("export");
    let file: Value = serde_json::from_str(&text).expect("json");
    assert_eq!(file["version"], json!(ASSEMBLY_VERSION));
    let holder = file["components"].as_array().and_then(|r| r.iter().find(|c| c["id"] == json!(HOLDER_M)));
    let facets = holder.and_then(|r| r["lathe_body"]["facets"].as_array().map(Vec::len));
    assert_eq!(facets, Some(24), "the holder row writes its lathe body: {holder:?}");
    let w2 = new_world();
    let mut h2 = History::default();
    import(&w2, &mut h2, &text, true).expect("import");
    assert_eq!(lathe_snapshot(&w2), lathe_snapshot(&w), "the rows, the lathe bodies, the records and the bodies");
    assert_eq!(export_tool_library(&w2).expect("export"), text, "export -> import -> export is byte-stable");
}

/// The document row of `id` as the file `text` states it.
fn file_row(text: &str, id: &str) -> camcraft_app::schema::DocumentComponent {
    let mut v: Value = serde_json::from_str(text).expect("json");
    let dto = serde_json::from_value(row_mut(&mut v, id).clone()).expect("row dto");
    camcraft_app::project::component_from_dto(&dto).expect("row")
}

#[test]
fn a_same_id_row_with_other_facts_running_data_or_lathe_body_refuses_the_import() {
    let text = v8_text();
    let facet = json!({ "facets": [[[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 10.0, 0.0]]] });
    let cases: [(&str, &str, Value); 3] = [
        ("fly-25", "milling", json!({ "dc": 25.0, "kapr_deg": 90.0, "apmx": 5.0, "center_cutting": false })),
        (INSERT, "running", json!({ "tool_material": "carbide" })),
        ("fly-25", "lathe_body", facet),
    ];
    for (id, key, value) in cases {
        let w = new_world();
        let mut h = History::default();
        let doc = file_row(&text, id);
        h.execute(&w, UpsertDocumentComponent::with_row(&w, doc).expect("upsert"));
        let before = components(&w);
        let file = edited(&text, |v| row_mut(v, id)[key] = value.clone());
        let Err(e) = import(&w, &mut h, &file, true) else { panic!("a row with another {key} was reused") };
        assert!(e.contains(&format!("tool component '{id}'")) && e.contains("nothing imported"), "{key}: {e}");
        assert_eq!(components(&w), before, "{key}: the document row is unchanged");
        assert!(model::find_by_name(&w, A).is_none() && model::find_by_name(&w, B).is_none(), "{key}: no tool landed");
    }
}

#[test]
fn a_skipped_identical_tool_whose_row_changed_refuses_the_import() {
    let (w, mut h) = shop();
    let text = export_tool_library(&w).expect("export");
    let moved = camcraft_app::project::component_to_dto(&camcraft_app::schema::DocumentComponent::new(
        fly("fly-25", 14.0).0,
        fly("fly-25", 14.0).1,
    ));
    let file = edited(&text, |v| {
        *row_mut(v, "fly-25") = serde_json::to_value(&moved).expect("row");
        let tools = v["tools"].as_array_mut().expect("tools");
        let mut plain = tools[0].clone();
        plain["name"] = json!("Plain");
        if let Some(d) = plain["definition"].as_object_mut() {
            d.remove("assembly");
            d.remove("number");
        }
        tools.push(plain);
    });
    let before = (components(&w), snapshot(&w));
    let Err(e) = import(&w, &mut h, &file, true) else { panic!("a changed row of a skipped tool imported") };
    assert!(e.contains("tool component 'fly-25'") && e.contains("nothing imported"), "{e}");
    assert_eq!((components(&w), snapshot(&w)), before, "nothing changed");
    assert!(model::find_by_name(&w, "Plain").is_none(), "the plain tool did not land");
}
