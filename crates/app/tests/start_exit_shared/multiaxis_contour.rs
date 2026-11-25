//! The Multi-Axis Contour's start and exit picks (#3330) through the
//! application: a pick re-sequences the drive curve nearest it, an open
//! curve starts and leaves at the picked ends, and the cut geometry
//! stays the automatic path's, sampled along every cut move.
//! A module of the `start_exit_shared` suite.

use super::*;
use camcraft_app::cl::{MoveKind, Toolpath};
use camcraft_app::geometry::Point3;

/// The ball radius of the catalog ball `BN-0600`.
const R: f64 = 3.0;

/// The step that samples each cut move along its length.
const STEP: f64 = 0.25;

/// The 10 x 10 x 5 box at the origin, with the catalog ball.
fn box_world() -> flecs_ecs::prelude::World {
    use camcraft_app::commands::{AddTool, History, ImportModel};
    let world = camcraft_app::world::new_world();
    let mut h = History::default();
    let t = camcraft_app::tools::catalog().into_iter().find(|(n, _)| *n == "BN-0600").map(|(_, t)| t).expect("catalog ball");
    h.execute(&world, AddTool::new(&world, "BN-0600", t).expect("tool"));
    h.execute(&world, ImportModel::from_mesh("part", camcraft_app::geometry::cube(Point3::ZERO, Point3::new(10.0, 10.0, 5.0))));
    world
}

/// The box's four top edges: four open drive curves at z = 5.
fn top_edges(world: &flecs_ecs::prelude::World) -> camcraft_app::operation::SrcRef {
    use camcraft_app::geometry::feature;
    use camcraft_app::operation::{FeatureSel, GeoSel, SrcRef};
    let e = camcraft_app::model::find_by_name(world, "part").expect("part");
    let f = camcraft_app::model::features_of(world, &e).expect("features");
    let items: Vec<GeoSel> = feature::chains(&f)
        .into_iter()
        .filter(|c| {
            let pts = feature::chain_points(&f, c);
            pts.len() >= 2 && pts.iter().all(|p| (p[2] - 5.0).abs() < 1e-9)
        })
        .map(|c| GeoSel { entity: "part".into(), feature: FeatureSel::Chain(c.id) })
        .collect();
    assert_eq!(items.len(), 4, "a box has four top edges");
    SrcRef::Selection(items)
}

fn op(world: &flecs_ecs::prelude::World, start: Option<(f64, f64)>, exit: Option<(f64, f64)>) -> camcraft_app::schema::Operation {
    let pick = |(x, y)| camcraft_app::path_picks::PathPick::xy(Point2::new(x, y));
    let params = OpParams::MultiAxisContour {
        tolerance: 0.05,
        clearance_z: Height::fixed(25.0),
        retract_z: None,
        top_z: Height::fixed(6.0),
        bottom_z: Height::fixed(4.0),
        feed: 500.0,
        plunge_feed: 120.0,
        climb: true,
        lead_deg: 0.0,
        lag_deg: 0.0,
        max_tilt_deg: 60.0,
        shank_avoid: false,
        picks: camcraft_app::path_picks::PathPicks { start: start.map(pick).into_iter().collect(), exit: exit.map(pick).into_iter().collect() },
    };
    camcraft_app::schema::Operation {
        tool: "BN-0600".into(),
        src: top_edges(world),
        params,
        stamp: None,
        setup: None,
        area: Default::default(),
        coolant: Default::default(),
        after: Default::default(),
        suppressed: false,
    }
}

/// Each move's (kind, tip, ball centre from, ball centre to): the centre
/// is the tip backed up R along the pose the machine holds there.
fn moves(t: &Toolpath) -> Vec<(MoveKind, Point3, Point3, Point3)> {
    let mut held = [0.0, 0.0, 1.0];
    t.moves
        .iter()
        .map(|m| {
            let a = Point3::new(m.from.x + R * held[0], m.from.y + R * held[1], m.from.z + R * held[2]);
            held = m.axis.unwrap_or(held);
            (m.kind, m.to, a, Point3::new(m.to.x + R * held[0], m.to.y + R * held[1], m.to.z + R * held[2]))
        })
        .collect()
}

/// Each drive curve's cut, as tips: from its plunge to its last feed.
fn runs(t: &Toolpath) -> Vec<Vec<Point3>> {
    let mut out: Vec<Vec<Point3>> = Vec::new();
    for (k, tip, _, _) in moves(t) {
        match k {
            MoveKind::Plunge => out.push(vec![tip]),
            MoveKind::Feed => {
                if let Some(r) = out.last_mut() {
                    r.push(tip);
                }
            }
            _ => {}
        }
    }
    out
}

fn cuts(t: &Toolpath) -> Vec<(Point3, Point3)> {
    moves(t).into_iter().filter(|m| m.0 == MoveKind::Feed).map(|m| (m.2, m.3)).collect()
}

fn off(segs: &[(Point3, Point3)], c: Point3) -> f64 {
    segs.iter()
        .map(|&(a, b)| {
            let d = b - a;
            let l2 = d.dot(d);
            let t = if l2 > 0.0 { ((c - a).dot(d) / l2).clamp(0.0, 1.0) } else { 0.0 };
            let e = c - Point3::new(a.x + d.x * t, a.y + d.y * t, a.z + d.z * t);
            e.dot(e).sqrt()
        })
        .fold(f64::INFINITY, f64::min)
}

/// Every cut of `got` rides a cut of `auto` along its whole length, and
/// every cut of `auto` rides a cut of `got`: each ball centre move is
/// sampled every STEP and each sample sits within 0.02 mm of the other.
fn same_cut(auto: &Toolpath, got: &Toolpath) {
    let (a, g) = (cuts(auto), cuts(got));
    for (from, to, what) in [(&g, &a, "a picked cut leaves the automatic path"), (&a, &g, "an automatic cut is left out")] {
        for &(p, q) in from {
            let n = (((q - p).dot(q - p).sqrt() / STEP).ceil() as usize).max(1);
            for k in 0..=n {
                let t = k as f64 / n as f64;
                let c = Point3::new(p.x + (q.x - p.x) * t, p.y + (q.y - p.y) * t, p.z + (q.z - p.z) * t);
                let e = off(to, c);
                assert!(e < 0.02, "{what}: {e:.4} mm at {c:?}");
            }
        }
    }
}

/// The run on the y = 0 edge: every tip on y = 0, z = 5.
fn front(t: &Toolpath) -> Vec<Point3> {
    runs(t).into_iter().find(|r| r.iter().all(|p| p.y.abs() < 1e-6 && (p.z - 5.0).abs() < 1e-6)).expect("the y = 0 edge is cut")
}

#[test]
fn multiaxis_contour_starts_and_leaves_at_the_picked_ends() {
    let world = box_world();
    let generate = |o| camcraft_app::operation::generate(&world, &o, "").map(|(t, _)| t);
    let auto = generate(op(&world, None, None)).expect("the automatic path");
    assert!(front(&auto).len() > 1, "the automatic path cuts the y = 0 edge");
    // The start pick beside the (10, 0) corner, the exit beside (0, 0).
    let got = generate(op(&world, Some((9.5, -1.0)), Some((0.5, -1.0)))).expect("the picked path");
    let g = front(&got);
    assert!((g[0].x - 10.0).abs() < 0.05, "the cut does not start at the picked end: {:?}", g[0]);
    assert!(g[g.len() - 1].x.abs() < 0.05, "the cut does not leave at the picked end: {:?}", g[g.len() - 1]);
    assert_eq!(runs(&got).len(), runs(&auto).len(), "the picks re-sequence the curves, they add none");
    let other = |t: &Toolpath| runs(t).into_iter().filter(|r| !r.iter().all(|p| p.y.abs() < 1e-6)).collect::<Vec<_>>();
    assert_eq!(other(&got), other(&auto), "the curves far from the picks keep their automatic cut");
    same_cut(&auto, &got);
    assert_eq!(generate(op(&world, None, None)).expect("again"), auto, "no pick is the automatic path");
}

#[test]
fn multiaxis_contour_refuses_impossible_picks_by_name() {
    let world = box_world();
    let cases = [
        (op(&world, Some((40.0, 40.0)), None), "start point (40.000, 40.000): the nearest drive curve stands"),
        (op(&world, Some((9.5, -1.0)), Some((8.5, -1.0))), "exit point (8.500, -1.000): it is nearer the end of open drive curve"),
    ];
    for (o, want) in cases {
        let e = camcraft_app::operation::generate(&world, &o, "").err().map(|e| format!("{e:?}"));
        assert!(e.as_deref().is_some_and(|m| m.contains(want)), "want {want:?}, got {e:?}");
    }
}

#[test]
fn multiaxis_contour_picks_ride_edit_operation_and_undo_restores_them() {
    let mut s = job();
    assert!(!call(&mut s, "add_tool", json!({ "catalog": "BN-0600" })).0);
    let (err, r) = call(&mut s, "add_operation", json!({ "kind": "Multi-Axis Contour", "tool": "BN-0600", "name": "MAC" }));
    assert!(!err, "{r}");
    let patch = json!({ "name": "MAC", "params": { "picks": { "start": [[31.25, 0.75, 5.5]], "exit": [[0.75, 27.25]] } } });
    let (err, r) = call(&mut s, "edit_operation", patch);
    assert!(!err, "the multi-axis contour takes both picks: {r}");
    let held = |s: &mut Server| call(s, "get_job", json!({})).1.to_string();
    let text = held(&mut s);
    assert!(text.contains("31.25") && text.contains("27.25"), "the job carries the picks: {text:.600}");
    assert!(!call(&mut s, "undo", json!({})).0);
    let text = held(&mut s);
    assert!(!text.contains("31.25") && !text.contains("27.25"), "undo restores the automatic path");
    assert!(!call(&mut s, "redo", json!({})).0);
    assert!(held(&mut s).contains("31.25"), "redo states the picks again");
}
