//! The 3D Pocket's start and exit picks (#3329, slice 3) through the
//! application: each level descends on its first ring at its point
//! nearest the start pick, and the operation ends on its last ring at
//! its point nearest the exit pick. A module of the `start_exit_shared`
//! suite.

use super::*;

const MILL: &str = "EM-0600-4FL";

/// A 10 mm square block (z 0..10) in a 40 x 40 box of stock, with a
/// 6 mm end mill: the pocket clears the stock round the block.
fn block_world() -> flecs_ecs::prelude::World {
    use camcraft_app::commands::{AddTool, History, ImportModel, SetStock};
    use camcraft_app::geometry::{cube, Point3};
    let world = camcraft_app::world::new_world();
    let mut h = History::default();
    h.execute(&world, ImportModel::from_mesh("block", cube(Point3::new(-5.0, -5.0, 0.0), Point3::new(5.0, 5.0, 10.0))));
    let shape = camcraft_app::stock::StockShape::Box { dx: 40.0, dy: 40.0, dz: 10.0 };
    h.execute(&world, SetStock::new(&world, shape, camcraft_app::schema::Location::default()).expect("stock"));
    let t = camcraft_app::tools::catalog().into_iter().find(|(n, _)| *n == MILL).map(|(_, t)| t).expect("catalog mill");
    h.execute(&world, AddTool::new(&world, MILL, t).expect("tool"));
    world
}

type Pick = Option<(f64, f64)>;

/// Waterline rings, plunge entry, three levels down to the stock bottom.
fn pocket(start: Pick, exit: Pick, strategy: camcraft_app::machining2d::PocketStrategy) -> camcraft_app::schema::Operation {
    let pick = |(x, y)| camcraft_app::path_picks::PathPick::xy(Point2::new(x, y));
    let mut params = camcraft_app::operation::seed_params(OpType::Pocket3D.ordinal());
    if let OpParams::Pocket3D { strategy: s, entry, stepdown, top_z, bottom_z, clearance_z, picks, .. } = &mut params {
        *s = strategy;
        *entry = camcraft_app::machining3d::Entry3D::Plunge;
        *stepdown = 4.0;
        *top_z = Height::fixed(10.0);
        *bottom_z = Height::fixed(2.0);
        *clearance_z = Height::fixed(25.0);
        *picks = camcraft_app::path_picks::PathPicks { start: start.map(pick).into_iter().collect(), exit: exit.map(pick).into_iter().collect() };
    }
    camcraft_app::schema::Operation {
        tool: MILL.into(),
        src: camcraft_app::operation::SrcRef::Entity("block".into()),
        params,
        stamp: None,
        setup: None,
        area: Default::default(),
        coolant: Default::default(),
        after: Default::default(),
        suppressed: false,
    }
}

type Pocket = camcraft_app::machining2d::PocketStrategy;

/// Every cut move, sampled each 0.25 mm, keeps the tool radius (3 mm,
/// less 0.01) off the block and stays inside the 40 mm stock.
fn no_gouge(t: &camcraft_app::cl::Toolpath) {
    for m in t.moves.iter().filter(|m| m.kind != camcraft_app::cl::MoveKind::Rapid) {
        for (a, b) in m.chords(0.01) {
            let len = (b.x - a.x).hypot(b.y - a.y);
            let n = ((len / 0.25).ceil() as usize).max(1);
            for k in 0..=n {
                let f = k as f64 / n as f64;
                let (x, y) = (a.x + (b.x - a.x) * f, a.y + (b.y - a.y) * f);
                let (dx, dy) = ((x.abs() - 5.0).max(0.0), (y.abs() - 5.0).max(0.0));
                assert!(dx.hypot(dy) >= 2.99, "a cut gouges the block at ({x}, {y})");
                assert!(x.abs() <= 20.0 + 1e-6 && y.abs() <= 20.0 + 1e-6, "a cut leaves the stock at ({x}, {y})");
            }
        }
    }
}

/// `p` at (x, y, z), XY read to the 0.001 mm of the readings.
fn at(p: camcraft_app::geometry::Point3, x: f64, y: f64, z: f64) -> bool {
    (p.x - x).abs() < 1e-3 && (p.y - y).abs() < 1e-3 && (p.z - z).abs() < 1e-6
}

#[test]
fn pocket3d_starts_at_the_picked_point_and_ends_at_the_picked_exit() {
    use camcraft_app::cl::MoveKind;
    let world = block_world();
    let generate = |op| camcraft_app::operation::generate(&world, &op, "").map(|(t, _)| t);
    let auto = generate(pocket(None, None, Pocket::OffsetRings)).expect("the automatic pocket");
    let t = generate(pocket(Some((-14.0, 10.0)), Some((10.0, -21.0)), Pocket::OffsetRings)).expect("the picked pocket");
    for (path, x, y) in [(&auto, -12.636, 11.502), (&t, -12.961, 11.095)] {
        for z in [6.0, 2.0] {
            let dive = path.moves.iter().find(|m| m.kind == MoveKind::Plunge && (m.to.z - z).abs() < 1e-6).map(|m| m.to);
            assert!(dive.is_some_and(|d| at(d, x, y, z)), "Z {z}: the level descends at ({x}, {y}): {dive:?}");
        }
    }
    let n = t.moves.len();
    assert!(t.moves[n - 2].kind == MoveKind::Feed && at(t.moves[n - 2].to, 10.0, -19.99, 2.0), "the cut ends at the exit: {:?}", t.moves[n - 2]);
    assert!(at(t.moves[n - 1].to, 10.0, -19.99, 25.0), "the rise is straight up: {:?}", t.moves[n - 1]);
    no_gouge(&t);
    // Without a pick the path is the automatic one, move for move.
    assert_eq!(generate(pocket(None, None, Pocket::OffsetRings)).expect("again"), auto);
}

#[test]
fn pocket3d_refuses_an_impossible_pick_by_name() {
    let world = block_world();
    let e = camcraft_app::operation::generate(&world, &pocket(Some((0.0, -21.0)), None, Pocket::OffsetRings), "").err().map(|e| format!("{e:?}"));
    assert!(e.as_deref().is_some_and(|m| m.contains("start point (0.000, -21.000)") && m.contains("first pass") && m.contains("Z 6.000")), "{e:?}");
    let mut s = job();
    assert!(!call(&mut s, "add_operation", json!({ "kind": "3D Pocket", "tool": "EM-1000-4FL", "name": "P3" })).0);
    for (field, value, word) in [("strategy", json!("adaptive"), "Adaptive"), ("rest_rough", json!(true), "Rest roughing")] {
        let patch = json!({ "name": "P3", "params": { field: value, "picks": { "start": [[0.0, 0.0]] } } });
        let (err, r) = call(&mut s, "edit_operation", patch);
        let text = r.to_string();
        assert!(err && text.contains("3D Pocket") && text.contains(word), "{field}: refused by name: {text}");
    }
}

#[test]
fn pocket3d_picks_ride_edit_operation_and_undo_restores_them() {
    let mut s = job();
    let (err, r) = call(&mut s, "add_operation", json!({ "kind": "3D Pocket", "tool": "EM-1000-4FL", "name": "P3" }));
    assert!(!err, "{r}");
    let patch = json!({ "name": "P3", "params": { "picks": { "start": [[12.25, 5.75]], "exit": [[-7.75, 3.25]] } } });
    let (err, r) = call(&mut s, "edit_operation", patch);
    assert!(!err, "the 3D Pocket takes both picks: {r}");
    let held = |s: &mut Server| call(s, "get_job", json!({})).1.to_string();
    let text = held(&mut s);
    assert!(text.contains("12.25") && text.contains("-7.75"), "the job carries the picks: {text:.600}");
    assert!(!call(&mut s, "undo", json!({})).0);
    let text = held(&mut s);
    assert!(!text.contains("12.25") && !text.contains("-7.75"), "undo restores the automatic entry");
    assert!(!call(&mut s, "redo", json!({})).0);
    assert!(held(&mut s).contains("12.25"), "redo states the picks again");
    let reply = s.handle(&json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {} })).expect("an answer");
    let doc = reply["result"]["tools"]
        .as_array()
        .and_then(|t| t.iter().find(|t| t["name"] == "edit_operation"))
        .map(|t| t["inputSchema"]["properties"]["params"]["description"].as_str().unwrap_or_default().to_owned())
        .unwrap_or_default();
    assert!(doc.contains("Region picks, taken by 3D Pocket"), "the schema names the region kind: {doc}");
}
