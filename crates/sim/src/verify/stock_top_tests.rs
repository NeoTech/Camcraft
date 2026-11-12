//! #1091: the stock-top re-reading of law (d)'s dives - pinned on a
//! stepped slab (Z 20, a boss to Z 30 over x 10..20) with an 8 mm tool.

use super::*;
use crate::cl::Move;
use crate::geometry::{Point3, cube};

fn tool() -> ToolGeom {
    ToolGeom {
        radius: 4.0,
        nose_radius: 0.0,
        flute_len: 20.0,
        shank_radius: 4.0,
        shank_len: 30.0,
        holder_radius: 16.0,
        holder_len: 40.0,
    }
}

/// The slab and the boss as two closed boxes (the bound only needs each
/// facet's plan and height).
fn stepped() -> Vec<Triangle> {
    let mut f = cube(Point3::new(-30.0, -30.0, -10.0), Point3::new(30.0, 30.0, 20.0)).facets.to_vec();
    f.extend(cube(Point3::new(10.0, -5.0, 20.0), Point3::new(20.0, 5.0, 30.0)).facets.iter().cloned());
    f
}

fn dive_at(x: f64, z1: f64, axis: Option<[f64; 3]>) -> Toolpath {
    Toolpath {
        moves: vec![Move {
            rotary: None,
            kind: MoveKind::Rapid,
            from: Point3::new(x, 0.0, 40.0),
            to: Point3::new(x, 0.0, z1),
            feed: None,
            arc: None,
            axis,
        }],
        cycles: Vec::new(),
        notes: Vec::new(),
        comps: Vec::new(),
        threads: Vec::new(),
        probes: Vec::new(),
        breaks: Vec::new(),
        plasma: None,
    }
}

fn judged(path: &Toolpath, mode: Mode, solids: &[&[Triangle]], t: &ToolGeom) -> Vec<Warning> {
    let mut w = vec![Warning::RapidInStock { at: 0 }];
    acquit_by_stock_top(path, mode, solids, t, 1e-3, &mut w);
    w
}

const CONVICTED: [Warning; 1] = [Warning::RapidInStock { at: 0 }];

#[test]
fn a_dive_over_the_low_step_is_acquitted() {
    let s = stepped();
    assert!(judged(&dive_at(0.0, 22.0, None), Mode::Area, &[&s], &tool()).is_empty());
}

#[test]
fn a_dive_over_the_boss_stands() {
    let s = stepped();
    assert_eq!(judged(&dive_at(15.0, 22.0, None), Mode::Area, &[&s], &tool()), CONVICTED);
}

#[test]
fn a_boss_under_the_rim_only_stands() {
    // The centre 3 mm off the boss's edge, the 4 mm rim over it.
    let s = stepped();
    assert_eq!(judged(&dive_at(7.0, 22.0, None), Mode::Area, &[&s], &tool()), CONVICTED);
    // 4.5 mm off: the flute clears it, and so do shank and holder (the
    // holder's bottom stands 50 mm over the tip).
    assert!(judged(&dive_at(5.5, 22.0, None), Mode::Area, &[&s], &tool()).is_empty());
}

#[test]
fn a_holder_band_over_the_boss_stands() {
    // A stubby stack: the holder (radius 16) starts 6 mm over the tip at
    // Z 28, under the boss's top 6 mm away - a crash the flute disc misses.
    let s = stepped();
    let stub = ToolGeom { flute_len: 3.0, shank_len: 3.0, ..tool() };
    assert_eq!(judged(&dive_at(0.0, 22.0, None), Mode::Area, &[&s], &stub), CONVICTED);
}

#[test]
fn the_part_reading_counts_too() {
    // A part standing to Z 25 under the dive, stock low: still convicted.
    let s = stepped();
    let part = cube(Point3::new(-3.0, -3.0, 0.0), Point3::new(3.0, 3.0, 25.0)).facets.to_vec();
    assert_eq!(judged(&dive_at(0.0, 22.0, None), Mode::Area, &[&s, &part], &tool()), CONVICTED);
}

#[test]
fn unknowns_fail_closed() {
    let s = stepped();
    // No facet under the flute: no proof.
    assert_eq!(judged(&dive_at(100.0, 22.0, None), Mode::Area, &[&s], &tool()), CONVICTED);
    // No solids at all.
    assert_eq!(judged(&dive_at(0.0, 22.0, None), Mode::Area, &[], &tool()), CONVICTED);
    // A non-finite vertex anywhere.
    let mut bad = s.clone();
    if let Some(t) = bad.get_mut(0) {
        t.a.z = f64::NAN;
    }
    assert_eq!(judged(&dive_at(0.0, 22.0, None), Mode::Area, &[&bad], &tool()), CONVICTED);
    // A stack band that is not a number.
    let nan = ToolGeom { holder_radius: f64::NAN, ..tool() };
    assert_eq!(judged(&dive_at(0.0, 22.0, None), Mode::Area, &[&s], &nan), CONVICTED);
}

#[test]
fn only_upright_column_dives_are_re_read() {
    let s = stepped();
    // A posed dive, and the radial / revolved / rotary modes, keep it.
    assert_eq!(judged(&dive_at(0.0, 22.0, Some([0.0, 0.6, 0.8])), Mode::Area, &[&s], &tool()), CONVICTED);
    for mode in [Mode::Radial, Mode::Revolve, Mode::Rotary] {
        assert_eq!(judged(&dive_at(0.0, 22.0, None), mode, &[&s], &tool()), CONVICTED);
    }
    // A lateral rapid is never re-read.
    let mut lateral = dive_at(0.0, 22.0, None);
    if let Some(m) = lateral.moves.get_mut(0) {
        m.to.x = 3.0;
    }
    assert_eq!(judged(&lateral, Mode::Area, &[&s], &tool()), CONVICTED);
}
