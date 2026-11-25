//! #3327: the 2D Adaptive's start and exit points through the app. The
//! entry goes down at the start point, the last pass of each level ends at
//! the exit point, a point that the plan cannot hold is refused by name
//! and nothing is cut, and the picks ride the undoable parameter command,
//! the project file, the stamp and the MCP `edit_operation` patch. An op
//! without a pick plans the earlier path.

use camcraft_app::cl::{MoveKind, Toolpath};
use camcraft_app::commands::{AddTool, CreateOperation, EditOperationParams, GenerateOperation, History, SetStock, SetWcsOrigin};
use camcraft_app::geometry::{Point3, quat_from_euler};
use camcraft_app::geo2d::Point2;
use camcraft_app::machining2d::{AdaptiveMethod, AdaptiveTrochParams, PassStart};
use camcraft_app::machining3d::Entry3D;
use camcraft_app::mcp::Server;
use camcraft_app::operation::{self, Height, KindInfo, OpParams, OpType, PickCapability, SrcRef, Tilt};
use camcraft_app::path_picks::{PathPick, PathPicks, PickRole};
use camcraft_app::schema::Location;
use camcraft_app::stock::StockShape;
use camcraft_app::tools::catalog;
use camcraft_app::world::new_world;
use camcraft_app::{model, project};
use flecs_ecs::prelude::World;
use serde_json::json;

use super::call;

const TOOL: &str = "EM-0800-4FL";
const OP: &str = "Adaptive1";
/// The stock box side (mm): its footprint is the region, centred on the
/// origin, so the tool centre reaches X and Y in -11 .. 11.
const SIDE: f64 = 30.0;
const HELIX: f64 = 1.0;
const START: Point2 = Point2 { x: -4.0, y: -3.0 };
const EXIT: Point2 = Point2 { x: 14.0, y: 0.0 };

fn picks(start: Option<Point2>, exit: Option<Point2>) -> PathPicks {
    PathPicks {
        start: start.map(PathPick::xy).into_iter().collect(),
        exit: exit.map(PathPick::xy).into_iter().collect(),
    }
}

/// One level, z 20 to 15, a helix entry of radius 1.
fn adaptive(picks: PathPicks) -> OpParams {
    OpParams::Adaptive2D {
        rest: None,
        blend_radius: None,
        blend_turns: false,
        links: camcraft_app::links::Links::PLANAR,
        stepover: 3.0,
        stock_to_leave: 0.0,
        clearance_z: Height::fixed(25.0),
        retract_z: None,
        top_z: Height::fixed(20.0),
        bottom_z: Height::fixed(15.0),
        stepdown: 5.0,
        feed: 800.0,
        plunge_feed: 200.0,
        climb: true,
        entry: Entry3D::Helix,
        helix_radius: HELIX,
        optimal_load: camcraft_app::load::DEFAULT_OPTIMAL_LOAD,
        method: AdaptiveMethod::Spiral,
        troch: AdaptiveTrochParams::default(),
        tilt: None,
        corner_relief: Default::default(),
        pass_start: PassStart::Auto,
        picks,
    }
}

fn job(params: OpParams) -> (World, History) {
    let world = new_world();
    let mut h = History::default();
    let stock = SetStock::new(&world, StockShape::Box { dx: SIDE, dy: SIDE, dz: 20.0 }, Location::default());
    h.execute(&world, stock.expect("stock"));
    let tool = catalog().into_iter().find(|(n, _)| *n == TOOL).map(|(_, t)| t).expect("catalog tool");
    h.execute(&world, AddTool::new(&world, TOOL, tool).expect("tool"));
    h.execute(&world, CreateOperation::new(&world, OP, TOOL, SrcRef::Stock, params).expect("create"));
    (world, h)
}

fn generate(world: &World, h: &mut History) -> Result<Toolpath, String> {
    let g = GenerateOperation::new(world, OP).map_err(|e| e.to_string())?;
    h.execute(world, g);
    let e = model::find_by_name(world, OP).ok_or("no op")?;
    model::toolpath_of(world, &e).map(|c| c.path).ok_or_else(|| "no cached path".to_owned())
}

fn params_of(world: &World) -> OpParams {
    let e = model::find_by_name(world, OP).expect("op");
    model::operation_of(world, &e).expect("op").params
}

/// The XY of the first planar feed at the floor and of the last one.
fn first_last(t: &Toolpath) -> (Point2, Point2) {
    let mut planar = t.moves.iter().filter(|m| m.kind == MoveKind::Feed && (m.from.z - m.to.z).abs() < 1e-9 && m.to.z < 20.0 - 1e-6);
    let first = planar.next().expect("a planar feed");
    let last = planar.next_back().unwrap_or(first);
    (Point2::new(first.from.x, first.from.y), Point2::new(last.to.x, last.to.y))
}

fn dist(a: Point2, b: Point2) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

#[test]
fn the_2d_adaptive_takes_a_start_and_an_exit_point() {
    let cap = KindInfo::ALL.iter().find(|k| k.ty == OpType::Adaptive2D).map(KindInfo::pick).expect("the 2D Adaptive");
    assert_eq!(cap, PickCapability::Region, "a 2D Adaptive pick is a point in its area");
    assert!(cap.takes(PickRole::Start) && cap.takes(PickRole::Exit));
}

#[test]
fn the_entry_is_at_the_start_point_and_the_last_pass_ends_at_the_exit_point() {
    let (world, mut h) = job(adaptive(picks(Some(START), Some(EXIT))));
    let path = generate(&world, &mut h).expect("both points hold");
    let (first, last) = first_last(&path);
    assert!(dist(first, START) <= HELIX + 1e-6, "the cut starts on the helix about the start point: {first:?}");
    // The closing ride runs 4 mm off the wall at X 15: its point nearest the exit.
    assert!(dist(last, Point2::new(11.0, 0.0)) < 0.05, "the last pass ends beside the exit point: {last:?}");
    assert!(!path.notes.iter().any(|n| n.contains("exceeds the")), "no planned load over the optimal load: {:?}", path.notes);
    super::common::write_golden("adaptive2d_start_exit_picks.cl", &super::common::gen_to_cltext(&path));
}

#[test]
fn the_picks_hold_under_a_90_degree_wcs_yaw() {
    // The picks are model-frame points: under a Z-rotated work offset the
    // path still enters at the start point and ends beside the exit point.
    let (world, mut h) = job(adaptive(picks(Some(START), Some(EXIT))));
    let rot = quat_from_euler(0.0, 0.0, 90f64.to_radians());
    h.execute(&world, SetWcsOrigin::new(&world, Location { pos: Point3::new(4.0, -3.0, 0.0), rot }).expect("place G54"));
    let path = generate(&world, &mut h).expect("the model-frame picks hold under WCS yaw");
    let (first, last) = first_last(&path);
    assert!(dist(first, START) <= HELIX + 1e-6, "the cut starts on the helix about the start point: {first:?}");
    assert!(dist(last, Point2::new(11.0, 0.0)) < 0.05, "the last pass ends beside the exit point: {last:?}");
    assert_eq!(params_of(&world).path_picks(), picks(Some(START), Some(EXIT)), "the stored picks stay model-frame");
}

/// One setting of a door case: it changes the op, given a tilt.
type Setting = fn(&mut OpParams, Tilt);

#[test]
fn a_point_the_plan_cannot_hold_is_refused_by_name_and_nothing_is_cut() {
    for (p, want) in [
        (picks(Some(Point2::new(14.0, 14.0)), None), ["start point", "outside the area"]),
        (picks(Some(Point2::new(10.5, 0.0)), None), ["helix", "no room"]),
        (picks(None, Some(Point2::new(40.0, 0.0))), ["exit point", "closing ride"]),
    ] {
        let (world, mut h) = job(adaptive(p.clone()));
        let why = generate(&world, &mut h).expect_err("refused");
        assert!(want.iter().all(|w| why.contains(w)), "{p:?}: {why}");
        let e = model::find_by_name(&world, OP).expect("op");
        assert!(model::toolpath_of(&world, &e).is_none(), "{p:?}: nothing is cut");
    }
    // At the door: the settings that choose their own start.
    let (world, _h) = job(adaptive(PathPicks::default()));
    let start = picks(Some(START), None);
    let tilted = Tilt { basis_version: 1, a_deg: 0.0, b_deg: 90.0, origin: [0.0, 0.0, 0.0] };
    let cases: [(Setting, &str); 4] = [
        (|p, t| if let OpParams::Adaptive2D { tilt, .. } = p { *tilt = Some(t) }, "tilted"),
        (|p, _| if let OpParams::Adaptive2D { method, .. } = p { *method = AdaptiveMethod::Trochoidal }, "Trochoidal"),
        (|p, _| if let OpParams::Adaptive2D { pass_start, .. } = p { *pass_start = PassStart::Inside }, "Pass starts at Inside"),
        (|p, _| if let OpParams::Adaptive2D { entry, .. } = p { *entry = Entry3D::Ramp }, "Helix or Plunge"),
    ];
    for (set, want) in cases {
        let mut p = adaptive(start.clone());
        set(&mut p, tilted);
        match EditOperationParams::new(&world, OP, p) {
            Ok(_) => panic!("{want}: refused at the door"),
            Err(e) => assert!(e.to_string().contains(want) && e.to_string().contains("start point"), "{want}: {e}"),
        }
    }
}

#[test]
fn the_picks_ride_the_command_undo_the_file_and_the_stamp() {
    let (world, mut h) = job(adaptive(PathPicks::default()));
    let before = generate(&world, &mut h).expect("plain");
    let picked = picks(Some(START), Some(EXIT));
    let stamp = |p: OpParams| operation::current_stamp(&world, TOOL, &SrcRef::Stock, &p, "X", None);
    assert_ne!(stamp(adaptive(picked.clone())), stamp(adaptive(PathPicks::default())), "the stamp tells a pick apart");

    h.execute(&world, EditOperationParams::new(&world, OP, adaptive(picked.clone())).expect("the op takes the picks"));
    assert_eq!(params_of(&world).path_picks(), picked);
    let after = generate(&world, &mut h).expect("picked");
    assert!(dist(first_last(&after).0, START) <= HELIX + 1e-6, "the picked path enters at the start point");
    assert_ne!(after.moves, before.moves, "the picks change the path");

    let file = std::env::temp_dir().join(format!("camcraft-adaptive-picks-{}.camproj", std::process::id()));
    project::save(&world, &file).expect("save");
    let back = new_world();
    project::load(&back, &file).expect("load");
    let _ = std::fs::remove_file(&file);
    assert_eq!(params_of(&back).path_picks(), picked, "the file keeps the picks");

    h.undo(&world);
    h.undo(&world);
    assert!(params_of(&world).path_picks().is_empty(), "undo clears the picks");
    let e = model::find_by_name(&world, OP).expect("op");
    let restored = model::toolpath_of(&world, &e).expect("the earlier path is back").path;
    assert_eq!(restored.moves, before.moves, "undo restores the previous path");
    h.redo(&world);
    assert_eq!(params_of(&world).path_picks(), picked, "redo sets the picks again");
    // Without a pick the plan is the earlier one, to the move.
    let (plain, mut hp) = job(adaptive(PathPicks::default()));
    assert_eq!(generate(&plain, &mut hp).expect("plain again").moves, before.moves, "no pick plans the earlier path");
}

/// The `picks` of operation `A` as `get_job` shows it (`null` without).
fn job_picks(s: &mut Server) -> serde_json::Value {
    let (err, r) = call(s, "get_job", json!({}));
    assert!(!err, "{r}");
    let ops = r["structuredContent"]["operations"].as_array().cloned().unwrap_or_default();
    let op = ops.iter().find(|o| o["name"] == "A").cloned().unwrap_or_else(|| panic!("operation A in {r:.400}"));
    op["params"]["adaptive2_d"]["picks"].clone()
}

#[test]
fn mcp_edit_operation_sets_the_adaptive_picks() {
    let mut s = Server::new();
    assert!(!call(&mut s, "set_stock", json!({ "box": { "dx": SIDE, "dy": SIDE, "dz": 20.0 } })).0);
    assert!(!call(&mut s, "add_tool", json!({ "catalog": TOOL })).0);
    let (err, r) = call(&mut s, "add_operation", json!({ "kind": "2D Adaptive", "tool": TOOL, "name": "A" }));
    assert!(!err, "{r}");
    let patch = json!({ "name": "A", "params": { "picks": { "start": [[START.x, START.y]], "exit": [[EXIT.x, EXIT.y]] } } });
    let (err, r) = call(&mut s, "edit_operation", patch);
    assert!(!err, "the 2D Adaptive takes a start and an exit point: {r}");
    let (err, kinds) = call(&mut s, "list_operation_types", json!({ "kind": "2D Adaptive" }));
    assert!(!err, "{kinds}");
    let text = kinds.to_string().replace("\\\"", "\"").replace(' ', "");
    for want in ["\"start\":true", "\"exit\":true", "\"field\":\"picks\""] {
        assert!(text.contains(want), "the 2D Adaptive's start_exit says {want}: {text:.600}");
    }
    // The round trip: get_job reads the picks back, and undo takes them away.
    assert_eq!(job_picks(&mut s), json!({ "start": [[START.x, START.y]], "exit": [[EXIT.x, EXIT.y]] }), "get_job reads the picks back");
    let (err, r) = call(&mut s, "undo", json!({}));
    assert!(!err, "undo: {r}");
    assert!(job_picks(&mut s).as_object().is_none_or(|o| o.values().all(|v| v.as_array().is_none_or(Vec::is_empty))), "undo clears the picks");
    let (err, r) = call(&mut s, "edit_operation", json!({ "name": "A", "params": { "picks": {} } }));
    assert!(!err, "an empty object clears the picks: {r}");
}
