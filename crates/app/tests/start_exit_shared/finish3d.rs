//! The 3D Finish's start and exit picks (#3329, slice 2) through the
//! application: on the Parallel pattern a pick chooses the raster corner
//! where the cut starts and where the last pass ends, the cut geometry
//! stays the automatic raster's, and what moves a picked corner (the
//! waterline rings, the area controls, another pattern) is refused by
//! name. A module of the `start_exit_shared` suite.

use super::*;

/// The catalog ball, 6 mm across.
const BALL: &str = "BN-0600";
const R: f64 = 3.0;

/// A closed 30 x 20 x 10 box on z = 0, centred on the origin, in a
/// 40 x 40 x 12 stock, with the ball.
fn block_world() -> flecs_ecs::prelude::World {
    use camcraft_app::commands::{AddTool, History, ImportModel, SetStock};
    use camcraft_app::geometry::{cube, Point3, IDENTITY_Q};
    let world = camcraft_app::world::new_world();
    let mut h = History::default();
    let at = camcraft_app::schema::Location { pos: Point3::ZERO, rot: IDENTITY_Q };
    let stock = SetStock::new(&world, camcraft_app::stock::StockShape::Box { dx: 40.0, dy: 40.0, dz: 12.0 }, at);
    h.execute(&world, stock.expect("stock"));
    let t = camcraft_app::tools::catalog().into_iter().find(|(n, _)| *n == BALL).map(|(_, t)| t).expect("catalog ball");
    h.execute(&world, AddTool::new(&world, BALL, t).expect("tool"));
    h.execute(&world, ImportModel::from_mesh("part", cube(Point3::new(-15.0, -10.0, 0.0), Point3::new(15.0, 10.0, 10.0))));
    world
}

type Pick = Option<(f64, f64)>;

/// The Parallel 3D Finish on the block's top, passes along +Y.
fn finish3d(start: Pick, exit: Pick, waterlines: bool) -> camcraft_app::schema::Operation {
    let pick = |(x, y)| camcraft_app::path_picks::PathPick::xy(Point2::new(x, y));
    let params = OpParams::Finish3D {
        pattern: camcraft_app::finishing3d::FinishPattern::Parallel,
        direction_angle: 90.0,
        stepover: Some(4.0),
        tolerance: 0.1,
        clearance_z: Height::fixed(25.0),
        retract_z: None,
        top_z: Height::fixed(11.0),
        bottom_z: Height::fixed(7.0),
        feed: 500.0,
        plunge_feed: 100.0,
        climb: true,
        ramp: false,
        min_step: None,
        ramp_angle: 45.0,
        entry_ramp: false,
        steep_stepover: None,
        pencil_reference: None,
        fill_openings: None,
        gap_link: camcraft_app::finishing3d::GapLink::Jump,
        waterlines,
        pass_start: Default::default(),
        picks: camcraft_app::path_picks::PathPicks { start: start.map(pick).into_iter().collect(), exit: exit.map(pick).into_iter().collect() },
    };
    camcraft_app::schema::Operation {
        tool: BALL.into(),
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

type Seg = ([f64; 3], [f64; 3]);

/// The ball centre segments of the cuts on the top: feeds that travel in
/// XY at the top's centre height.
fn top_cuts(t: &camcraft_app::cl::Toolpath) -> Vec<Seg> {
    let c = |p: camcraft_app::geometry::Point3| [p.x, p.y, p.z + R];
    t.moves
        .iter()
        .filter(|m| m.kind == camcraft_app::cl::MoveKind::Feed && (m.from.x - m.to.x).hypot(m.from.y - m.to.y) > 1e-9)
        .map(|m| (c(m.from), c(m.to)))
        .filter(|(a, b)| (a[2] - 13.0).abs() < 0.2 && (b[2] - 13.0).abs() < 0.2)
        .collect()
}

fn first_plunge(t: &camcraft_app::cl::Toolpath) -> [f64; 3] {
    t.moves.iter().find(|m| m.kind == camcraft_app::cl::MoveKind::Plunge).map(|m| [m.to.x, m.to.y, m.to.z]).expect("a plunge")
}

fn last_feed(t: &camcraft_app::cl::Toolpath) -> usize {
    t.moves.iter().rposition(|m| m.kind == camcraft_app::cl::MoveKind::Feed).expect("a feed")
}

/// Every top cut of `got` lies on a top cut of `auto` along its whole
/// length, sampled every 0.25 mm, and the other way round.
fn same_top_cuts(auto: &camcraft_app::cl::Toolpath, got: &camcraft_app::cl::Toolpath) {
    let off = |c: [f64; 3], segs: &[Seg]| {
        segs.iter().map(|(a, b)| {
            let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let l2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
            let t = if l2 > 0.0 { (((c[0] - a[0]) * d[0] + (c[1] - a[1]) * d[1] + (c[2] - a[2]) * d[2]) / l2).clamp(0.0, 1.0) } else { 0.0 };
            (0..3).map(|i| (c[i] - a[i] - d[i] * t).powi(2)).sum::<f64>().sqrt()
        }).fold(f64::INFINITY, f64::min)
    };
    for (from, to) in [(got, auto), (auto, got)] {
        let segs = top_cuts(to);
        assert!(!segs.is_empty(), "the raster cuts the top");
        for (a, b) in top_cuts(from) {
            let len = (0..3).map(|i| (b[i] - a[i]).powi(2)).sum::<f64>().sqrt();
            let n = ((len / 0.25).ceil() as usize).max(1);
            for k in 0..=n {
                let t = k as f64 / n as f64;
                let c = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t];
                assert!(off(c, &segs) < 1e-6, "a top cut leaves the automatic raster at {c:?}");
            }
        }
    }
}

fn same(p: [f64; 3], q: [f64; 3]) -> bool {
    (0..3).all(|i| (p[i] - q[i]).abs() < 1e-6)
}

#[test]
fn finish3d_starts_at_the_picked_corner_and_ends_at_the_picked_exit() {
    let world = block_world();
    let generate = |op| camcraft_app::operation::generate(&world, &op, "").map(|(t, _)| t);
    let auto = generate(finish3d(None, None, false)).expect("the automatic raster");
    let a = first_plunge(&auto);
    // The far end of the first pass (the passes run along Y).
    let started = generate(finish3d(Some((a[0], -a[1])), None, false)).expect("the started raster");
    let s = first_plunge(&started);
    assert!(s[1] * a[1] < 0.0 && (s[1] + a[1]).abs() < 0.5 && (s[0] - a[0]).abs() < 1e-6, "the cut starts at the picked corner: {s:?} (automatic {a:?})");
    same_top_cuts(&auto, &started);
    // An exit at the automatic start: the whole raster runs back, the
    // last cut ends at the automatic first corner, and the tool rises
    // straight up from there.
    let exited = generate(finish3d(None, Some((a[0], a[1])), false)).expect("the exited raster");
    let k = last_feed(&exited);
    let end = exited.moves[k].to;
    assert!(same([end.x, end.y, end.z], a), "the cut ends at the exit corner: {end:?} (automatic start {a:?})");
    let rise = &exited.moves[k + 1];
    assert!(rise.from.x == rise.to.x && rise.from.y == rise.to.y && rise.to.z > rise.from.z, "the tool leaves straight up: {rise:?}");
    same_top_cuts(&auto, &exited);
    // Without a pick the path is the automatic raster, move for move.
    assert_eq!(generate(finish3d(None, None, false)).expect("again"), auto);
}

#[test]
fn finish3d_refuses_an_impossible_pick_by_name() {
    let world = block_world();
    let auto = camcraft_app::operation::generate(&world, &finish3d(None, None, false), "").map(|(t, _)| t).expect("auto");
    let a = first_plunge(&auto);
    let mut clipped = finish3d(Some((a[0], a[1])), None, false);
    clipped.area.offset = -6.0;
    let cases = [
        (finish3d(Some((0.0, 0.0)), None, false), "start point (0.000, 0.000)", "only at a corner"),
        (finish3d(None, Some((a[0], a[1])), true), "exit point", "the waterline rings cut after the raster"),
        (clipped, "start point", "the area controls clip the raster"),
    ];
    for (op, what, why) in cases {
        let e = camcraft_app::operation::generate(&world, &op, "").err().map(|e| format!("{e:?}"));
        assert!(e.as_deref().is_some_and(|m| m.contains(what) && m.contains(why)), "{what}, {why}: {e:?}");
    }
    // The same start with the rings on holds: the raster cuts first.
    let t = camcraft_app::operation::generate(&world, &finish3d(Some((a[0], -a[1])), None, true), "").map(|(t, _)| t).expect("a start with rings");
    assert!(first_plunge(&t)[1] * a[1] < 0.0, "the cut starts at the picked corner: {:?}", first_plunge(&t));
}

#[test]
fn finish3d_picks_ride_edit_operation_and_another_pattern_is_refused_at_the_door() {
    let mut s = job();
    assert!(!call(&mut s, "add_tool", json!({ "catalog": BALL })).0);
    let (err, r) = call(&mut s, "add_operation", json!({ "kind": "3D Finish", "tool": BALL, "name": "F3" }));
    assert!(!err, "{r}");
    let patch = json!({ "name": "F3", "params": { "pattern": "parallel", "picks": { "start": [[-27.25, -17.75]], "exit": [[27.25, 17.75]] } } });
    let (err, r) = call(&mut s, "edit_operation", patch);
    assert!(!err, "the Parallel 3D Finish takes both picks: {r}");
    let held = |s: &mut Server| call(s, "get_job", json!({})).1.to_string();
    assert!(held(&mut s).contains("27.25"), "the job carries the picks");
    // Another pattern with the picks is refused by name, and changes nothing.
    let (err, r) = call(&mut s, "edit_operation", json!({ "name": "F3", "params": { "pattern": "waterline" } }));
    let text = r.to_string();
    assert!(err && text.contains("3D Finish") && text.contains("Parallel pattern only"), "{text}");
    assert!(held(&mut s).contains("27.25"), "the refused edit keeps the picks");
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
    assert!(doc.contains("3D Finish (the Parallel pattern only"), "the schema names the 3D Finish and its limit: {doc}");
}
