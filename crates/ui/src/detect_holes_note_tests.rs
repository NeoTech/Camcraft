//! #1556: the Detect holes button on a Bore op leaves out a hole wider
//! than its tool's ring clears, and the status line names it - the same
//! sentence the MCP `detect_holes` tool answers in `skipped`.
//!
//! Provenance: our own tests over our own detect-holes rule; no vendor
//! material consulted.

use camcraft_app::commands::{AddTool, CreateOperation, ImportModel};
use camcraft_app::geometry::{Mesh, Point3, Triangle};
use camcraft_app::model;
use camcraft_app::operation::{Height, OpParams, SrcRef};
use camcraft_app::tools::catalog;

use crate::state::{ErrorScope, State, UiTarget};

const TOOL: &str = "EM-0400-2FL";
const H: f64 = 50.0;
const N: usize = 64;

/// 100 x 100 x 10 cells along X, each bored through at its centre with
/// the given radius; bore vertices pushed radially onto the cell's
/// square so neighbours share their edge vertices.
fn plate(radii: &[f64]) -> Mesh {
    let mut facets = Vec::new();
    let mut quad = |a: Point3, b: Point3, c: Point3, d: Point3, want: [f64; 3]| {
        for [p, q, r] in [[a, b, c], [a, c, d]] {
            let (u, v) = ([q.x - p.x, q.y - p.y, q.z - p.z], [r.x - p.x, r.y - p.y, r.z - p.z]);
            let n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
            let dot = n[0] * want[0] + n[1] * want[1] + n[2] * want[2];
            facets.push(if dot >= 0.0 { Triangle::new(p, q, r) } else { Triangle::new(p, r, q) });
        }
    };
    let cells = radii.len();
    for (k, &rad) in radii.iter().enumerate() {
        let cx = (2.0 * k as f64 + 1.0) * H - cells as f64 * H;
        let ang = |i: usize| std::f64::consts::TAU * (i % N) as f64 / N as f64;
        let bp = |i: usize, z: f64| Point3::new(cx + rad * ang(i).cos(), rad * ang(i).sin(), z);
        let sq = |i: usize, z: f64| {
            let (x, y) = (ang(i).cos(), ang(i).sin());
            let s = x.abs().max(y.abs());
            Point3::new(cx + x * H / s, y * H / s, z)
        };
        for i in 0..N {
            let j = i + 1;
            quad(bp(i, 10.0), bp(j, 10.0), sq(j, 10.0), sq(i, 10.0), [0.0, 0.0, 1.0]);
            quad(bp(i, 0.0), bp(j, 0.0), sq(j, 0.0), sq(i, 0.0), [0.0, 0.0, -1.0]);
            let mid = ang(i) + std::f64::consts::PI / N as f64;
            quad(bp(i, 10.0), bp(j, 10.0), bp(j, 0.0), bp(i, 0.0), [-mid.cos(), -mid.sin(), 0.0]);
            let (a, b) = (sq(i, 10.0), sq(j, 10.0));
            let on = |x: f64| (a.x - x).abs() < 1e-9 && (b.x - x).abs() < 1e-9;
            if !((k > 0 && on(cx - H)) || (k + 1 < cells && on(cx + H))) {
                quad(a, b, sq(j, 0.0), sq(i, 0.0), [(a.x + b.x) * 0.5 - cx, (a.y + b.y) * 0.5, 0.0]);
            }
        }
    }
    Mesh::from_facets(facets)
}

fn bore_state(radii: &[f64]) -> State {
    let mut state = State::new();
    state.history.execute(&state.world, ImportModel::from_mesh("Plate", plate(radii)));
    let tool = catalog().into_iter().find(|(n, _)| *n == TOOL).map(|(_, t)| t).expect("catalog entry");
    state.history.execute(&state.world, AddTool::new(&state.world, TOOL, tool).expect("tool"));
    let params = OpParams::Bore {
        links: camcraft_app::links::Links::PLANAR,
        clearance_z: Height::fixed(25.0),
        retract_z: None,
        top_z: Height::fixed(10.0),
        bottom_z: None,
        stock_to_leave: 0.0,
        stepdown: 2.0,
        feed: 500.0,
        plunge_feed: 150.0,
        climb: true,
        tolerance: 0.01,
        holes: Vec::new(),
    };
    let src = SrcRef::Entity("Plate".into());
    state.history.execute(&state.world, CreateOperation::new(&state.world, "BOR", TOOL, src, params).expect("create"));
    state.ui_target = UiTarget::Operation("BOR".into());
    state.sync_ops_ui();
    state
}

fn stored_holes(state: &State) -> usize {
    match model::find_by_name(&state.world, "BOR").and_then(|e| model::operation_of(&state.world, &e)).map(|o| o.params) {
        Some(OpParams::Bore { holes, .. }) => holes.len(),
        _ => usize::MAX,
    }
}

#[test]
fn detect_holes_on_a_bore_op_stores_the_small_hole_and_names_the_wide_one() {
    let mut state = bore_state(&[37.15, 3.0]);
    state.detect_holes("BOR");
    state.drain_actions();
    assert_eq!(stored_holes(&state), 1, "only the Ø6 hole is stored");
    for want in ["holes detected for BOR", "74.300 mm hole at (-50.000, 0.000) was skipped", "12.000 mm"] {
        assert!(state.status.contains(want), "the status names `{want}`: {}", state.status);
    }
}

#[test]
fn detect_holes_on_a_bore_op_with_only_a_wide_hole_is_refused() {
    let mut state = bore_state(&[37.15]);
    state.detect_holes("BOR");
    state.drain_actions();
    assert_eq!(stored_holes(&state), 0, "a refused detection stores nothing");
    let msg = state.error_for(ErrorScope::Operation).unwrap_or_default();
    assert!(msg.contains("no hole this Bore op can make") && msg.contains("74.300 mm hole"), "{msg}");
}
