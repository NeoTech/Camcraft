//! #3316 / #3319: 2D Offset "Pass starts at" (Auto / Inside / Outside).
//! With an Outside containment on `fig8.step` and EM-0800-4FL, the
//! maintainer's heights (clearance top of stock +10, top of stock +0,
//! bottom of the selection +1.5, stepdown 10, no hop below clearance)
//! cut each level from the core of the band outward: the first ring is a
//! full slot in the stock. Outside starts every level at the stock edge,
//! cuts no move with uncut stock on both flanks along the whole level,
//! and ends it on the part wall. Inside names its part-wall start on the
//! toolpath. The option rides the undoable
//! EditOperationParams through MCP `edit_operation` and the project
//! file, and Auto is absent from both. The #3319 law rides along: no
//! lateral rapid below the clearance plane.
//!
//! Gate map: covers app::operation (generate, params digest), app::mcp
//! (edit_operation, undo), app::project (params, params_impl),
//! motion::machining2d (pass_start, level_rings, level_order), sim::verify.
//! Provenance: our own regression of the maintainer's report; no vendor
//! material consulted.

mod area_support;
mod common;
#[path = "offset_pass_start/flanks.rs"]
mod flanks;

use area_support::*;
use camcraft_app::machining2d::{PASS_START_NOTE, PassStart};
use camcraft_app::mcp::Server;
use camcraft_app::operation::HeightReference;
use serde_json::{Value, json};

const EM8: &str = "EM-0800-4FL";
const R8: f64 = 4.0;
const CLEARANCE: f64 = 10.0;

fn fig8_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/models/fig8.step")
}

/// The part's largest horizontal face at `z` (the fig8 floor).
fn horizontal_face(world: &World, name: &str, z: f64) -> FeatureSel {
    let e = model::find_by_name(world, name).expect("model");
    let f = model::features_of(world, &e).expect("features");
    let mut best: Option<(f64, FeatureSel)> = None;
    for face in f.faces.iter().filter(|face| face.normal[2].abs() > 0.99) {
        let Some(l) = face.loops.iter().find(|l| l.kind == feature::LoopKind::Outer) else { continue };
        let pts: Vec<[f64; 3]> = l.verts.iter().filter_map(|&v| f.verts.get(v as usize).copied()).collect();
        if pts.is_empty() || pts.iter().any(|p| (p[2] - z).abs() > 1e-6) {
            continue;
        }
        let ext = pts.iter().map(|p| p[0].abs().max(p[1].abs())).fold(0.0, f64::max);
        if best.as_ref().is_none_or(|(b, _)| ext > *b) {
            best = Some((ext, FeatureSel::Face(face.id)));
        }
    }
    best.expect("a horizontal face").1
}

/// fig8.step, EM-0800-4FL, a setup, and a stock box 5 mm past the part
/// in XY and 2 mm over its top. Returns the stock box corners.
fn fig8_world() -> (World, History, String, (Point3, Point3)) {
    let world = new_world();
    let mut h = History::default();
    let imp = ImportModel::new(&fig8_path()).expect("fig8.step imports");
    let name = imp.name.clone();
    h.execute(&world, imp);
    let e = model::find_by_name(&world, &name).expect("part");
    let b = model::entity_world_aabb(&world, &e).expect("part aabb");
    let tool = catalog().into_iter().find(|(n, _)| *n == EM8).map(|(_, t)| t).expect("catalog tool");
    h.execute(&world, AddTool::new(&world, EM8, tool).expect("tool"));
    h.execute(&world, camcraft_app::commands::CreateSetup::new(&world, None, None).expect("setup"));
    let m = 5.0;
    let (dx, dy, dz) = ((b.max.x - b.min.x) + 2.0 * m, (b.max.y - b.min.y) + 2.0 * m, (b.max.z - b.min.z) + 2.0);
    let loc = Location { pos: Point3::new(0.5 * (b.min.x + b.max.x), 0.5 * (b.min.y + b.max.y), b.min.z), ..Default::default() };
    h.execute(&world, SetStock::new(&world, StockShape::Box { dx, dy, dz }, loc).expect("stock"));
    let lo = Point3::new(b.min.x - m, b.min.y - m, b.min.z);
    let hi = Point3::new(b.max.x + m, b.max.y + m, b.max.z + 2.0);
    (world, h, name, (lo, hi))
}

/// The maintainer's 2D Offset1: Outside containment over the floor face,
/// generated with `start`.
fn offset_op(world: &World, h: &mut History, name: &str, zmin: f64, start: PassStart) -> (Toolpath, Vec<String>) {
    let src = sel(name, horizontal_face(world, name, zmin));
    let mut p = operation::retype_params(&pocket(zmin + 20.0, zmin), OpType::Offset2D.ordinal());
    if let OpParams::Offset2D { clearance_z, retract_z, top_z, bottom_z, stepdown, entry, helix_radius, pass_start, .. } =
        &mut p
    {
        *clearance_z = Height { reference: HeightReference::TopOfStock, offset: CLEARANCE };
        *retract_z = None;
        *top_z = Height { reference: HeightReference::TopOfStock, offset: 0.0 };
        *bottom_z = Height { reference: HeightReference::BottomOfSelection, offset: 1.5 };
        *stepdown = 10.0;
        *entry = Entry3D::Helix;
        *helix_radius = 1.6;
        *pass_start = start;
    }
    let area = AreaControls { containment: Some(Containment::Outside), ..Default::default() };
    h.execute(world, CreateOperation::new(world, "Off1", EM8, src, p.clone()).expect("create"));
    h.execute(world, EditOperationParams::with_area(world, "Off1", p, area).expect("area"));
    h.execute(world, camcraft_app::commands::GenerateOperation::new(world, "Off1").expect("generate"));
    let e = model::find_by_name(world, "Off1").expect("op");
    let cached = model::toolpath_of(world, &e).expect("cached");
    let warnings = cached.warnings.iter().map(|w| format!("{w:?}")).collect();
    (cached.path.clone(), warnings)
}

/// #3319: every lateral rapid rides at the clearance plane (hop off).
fn lateral_rapids_below(t: &Toolpath, clear_z: f64) -> Vec<(usize, Point3, Point3)> {
    t.moves
        .iter()
        .enumerate()
        .filter(|(_, m)| m.kind == MoveKind::Rapid)
        .filter(|(_, m)| (m.from.x - m.to.x).hypot(m.from.y - m.to.y) > 1e-6)
        .filter(|(_, m)| m.from.z.min(m.to.z) < clear_z - 1e-6)
        .map(|(i, m)| (i, m.from, m.to))
        .collect()
}

/// Each cut level, by Z: where its first ring starts (the end of the
/// level's first planar feed - the entry's own bore stands before it)
/// and where its last planar feed ends.
fn level_ends(t: &Toolpath) -> Vec<(f64, Point3, Point3)> {
    let mut out: Vec<(f64, Point3, Point3)> = Vec::new();
    for m in t.moves.iter().filter(|m| m.kind == MoveKind::Feed && (m.from.z - m.to.z).abs() < 1e-9) {
        match out.iter_mut().find(|(z, _, _)| (z - m.from.z).abs() < 1e-6) {
            Some(level) => level.2 = m.to,
            None => out.push((m.from.z, m.to, m.to)),
        }
    }
    out
}

/// How far `p` stands outside the stock box in XY (0 inside it).
fn outside_box(p: Point3, (lo, hi): (Point3, Point3)) -> f64 {
    let dx = (lo.x - p.x).max(p.x - hi.x).max(0.0);
    let dy = (lo.y - p.y).max(p.y - hi.y).max(0.0);
    dx.hypot(dy)
}

/// Distance from `pt` to the part's projected outline, in XY.
fn to_part(world: &World, name: &str, pt: Point3) -> f64 {
    let e = model::find_by_name(world, name).expect("part");
    let facets = model::world_facets(world, &e).expect("facets");
    let reg = camcraft_app::geometry::slice::project(&facets, 64).expect("part outline");
    let mut best = f64::INFINITY;
    for l in reg.loops() {
        let pts = l.points();
        for i in 0..pts.len() {
            let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
            let (dx, dy) = (b.x - a.x, b.y - a.y);
            let dd = dx * dx + dy * dy;
            let t = if dd > 0.0 { (((pt.x - a.x) * dx + (pt.y - a.y) * dy) / dd).clamp(0.0, 1.0) } else { 0.0 };
            best = best.min((pt.x - a.x - dx * t).hypot(pt.y - a.y - dy * t));
        }
    }
    best
}

/// #3316 pin: Outside starts each level with the tool at the stock edge
/// (its disc outside the stock box, so the first ring cuts one stepover,
/// never a slot) and ends it on the part wall. No lateral rapid runs
/// below the clearance plane, and the verifier says no rapid crosses
/// stock.
#[test]
#[ignore = "heavy lane (#3316 fig8 Outside): one generation and its verify run past the 60 s debug budget (about 170 s) - run with --release -- --ignored"]
fn fig8_outside_offset_starts_each_level_at_the_stock_edge() {
    let (world, mut h, name, stock) = fig8_world();
    let (path, warnings) = offset_op(&world, &mut h, &name, stock.0.z, PassStart::Outside);
    let levels = level_ends(&path);
    assert!(levels.len() >= 2, "the levels: {levels:?}");
    for (z, first, last) in &levels {
        let out = outside_box(*first, stock);
        assert!(out >= R8 - 0.05, "Z {z:.3}: the first ring starts {out:.3} mm outside the stock, in the stock: {first:?}");
        let d = to_part(&world, &name, *last);
        assert!((d - R8).abs() < 0.05, "Z {z:.3}: the level ends {d:.3} mm from the part, not on its wall");
    }
    let clear_z = stock.1.z + CLEARANCE;
    assert_eq!(lateral_rapids_below(&path, clear_z), vec![], "#3319: a lateral rapid below Z {clear_z}");
    assert!(!warnings.iter().any(|w| w.starts_with("RapidInStock")), "#3319: a rapid through stock: {warnings:?}");
}

/// The part's outline and the material of the band: the stock box less
/// the part, in XY.
fn band_material(world: &World, name: &str, (lo, hi): (Point3, Point3)) -> camcraft_app::geo2d::Region {
    use camcraft_app::geo2d::{Loop, Point2, Region};
    let e = model::find_by_name(world, name).expect("part");
    let facets = model::world_facets(world, &e).expect("facets");
    let part = camcraft_app::geometry::slice::project(&facets, 64).expect("part outline");
    let pts = vec![Point2::new(lo.x, lo.y), Point2::new(hi.x, lo.y), Point2::new(hi.x, hi.y), Point2::new(lo.x, hi.y)];
    let stock = Region::new(vec![Loop::new(pts).expect("box")]).expect("stock");
    stock.difference(&part).expect("band")
}

/// #3316 second read: along each WHOLE level (not only its ends), no
/// Outside cut has uncut stock on both flanks - no cut takes the full
/// tool width, where the band at the lobe tips splits the ring.
#[test]
#[ignore = "heavy lane (#3316 fig8 Outside): one generation and its verify run past the 60 s debug budget (about 170 s) - run with --release -- --ignored"]
fn fig8_outside_offset_cuts_no_move_at_full_width() {
    let (world, mut h, name, stock) = fig8_world();
    let (path, _) = offset_op(&world, &mut h, &name, stock.0.z, PassStart::Outside);
    let full = flanks::full_width_points(&path, &band_material(&world, &name, stock), R8);
    assert!(full.is_empty(), "{} full-width cut point(s), the first at {:?}", full.len(), full.first());
    let named: Vec<&String> = path.notes.iter().filter(|n| n.starts_with(PASS_START_NOTE)).collect();
    assert!(named.is_empty(), "the band starts in air, so no pass start warning: {named:?}");
}

/// #3316: Inside on the Outside band starts each level on the part wall,
/// a cut of the full tool width - the toolpath names that case.
#[test]
fn fig8_inside_offset_names_its_part_wall_start() {
    let (world, mut h, name, stock) = fig8_world();
    let (path, warnings) = offset_op(&world, &mut h, &name, stock.0.z, PassStart::Inside);
    let named: Vec<&String> = path.notes.iter().filter(|n| n.starts_with(PASS_START_NOTE)).collect();
    assert!(named.iter().any(|n| n.contains("the part wall")), "{named:?}");
    // #3035: this band plans complete, so no fallback warning is posted.
    assert!(!warnings.iter().any(|w| w.starts_with("StrategyFallback")), "no fallback expected: {warnings:?}");
}

/// #3316/#3035: the completeness guard's fallback note is an Offset-rings
/// fallback, so `rings_fallbacks` counts one area and the op posts the
/// advisory `Warning::StrategyFallback` (the posting law is pinned by
/// `strategy_fallback`). The other "Pass start" notes count none.
#[test]
fn the_offset_guard_fallback_is_a_strategy_fallback() {
    use camcraft_app::machining2d::{StartSide, rings_fallbacks};
    let side = StartSide::from(PassStart::Outside);
    let note = side.fallback_note(12.3);
    assert!(note.starts_with(PASS_START_NOTE), "{note}");
    assert_eq!(rings_fallbacks(std::slice::from_ref(&note)), 1, "{note}");
    let wall = side.wall_first_note().unwrap_or_default();
    assert_eq!(rings_fallbacks(std::slice::from_ref(&wall)), 0, "{wall}");
}

/// #3319 guard on Auto, the maintainer's own setting: the order is the
/// one every earlier file cut, and no lateral rapid runs below the
/// clearance plane with "Hop below clearance" off.
#[test]
fn fig8_auto_offset_keeps_every_lateral_rapid_at_the_clearance_plane() {
    let (world, mut h, name, stock) = fig8_world();
    let (path, warnings) = offset_op(&world, &mut h, &name, stock.0.z, PassStart::Auto);
    let clear_z = stock.1.z + CLEARANCE;
    assert_eq!(lateral_rapids_below(&path, clear_z), vec![], "#3319: a lateral rapid below Z {clear_z}");
    assert!(!warnings.iter().any(|w| w.starts_with("RapidInStock")), "#3319: a rapid through stock: {warnings:?}");
    // Auto is the core-first order: the first ring is not at the stock edge.
    let levels = level_ends(&path);
    assert!(levels.iter().all(|(_, first, _)| outside_box(*first, stock) < R8 - 0.05), "Auto moved: {levels:?}");
}

fn call(s: &mut Server, id: i64, name: &str, args: Value) -> Value {
    let msg = json!({ "jsonrpc": "2.0", "id": id, "method": "tools/call", "params": { "name": name, "arguments": args } });
    s.handle(&msg).expect("a response")["result"].clone()
}

/// The option through MCP and the project file: `outside` and `inside`
/// are kept, `auto` is accepted and never written, an unknown word is
/// refused, and undo restores the value before.
#[test]
fn mcp_sets_pass_start_refuses_an_unknown_word_and_undo_restores_it() {
    let mut s = Server::new();
    let ok = |r: &Value| r["isError"] != json!(true);
    let imp = call(&mut s, 1, "import_model", json!({ "path": fig8_path().display().to_string() }));
    assert!(ok(&imp), "{imp}");
    assert!(ok(&call(&mut s, 2, "add_tool", json!({ "catalog": EM8 }))));
    let add = call(&mut s, 3, "add_operation", json!({ "kind": "2D Offset", "tool": EM8, "name": "O1", "source": "fig8" }));
    assert!(ok(&add), "{add}");
    assert!(!add.to_string().contains("pass_start"), "Auto is absent: {add}");
    let edit = call(&mut s, 4, "edit_operation", json!({ "name": "O1", "params": { "pass_start": "outside" } }));
    assert!(ok(&edit), "{edit}");
    assert!(edit.to_string().contains("\"pass_start\":\"outside\""), "{edit}");
    let bad = call(&mut s, 5, "edit_operation", json!({ "name": "O1", "params": { "pass_start": "sideways" } }));
    assert_eq!(bad["isError"], json!(true), "an unknown word is refused: {bad}");
    let inside = call(&mut s, 6, "edit_operation", json!({ "name": "O1", "params": { "pass_start": "inside" } }));
    assert!(inside.to_string().contains("\"pass_start\":\"inside\""), "{inside}");
    let auto = call(&mut s, 7, "edit_operation", json!({ "name": "O1", "params": { "pass_start": "auto" } }));
    assert!(ok(&auto), "auto is a known word: {auto}");
    assert!(!auto.to_string().contains("pass_start"), "Auto is not written: {auto}");
    assert!(ok(&call(&mut s, 8, "undo", json!({}))));
    let job = call(&mut s, 9, "get_job", json!({})).to_string();
    assert!(job.contains("\"pass_start\":\"inside\""), "undo restores Inside: {job}");
    // The project file: the domain value round-trips by its one key.
    let mut p = operation::retype_params(&pocket(20.0, 0.0), OpType::Offset2D.ordinal());
    assert!(!project::params_to_json(&p).to_string().contains("pass_start"), "Auto stays off the wire");
    if let OpParams::Offset2D { pass_start, .. } = &mut p {
        *pass_start = PassStart::Outside;
    }
    let wire = project::params_to_json(&p);
    assert!(wire.to_string().contains("\"pass_start\":\"outside\""), "{wire}");
    assert_eq!(project::params_from_json(&wire).expect("reads back"), p);
}
