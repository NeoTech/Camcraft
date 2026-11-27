//! #3915: a model that holds a SolidCraft source face record shows the name
//! of each face and patch: `source_name` (one name) or `source_names`
//! (several), on the `list_features` rows, compact rows included, and on
//! the picks that `get_operation_geometry` and `get_selection` show.
//! `source_unverified` flags a name that SolidCraft has not verified. A
//! model with no record, a record that does not fit the facets, and a pick
//! that is no longer in the model show none of these keys.
//! Provenance: our own tests; the meshes and their names are the test's own.
//!
//! Gate map: covers app::mcp (source_names, features list_features, pick
//! picks_view, selection get_selection), camcraft_geometry::geometry::source_names.

use std::path::Path;
use std::sync::Arc;

use super::Client;
use camcraft_app::commands::{History, ImportModel};
use camcraft_app::geometry::source_faces::{FaceName, FacetRange, SourceFaces};
use camcraft_app::geometry::{Mesh, Point3, Triangle, cylinder};
use camcraft_app::mcp::{LiveOptions, SelectionContext, answer_live, answer_live_with};
use camcraft_app::operation::{FeatureSel, GeoSel};
use camcraft_app::world::new_world;
use flecs_ecs::prelude::World;
use serde_json::{Value, json};

/// The sides of the 10 mm box, in the order `box_mesh` writes them.
const SIDES: [&str; 6] = ["F1:bottom", "F1:top", "F1:side:y0", "F1:side:y1", "F1:side:x0", "F1:side:x1"];

/// A 10 mm box from the origin, wound outward: six quads of two facets
/// each, in the order of `SIDES`.
fn box_mesh() -> Mesh {
    let v = |i: usize| {
        let c = |bit: usize| if i & bit == 0 { 0.0 } else { 10.0 };
        Point3::new(c(1), c(2), c(4))
    };
    let quads = [[0, 2, 3, 1], [4, 5, 7, 6], [0, 1, 5, 4], [2, 6, 7, 3], [0, 4, 6, 2], [1, 3, 7, 5]];
    let facets: Vec<Triangle> = quads
        .iter()
        .flat_map(|q| [Triangle::new(v(q[0]), v(q[1]), v(q[2])), Triangle::new(v(q[0]), v(q[2]), v(q[3]))])
        .collect();
    Mesh::from_facets(facets)
}

fn names(list: &[&str]) -> Vec<FaceName> {
    list.iter().map(|n| FaceName { name: (*n).to_owned(), verified: true, former: None }).collect()
}

/// A record from the record face of each facet (C13 PR 1's builder).
fn record(per_facet: &[u32], list: Vec<FaceName>) -> SourceFaces {
    SourceFaces::from_tri_face(per_facet, list).expect("every face index is below the name count")
}

fn one_per_quad() -> Vec<u32> {
    (0..12).map(|i| i / 2).collect()
}

/// Land `mesh` as model `name` through the import command, with `record`
/// as a `.sc` import carries it (`ImportModel::apply` sets it).
fn land(world: &World, history: &mut History, name: &str, mesh: Mesh, record: Option<SourceFaces>) {
    let mut cmd = ImportModel::from_mesh(name, mesh);
    cmd.source_faces = record.map(Arc::new);
    history.execute(world, cmd);
}

fn line(id: u64, name: &str, args: Value) -> String {
    json!({ "jsonrpc": "2.0", "id": id, "method": "tools/call", "params": { "name": name, "arguments": args } })
        .to_string()
}

fn success(reply: Option<String>, name: &str) -> Value {
    let reply: Value = serde_json::from_str(&reply.expect("a reply")).expect("json");
    assert_eq!(reply["result"]["isError"], json!(false), "{name}: {reply}");
    reply["result"]["structuredContent"].clone()
}

/// One live call against `world`; the answer must be a tool success.
fn ok(world: &World, history: &mut History, id: u64, name: &str, args: Value) -> Value {
    success(answer_live(world, history, None, &line(id, name, args)).reply, name)
}

/// The box side a `list_features` face row lies on, by its outward normal.
fn side_of(row: &Value) -> &'static str {
    let n: Vec<f64> = row["normal"].as_array().expect("an outward normal").iter().filter_map(Value::as_f64).collect();
    let axis = n.iter().position(|c| c.abs() > 0.5).expect("an axis normal");
    // bottom, top, y-, y+, x-, x+ (the order of `SIDES`)
    let slot = match axis {
        2 => 0,
        1 => 2,
        _ => 4,
    };
    SIDES[slot + usize::from(n[axis] > 0.0)]
}

fn faces(listing: &Value) -> Vec<Value> {
    listing["faces"].as_array().expect("faces").clone()
}

fn no_name_keys(row: &Value) -> bool {
    ["source_name", "source_names", "source_unverified"].iter().all(|k| row.get(k).is_none())
}

#[test]
fn each_face_row_names_its_solidcraft_face() {
    let world = new_world();
    let mut h = History::default();
    land(&world, &mut h, "part", box_mesh(), Some(record(&one_per_quad(), names(&SIDES))));
    let f = ok(&world, &mut h, 1, "list_features", json!({ "model": "part", "kinds": ["faces", "chains"] }));
    let rows = faces(&f);
    assert_eq!(rows.len(), 6, "{f}");
    for row in &rows {
        assert_eq!(row["source_name"], json!(side_of(row)), "{row}");
        assert!(row.get("source_names").is_none() && row.get("source_unverified").is_none(), "{row}");
    }
    // Chains carry no names in this PR.
    for row in f["chains"].as_array().expect("chains") {
        assert!(no_name_keys(row), "{row}");
    }
    // A compact listing keeps the names: they are what a pick is chosen by.
    let c = ok(&world, &mut h, 2, "list_features", json!({ "model": "part", "kinds": ["faces"], "compact": true }));
    let mut got: Vec<String> = faces(&c).iter().filter_map(|r| r["source_name"].as_str().map(str::to_owned)).collect();
    got.sort_unstable();
    let mut want: Vec<String> = SIDES.iter().map(|s| (*s).to_owned()).collect();
    want.sort_unstable();
    assert_eq!(got, want, "{c}");
}

#[test]
fn a_face_over_two_record_faces_lists_both_and_flags_an_unverified_one() {
    let world = new_world();
    let mut h = History::default();
    // The top's two facets (2 and 3) are two SolidCraft faces, and the
    // second one is not verified.
    let mut per_facet = one_per_quad();
    per_facet[3] = 6;
    let mut list = names(&SIDES);
    list.push(FaceName { name: "F1:top:split".to_owned(), verified: false, former: None });
    land(&world, &mut h, "part", box_mesh(), Some(record(&per_facet, list)));
    let f = ok(&world, &mut h, 1, "list_features", json!({ "model": "part", "kinds": ["faces"] }));
    let top = faces(&f).into_iter().find(|r| side_of(r) == SIDES[1]).expect("the top face");
    assert_eq!(top["source_names"], json!(["F1:top", "F1:top:split"]), "{top}");
    assert!(top.get("source_name").is_none(), "{top}");
    assert_eq!(top["source_unverified"], json!(true), "{top}");
    let bottom = faces(&f).into_iter().find(|r| side_of(r) == SIDES[0]).expect("the bottom face");
    assert!(bottom.get("source_unverified").is_none(), "{bottom}");
}

#[test]
fn no_record_or_a_record_that_does_not_fit_shows_no_names() {
    let world = new_world();
    let mut h = History::default();
    land(&world, &mut h, "plain", box_mesh(), None);
    // Ranges over 10 of the 12 facets: the record does not describe them.
    let short: Vec<u32> = (0..10).map(|i| i / 2).collect();
    land(&world, &mut h, "short", box_mesh(), Some(record(&short, names(&SIDES))));
    // A range that names a record face past the name list.
    let past = SourceFaces { names: names(&SIDES), ranges: vec![FacetRange { start: 0, end: 12, face: 9 }] };
    land(&world, &mut h, "past", box_mesh(), Some(past));
    for (id, model) in [(1, "plain"), (2, "short"), (3, "past")] {
        let f = ok(&world, &mut h, id, "list_features", json!({ "model": model, "kinds": ["faces", "patches"] }));
        for row in faces(&f).iter().chain(f["patches"].as_array().expect("patches")) {
            assert!(no_name_keys(row), "{model}: {row}");
        }
    }
}

#[test]
fn a_face_pick_shows_its_name_in_the_operation_geometry() {
    let world = new_world();
    let mut h = History::default();
    land(&world, &mut h, "part", box_mesh(), Some(record(&one_per_quad(), names(&SIDES))));
    ok(&world, &mut h, 1, "set_stock", json!({ "box": { "dx": 30.0, "dy": 30.0, "dz": 15.0 } }));
    ok(&world, &mut h, 2, "add_tool", json!({ "catalog": "EM-0400-2FL" }));
    ok(
        &world,
        &mut h,
        3,
        "add_operation",
        json!({
            "kind": "pocket", "tool": "EM-0400-2FL", "name": "P1", "source": "part",
            "params": {
                "top_z": { "reference": "top_of_stock", "offset": 0.0 },
                "bottom_z": { "reference": "bottom_of_selection", "offset": 0.0 },
                "stepdown": 2.5,
            },
        }),
    );
    let f = ok(&world, &mut h, 4, "list_features", json!({ "model": "part", "kinds": ["faces"] }));
    let top = faces(&f).into_iter().find(|r| side_of(r) == SIDES[1]).expect("the top face");
    let pick = json!([{ "model": "part", "id": top["id"] }]);
    let set = ok(&world, &mut h, 5, "set_operation_geometry", json!({ "operation": "P1", "picks": pick }));
    assert_eq!(set["source"]["picks"][0]["source_name"], json!(SIDES[1]), "{set}");
    let g = ok(&world, &mut h, 6, "get_operation_geometry", json!({ "operation": "P1" }));
    let shown = &g["source"]["picks"][0];
    assert_eq!(shown["id"], top["id"], "{g}");
    assert_eq!(shown["source_name"], json!(SIDES[1]), "{g}");
}

#[test]
fn a_patch_pick_names_all_its_faces_and_a_missing_pick_names_none() {
    let world = new_world();
    let mut h = History::default();
    // `cylinder` writes four facets for each of its 16 segments: two wall
    // facets, a top cap facet, a bottom cap facet.
    let m = cylinder(20.0, 10.0, 16);
    let per_facet: Vec<u32> = (0..m.facets.len())
        .map(|i| match i % 4 {
            0 | 1 if i / 4 < 8 => 0,
            0 | 1 => 1,
            2 => 2,
            _ => 3,
        })
        .collect();
    let list = names(&["F1:side:a", "F1:side:b", "F1:top", "F1:bottom"]);
    land(&world, &mut h, "post", m, Some(record(&per_facet, list)));
    let f = ok(&world, &mut h, 1, "list_features", json!({ "model": "post", "kinds": ["patches"] }));
    let patches = f["patches"].as_array().expect("patches");
    assert_eq!(patches.len(), 1, "the wall is one patch: {f}");
    assert_eq!(patches[0]["source_names"], json!(["F1:side:a", "F1:side:b"]), "{f}");
    let hex = patches[0]["id"].as_str().and_then(|s| s.strip_prefix("patch:0x")).expect("a patch id");
    let patch = u64::from_str_radix(hex, 16).expect("hex");
    // The window's selection, read through get_selection: the same picks view.
    let selection = SelectionContext {
        pick_mode: None,
        geometry_picks: vec![
            GeoSel { entity: "post".into(), feature: FeatureSel::Patch(patch) },
            GeoSel { entity: "post".into(), feature: FeatureSel::Face(1) },
        ],
        active_operation: None,
    };
    let options = LiveOptions { selection: Some(selection), ..LiveOptions::default() };
    let answer = answer_live_with(&world, &mut h, None, &line(2, "get_selection", json!({})), &options);
    let s = success(answer.reply, "get_selection");
    let picks = s["geometry_picks"].as_array().expect("geometry picks");
    assert_eq!(picks.len(), 2, "{s}");
    assert_eq!(picks[0]["source_names"], json!(["F1:side:a", "F1:side:b"]), "{s}");
    assert!(no_name_keys(&picks[1]), "a face id that is not in the model has no name: {s}");
}

/// The format-10 fixture through the real import (C13 PR 1's reader): each
/// face and patch row gets one of the two keys, and each name is a string
/// that is not empty; a face pick in `get_operation_geometry` shows the
/// same name as its row (#4214's "Done means"). The names themselves are
/// not pinned here.
#[test]
fn a_sc_import_names_every_face_and_patch() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/models/hole_on_pocket_wall.sc");
    let mut c = Client::new();
    let out = c.ok("import_model", json!({ "path": path.display().to_string(), "name": "plate" }));
    assert_eq!(out["model"], json!("plate"), "{out}");
    let f = c.ok("list_features", json!({ "model": "plate", "kinds": ["faces", "patches"], "limit": 20000 }));
    let rows: Vec<&Value> =
        f["faces"].as_array().expect("faces").iter().chain(f["patches"].as_array().expect("patches")).collect();
    assert!(!rows.is_empty(), "{f}");
    for row in rows {
        let set: Vec<&str> = match (row.get("source_name"), row.get("source_names")) {
            (Some(one), None) => vec![one.as_str().expect("a string")],
            (None, Some(many)) => many.as_array().expect("an array").iter().filter_map(Value::as_str).collect(),
            _ => panic!("one of source_name and source_names: {row}"),
        };
        assert!(!set.is_empty() && set.iter().all(|n| !n.is_empty()), "{row}");
    }
    // A face that looks up, picked into a 2D Pocket: the pick shows the
    // name keys of its `list_features` row. The pocket's top is fixed 1 mm
    // above the face, so its heights do not depend on the stock.
    let up = f["faces"].as_array().expect("faces").iter().find(|r| r["facing"] == json!("up")).expect("an up face");
    let face_z = up["bounds"]["max"][2].as_f64().expect("the face's top z");
    c.ok("set_stock", json!({ "from_model": "plate" }));
    c.ok("add_tool", json!({ "catalog": "EM-0400-2FL" }));
    c.ok(
        "add_operation",
        json!({
            "kind": "pocket", "tool": "EM-0400-2FL", "name": "P1", "source": "plate",
            "params": {
                "top_z": { "reference": "fixed", "offset": face_z + 1.0 },
                "bottom_z": { "reference": "bottom_of_selection", "offset": 0.0 },
                "stepdown": 2.5,
            },
        }),
    );
    c.ok("set_operation_geometry", json!({ "operation": "P1", "picks": [{ "model": "plate", "id": up["id"] }] }));
    let g = c.ok("get_operation_geometry", json!({ "operation": "P1" }));
    let shown = &g["source"]["picks"][0];
    assert_eq!(shown["id"], up["id"], "{g}");
    for key in ["source_name", "source_names", "source_unverified"] {
        assert_eq!(shown.get(key), up.get(key), "{key}: {g}");
    }
}
