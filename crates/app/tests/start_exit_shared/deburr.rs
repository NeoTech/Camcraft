//! The Deburr's start and exit picks (#3329, slice 1) through the
//! application: the chain nearest the start pick is cut first from its
//! point nearest it, the chain nearest the exit pick last to its point
//! nearest it, and the cut geometry stays the automatic ride's. A
//! module of the `start_exit_shared` suite.

use super::*;

/// The 90-degree chamfer mill, 6 mm across.
const CONE: &str = "CM-0600-90";

/// A 40 x 40 brick, z 0..10, as its own stock, with the cone.
fn brick_world() -> flecs_ecs::prelude::World {
    use camcraft_app::commands::{AddTool, History, ImportModel, SetStock};
    use camcraft_app::geometry::{cube, Point3};
    let world = camcraft_app::world::new_world();
    let mut h = History::default();
    h.execute(&world, ImportModel::from_mesh("plate", cube(Point3::new(-20.0, -20.0, 0.0), Point3::new(20.0, 20.0, 10.0))));
    let stock = SetStock::new(&world, camcraft_app::stock::StockShape::FromModel { source: "plate".into() }, Default::default());
    h.execute(&world, stock.expect("stock"));
    let t = camcraft_app::tools::catalog().into_iter().find(|(n, _)| *n == CONE).map(|(_, t)| t).expect("catalog cone");
    h.execute(&world, AddTool::new(&world, CONE, t).expect("tool"));
    world
}

type Pick = Option<(f64, f64, Option<f64>)>;

/// A 1 mm setback whose tips ride z 8.5..9.5: the top ring of the brick.
fn deburr(start: Pick, exit: Pick) -> camcraft_app::schema::Operation {
    let pick = |(x, y, z)| camcraft_app::path_picks::PathPick { xy: Point2::new(x, y), z };
    let params = OpParams::Deburr {
        width: std::f64::consts::SQRT_2,
        clearance_z: Height::fixed(25.0),
        top_z: Height::fixed(9.5),
        bottom_z: Height::fixed(8.5),
        feed: 500.0,
        plunge_feed: 100.0,
        exclude_edges: Vec::new(),
        min_edge_angle_deg: 0.0,
        exclude_holes: false,
        hole_min_diameter: 0.0,
        hole_max_diameter: 0.0,
        unreachable: camcraft_app::finishing3d::UnreachableEdges::SkipWithNote,
        sizing: camcraft_app::finishing3d::DeburrBreak::default(),
        picks: camcraft_app::path_picks::PathPicks { start: start.map(pick).into_iter().collect(), exit: exit.map(pick).into_iter().collect() },
    };
    camcraft_app::schema::Operation {
        tool: CONE.into(),
        src: camcraft_app::operation::SrcRef::Entity("plate".into()),
        params,
        stamp: None,
        setup: None,
        area: Default::default(),
        coolant: Default::default(),
        after: Default::default(),
        suppressed: false,
    }
}

type Seg = ([f64; 3], [f64; 3]);

fn feeds(t: &camcraft_app::cl::Toolpath) -> Vec<Seg> {
    t.moves.iter().filter(|m| m.kind == camcraft_app::cl::MoveKind::Feed).map(|m| ([m.from.x, m.from.y, m.from.z], [m.to.x, m.to.y, m.to.z])).collect()
}

/// Every cut of `got`, sampled each 0.25 mm along its length, lies on
/// a cut of `auto`, and the other way round.
fn same_cut(auto: &camcraft_app::cl::Toolpath, got: &camcraft_app::cl::Toolpath) {
    let off = |c: [f64; 3], segs: &[Seg]| {
        segs.iter().map(|(a, b)| {
            let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let l2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
            let t = if l2 > 0.0 { (((c[0] - a[0]) * d[0] + (c[1] - a[1]) * d[1] + (c[2] - a[2]) * d[2]) / l2).clamp(0.0, 1.0) } else { 0.0 };
            (0..3).map(|i| (c[i] - a[i] - d[i] * t).powi(2)).sum::<f64>().sqrt()
        }).fold(f64::INFINITY, f64::min)
    };
    for (from, to) in [(got, auto), (auto, got)] {
        let segs = feeds(to);
        for (a, b) in feeds(from) {
            let len = (0..3).map(|i| (b[i] - a[i]).powi(2)).sum::<f64>().sqrt();
            let n = ((len / 0.25).ceil() as usize).max(1);
            for k in 0..=n {
                let t = k as f64 / n as f64;
                let c = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t];
                assert!(off(c, &segs) < 1e-6, "a cut leaves the automatic ride at {c:?}");
            }
        }
    }
}

fn near(p: [f64; 3], q: [f64; 3]) -> bool {
    (0..3).all(|i| (p[i] - q[i]).abs() < 1e-6)
}

#[test]
fn deburr_starts_at_the_picked_point_and_ends_at_the_picked_exit() {
    let world = brick_world();
    let generate = |op| camcraft_app::operation::generate(&world, &op, "").map(|(t, _)| t);
    let auto = generate(deburr(None, None)).expect("the automatic ride");
    let t = generate(deburr(Some((20.5, 5.0, Some(10.0))), Some((5.0, -20.5, None)))).expect("the picked ride");
    let plunge = t.moves.iter().find(|m| m.kind == camcraft_app::cl::MoveKind::Plunge).map(|m| [m.to.x, m.to.y, m.to.z]).expect("a plunge");
    assert!(near(plunge, [20.0, 5.0, 9.0]), "the cut starts at the start pick: {plunge:?}");
    let last = feeds(&t).last().map(|s| s.1).expect("a cut");
    assert!(near(last, [5.0, -20.0, 9.0]), "the cut ends at the exit pick: {last:?}");
    same_cut(&auto, &t);
    // Without a pick the path is the automatic ride, move for move.
    assert_eq!(generate(deburr(None, None)).expect("again"), auto);
}

#[test]
fn deburr_refuses_an_impossible_pick_by_name() {
    let world = brick_world();
    let e = camcraft_app::operation::generate(&world, &deburr(Some((60.0, 0.0, None)), None), "").err().map(|e| format!("{e:?}"));
    assert!(e.as_deref().is_some_and(|m| m.contains("start point (60.000, 0.000)") && m.contains("farther than")), "{e:?}");
    let e = camcraft_app::operation::generate(&world, &deburr(None, Some((0.0, 0.0, Some(40.0)))), "").err().map(|e| format!("{e:?}"));
    assert!(e.as_deref().is_some_and(|m| m.contains("exit point (0.000, 0.000, 40.000)") && m.contains("nearest edge chain")), "{e:?}");
}

#[test]
fn deburr_picks_ride_edit_operation_and_undo_restores_them() {
    let mut s = job();
    assert!(!call(&mut s, "add_tool", json!({ "catalog": CONE })).0);
    let (err, r) = call(&mut s, "add_operation", json!({ "kind": "Deburr", "tool": CONE, "name": "DB" }));
    assert!(!err, "{r}");
    let patch = json!({ "name": "DB", "params": { "picks": { "start": [[20.25, 5.75, 10.0]], "exit": [[5.75, -20.25]] } } });
    let (err, r) = call(&mut s, "edit_operation", patch);
    assert!(!err, "the Deburr takes both picks: {r}");
    let held = |s: &mut Server| call(s, "get_job", json!({})).1.to_string();
    let job_text = held(&mut s);
    assert!(job_text.contains("20.25") && job_text.contains("5.75"), "the job carries the picks: {job_text:.600}");
    assert!(!call(&mut s, "undo", json!({})).0);
    let job_text = held(&mut s);
    assert!(!job_text.contains("20.25") && !job_text.contains("5.75"), "undo restores the automatic order");
    assert!(!call(&mut s, "redo", json!({})).0);
    assert!(held(&mut s).contains("20.25"), "redo states the picks again");
    let reply = s.handle(&json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {} })).expect("an answer");
    let doc = reply["result"]["tools"]
        .as_array()
        .and_then(|t| t.iter().find(|t| t["name"] == "edit_operation"))
        .map(|t| t["inputSchema"]["properties"]["params"]["description"].as_str().unwrap_or_default().to_owned())
        .unwrap_or_default();
    assert!(doc.contains("Deburr") && doc.contains("edge chain"), "the schema names the chain kind: {doc}");
}
