//! Unit tests of `step_division` (#1602): a direction is split only where
//! it is curved, a twist still splits both, and every cell of the grid
//! meets truck's own bilinear test.

use super::*;
use crate::geometry::import::step_thread_fixture::flank;
use truck_stepio::r#in::alias::{algo, BSplineSurface, KnotVec, Surface};

/// Every cell of `(udiv, vdiv)` stands within `tol` of `s` at truck's
/// jittered interior point (the acceptance test of truck's division).
fn assert_cells_within(s: &Surface, (udiv, vdiv): (&[f64], &[f64]), tol: f64) {
    for u in udiv.windows(2) {
        for v in vdiv.windows(2) {
            let centre = s.subs((u[0] + u[1]) / 2.0, (v[0] + v[1]) / 2.0);
            let t = 0.5 + (0.2 * HashGen::hash1(centre) - 0.1);
            let (pu, pv) = (u[0] + (u[1] - u[0]) * t, v[0] + (v[1] - v[0]) * t);
            let c = |a: f64, b: f64| s.subs(a, b).to_vec();
            let bilinear = c(u[0], v[0]) * (1.0 - t) * (1.0 - t)
                + c(u[0], v[1]) * (1.0 - t) * t
                + c(u[1], v[0]) * t * (1.0 - t)
                + c(u[1], v[1]) * t * t;
            let off = s.subs(pu, pv).distance(Point3::from_vec(bilinear));
            assert!(off <= tol, "cell u {u:?} v {v:?} stands {off} off at tol {tol}");
        }
    }
}

/// The #1602 runaway: a thread flank is straight across u, so u needs no
/// split - truck's division split it every sweep (2^sweeps intervals,
/// 2 M facets on the real face). Ours leaves u whole and meets the
/// tolerance along v.
#[test]
fn a_strip_curved_only_along_v_is_not_split_across_u() {
    let s = flank(3);
    let range = ((0.0, 1.0), (0.0, 1.0));
    let tol = 0.005;
    let (u, v) = parameter_division(&s, range, tol).expect("converges");
    assert_eq!(u, vec![0.0, 1.0], "a straight direction must stay whole");
    assert!(v.len() > 100, "three turns at 5 um need hundreds of steps along v: {}", v.len());
    assert_cells_within(&s, (&u, &v), tol);
    let (tu, _) = algo::surface::parameter_division(&s, range, tol);
    assert!(tu.len() > 64, "truck's isotropic split is the runaway this replaces: {}", tu.len());
}

/// A doubly curved patch still splits both ways and meets the test.
#[test]
fn a_doubly_curved_patch_splits_both_ways() {
    let k = KnotVec::bezier_knot(2);
    let p = |x: f64, y: f64, z: f64| Point3::new(x, y, z);
    let cps = vec![
        vec![p(0.0, 0.0, 0.0), p(0.0, 1.0, 0.5), p(0.0, 2.0, 0.0)],
        vec![p(1.0, 0.0, 0.5), p(1.0, 1.0, 2.0), p(1.0, 2.0, 0.5)],
        vec![p(2.0, 0.0, 0.0), p(2.0, 1.0, 0.5), p(2.0, 2.0, 0.0)],
    ];
    let s = Surface::BSplineSurface(Box::new(BSplineSurface::new((k.clone(), k), cps)));
    for tol in [0.1, 1e-2, 1e-3] {
        let (u, v) = parameter_division(&s, ((0.0, 1.0), (0.0, 1.0)), tol).expect("converges");
        assert!(u.len() > 2 && v.len() > 2, "tol {tol}: {} x {}", u.len(), v.len());
        assert_cells_within(&s, (&u, &v), tol);
    }
}

/// A cell whose iso-curves through the probe are within the tolerance of
/// their chords but whose corners are not (a twist) splits both ways:
/// z = c (1 - phi(u) phi(v)), phi(x) = 4 (x - 1/2)^2, is c along both
/// mid-lines and 0 at the corners.
#[test]
fn a_twist_the_chords_miss_still_splits_both_ways() {
    let c = 1.0;
    // phi as a quadratic Bezier on [0, 1]: values 1, 0, 1 need control
    // values 1, -1, 1; f = c - c phi(u) phi(v) is their tensor product.
    let phi = [1.0, -1.0, 1.0];
    let k = KnotVec::bezier_knot(2);
    let cps = (0..3)
        .map(|i| {
            (0..3)
                .map(|j| Point3::new(i as f64 / 2.0, j as f64 / 2.0, c - c * phi[i] * phi[j]))
                .collect()
        })
        .collect();
    let s = Surface::BSplineSurface(Box::new(BSplineSurface::new((k.clone(), k), cps)));
    // The chords stand at most 0.04 c off at the jittered probe; the
    // corners' bilinear patch stands ~c off.
    let (u, v) = parameter_division(&s, ((0.0, 1.0), (0.0, 1.0)), 0.5 * c).expect("converges");
    assert!(u.len() > 2 && v.len() > 2, "a twist must split both ways: {} x {}", u.len(), v.len());
}

/// A tolerance that is not positive and finite divides nothing. Without
/// the guard a zero tolerance finds every cell far and doubles both
/// directions each sweep - 2^128 cells - so the call runs on a thread and
/// a hang fails the test instead of the suite.
#[test]
fn a_bad_tolerance_returns_the_corners() {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let s = flank(1);
        let got: Vec<_> = [0.0, -1.0, f64::NAN, f64::INFINITY]
            .into_iter()
            .map(|tol| parameter_division(&s, ((0.0, 1.0), (0.0, 1.0)), tol))
            .collect();
        let _ = tx.send(got);
    });
    let got = rx.recv_timeout(std::time::Duration::from_secs(20)).expect("a bad tolerance must return at once");
    for division in got {
        assert_eq!(division, Some((vec![0.0, 1.0], vec![0.0, 1.0])));
    }
}

/// A surface that jumps (an interior knot repeated past its degree) never
/// converges: the division says so instead of returning a grid that
/// smears the jump into a wall. Split only across the jump, it grows one
/// column a sweep, far too slowly for the evaluation meter to stop it.
#[test]
fn a_jump_is_reported_as_not_converged() {
    let uk = KnotVec::from(vec![0.0, 0.0, 1.0, 1.0, 2.0, 2.0]);
    let vk = KnotVec::bezier_knot(1);
    let p = |x: f64, y: f64, z: f64| Point3::new(x, y, z);
    let cps = vec![
        vec![p(0.0, 0.0, 0.0), p(0.0, 1.0, 0.0)],
        vec![p(1.0, 0.0, 0.0), p(1.0, 1.0, 0.0)],
        vec![p(1.0, 0.0, 1.0), p(1.0, 1.0, 1.0)],
        vec![p(2.0, 0.0, 1.0), p(2.0, 1.0, 1.0)],
    ];
    let s = Surface::BSplineSurface(Box::new(BSplineSurface::new((uk, vk), cps)));
    let started = std::time::Instant::now();
    assert_eq!(parameter_division(&s, ((0.0, 2.0), (0.0, 1.0)), 1e-3), None);
    assert!(started.elapsed().as_secs() < 5, "bounded: {:?}", started.elapsed());
}
