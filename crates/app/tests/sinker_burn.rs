//! #541 (part of #290): the Sinker Burn - one rectangular electrode
//! burned straight down Z to a depth read from a stated datum, on a
//! sinker EDM only. The electrode is a tool-library tool (validated at
//! AddTool), the condition's spark gap places the tip (#531), a depth
//! past the cavity floor datum is refused by name, and the 3-axis stock
//! sim verifies the plunge with the electrode's box atom (#290 D7): a
//! burn deeper than the part's cavity is convicted. The CL is golden.
//!
//! Gate map (#541): covers app::operation (sinker_op, kinds, params,
//! stamp), app::commands::operation (family gate), app::stock_sim
//! (plan_program, carry walk), app::project (sinker DTO), motion::sinker,
//! sim::stock_sim (dexel_box), tools::electrode.
//! Provenance: our own SYNTHETIC fixtures - the sinker template of #530
//! with an invented condition (no machine's manual), and a hand-built
//! block with a rectangular cavity. Not exercised on a real EDM control.

mod common;

use flecs_ecs::prelude::World;

use camcraft_app::commands::{
    AddTool, CreateOperation, EditOperationParams, GenerateOperation, History, ImportModel, LoadMachine, SetStock,
};
use camcraft_app::geometry::{Mesh, Point3, Triangle};
use camcraft_app::machine::parse_mch;
use camcraft_app::operation::{Height, OpParams, SinkerBurnOp, SinkerDatum, SrcRef};
use camcraft_app::sinker::{burn_to_depth, SinkerParams};
use camcraft_app::stock::StockShape;
use camcraft_app::stock_sim::{simulate_box, simulate_document, Dexel};
use camcraft_app::tools::Tool;
use camcraft_app::world::new_world;
use camcraft_app::{model, project};

const SINKER: &str = include_str!("edm/sinker-edm.mch");
const MILL_TOOL: &str = "EM-0400-2FL";
const ELECTRODE: &str = "E-10x6";
/// Synthetic spark gap per side, mm.
const GAP: f64 = 0.05;

/// The sinker template with one synthetic condition.
fn sinker_machine() -> String {
    let mut v: serde_json::Value = serde_json::from_str(SINKER).expect("template json");
    v["edm"]["conditions"] = serde_json::json!([
        { "id": "S1", "name": "synthetic rough", "gap": GAP, "undersize": 0.1, "role": "rough" }
    ]);
    v.to_string()
}

fn electrode() -> Tool {
    camcraft_tools::electrode::electrode_tool(10.0, 6.0, 30.0, 15.0)
}

fn quad(out: &mut Vec<Triangle>, a: Point3, b: Point3, c: Point3, d: Point3) {
    out.push(Triangle::new(a, b, c));
    out.push(Triangle::new(a, c, d));
}

/// A 60 x 60 x 20 block (z 0..20) with a rectangular blind cavity
/// `2hx` x `2hy`, `depth` deep, centred on the origin.
fn block_with_cavity(hx: f64, hy: f64, depth: f64) -> Mesh {
    let p = Point3::new;
    let (s, top, fl) = (30.0, 20.0, 20.0 - depth);
    let mut f = Vec::new();
    // Top, around the cavity: four strips.
    quad(&mut f, p(-s, -s, top), p(-hx, -s, top), p(-hx, s, top), p(-s, s, top));
    quad(&mut f, p(hx, -s, top), p(s, -s, top), p(s, s, top), p(hx, s, top));
    quad(&mut f, p(-hx, -s, top), p(hx, -s, top), p(hx, -hy, top), p(-hx, -hy, top));
    quad(&mut f, p(-hx, hy, top), p(hx, hy, top), p(hx, s, top), p(-hx, s, top));
    // Cavity floor and walls.
    quad(&mut f, p(-hx, -hy, fl), p(hx, -hy, fl), p(hx, hy, fl), p(-hx, hy, fl));
    quad(&mut f, p(-hx, -hy, fl), p(-hx, -hy, top), p(hx, -hy, top), p(hx, -hy, fl));
    quad(&mut f, p(hx, hy, fl), p(hx, hy, top), p(-hx, hy, top), p(-hx, hy, fl));
    quad(&mut f, p(-hx, hy, fl), p(-hx, hy, top), p(-hx, -hy, top), p(-hx, -hy, fl));
    quad(&mut f, p(hx, -hy, fl), p(hx, -hy, top), p(hx, hy, top), p(hx, hy, fl));
    // Outer walls and bottom.
    quad(&mut f, p(-s, -s, 0.0), p(s, -s, 0.0), p(s, -s, top), p(-s, -s, top));
    quad(&mut f, p(s, s, 0.0), p(-s, s, 0.0), p(-s, s, top), p(s, s, top));
    quad(&mut f, p(-s, s, 0.0), p(-s, -s, 0.0), p(-s, -s, top), p(-s, s, top));
    quad(&mut f, p(s, -s, 0.0), p(s, s, 0.0), p(s, s, top), p(s, -s, top));
    quad(&mut f, p(-s, -s, 0.0), p(-s, s, 0.0), p(s, s, 0.0), p(s, -s, 0.0));
    Mesh::from_facets(f)
}

fn burn(datum: SinkerDatum, depth: f64, floor: f64) -> OpParams {
    OpParams::SinkerBurn(SinkerBurnOp {
        clearance_z: Height::fixed(25.0),
        top_z: Height::fixed(20.0),
        floor_z: Height::fixed(floor),
        datum,
        depth,
        x: 0.0,
        y: 0.0,
        feed: 40.0,
        condition: "S1".into(),
        orbit: None,
    })
}

/// A sinker document: the part (cavity `cavity_depth` deep, sized to the
/// electrode plus the gap), a matching block of stock, the sinker
/// machine, the electrode.
fn sinker_doc(cavity_depth: f64) -> (World, History) {
    let world = new_world();
    let mut h = History::default();
    h.execute(&world, ImportModel::from_mesh("part", block_with_cavity(5.0 + GAP, 3.0 + GAP, cavity_depth)));
    let stock = SetStock::new(&world, StockShape::Box { dx: 60.0, dy: 60.0, dz: 20.0 }, Default::default());
    h.execute(&world, stock.expect("stock"));
    let m = parse_mch(&sinker_machine()).expect("sinker parses");
    h.execute(&world, LoadMachine::new(&world, "sinker.mch".into(), Some(m)).expect("mount"));
    h.execute(&world, AddTool::new(&world, ELECTRODE, electrode()).expect("an electrode is a tool"));
    (world, h)
}

fn feed_tip(world: &World, name: &str) -> f64 {
    let e = model::find_by_name(world, name).expect("op");
    let path = model::toolpath_of(world, &e).expect("generated").path;
    path.moves.iter().filter(|m| m.kind == camcraft_app::cl::MoveKind::Feed).map(|m| m.to.z).fold(f64::INFINITY, f64::min)
}

#[test]
fn sinker_burn_cavity_matches_golden() {
    // A 6 mm burn from the work top of a 20 mm block: approach at 25,
    // the tip fed to one 0.05 gap above the floor at Z 14, retract.
    let p = SinkerParams {
        x: 12.5,
        y: -7.5,
        approach_z: 25.0,
        top_z: 20.0,
        floor_z: 10.0,
        model_floor_z: Some(0.0),
        datum: SinkerDatum::WorkTop,
        depth: 6.0,
        electrode_len: 30.0,
        feed: 40.0,
        gap: GAP,
        orbit: None,
    };
    let path = burn_to_depth(&p).expect("burns");
    let text = common::gen_to_cltext(&path);
    assert_eq!(camcraft_app::cl::text::to_text(&path).expect("text"), text);
    common::write_golden("sinker_burn_cavity.cl", &text);
}

#[test]
fn a_rectangular_electrode_burns_its_cavity_to_the_stated_depth() {
    let (world, mut h) = sinker_doc(6.0);
    h.execute(&world, CreateOperation::new(&world, "Burn1", ELECTRODE, SrcRef::Stock, burn(SinkerDatum::WorkTop, 6.0, 14.0)).expect("a sinker burn on a sinker"));
    h.execute(&world, GenerateOperation::new(&world, "Burn1").expect("generates"));
    assert!((feed_tip(&world, "Burn1") - (14.0 + GAP)).abs() < 1e-9, "the tip stops one gap above the floor");

    // The column engine, read directly: inside the face grown by the gap
    // the block stands at the floor, outside it untouched.
    let e = model::find_by_name(&world, "Burn1").expect("op");
    let moves = model::toolpath_of(&world, &e).expect("path").path.moves;
    let mut dex = Dexel::block(-30.0, -30.0, 30.0, 30.0, 20.0, 0.25).expect("grid");
    simulate_box(&mut dex, &moves, 5.0, 3.0, GAP).expect("burns");
    let top = |x: f64, y: f64| {
        let k = dex.col(camcraft_app::geo2d::Point2::new(x, y)).expect("in the grid");
        dex.top_at(k)
    };
    assert!((top(0.0, 0.0) - 14.0).abs() < 1e-9 && (top(4.9, 2.9) - 14.0).abs() < 1e-9, "the cavity floor");
    assert_eq!(top(5.3, 0.0), 20.0, "beside the face, past the gap");
    assert_eq!(top(0.0, 3.3), 20.0);

    // The program verdict judges it clean against the part.
    let sim = simulate_document(&world).expect("judged");
    assert_eq!(sim.dev.gouge_cells, 0, "a burn to the cavity's own floor gouges nothing: {:?}", sim.dev.max_gouge_at);
}

#[test]
fn a_burn_deeper_than_the_cavity_is_convicted_by_the_stock_sim() {
    // The floor datum admits 7 mm, but the part's cavity is 6 deep: the
    // door passes it, the box atom convicts it.
    let (world, mut h) = sinker_doc(6.0);
    h.execute(&world, CreateOperation::new(&world, "Burn1", ELECTRODE, SrcRef::Stock, burn(SinkerDatum::WorkTop, 7.0, 13.0)).expect("create"));
    h.execute(&world, GenerateOperation::new(&world, "Burn1").expect("generates"));
    let sim = simulate_document(&world).expect("judged");
    assert!(sim.dev.gouge_cells > 0, "a 1 mm overburn is a gouge");
    assert!((sim.dev.max_gouge - 1.0).abs() < 0.05, "max gouge {}", sim.dev.max_gouge);
}

#[test]
fn a_depth_past_the_floor_datum_is_refused_by_name() {
    let (world, mut h) = sinker_doc(6.0);
    h.execute(&world, CreateOperation::new(&world, "Burn1", ELECTRODE, SrcRef::Stock, burn(SinkerDatum::WorkTop, 6.5, 14.0)).expect("create"));
    let Err(err) = GenerateOperation::new(&world, "Burn1") else { panic!("a burn past the floor datum generated") };
    assert!(err.to_string().contains("past the cavity floor datum"), "{err}");
    // From the cavity floor: the stand-off is what is left above it.
    let next = burn(SinkerDatum::CavityFloor, 0.0, 14.0);
    h.execute(&world, EditOperationParams::new(&world, "Burn1", next).expect("edit"));
    h.execute(&world, GenerateOperation::new(&world, "Burn1").expect("burns to the floor"));
    assert!((feed_tip(&world, "Burn1") - (14.0 + GAP)).abs() < 1e-9);
    // From the electrode tip at the approach plane (Z 25): 11 reaches Z 14.
    h.execute(&world, EditOperationParams::new(&world, "Burn1", burn(SinkerDatum::ElectrodeTip, 11.0, 14.0)).expect("edit"));
    h.execute(&world, GenerateOperation::new(&world, "Burn1").expect("burns"));
    assert!((feed_tip(&world, "Burn1") - (14.0 + GAP)).abs() < 1e-9);
    h.execute(&world, EditOperationParams::new(&world, "Burn1", burn(SinkerDatum::ElectrodeTip, 11.5, 14.0)).expect("edit"));
    let Err(err) = GenerateOperation::new(&world, "Burn1") else { panic!("past the floor from the tip") };
    assert!(err.to_string().contains("past the cavity floor datum"), "{err}");
}

#[test]
fn a_sinker_burn_needs_a_sinker_and_a_stated_condition() {
    // On a mill: refused at the door, by name.
    let world = new_world();
    let mut h = History::default();
    h.execute(&world, SetStock::new(&world, StockShape::Box { dx: 60.0, dy: 60.0, dz: 20.0 }, Default::default()).expect("stock"));
    h.execute(&world, AddTool::new(&world, ELECTRODE, electrode()).expect("tool"));
    let Err(err) = CreateOperation::new(&world, "Burn1", ELECTRODE, SrcRef::Stock, burn(SinkerDatum::WorkTop, 6.0, 14.0)) else {
        panic!("a sinker burn entered a mill");
    };
    assert!(err.to_string().contains("sinker EDM work") && err.to_string().contains("mount a sinker EDM"), "{err}");

    // On the sinker: a milling tool is refused by the tool law, a milling
    // op stays refused (#530), and an unknown condition is refused by name.
    let (world, mut h) = sinker_doc(6.0);
    let mill = camcraft_app::tools::catalog().into_iter().find(|(n, _)| *n == MILL_TOOL).map(|(_, t)| t).expect("catalog");
    h.execute(&world, AddTool::new(&world, MILL_TOOL, mill).expect("tool"));
    assert!(CreateOperation::new(&world, "Burn0", MILL_TOOL, SrcRef::Stock, burn(SinkerDatum::WorkTop, 6.0, 14.0)).is_err(), "an end mill is no electrode");
    let mut p = burn(SinkerDatum::WorkTop, 6.0, 14.0);
    if let OpParams::SinkerBurn(op) = &mut p {
        op.condition = "S9".into();
    }
    h.execute(&world, CreateOperation::new(&world, "Burn1", ELECTRODE, SrcRef::Stock, p).expect("create"));
    let Err(err) = GenerateOperation::new(&world, "Burn1") else { panic!("an unknown condition generated") };
    assert!(err.to_string().contains("no EDM condition \"S9\""), "{err}");
}

#[test]
fn an_electrode_is_refused_at_add_tool_when_garbled_and_round_trips_the_file() {
    let world = new_world();
    let spun = Tool { rpm: 500.0, ..electrode() };
    let Err(err) = AddTool::new(&world, "E-bad", spun) else { panic!("a spinning electrode was admitted") };
    assert!(err.to_string().contains("turns no spindle"), "{err}");
    let flat = Tool { width: 0.0, ..electrode() };
    assert!(AddTool::new(&world, "E-flat", flat).is_err(), "no Y size");

    let (world, mut h) = sinker_doc(6.0);
    h.execute(&world, CreateOperation::new(&world, "Burn1", ELECTRODE, SrcRef::Stock, burn(SinkerDatum::CavityFloor, 0.25, 14.0)).expect("create"));
    let path = std::env::temp_dir().join(format!("camcraft-541-{}.ccam", std::process::id()));
    project::save(&world, &path).expect("save");
    let back = new_world();
    project::load(&back, &path).expect("load");
    let _ = std::fs::remove_file(&path);
    let e = model::find_by_name(&back, "Burn1").expect("op survives");
    assert_eq!(model::operation_of(&back, &e).expect("op").params, burn(SinkerDatum::CavityFloor, 0.25, 14.0));
    let t = model::find_by_name(&back, ELECTRODE).and_then(|t| model::tool_of(&back, &t)).expect("tool survives");
    assert_eq!(t, electrode());
}

#[test]
fn the_sinker_burn_is_driven_over_mcp_with_the_gui_commands() {
    use camcraft_app::mcp::Server;
    use serde_json::{json, Value};
    fn call(s: &mut Server, id: &mut i64, name: &str, args: Value) -> Value {
        *id += 1;
        let msg = json!({ "jsonrpc": "2.0", "id": *id, "method": "tools/call", "params": { "name": name, "arguments": args } });
        let reply = s.handle(&msg).expect("a request gets a response");
        let result = &reply["result"];
        assert_ne!(result["isError"], json!(true), "{name} refused: {}", result["content"][0]["text"]);
        result["structuredContent"].clone()
    }
    let mch = std::env::temp_dir().join(format!("camcraft-541-{}.mch", std::process::id()));
    std::fs::write(&mch, sinker_machine()).expect("write the machine");
    let (mut s, mut id) = (Server::new(), 0);
    let _ = s.handle(&json!({ "jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {} }));
    call(&mut s, &mut id, "set_stock", json!({ "box": { "dx": 60, "dy": 60, "dz": 20 } }));
    call(&mut s, &mut id, "load_machine", json!({ "path": mch.display().to_string() }));
    let _ = std::fs::remove_file(&mch);
    let def = project::tool_to_json(&electrode());
    call(&mut s, &mut id, "add_tool", json!({ "name": ELECTRODE, "definition": def, "holder": "none" }));
    let params = json!({ "depth": 6, "condition": "S1", "top_z": { "reference": "fixed", "offset": 20 }, "floor_z": { "reference": "fixed", "offset": 14 } });
    call(&mut s, &mut id, "add_operation", json!({ "kind": "Sinker Burn", "tool": ELECTRODE, "source": "stock", "name": "Burn1", "params": params }));
    call(&mut s, &mut id, "generate", json!({ "name": "Burn1" }));
    // Deeper than the floor datum: the edit lands (one undo step), the
    // generation refuses by name.
    call(&mut s, &mut id, "edit_operation", json!({ "name": "Burn1", "params": { "depth": 6.5 } }));
    id += 1;
    let reply = s
        .handle(&json!({ "jsonrpc": "2.0", "id": id, "method": "tools/call", "params": { "name": "generate", "arguments": { "name": "Burn1" } } }))
        .expect("a response");
    let text = reply["result"]["content"][0]["text"].as_str().unwrap_or_default().to_owned();
    assert_eq!(reply["result"]["isError"], json!(true), "{reply}");
    assert!(text.contains("past the cavity floor datum"), "{text}");
    call(&mut s, &mut id, "undo", json!({}));
    call(&mut s, &mut id, "generate", json!({ "name": "Burn1" }));
}
