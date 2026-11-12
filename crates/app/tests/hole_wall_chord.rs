//! The containment kinds' hole-wall excuse credits tessellation, not
//! flats (#1217, Prime Directive): a hole that really is a coarse
//! polygon (an octagon, a 16-gon, corners on r 5) cut out to its corner
//! circle by a bore orbit or a thread-mill helix declared at another
//! radius (r 4.4, off the corners, as the issue's thread was) is a
//! Gouge on its flats - only a chord of a declared rim is excused its
//! whole dip, any other at most the mesh tolerance. A full-fidelity
//! round hole (a 96-gon) stays clean for every containment kind with
//! or without its rim, and a coarse round hole (a 32-gon) stays clean
//! when its rim is the one declared. A polygon declared at its own
//! corner circle (#1231) is still a polygon: a chord dipping past a
//! tessellation's depth is a flat and convicts, so a drill, bore or
//! thread of the corner-circle size in an octagon is a Gouge.
//!
//! Provenance: our own fixtures, specced from the Prime Directive and
//! the chord sagitta s = r - sqrt(r^2 - (l/2)^2); no vendor material.
//!
//! Gate map (solid law): covers sim::verify (solid_hits_rims,
//! swept::chord_dip, solid_index hole_walls).

mod common;

use common::solid_part::{circle, mill, plate, rect};

use camcraft_app::cl::{Move, MoveKind, Toolpath};
use camcraft_app::geo2d::Point2;
use camcraft_app::geometry::{Point3, Triangle};
use camcraft_app::verify::{Cutter, HoleRim, SolidLaw, Warning, solid_hits_rims};

const C: (f64, f64) = (20.0, 20.0);
const R: f64 = 5.0;

/// A 40 x 40 plate, z -10..0, with a blind n-gon hole (corners on r 5)
/// down to z -6.
fn part(n: usize) -> Vec<Triangle> {
    let hole = circle(C, R, n);
    let mut f = Vec::new();
    plate(&rect(0.0, 0.0, 40.0, 40.0), Some((&hole, -6.0)), -10.0, 0.0, &mut f);
    f
}

fn p(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

/// The containment law as the operation layer runs it, with the op's
/// declared rims at the hole's centre.
fn contain(t: &Toolpath, part: &[Triangle], d: f64, rims: &[f64]) -> Vec<Warning> {
    let law = SolidLaw { step: 0.25, allow: 0.01, hole_walls: true };
    let rims: Vec<HoleRim> = rims.iter().map(|&r| HoleRim { x: C.0, y: C.1, r }).collect();
    solid_hits_rims(t, part, &Cutter::flat(&mill(d)), &law, &rims)
}

/// A drill of the corner circle's diameter plunged at the centre.
fn drill() -> Toolpath {
    Toolpath::new(vec![
        Move::new(MoveKind::Rapid, p(C.0, C.1, 5.0), p(C.0, C.1, 1.0), None),
        Move::new(MoveKind::Plunge, p(C.0, C.1, 1.0), p(C.0, C.1, -3.0), Some(100.0)),
        Move::new(MoveKind::Rapid, p(C.0, C.1, -3.0), p(C.0, C.1, 5.0), None),
    ])
}

/// A 6 mm cutter orbiting r 2 (reach r 5): flat at z -3 (a bore) or
/// falling 1 mm a turn (a thread mill's helix).
fn orbit(pitch: f64) -> Toolpath {
    let (x0, x1) = (C.0 + 2.0, C.0 - 2.0);
    let c = Point2::new(C.0, C.1);
    let z = -2.0;
    let half = 0.5 * pitch;
    Toolpath::new(vec![
        Move::new(MoveKind::Rapid, p(x0, C.1, 5.0), p(x0, C.1, 1.0), None),
        Move::new(MoveKind::Plunge, p(x0, C.1, 1.0), p(x0, C.1, z), Some(100.0)),
        Move::new(MoveKind::Feed, p(x0, C.1, z), p(x1, C.1, z - half), Some(300.0)).with_arc(c, true),
        Move::new(MoveKind::Feed, p(x1, C.1, z - half), p(x0, C.1, z - pitch), Some(300.0)).with_arc(c, true),
        Move::new(MoveKind::Rapid, p(x0, C.1, z - pitch), p(x0, C.1, 5.0), None),
    ])
}

fn gouged(w: &[Warning]) -> bool {
    w.iter().any(|w| matches!(w, Warning::Gouge { .. }))
}

#[test]
fn a_coarse_polygon_holes_flats_cut_by_a_containment_kind_convict() {
    for n in [8, 16] {
        let f = part(n);
        // Declared off the corner circle (the issue's thread, r 4.4), or
        // with no rim at all: the flats are judged.
        for rims in [&[4.4][..], &[]] {
            assert!(gouged(&contain(&orbit(0.0), &f, 6.0, rims)), "{n}-gon {rims:?}: bore cut its flats silently");
            assert!(gouged(&contain(&orbit(1.0), &f, 6.0, rims)), "{n}-gon {rims:?}: thread helix cut its flats silently");
        }
        assert!(gouged(&contain(&drill(), &f, 2.0 * R, &[])), "{n}-gon: drill cut its flats silently");
    }
}

#[test]
fn a_full_fidelity_round_hole_stays_clean_for_every_containment_kind() {
    let f = part(96);
    for rims in [&[R][..], &[]] {
        assert_eq!(contain(&drill(), &f, 2.0 * R, rims), Vec::new(), "drill {rims:?}");
        assert_eq!(contain(&orbit(0.0), &f, 6.0, rims), Vec::new(), "bore {rims:?}");
        assert_eq!(contain(&orbit(1.0), &f, 6.0, rims), Vec::new(), "thread helix {rims:?}");
    }
}

#[test]
fn a_coarse_round_hole_is_excused_only_on_its_declared_rim() {
    // A 32-gon at r 5 dips 0.024 mm: past the mesh tolerance, so only
    // its declared rim excuses it - a matched drill and bore stay clean.
    let f = part(32);
    assert_eq!(contain(&drill(), &f, 2.0 * R, &[R]), Vec::new(), "drill on its rim");
    assert_eq!(contain(&orbit(0.0), &f, 6.0, &[R]), Vec::new(), "bore on its rim");
    assert!(gouged(&contain(&drill(), &f, 2.0 * R, &[4.4])), "drill off its rim");
}

#[test]
fn a_polygon_declared_at_its_corner_circle_still_convicts_on_its_flats() {
    // #1231: the octagon's corners lie on the declared r 5 rim, but its
    // flats dip 0.38 mm (a 16-gon's 0.096 mm) - flats, not tessellation.
    for n in [8, 16] {
        let f = part(n);
        assert!(gouged(&contain(&drill(), &f, 2.0 * R, &[R])), "{n}-gon: corner-circle drill passed");
        assert!(gouged(&contain(&orbit(0.0), &f, 6.0, &[R])), "{n}-gon: corner-circle bore passed");
        assert!(gouged(&contain(&orbit(1.0), &f, 6.0, &[R])), "{n}-gon: corner-circle thread passed");
    }
}

#[test]
fn a_small_tessellated_round_hole_keeps_its_rim_credit() {
    // The operations fixture's 16-gon r 2 hole dips 0.038 mm: a round
    // hole's tessellation, so a matched 4 mm drill on its rim is clean.
    let hole = circle(C, 2.0, 16);
    let mut f = Vec::new();
    plate(&rect(0.0, 0.0, 40.0, 40.0), Some((&hole, -6.0)), -10.0, 0.0, &mut f);
    assert_eq!(contain(&drill(), &f, 4.0, &[2.0]), Vec::new());
    assert!(gouged(&contain(&drill(), &f, 4.0, &[])), "no rim, no full credit");
}
