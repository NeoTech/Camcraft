//! Unit tests of `step_nearest` (#1602): a boundary point's parameters are
//! found on its own turn of a many-turn strip, inside the knot domain.

use super::*;
use crate::geometry::import::step_thread_fixture::{flank, root};
use truck_stepio::r#in::alias::{BSplineSurface, KnotVec, ParametricSurface3D, SPHint2D};

fn on_surface_inside(s: &Surface, p: Point3, uv: Option<(f64, f64)>) -> bool {
    let Some(domain) = super::super::step_pole::knot_domain(s) else { return false };
    uv.is_some_and(|uv| inside(uv, domain) && s.subs(uv.0, uv.1).distance(p) <= SAME_POINT)
}

/// The first boundary point of a 14-turn flank, searched with no hint:
/// truck's 51 x 51 presearch lands on the neighbouring turn and Newton
/// settles off the surface (the #1602 hole). The rescue finds it.
#[test]
fn a_point_on_a_many_turn_flank_is_found_on_its_own_turn() {
    let s = flank(14);
    let mut fixed = 0;
    for k in 0..40 {
        let v = (k as f64 + 0.37) / 40.0;
        let p = s.subs(0.0, v);
        let truck = s.search_nearest_parameter(p, SPHint2D::None, TRIALS);
        let found = rescue(&s, p, truck);
        assert!(on_surface_inside(&s, p, found), "v {v}: truck {truck:?} -> {found:?}");
        fixed += usize::from(!on_surface_inside(&s, p, truck));
    }
    assert!(fixed > 0, "the fixture must reproduce truck's miss on some point");
}

/// A root band's straight rulings, extrapolated past u = 1, meet its other
/// turns: truck can find the point exactly at a u outside [0, 1]. The
/// answer must come back inside the domain, on the point's own turn.
#[test]
fn a_point_on_a_root_band_is_found_inside_the_knot_domain() {
    let s = root(14);
    let mut fixed = 0;
    for k in 0..40 {
        let v = (k as f64 + 0.37) / 40.0;
        let p = s.subs(1.0, v);
        // The hint a mesher would carry from a neighbouring turn.
        let hint = (0.5, (v - 1.0 / 14.0).max(0.0));
        let truck = s.search_nearest_parameter(p, hint, TRIALS);
        let found = rescue(&s, p, truck);
        assert!(on_surface_inside(&s, p, found), "v {v}: truck {truck:?} -> {found:?}");
        fixed += usize::from(!on_surface_inside(&s, p, truck));
    }
    assert!(fixed > 0, "the fixture must reproduce truck's out-of-domain answer on some point");
}

/// A point the face really does not reach keeps truck's answer: on a flat
/// unit square, a point past its edge is found on the extended plane,
/// which no parameter inside the domain matches.
#[test]
fn a_point_off_the_face_keeps_trucks_answer() {
    let k = KnotVec::bezier_knot(1);
    let p = |x: f64, y: f64| Point3::new(x, y, 0.0);
    let cps = vec![vec![p(0.0, 0.0), p(0.0, 1.0)], vec![p(1.0, 0.0), p(1.0, 1.0)]];
    let s = Surface::BSplineSurface(Box::new(BSplineSurface::new((k.clone(), k), cps)));
    let off = Point3::new(1.5, 0.5, 0.0);
    let truck = s.search_nearest_parameter(off, (0.9, 0.5), TRIALS);
    assert!(truck.is_some_and(|(u, _)| u > 1.4), "truck extends the plane: {truck:?}");
    assert_eq!(rescue(&s, off, truck), truck);
    // An answer already inside and on the surface is returned untouched.
    let on = Point3::new(0.25, 0.75, 0.0);
    assert_eq!(rescue(&s, on, Some((0.25, 0.75))), Some((0.25, 0.75)));
}

/// The sample grid has a few samples in every knot span and stays inside
/// the evaluation bound however many spans a face has.
#[test]
fn the_sample_grid_covers_every_span_within_its_bound() {
    let s = flank(14);
    let Some(domain) = super::super::step_pole::knot_domain(&s) else { panic!("a NURBS has a knot domain") };
    let (us, vs) = samples(&s, domain);
    assert!(us.len() * vs.len() <= MAX_SAMPLES, "{} x {}", us.len(), vs.len());
    assert!(vs.len() > 140, "every one of the 140 spans is sampled: {}", vs.len());
    assert_eq!((us.first(), us.last()), (Some(&0.0), Some(&1.0)));
}

/// Whatever the samples' Newton runs find, the rescue never answers
/// outside the knot domain: on a flat square, a point on the plane past
/// its edge draws every run there, and truck's own (bogus) answer stands.
#[test]
fn a_rescue_never_answers_outside_the_knot_domain() {
    let k = KnotVec::bezier_knot(1);
    let p = |x: f64, y: f64| Point3::new(x, y, 0.0);
    let cps = vec![vec![p(0.0, 0.0), p(0.0, 1.0)], vec![p(1.0, 0.0), p(1.0, 1.0)]];
    let s = Surface::BSplineSurface(Box::new(BSplineSurface::new((k.clone(), k), cps)));
    let bogus = Some((5.0, 0.5));
    assert_eq!(rescue(&s, Point3::new(1.5, 0.5, 0.0), bogus), bogus);
}

/// A root band's point found exactly on an extrapolated ruling (u = 19.5,
/// one pitch down at the same angle) is replaced by its own in-domain
/// parameter, although that one is a rounding error farther away.
#[test]
fn an_exact_answer_outside_the_domain_gives_way_to_one_on_the_surface_inside() {
    let s = root(14);
    let v = 0.5;
    let outside = (19.5, v + 1.0 / 14.0);
    let p = s.subs(outside.0, outside.1);
    let found = rescue(&s, p, Some(outside));
    assert!(on_surface_inside(&s, p, found), "{found:?}");
}

/// A boundary point a little off its face (a file's own tolerance), whose
/// first answer is a turn away, takes the nearer answer on its own turn
/// although that one is not within the acceptance distance either.
#[test]
fn a_nearer_answer_replaces_a_far_one_off_the_surface() {
    let s = flank(14);
    let (u, v) = (0.0, 0.437);
    let p = s.subs(u, v) + s.normal(u, v) * 1e-4;
    let wrong_turn = Some((0.0, v - 1.0 / 14.0));
    let found = rescue(&s, p, wrong_turn).expect("an answer");
    assert!((found.1 - v).abs() < 1e-3, "{found:?}");
}
