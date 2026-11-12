//! The containment kinds' hole-wall excuse credits tessellation, not
//! flats (#1217, Prime Directive): a hole that really is a coarse
//! polygon (an octagon, a 16-gon, corners on r 5) cut out to its corner
//! circle by a drill plunge, a bore orbit or a thread-mill helix is a
//! Gouge on its flats; the same cuts in a full-fidelity round hole (a
//! 96-gon, dip under the mesh tolerance) stay clean.
//!
//! Provenance: our own fixtures, specced from the Prime Directive and
//! the chord sagitta s = r - sqrt(r^2 - (l/2)^2); no vendor material.
//!
//! Gate map (solid law): covers sim::verify (solid_hits, swept::chord_dip,
//! solid_index hole_walls).

mod common;

use common::solid_part::{circle, mill, plate, rect};

use camcraft_app::cl::{Move, MoveKind, Toolpath};
use camcraft_app::geo2d::Point2;
use camcraft_app::geometry::{Point3, Triangle};
use camcraft_app::verify::{Cutter, SolidLaw, Warning, solid_hits};

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

/// The containment law as the operation layer runs it.
fn contain(t: &Toolpath, part: &[Triangle], d: f64) -> Vec<Warning> {
    let law = SolidLaw { step: 0.25, allow: 0.01, hole_walls: true };
    solid_hits(t, part, &Cutter::flat(&mill(d)), &law)
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
        assert!(gouged(&contain(&drill(), &f, 2.0 * R)), "{n}-gon: drill cut its flats silently");
        assert!(gouged(&contain(&orbit(0.0), &f, 6.0)), "{n}-gon: bore cut its flats silently");
        assert!(gouged(&contain(&orbit(1.0), &f, 6.0)), "{n}-gon: thread helix cut its flats silently");
    }
}

#[test]
fn a_full_fidelity_round_hole_stays_clean_for_every_containment_kind() {
    let f = part(96);
    assert_eq!(contain(&drill(), &f, 2.0 * R), Vec::new(), "drill");
    assert_eq!(contain(&orbit(0.0), &f, 6.0), Vec::new(), "bore");
    assert_eq!(contain(&orbit(1.0), &f, 6.0), Vec::new(), "thread helix");
}
