//! The radial rings' geometry (#2774, review of #3490): the ring stands in
//! the plane SQUARE to the axis, at the stated radius, on every axis - the
//! second perpendicular is `dir × u`, never re-perpendicularized through
//! `off_axis` (which turned an X axis's ring into a line through
//! ±sqrt(2)·r and a Z axis's second vector into the axis itself).
//!
//! Provenance: our own fixtures; a circle about a line is elementary
//! geometry. No vendor material consulted.

use super::*;
use crate::state::{CommandKind, State};
use camcraft_app::commands::{AddTool, CreateOperation, ImportModel};
use camcraft_app::geometry::{Mesh, Triangle};
use camcraft_app::operation::{Height, OpParams, RadialHeight, RadialSource, SrcRef};
use camcraft_app::stock::StockShape;
use camcraft_app::tools::catalog;

/// The axis directions the review names: the three world axes and one
/// oblique line (a fitted axis is a unit direction; the oblique one stands
/// for any of them).
const DIRS: [&str; 4] = ["X", "Y", "Z", "oblique"];

fn dir_of(name: &str) -> [f64; 3] {
    match name {
        "X" => [1.0, 0.0, 0.0],
        "Y" => [0.0, 1.0, 0.0],
        "Z" => [0.0, 0.0, 1.0],
        _ => {
            let d: [f64; 3] = [1.0, 2.0, 3.0];
            let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            [d[0] / l, d[1] / l, d[2] / l]
        }
    }
}

#[test]
fn the_ring_basis_is_orthonormal_on_every_axis() {
    for name in DIRS {
        let d = dir_of(name);
        let (u, v) = ring_basis(d);
        let dot = |a: [f64; 3], b: [f64; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
        let len = |a: [f64; 3]| dot(a, a).sqrt();
        assert!((len(u) - 1.0).abs() < 1e-9, "{name}: u is not unit: {u:?}");
        assert!((len(v) - 1.0).abs() < 1e-9, "{name}: v is not unit: {v:?}");
        assert!(dot(u, d).abs() < 1e-9, "{name}: u does not stand off the axis: {u:?}");
        assert!(dot(v, d).abs() < 1e-9, "{name}: v does not stand off the axis: {v:?}");
        assert!(dot(u, v).abs() < 1e-9, "{name}: u and v are not square to each other");
    }
}

#[test]
fn every_ring_point_stands_in_its_plane_at_its_radius() {
    let on_axis = [2.0, -1.0, 0.5];
    let radius = 4.0;
    for name in DIRS {
        let d = dir_of(name);
        let points = ring_points(on_axis, d, radius);
        assert_eq!(points.len(), SEGMENTS, "{name}: a whole ring");
        for (i, p) in points.iter().enumerate() {
            let w = [p[0] - on_axis[0], p[1] - on_axis[1], p[2] - on_axis[2]];
            let plane = w[0] * d[0] + w[1] * d[1] + w[2] * d[2];
            assert!(plane.abs() < 1e-9, "{name}: ring point {i} leaves the plane square to the axis: {plane:.9}");
            let r = (w[0] * w[0] + w[1] * w[1] + w[2] * w[2]).sqrt();
            assert!((r - radius).abs() < 1e-9, "{name}: ring point {i} stands at {r:.9}, not the stated {radius}");
        }
        // The ring is closed and spread: consecutive points are distinct
        // (the collapsed-to-a-line signature of the old basis).
        let spread: f64 = (0..points.len())
            .map(|i| {
                let a = points[i];
                let b = points[(i + 1) % points.len()];
                ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
            })
            .fold(f64::INFINITY, f64::min);
        assert!(spread > 0.1 * radius, "{name}: the ring collapsed (shortest chord {spread:.3})");
    }
}

/// A standing turned bar (radius `r`, z from 0 to 16) swept at `n` turns.
fn turned_bar(n: usize, r: f64) -> Vec<Triangle> {
    let v = |j: usize, rho: f64, z: f64| {
        let th = std::f64::consts::TAU * j as f64 / n as f64;
        Point3::new(rho * th.cos(), rho * th.sin(), z)
    };
    let mut t = Vec::new();
    for (seg, next) in [((0.0, 0.0), (r, 0.0)), ((r, 0.0), (r, 16.0)), ((r, 16.0), (0.0, 16.0))] {
        for j in 0..n {
            let k = (j + 1) % n;
            if seg.0 <= 1e-12 {
                t.push(Triangle::new(v(j, seg.0, seg.1), v(k, next.0, next.1), v(j, next.0, next.1)));
            } else if next.0 <= 1e-12 {
                t.push(Triangle::new(v(j, seg.0, seg.1), v(j + 1, seg.0, seg.1), v(k, next.0, next.1)));
            } else {
                t.push(Triangle::new(v(j, seg.0, seg.1), v(j + 1, seg.0, seg.1), v(k, next.0, next.1)));
                t.push(Triangle::new(v(j, seg.0, seg.1), v(k, next.0, next.1), v(j, next.0, next.1)));
            }
        }
    }
    t
}

/// Stock, tool, model and one Rotary Contour with its radial heights
/// stated, opened in the pane the way a tree click opens it - the state
/// the viewport's ring overlay reads.
fn opened() -> State {
    let mut state = State::new();
    state.pending.push_back(CommandKind::SetStock {
        shape: StockShape::Cylinder { diameter: 6.4, height: 16.0 },
        pos: [0.0, 0.0, 0.0],
        yaw_deg: 0.0,
    });
    state.drain_actions();
    let bn = catalog().into_iter().find(|(n, _)| *n == "BN-0600").map(|(_, t)| t).expect("catalog entry");
    state.history.execute(&state.world, AddTool::new(&state.world, "BN-0600", bn).expect("tool"));
    state.history.execute(&state.world, ImportModel::from_mesh("bar", Mesh::from_facets(turned_bar(16, 3.0))));
    let params = OpParams::RotaryContour {
        tolerance: 0.02,
        clearance_z: Height::fixed(4.0),
        retract_z: None,
        top_z: Height::fixed(3.0),
        bottom_z: Height::fixed(-3.0),
        clearance_r: Some(RadialHeight::of(RadialSource::StockOd, 0.0)),
        retract_r: None,
        top_r: None,
        bottom_r: Some(RadialHeight::of(RadialSource::Absolute, 4.0)),
        feed: 200.0,
        plunge_feed: 50.0,
        climb: true,
        lead_deg: 0.0,
        lag_deg: 0.0,
        max_tilt_deg: 60.0,
        stated_axis: None,
        picks: Default::default(),
    };
    state.history.execute(
        &state.world,
        CreateOperation::new(&state.world, "W1", "BN-0600", SrcRef::Entity("bar".into()), params)
            .expect("rotary operation"),
    );
    state.pending.push_back(CommandKind::Select(Some("W1".into())));
    state.drain_actions();
    state.sync_ops_ui();
    state
}

/// The wiring, end to end: the overlay's rings for the open operation
/// stand in planes square to the judged axis, at the stated radii - the
/// same line the generators turn about.
#[test]
fn the_overlay_rings_stand_in_planes_about_the_judged_axis() {
    let state = opened();
    let view = crate::height_memo::radial_axis(&state.world, None, &SrcRef::Entity("bar".into()), None)
        .expect("the standing bar has an axis");
    let lines = ring_lines(&state);
    assert!(!lines.is_empty(), "the stated radii draw their rings");
    // Two stated radii, one ring each: clearance 3.2 (the stock OD) and
    // bottom 4.0, about the bar's own axis through the fit's origin. Each
    // entry is one endpoint with its colour ([x, y, z, r, g, b]); the
    // chords pair consecutive entries.
    let radii = [3.2_f64, 4.0];
    let mut seen = [0usize; 2];
    let mut n = 0usize;
    for line in &lines {
        let p = [line[0] as f64, line[1] as f64, line[2] as f64];
        n += 1;
        // In the plane square to the judged axis (Z for the standing
        // bar): every point keeps the axis coordinate of the origin.
        assert!((p[2] - view.origin[2]).abs() < 1e-6, "a ring point leaves the axis plane: {p:?}");
        let r = ((p[0] - view.origin[0]).powi(2) + (p[1] - view.origin[1]).powi(2)).sqrt();
        assert!(
            radii.iter().enumerate().any(|(i, want)| {
                let hit = (r - want).abs() < 1e-3;
                if hit {
                    seen[i] += 1;
                }
                hit
            }),
            "a ring point stands at {r:.6}, not a stated radius {radii:?}"
        );
    }
    assert!(n >= 64, "two whole rings of chords, got {n} endpoints");
    assert!(seen.iter().all(|s| *s > 0), "both stated radii draw their ring: {seen:?}");
}
