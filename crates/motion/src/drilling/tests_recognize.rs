//! Hole recognition from a mesh's vertical walls: through and blind holes,
//! refusals (walls, bosses, slots), counterbores, vertex noise and
//! degenerate input. Split from `tests.rs` (#2528); structure only.
//! Provenance: our own hole-recognition law (see `hole.rs`).

use crate::geometry::{Point3, Triangle};
use super::*;
use super::tests::{circle, circle_cw, ring_walls};

fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<(f64, f64)> {
    vec![(x0, y0), (x1, y0), (x1, y1), (x0, y1)]
}

/// A 20x20x10 plate's walls plus `extra` void/collar walls.
fn plate(extra: Vec<Triangle>) -> Vec<Triangle> {
    let mut t = ring_walls(&rect(0.0, 0.0, 20.0, 20.0), 0.0, 10.0);
    t.extend(extra);
    t
}

#[test]
fn recognize_finds_through_and_blind_holes() {
    let mut t = plate(ring_walls(&circle_cw(10.0, 10.0, 2.0, 16), 0.0, 10.0));
    t.extend(ring_walls(&circle_cw(5.0, 5.0, 1.5, 16), 4.0, 10.0));
    let holes = recognize(&t);
    assert_eq!(holes.len(), 2, "{holes:?}");
    let through = holes.iter().find(|h| (h.xy.x - 10.0).abs() < 1e-6).unwrap();
    assert!(through.through);
    assert!((through.radius - 2.0).abs() < 1e-6);
    assert_eq!(through.entry_z, 10.0);
    assert_eq!(through.floor_z, 0.0);
    let blind = holes.iter().find(|h| (h.xy.x - 5.0).abs() < 1e-6).unwrap();
    assert!(!blind.through);
    assert_eq!(blind.floor_z, 4.0);
}

#[test]
fn recognize_refuses_walls_bosses_and_slots() {
    // The plate's own outer wall: straight, normals face away.
    assert!(recognize(&plate(Vec::new())).is_empty(), "outer wall is not a hole");
    // A raised boss: closed round wall, normals face away.
    let mut m = ring_walls(&rect(-20.0, -20.0, 20.0, 20.0), 0.0, 2.0);
    m.extend(ring_walls(&circle(0.0, 0.0, 5.0, 32), 2.0, 8.0)); // CCW = outward-facing
    assert!(recognize(&m).is_empty(), "a boss is not a hole");
    // A racetrack slot: a void, but its vertices fit no circle.
    let mut track: Vec<(f64, f64)> = (1..16)
        .map(|i| {
            let a = -std::f64::consts::PI / 2.0 + std::f64::consts::PI * (i as f64) / 16.0;
            (13.0 + 3.0 * a.cos(), 10.0 + 3.0 * a.sin())
        })
        .collect();
    track.extend((1..16).map(|i| {
        let a = std::f64::consts::PI / 2.0 + std::f64::consts::PI * (i as f64) / 16.0;
        (7.0 + 3.0 * a.cos(), 10.0 + 3.0 * a.sin())
    }));
    let mut m2 = plate(ring_walls(&rect(0.0, 0.0, 40.0, 20.0), 0.0, 10.0));
    m2.extend(ring_walls(&track, 0.0, 10.0));
    assert!(recognize(&m2).is_empty(), "a slot is not a drilled hole");
}

#[test]
fn recognize_merges_a_counterbore() {
    // A wider bore stepped above the main wall: one axis, two z-spans.
    let mut m = plate(ring_walls(&circle_cw(10.0, 10.0, 2.0, 24), 0.0, 8.0));
    m.extend(ring_walls(&circle_cw(10.0, 10.0, 3.0, 24), 8.0, 10.0));
    let h = recognize(&m);
    assert_eq!(h.len(), 1, "counterbore is one hole: {h:?}");
    assert!((h[0].radius - 2.0).abs() < 1e-6, "the narrow wall is the bore");
    assert_eq!(h[0].entry_z, 10.0);
    assert_eq!(h[0].floor_z, 0.0);
    assert!(h[0].through);
}

#[test]
fn recognize_is_stable_under_vertex_noise() {
    let base = plate(ring_walls(&circle_cw(10.0, 10.0, 2.0, 16), 0.0, 10.0));
    let noisy = base
        .iter()
        .map(|t| {
            let jit = |p: Point3| {
                let k = p.x.to_bits() ^ (p.y.to_bits() << 1) ^ (p.z.to_bits() >> 3);
                let e = ((k & 0xFFFF) as f64 / 65535.0 - 0.5) * 2e-7;
                Point3::new(p.x + e, p.y + e, p.z)
            };
            Triangle::new(jit(t.a), jit(t.b), jit(t.c))
        })
        .collect::<Vec<_>>();
    let (a, b) = (recognize(&base), recognize(&noisy));
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(&b) {
        assert!((x.xy.x - y.xy.x).abs() < 1e-4 && (x.xy.y - y.xy.y).abs() < 1e-4);
        assert!((x.radius - y.radius).abs() < 1e-4);
    }
}

#[test]
fn recognize_degenerate_input_never_panics() {
    assert!(recognize(&[]).is_empty());
    let nan = Triangle::new(
        Point3::new(f64::NAN, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    );
    assert!(recognize(&[nan]).is_empty());
    let mut t = plate(ring_walls(&circle_cw(10.0, 10.0, 2.0, 16), 0.0, 10.0));
    for x in t.iter_mut() {
        x.b = x.a; // collapse every triangle to zero area
    }
    assert!(recognize(&t).is_empty(), "zero-area facets must drop, not divide");
}
