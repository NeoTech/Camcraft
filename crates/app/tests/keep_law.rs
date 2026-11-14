//! #1603: the pocket family (2D Pocket, 2D Adaptive, 2D Offset) keeps
//! the part, and says so on the operation (`DepthLaw::Keep`,
//! `operation::keep_law`). A whole-model source is cleared AROUND (#12);
//! an area offset or extension that carries the boundary into the part,
//! or a stock source that covers it (#1554), is a blocking
//! `Warning::Gouge` on the operation and the post is refused by name on
//! its moves - with the stock aligned and with it turned (where the
//! program sim refuses to judge). Before #1603 these carried
//! `DepthLaw::None`: 0 findings on the op, and with the stock turned the
//! post went out 4-7 mm into the part. A legal pocket or adaptive with
//! the default area controls stays clean and posts on both stocks.
//!
//! Provenance: our own fixtures (the #12 block with a boss) over the
//! sim crate's solid law; no vendor material consulted.
//!
//! Gate map: covers app::operation::keep_law, app::operation::generate
//! (depth_law_of, the Keep arm), app::operation::job (keep_parts),
//! app::posting, app::posting_sim (uncovered), app::mcp (add_operation,
//! set_operation_area, verify).

mod area_support;
mod pocket_support;

use area_support::*;
use camcraft_app::commands::GenerateOperation;
use camcraft_app::geometry::{quat_from_yaw, Mesh, Triangle};
use camcraft_app::mcp::Server;
use camcraft_app::posting;
use camcraft_app::verify::Warning;
use serde_json::{json, Value};

/// A 30 x 30 x [0, 10] block with a 12 x 12 boss to Z 20 (the #12
/// fixture). Outward-wound.
fn block_with_boss() -> Mesh {
    let quad = |a: Point3, b: Point3, c: Point3, d: Point3, f: &mut Vec<Triangle>| {
        f.push(Triangle::new(a, b, c));
        f.push(Triangle::new(a, c, d));
    };
    let mut f = Vec::new();
    let block = cube(Point3::new(-15.0, -15.0, 0.0), Point3::new(15.0, 15.0, 10.0));
    f.extend(block.facets[0..2].iter().cloned()); // bottom
    f.extend(block.facets[4..12].iter().cloned()); // walls
    let (o, i) = (15.0, 6.0);
    let oo = [(-o, -o), (o, -o), (o, o), (-o, o)];
    let ii = [(-i, -i), (i, -i), (i, i), (-i, i)];
    let p = |(x, y): (f64, f64)| Point3::new(x, y, 10.0);
    for k in 0..4 {
        let n = (k + 1) % 4;
        quad(p(oo[k]), p(oo[n]), p(ii[n]), p(ii[k]), &mut f); // shoulder
    }
    let boss = cube(Point3::new(-i, -i, 10.0), Point3::new(i, i, 20.0));
    f.extend(boss.facets[2..12].iter().cloned()); // boss top + walls
    Mesh::from_facets(f)
}

/// The stock's side; it stands on [-25, 25]^2 x [0, 20] either way.
const SIDE: f64 = 50.0;

/// `part` (when given) in the 50 x 50 x 20 stock: axis-aligned, or the
/// same box turned a quarter about Z (the same footprint, but a rotation
/// the program sim refuses to model).
fn job(part: Option<Mesh>, rotated: bool) -> (World, History) {
    let world = new_world();
    let mut h = History::default();
    if let Some(m) = part {
        h.execute(&world, ImportModel::from_mesh("part", m));
    }
    // A box stock stands centred on its place, so a quarter turn about
    // Z leaves the footprint where it was.
    let rot = if rotated { quat_from_yaw(std::f64::consts::FRAC_PI_2) } else { Location::default().rot };
    let at = Location { pos: Point3::new(0.0, 0.0, 0.0), rot };
    h.execute(&world, SetStock::new(&world, StockShape::Box { dx: SIDE, dy: SIDE, dz: 20.0 }, at).expect("stock"));
    let ((lo, hi), half) = (stock_box(&world), SIDE / 2.0);
    let on = |v: f64, want: f64| (v - want).abs() < 1e-9;
    assert!(on(lo.x, -half) && on(lo.y, -half) && on(hi.x, half) && on(hi.y, half), "stock at {lo:?}..{hi:?}");
    let tool = catalog().into_iter().find(|(n, _)| *n == TOOL).map(|(_, t)| t).expect("catalog");
    h.execute(&world, AddTool::new(&world, TOOL, tool).expect("tool"));
    (world, h)
}

fn retyped(to: OpType, top: f64, bottom: f64) -> OpParams {
    let mut p = operation::retype_params(&pocket(top, bottom), to as usize);
    match &mut p {
        OpParams::Adaptive2D { top_z, bottom_z, stepdown, stepover, .. }
        | OpParams::Offset2D { top_z, bottom_z, stepdown, stepover, .. } => {
            *top_z = Height::fixed(top);
            *bottom_z = Height::fixed(bottom);
            *stepdown = 5.0;
            *stepover = 4.0;
        }
        _ => {}
    }
    assert_eq!(p.kind(), to);
    p
}

/// The two kinds #1603 names, cut from `top` to `bottom`.
fn kinds(top: f64, bottom: f64) -> [(&'static str, OpParams); 2] {
    [("2D Pocket", pocket(top, bottom)), ("2D Adaptive", retyped(OpType::Adaptive2D, top, bottom))]
}

/// Create "P1" with `area`, generate it; its cached warnings.
fn generated(world: &World, h: &mut History, src: SrcRef, params: OpParams, area: AreaControls) -> Vec<Warning> {
    h.execute(world, CreateOperation::new(world, "P1", TOOL, src, params.clone()).expect("create"));
    if !area.is_default() {
        h.execute(world, EditOperationParams::with_area(world, "P1", params, area).expect("area"));
    }
    h.execute(world, GenerateOperation::new(world, "P1").expect("generate"));
    let e = model::find_by_name(world, "P1").expect("op");
    let tp = model::toolpath_of(world, &e).expect("cached path");
    assert!(!tp.path.moves.is_empty(), "the area was generated");
    tp.warnings.clone()
}

fn gouges(w: &[Warning]) -> usize {
    w.iter().filter(|w| matches!(w, Warning::Gouge { .. })).count()
}

fn nc(tag: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("camcraft-keep-law-{tag}-{}.nc", std::process::id()))
}

/// The post is refused and nothing is written. `by_op`: the refusal
/// names the operation's own gouge (where the program sim judges too,
/// its clause may be the one the message leads with).
fn assert_post_refused(world: &World, tag: &str, by_op: bool) {
    let out = nc(tag);
    let _ = std::fs::remove_file(&out);
    let err = posting::post_setup(world, &out).expect_err("a gouge blocks the post");
    assert!(err.contains("post blocked"), "{tag}: {err}");
    if by_op {
        assert!(err.contains("P1: move") && err.contains("cuts past the finish surface"), "{tag}: {err}");
    }
    assert!(!out.exists(), "{tag}: blocked output is never written");
}

fn assert_posts(world: &World, tag: &str) {
    let out = nc(tag);
    let _ = std::fs::remove_file(&out);
    posting::post_setup(world, &out).unwrap_or_else(|e| panic!("{tag}: a legal program posts: {e}"));
    assert!(out.exists(), "{tag}: the program was written");
    let _ = std::fs::remove_file(&out);
}

fn whole() -> SrcRef {
    SrcRef::Entity("part".into())
}

/// A whole-model `area` case: convicted on the op and never posted, on
/// both stocks.
fn refused_on_both_stocks(area: &AreaControls, tag: &str) {
    for (label, params) in kinds(20.0, 5.0) {
        for rotated in [false, true] {
            let (world, mut h) = job(Some(block_with_boss()), rotated);
            let warns = generated(&world, &mut h, whole(), params.clone(), area.clone());
            assert!(gouges(&warns) > 0, "{label} {tag} rotated {rotated}: convicted on the op: {warns:?}");
            assert_post_refused(&world, &format!("{tag}-{label}-{rotated}"), rotated);
        }
    }
}

#[test]
fn a_whole_model_area_offset_into_the_part_gouges_and_never_posts() {
    // Offset +3: the boundary grows 3 mm into the part's own section.
    // Before #1603: 0 findings, and it POSTED with the stock turned.
    refused_on_both_stocks(&AreaControls { offset: 3.0, ..Default::default() }, "offset");
}

#[test]
fn a_whole_model_extension_into_the_part_gouges_and_never_posts() {
    refused_on_both_stocks(&AreaControls { extension: 5.0, ..Default::default() }, "extension");
}

#[test]
fn a_stock_source_cut_through_the_part_gouges_and_never_posts() {
    // #1554: the stock as the source clears its whole footprint, the
    // part included. Before #1603 nothing on the op said so.
    for (label, params) in kinds(20.0, 5.0) {
        for rotated in [false, true] {
            let (world, mut h) = job(Some(block_with_boss()), rotated);
            let warns = generated(&world, &mut h, SrcRef::Stock, params.clone(), AreaControls::default());
            assert!(gouges(&warns) > 0, "{label} rotated {rotated}: the part under the stock is convicted: {warns:?}");
            assert_post_refused(&world, &format!("stock-{label}-{rotated}"), rotated);
        }
    }
}

#[test]
fn a_legal_whole_model_clearing_stays_clean_and_posts_on_both_stocks() {
    // Default area controls: every ring stands outside the part. A
    // turned stock must not refuse it - that would be a false refusal.
    let all = [
        ("2D Pocket", pocket(20.0, 5.0)),
        // Two levels, the last on the block's top: around the boss and
        // tangent to the shoulder (the Z5 ring round the block costs a
        // debug build a minute per run and adds no other contact).
        ("2D Adaptive", retyped(OpType::Adaptive2D, 20.0, 10.0)),
        ("2D Offset", retyped(OpType::Offset2D, 20.0, 5.0)),
    ];
    for (label, params) in all {
        for rotated in [false, true] {
            let (world, mut h) = job(Some(block_with_boss()), rotated);
            let warns = generated(&world, &mut h, whole(), params.clone(), AreaControls::default());
            let blocking: Vec<_> = warns.iter().filter(|w| w.blocks_post()).collect();
            assert!(blocking.is_empty(), "{label} rotated {rotated}: {blocking:?}");
            assert_posts(&world, &format!("legal-{label}-{rotated}"));
        }
    }
}

#[test]
fn a_stock_source_cut_that_stops_on_the_part_stays_clean_and_posts() {
    // A plain 30 x 30 x 10 block under 10 mm of stock: clearing the
    // stock down to the block's top touches it and no more.
    let block = || Mesh::from_facets(cube(Point3::new(-15.0, -15.0, 0.0), Point3::new(15.0, 15.0, 10.0)).facets.to_vec());
    for (label, params) in kinds(20.0, 10.0) {
        for rotated in [false, true] {
            let (world, mut h) = job(Some(block()), rotated);
            let warns = generated(&world, &mut h, SrcRef::Stock, params.clone(), AreaControls::default());
            let blocking: Vec<_> = warns.iter().filter(|w| w.blocks_post()).collect();
            assert!(blocking.is_empty(), "{label} rotated {rotated}: {blocking:?}");
            assert_posts(&world, &format!("stock-legal-{label}-{rotated}"));
        }
    }
}

#[test]
fn a_stock_source_cut_with_no_part_in_the_setup_judges_nothing() {
    for (label, params) in kinds(20.0, 5.0) {
        let (world, mut h) = job(None, false);
        let warns = generated(&world, &mut h, SrcRef::Stock, params, AreaControls::default());
        assert_eq!(gouges(&warns), 0, "{label}: no part, no part law: {warns:?}");
    }
}

// ---- MCP parity -----------------------------------------------------

fn call(s: &mut Server, id: &mut i64, name: &str, args: Value) -> Value {
    *id += 1;
    let msg = json!({ "jsonrpc": "2.0", "id": *id, "method": "tools/call",
        "params": { "name": name, "arguments": args } });
    let reply = s.handle(&msg).expect("a request gets a response");
    let result = &reply["result"];
    assert_ne!(result["isError"], json!(true), "{name} refused: {}", result["content"][0]["text"]);
    result["structuredContent"].clone()
}

/// The boss job over MCP: a `kind` clearing from `source` with `area`;
/// verify's reply.
fn mcp_verify(tag: &str, kind: &str, source: &str, area: Option<Value>) -> Value {
    let dir = std::env::temp_dir().join(format!("camcraft-keep-law-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let stl = dir.join("part.stl");
    pocket_support::write_stl(&block_with_boss(), &stl);
    let (mut s, mut id) = (Server::new(), 0);
    let _ = s.handle(&json!({ "jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {} }));
    call(&mut s, &mut id, "import_model", json!({ "name": "part", "path": stl.display().to_string() }));
    call(&mut s, &mut id, "set_stock", json!({ "box": { "dx": SIDE, "dy": SIDE, "dz": 20.0 } }));
    call(&mut s, &mut id, "add_tool", json!({ "catalog": TOOL }));
    let params = json!({ "top_z": 20.0, "bottom_z": 5.0, "stepdown": 5.0, "stepover": 4.0, "stock_to_leave": 0.0 });
    call(&mut s, &mut id, "add_operation", json!({ "kind": kind, "tool": TOOL, "source": source, "name": "P1", "params": params }));
    if let Some(area) = area {
        call(&mut s, &mut id, "set_operation_area", json!({ "operation": "P1", "area": area }));
    }
    let g = call(&mut s, &mut id, "generate", json!({}));
    assert_eq!(g["generated"][0]["status"], json!("ok"), "{g}");
    let ver = call(&mut s, &mut id, "verify", json!({}));
    let _ = std::fs::remove_dir_all(&dir);
    ver
}

/// The operation's own blocking gouge findings (not the program gate's).
fn op_gouges(ver: &Value) -> usize {
    ver["findings"]
        .as_array()
        .map_or(0, |f| f.iter().filter(|x| x["kind"] == json!("gouge") && x["op"] == json!("P1") && x["blocks_post"] == json!(true)).count())
}

#[test]
fn over_mcp_the_pocket_family_keeps_the_part_the_same() {
    // One kind over the wire: the law is the kinds' shared arm, pinned
    // per kind above; this pins that the tools reach it.
    let kind = "2D Pocket";
    let offset = mcp_verify("offset", kind, "part", Some(json!({ "offset": 3.0 })));
    assert!(op_gouges(&offset) > 0, "an offset into the part is the op's own gouge: {offset}");
    assert_eq!(offset["ok"], json!(false), "{offset}");
    let stock = mcp_verify("stock", kind, "stock", None);
    assert!(op_gouges(&stock) > 0, "the stock source over the part is the op's own gouge: {stock}");
    assert_eq!(stock["ok"], json!(false), "{stock}");
    let good = mcp_verify("legal", kind, "part", None);
    assert_eq!(op_gouges(&good), 0, "{good}");
    assert_eq!(good["ok"], json!(true), "a legal clearing verifies ok: {good}");
}
