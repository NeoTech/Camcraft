//! #1538: "Fit stock to models" wraps the PARTS, never the fixtures. A
//! 50 x 50 x 6 block with a 5 in vise closed on its stock (and with a
//! soft jaw, a dowel plate and tagged fixtures) fits the same box, the
//! same snug cylinder and the same per-side box as the bare block; the
//! fit is one undoable step; over MCP `fit_stock` does the same.
//! Provenance: our own fixtures (the #1529 reproduction: a 50 x 50 x 6
//! block, Z 4..10, in a 55 x 55 x 11 box stock); no vendor material
//! consulted.
//!
//! Gate map (#1538): covers app::stock_fit (part_bounds,
//! part_radius_about_in, fitted, fit_command), app::mcp
//! (stock_fit_tools), app::commands (SetStock, InsertVise,
//! InsertAccessory, InsertPlate, TagFixture).

use flecs_ecs::prelude::World;
use serde_json::{Value, json};

use camcraft_app::accessory::Accessory;
use camcraft_app::commands::{
    AccessoryPlacement, History, ImportModel, InsertAccessory, InsertPlate, InsertVise, PlatePlacement, SetStock,
    TagFixture, VisePlacement,
};
use camcraft_app::geometry::{Aabb, IDENTITY_Q, Point3, cube};
use camcraft_app::mcp::Server;
use camcraft_app::model;
use camcraft_app::plate;
use camcraft_app::schema::Location;
use camcraft_app::stock::StockShape;
use camcraft_app::stock_fit::{self, FitKind};
use camcraft_app::vise;
use camcraft_app::world::new_world;

const MARGIN: f64 = 2.0;

fn at(x: f64, y: f64, z: f64) -> Location {
    Location { pos: Point3::new(x, y, z), rot: IDENTITY_Q }
}

/// The 50 x 50 x 6 block (X/Y 2.5..52.5, Z 4..10) in a 55 x 55 x 11 box.
fn bare() -> (World, History) {
    let world = new_world();
    let mut h = History::default();
    h.execute(&world, ImportModel::from_mesh("block", cube(Point3::new(2.5, 2.5, 4.0), Point3::new(52.5, 52.5, 10.0))));
    h.execute(&world, SetStock::new(&world, StockShape::Box { dx: 55.0, dy: 55.0, dz: 11.0 }, Location::default()).expect("stock"));
    (world, h)
}

/// The bare block with a 5 in vise closed on its stock - the jaws stand
/// well outside the part in X, Y and Z.
fn with_vise() -> (World, History) {
    let (world, mut h) = bare();
    let p = vise::size("5 in").expect("5 in").half_open();
    let cmd = InsertVise::new(&world, &p, None, VisePlacement::FitStock).expect("fits");
    let name = cmd.name().to_owned();
    h.execute(&world, cmd);
    let jaws = model::find_by_name(&world, &name).and_then(|e| model::entity_world_aabb(&world, &e)).expect("vise box");
    assert!(jaws.max.z > 11.0 && jaws.max.x - jaws.min.x > 100.0, "the vise is bigger than the part: {jaws:?}");
    (world, h)
}

/// Every kind of workholding at once: the vise, a soft jaw, a dowel plate
/// and two tagged mesh fixtures.
fn with_all_fixtures() -> (World, History) {
    let (world, mut h) = with_vise();
    let jaw = Accessory::default_of("soft_jaw").expect("soft jaw");
    h.execute(&world, InsertAccessory::new(&world, &jaw, None, AccessoryPlacement::At(at(27.5, 55.0, 0.0))).expect("soft jaw"));
    let plate = plate::preset("300 x 300 mm").expect("preset").params();
    h.execute(&world, InsertPlate::new(&world, &plate, None, PlatePlacement::At(at(-122.5, -122.5, 0.0))).expect("plate"));
    h.execute(&world, ImportModel::from_mesh("Clamp", cube(Point3::new(60.0, 0.0, 0.0), Point3::new(70.0, 10.0, 80.0))));
    h.execute(&world, TagFixture::new(&world, "Clamp", None).expect("tag"));
    (world, h)
}

/// Fit the active setup's stock; the stock's world box after.
fn fit(world: &World, h: &mut History, kind: FitKind) -> (StockShape, Aabb) {
    let (cmd, shape) = stock_fit::fit_command(world, kind).expect("fits");
    h.execute(world, cmd);
    (shape, model::stock_world_aabb(world).expect("stock box"))
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

fn assert_box(b: &Aabb, min: [f64; 3], max: [f64; 3]) {
    let got = [b.min.x, b.min.y, b.min.z, b.max.x, b.max.y, b.max.z];
    let want = [min[0], min[1], min[2], max[0], max[1], max[2]];
    assert!(got.iter().zip(want).all(|(g, w)| near(*g, w)), "stock box {got:?}, want {want:?}");
}

#[test]
fn fit_box_with_a_vise_wraps_the_part_plus_the_margin() {
    let (world, mut h) = with_vise();
    let (shape, b) = fit(&world, &mut h, FitKind::Box(MARGIN));
    assert_eq!(shape, StockShape::Box { dx: 54.0, dy: 54.0, dz: 10.0 }, "the part plus 2 mm a side, not the vise");
    assert_box(&b, [0.5, 0.5, 2.0], [54.5, 54.5, 12.0]);
    let (bw, mut hb) = bare();
    assert_eq!(fit(&bw, &mut hb, FitKind::Box(MARGIN)).0, shape, "the vise changes nothing");
}

#[test]
fn fit_cylinder_with_a_vise_is_snug_about_the_part() {
    let (world, mut h) = with_vise();
    let (shape, b) = fit(&world, &mut h, FitKind::Cylinder(MARGIN));
    let StockShape::Cylinder { diameter, height } = shape else { panic!("a cylinder: {shape:?}") };
    let want = 2.0 * (25.0 * 2f64.sqrt() + MARGIN);
    assert!(near(diameter, want) && near(height, 10.0), "d {diameter} h {height}, want d {want} h 10");
    assert!(near(b.min.z, 2.0) && near(b.max.z, 12.0), "Z {} .. {}", b.min.z, b.max.z);
    let (bw, mut hb) = bare();
    assert_eq!(fit(&bw, &mut hb, FitKind::Cylinder(MARGIN)).0, shape, "the vise changes nothing");
}

#[test]
fn every_fixture_kind_is_left_out_of_every_fit() {
    for kind in [FitKind::Box(MARGIN), FitKind::Cylinder(MARGIN), FitKind::Sides([1.0, 2.0, 3.0, 4.0, 0.5, 1.5])] {
        let (bw, mut hb) = bare();
        let (fw, mut hf) = with_all_fixtures();
        assert_eq!(fit(&fw, &mut hf, kind), fit(&bw, &mut hb, kind), "{kind:?}");
    }
    let (fw, _) = with_all_fixtures();
    let p = stock_fit::part_bounds(&fw, None).expect("the part");
    assert_box(&p, [2.5, 2.5, 4.0], [52.5, 52.5, 10.0]);
}

#[test]
fn the_fit_is_one_undoable_step() {
    let (world, mut h) = with_vise();
    let before = model::stock_world_aabb(&world).expect("stock");
    fit(&world, &mut h, FitKind::Box(MARGIN));
    assert!(h.undo(&world));
    let after = model::stock_world_aabb(&world).expect("stock");
    assert_box(&after, [before.min.x, before.min.y, before.min.z], [before.max.x, before.max.y, before.max.z]);
}

#[test]
fn fixtures_alone_are_nothing_to_fit_and_a_bad_side_is_refused() {
    let world = new_world();
    let mut h = History::default();
    h.execute(&world, ImportModel::from_mesh("Clamp", cube(Point3::new(0.0, 0.0, 0.0), Point3::new(10.0, 10.0, 80.0))));
    h.execute(&world, TagFixture::new(&world, "Clamp", None).expect("tag"));
    let err = stock_fit::fit_command(&world, FitKind::Box(MARGIN)).err().expect("refused");
    assert_eq!(err, "nothing to fit: no models in the document");
    let (bw, _) = bare();
    let err = stock_fit::fit_command(&bw, FitKind::Sides([1.0, -1.0, 0.0, 0.0, 0.0, 0.0])).err().expect("refused");
    assert!(err.contains("allowance must be zero or more"), "{err}");
}

// ---- over MCP ----------------------------------------------------------

fn call(server: &mut Server, id: i64, name: &str, args: Value) -> Result<Value, String> {
    let msg = json!({ "jsonrpc": "2.0", "id": id, "method": "tools/call", "params": { "name": name, "arguments": args } });
    let reply = server.handle(&msg).expect("a response");
    let result = &reply["result"];
    if result["isError"] == json!(true) {
        Err(result["content"][0]["text"].as_str().unwrap_or_default().to_owned())
    } else {
        Ok(result["structuredContent"].clone())
    }
}

/// A 50 x 50 x 6 block as ASCII STL (X/Y 2.5..52.5, Z 4..10).
fn block_stl(tag: &str) -> std::path::PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!("camcraft-stock-fit-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("block.stl");
    let mut s = String::from("solid block\n");
    for t in cube(Point3::new(2.5, 2.5, 4.0), Point3::new(52.5, 52.5, 10.0)).facets.iter() {
        s.push_str(" facet normal 0 0 0\n  outer loop\n");
        for p in [t.a, t.b, t.c] {
            s.push_str(&format!("   vertex {} {} {}\n", p.x, p.y, p.z));
        }
        s.push_str("  endloop\n endfacet\n");
    }
    s.push_str("endsolid block\n");
    std::fs::write(&path, s).expect("write stl");
    path
}

#[test]
fn mcp_fit_stock_with_a_vise_gives_the_part_box() {
    let stl = block_stl("mcp");
    let mut s = Server::new();
    call(&mut s, 1, "import_model", json!({ "path": stl.display().to_string() })).expect("import");
    call(&mut s, 2, "set_stock", json!({ "box": { "dx": 55.0, "dy": 55.0, "dz": 11.0 } })).expect("stock");
    call(&mut s, 3, "add_builtin_vise", json!({ "size": "5 in", "fit_to_stock": true })).expect("vise");
    let fitted = call(&mut s, 4, "fit_stock", json!({ "margin": MARGIN })).expect("fit");
    assert_eq!(fitted["stock"], json!({ "box": { "dx": 54.0, "dy": 54.0, "dz": 10.0 } }), "{fitted}");
    let cyl = call(&mut s, 5, "fit_stock", json!({ "shape": "cylinder", "margin": MARGIN })).expect("fit");
    let d = cyl["stock"]["cylinder"]["diameter"].as_f64().expect("a cylinder");
    assert!(near(d, 2.0 * (25.0 * 2f64.sqrt() + MARGIN)), "{cyl}");
    let sides = call(&mut s, 6, "fit_stock", json!({ "sides": [1.0, 1.0, 1.0, 1.0, 0.0, 0.0] })).expect("fit");
    assert_eq!(sides["stock"], json!({ "box": { "dx": 52.0, "dy": 52.0, "dz": 6.0 } }), "{sides}");
    let bad = call(&mut s, 7, "fit_stock", json!({ "shape": "cone" })).expect_err("refused");
    assert!(bad.contains("\"box\" or \"cylinder\""), "{bad}");
    call(&mut s, 8, "undo", json!({})).expect("undo");
    let _ = std::fs::remove_dir_all(stl.parent().expect("dir"));
}
