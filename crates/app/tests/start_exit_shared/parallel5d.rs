//! The 5-Axis Parallel's start and exit picks (#3330) through the
//! application: a pick chooses the raster corner where the cut starts
//! and where the last pass ends, and the cut geometry stays the
//! automatic raster's. A module of the `start_exit_shared` suite.

use super::*;

/// A closed 30 x 20 x 10 box on z = 0, centred on the origin: a flat
/// top for the raster, steep walls for the rings.
fn block() -> camcraft_app::geometry::Mesh {
    use camcraft_app::geometry::{Mesh, Point3, Triangle};
    let (x, y, h) = (15.0, 10.0, 10.0);
    let v = |i: usize| Point3::new(if i & 1 == 0 { -x } else { x }, if i & 2 == 0 { -y } else { y }, if i & 4 == 0 { 0.0 } else { h });
    let quads = [[0, 2, 3, 1], [4, 5, 7, 6], [0, 1, 5, 4], [2, 6, 7, 3], [0, 4, 6, 2], [1, 3, 7, 5]];
    let mut f = Vec::new();
    for q in quads {
        f.push(Triangle::new(v(q[0]), v(q[1]), v(q[2])));
        f.push(Triangle::new(v(q[0]), v(q[2]), v(q[3])));
    }
    Mesh::from_facets(f)
}

/// The block in a 40 x 40 x 12 stock, with the catalog ball.
fn block_world() -> flecs_ecs::prelude::World {
    use camcraft_app::commands::{AddTool, History, ImportModel, SetStock};
    use camcraft_app::geometry::{Point3, IDENTITY_Q};
    let world = camcraft_app::world::new_world();
    let mut h = History::default();
    let at = camcraft_app::schema::Location { pos: Point3::ZERO, rot: IDENTITY_Q };
    let stock = SetStock::new(&world, camcraft_app::stock::StockShape::Box { dx: 40.0, dy: 40.0, dz: 12.0 }, at);
    h.execute(&world, stock.expect("stock"));
    let t = camcraft_app::tools::catalog().into_iter().find(|(n, _)| *n == "BN-0600").map(|(_, t)| t).expect("catalog ball");
    h.execute(&world, AddTool::new(&world, "BN-0600", t).expect("tool"));
    h.execute(&world, ImportModel::from_mesh("part", block()));
    world
}

fn parallel5d(start: Option<(f64, f64)>, exit: Option<(f64, f64)>) -> camcraft_app::schema::Operation {
    let pick = |(x, y)| camcraft_app::path_picks::PathPick::xy(Point2::new(x, y));
    let params = OpParams::Parallel5D {
        tolerance: 0.1,
        stepover: Some(4.0),
        clearance_z: Height::fixed(25.0),
        retract_z: None,
        top_z: Height::fixed(11.0),
        bottom_z: Height::fixed(7.0),
        feed: 500.0,
        plunge_feed: 100.0,
        climb: true,
        lead_deg: 15.0,
        lag_deg: 0.0,
        max_tilt_deg: 60.0,
        shank_avoid: false,
        picks: camcraft_app::path_picks::PathPicks { start: start.map(pick).into_iter().collect(), exit: exit.map(pick).into_iter().collect() },
    };
    camcraft_app::schema::Operation {
        tool: "BN-0600".into(),
        src: camcraft_app::operation::SrcRef::Entity("part".into()),
        params,
        stamp: None,
        setup: None,
        area: Default::default(),
        coolant: Default::default(),
        after: Default::default(),
        suppressed: false,
    }
}

/// Each move's (kind, start centre, end centre): its tips backed up
/// 3 mm (the ball's radius) along the pose the machine holds there.
fn centres(t: &camcraft_app::cl::Toolpath) -> Vec<(camcraft_app::cl::MoveKind, [f64; 3], [f64; 3])> {
    let mut held = [0.0, 0.0, 1.0];
    t.moves.iter().map(|m| {
        let from = [m.from.x + 3.0 * held[0], m.from.y + 3.0 * held[1], m.from.z + 3.0 * held[2]];
        held = m.axis.unwrap_or(held);
        (m.kind, from, [m.to.x + 3.0 * held[0], m.to.y + 3.0 * held[1], m.to.z + 3.0 * held[2]])
    }).collect()
}

/// The ball centre segments of the raster's cuts on the top: feed moves
/// that travel in XY at the top's height.
fn top_cuts(t: &camcraft_app::cl::Toolpath) -> Vec<([f64; 3], [f64; 3])> {
    use camcraft_app::cl::MoveKind;
    centres(t).into_iter().filter(|(k, a, b)| *k == MoveKind::Feed && (a[0] - b[0]).hypot(a[1] - b[1]) > 1e-9 && (a[2] - 13.0).abs() < 0.2 && (b[2] - 13.0).abs() < 0.2).map(|(_, a, b)| (a, b)).collect()
}

/// The end centre of the first plunge: where the cut starts.
fn first_plunge(t: &camcraft_app::cl::Toolpath) -> [f64; 3] {
    centres(t).into_iter().find(|(k, _, _)| *k == camcraft_app::cl::MoveKind::Plunge).map(|(_, _, b)| b).expect("a plunge")
}

/// Every top cut of `got` lies on a top cut of `auto` along its whole
/// length, sampled every 0.25 mm: the pick changes the order, never
/// the geometry.
fn cuts_ride_the_raster(auto: &camcraft_app::cl::Toolpath, got: &camcraft_app::cl::Toolpath) {
    let segs = top_cuts(auto);
    let off = |c: [f64; 3]| {
        segs.iter().map(|(a, b)| {
            let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let l2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
            let t = if l2 > 0.0 { (((c[0] - a[0]) * d[0] + (c[1] - a[1]) * d[1] + (c[2] - a[2]) * d[2]) / l2).clamp(0.0, 1.0) } else { 0.0 };
            (0..3).map(|i| (c[i] - a[i] - d[i] * t).powi(2)).sum::<f64>().sqrt()
        }).fold(f64::INFINITY, f64::min)
    };
    let cuts = top_cuts(got);
    assert_eq!(cuts.len(), segs.len(), "the pick adds and drops no top cut");
    for (a, b) in cuts {
        let len = (0..3).map(|i| (b[i] - a[i]).powi(2)).sum::<f64>().sqrt();
        let n = ((len / 0.25).ceil() as usize).max(1);
        for k in 0..=n {
            let t = k as f64 / n as f64;
            let c = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t];
            assert!(off(c) < 1e-6, "a cut stands {} mm off the automatic raster at {c:?}", off(c));
        }
    }
}

#[test]
fn parallel5d_starts_at_the_picked_corner_and_ends_at_the_picked_exit() {
    let world = block_world();
    let generate = |op| camcraft_app::operation::generate(&world, &op, "").map(|(t, _)| t);
    let auto = generate(parallel5d(None, None)).expect("the automatic raster");
    let a = first_plunge(&auto);
    // The far end of the first pass, mirrored across the block's Y axis.
    let started = generate(parallel5d(Some((-a[0], a[1])), None)).expect("the started raster");
    let s = first_plunge(&started);
    assert!(s[0] * a[0] < 0.0 && (s[0] + a[0]).abs() < 0.5 && (s[1] - a[1]).abs() < 1e-6, "the cut starts at the picked corner: {s:?} (automatic {a:?})");
    cuts_ride_the_raster(&auto, &started);
    // An exit at the automatic start: the whole raster runs back, so the
    // last pass ends at the automatic first corner and the tool rises
    // straight up from there.
    let exited = generate(parallel5d(None, Some((a[0], a[1])))).expect("the exited raster");
    let c = centres(&exited);
    let same = |p: [f64; 3], q: [f64; 3]| (0..3).all(|i| (p[i] - q[i]).abs() < 1e-6);
    let end = c.iter().rposition(|(k, _, b)| *k == camcraft_app::cl::MoveKind::Feed && same(*b, a)).expect("a cut ends at the exit corner");
    let rise = c[end + 1];
    assert!((rise.1[0] - rise.2[0]).hypot(rise.1[1] - rise.2[1]) < 1e-6 && rise.2[2] > rise.1[2], "the tool leaves straight up: {rise:?}");
    assert!(first_plunge(&exited)[1] * a[1] < 0.0, "the cut starts on the far pass: {:?}", first_plunge(&exited));
    cuts_ride_the_raster(&auto, &exited);
    // Without a pick the path is the automatic raster, move for move.
    assert_eq!(generate(parallel5d(None, None)).expect("again"), auto);
}

#[test]
fn parallel5d_refuses_an_impossible_pick_by_name() {
    let world = block_world();
    let auto = camcraft_app::operation::generate(&world, &parallel5d(None, None), "").map(|(t, _)| t).expect("auto");
    let a = first_plunge(&auto);
    let cases = [
        (parallel5d(Some((0.0, 0.0)), None), "start point (0.000, 0.000)", "only at a corner"),
        (parallel5d(None, Some((0.0, 0.0))), "exit point (0.000, 0.000)", "only at a corner"),
        (parallel5d(Some((a[0], a[1])), Some((a[0], a[1]))), "exit point", "a raster that starts at the start point ends at"),
    ];
    for (op, what, why) in cases {
        let e = camcraft_app::operation::generate(&world, &op, "").err().map(|e| format!("{e:?}"));
        assert!(e.as_deref().is_some_and(|m| m.contains(what) && m.contains(why)), "{what}: {e:?}");
    }
}

#[test]
fn parallel5d_picks_ride_edit_operation_and_undo_restores_them() {
    let mut s = job();
    assert!(!call(&mut s, "add_tool", json!({ "catalog": "BN-0600" })).0);
    let (err, r) = call(&mut s, "add_operation", json!({ "kind": "5-Axis Parallel", "tool": "BN-0600", "name": "P5" }));
    assert!(!err, "{r}");
    let patch = json!({ "name": "P5", "params": { "picks": { "start": [[-27.25, -17.75]], "exit": [[27.25, 17.75]] } } });
    let (err, r) = call(&mut s, "edit_operation", patch);
    assert!(!err, "the 5-axis parallel takes both picks: {r}");
    let held = |s: &mut Server| call(s, "get_job", json!({})).1.to_string();
    let job_text = held(&mut s);
    assert!(job_text.contains("27.25") && job_text.contains("17.75"), "the job carries the picks: {job_text:.600}");
    assert!(!call(&mut s, "undo", json!({})).0);
    let job_text = held(&mut s);
    assert!(!job_text.contains("27.25") && !job_text.contains("17.75"), "undo restores the automatic order");
    assert!(!call(&mut s, "redo", json!({})).0);
    assert!(held(&mut s).contains("27.25"), "redo states the picks again");
    let reply = s.handle(&json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {} })).expect("an answer");
    let doc = reply["result"]["tools"]
        .as_array()
        .and_then(|t| t.iter().find(|t| t["name"] == "edit_operation"))
        .map(|t| t["inputSchema"]["properties"]["params"]["description"].as_str().unwrap_or_default().to_owned())
        .unwrap_or_default();
    assert!(doc.contains("5-Axis Parallel") && doc.contains("raster corner"), "the schema names the raster kind: {doc}");
}
