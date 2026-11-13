//! Unit tests of `step_guard` (#1536, #1568): the meter, the division it
//! wraps, the out-of-range flag and the refusal text.

use super::*;
use truck_stepio::r#in::alias::{BSplineSurface, KnotVec};

/// Degree 1 in u with the knot 1.0 repeated twice: the left half sits
/// at z = 0, the right half at z = 1, so the surface jumps at u = 1.
fn discontinuous() -> Surface {
    let uk = KnotVec::from(vec![0.0, 0.0, 1.0, 1.0, 2.0, 2.0]);
    let vk = KnotVec::bezier_knot(1);
    let p = |x: f64, y: f64, z: f64| Point3::new(x, y, z);
    let cps = vec![
        vec![p(0.0, 0.0, 0.0), p(0.0, 1.0, 0.0)],
        vec![p(1.0, 0.0, 0.0), p(1.0, 1.0, 0.0)],
        vec![p(1.0, 0.0, 1.0), p(1.0, 1.0, 1.0)],
        vec![p(2.0, 0.0, 1.0), p(2.0, 1.0, 1.0)],
    ];
    Surface::BSplineSurface(Box::new(BSplineSurface::new((uk, vk), cps)))
}

fn smooth() -> Surface {
    let k = KnotVec::bezier_knot(2);
    let p = |x: f64, y: f64, z: f64| Point3::new(x, y, z);
    let cps = vec![
        vec![p(0.0, 0.0, 0.0), p(0.0, 1.0, 0.5), p(0.0, 2.0, 0.0)],
        vec![p(1.0, 0.0, 0.5), p(1.0, 1.0, 2.0), p(1.0, 2.0, 0.5)],
        vec![p(2.0, 0.0, 0.0), p(2.0, 1.0, 0.5), p(2.0, 2.0, 0.0)],
    ];
    Surface::BSplineSurface(Box::new(BSplineSurface::new((k.clone(), k), cps)))
}

/// The heaviest healthy face measured (#1568: 10208.STEP's face #1073, at
/// the 60 M facet budget) converged at 28.07 M evaluations; the cap keeps
/// at least 5x that. The default-lane twin of the heavy-lane egg crate.
#[test]
fn the_cap_keeps_headroom_over_the_heaviest_measured_healthy_face() {
    const HEAVIEST_HEALTHY: u64 = 28_070_000;
    let cap = std::hint::black_box(FACE_EVAL_CAP);
    assert!(cap >= 5 * HEAVIEST_HEALTHY, "cap {cap}");
}

/// The chord tolerance at which the egg crate of
/// `a_smooth_face_needing_more_than_the_old_cap_is_divided_not_tripped`
/// needs 39.3 M evaluations (a 1206 x 1206 grid; 8 s in release).
const TOL_PAST_OLD_CAP: f64 = 6e-5;

#[test]
fn a_discontinuous_bspline_division_stops_at_the_cap_and_trips_the_meter() {
    let m = Metered::new(discontinuous(), 200_000);
    let started = std::time::Instant::now();
    let (u, v) = m.parameter_division(((0.0, 2.0), (0.0, 1.0)), 1e-3);
    assert!(m.meter.tripped(), "the meter must report the runaway face");
    assert_eq!((u, v), (vec![0.0, 2.0], vec![0.0, 1.0]), "a tripped face hands truck no grid to triangulate");
    assert!(started.elapsed().as_secs() < 10, "bounded: {:?}", started.elapsed());
}

#[test]
fn a_smooth_bspline_divides_exactly_as_truck_does_and_does_not_trip() {
    let s = smooth();
    let m = Metered::new(s.clone(), FACE_EVAL_CAP);
    let range = ((0.0, 1.0), (0.0, 1.0));
    for tol in [0.1, 1e-2, 1e-3] {
        m.meter.reset();
        assert_eq!(m.parameter_division(range, tol), s.parameter_division(range, tol), "tol {tol}");
        assert!(!m.meter.tripped());
    }
}

/// A smooth face whose division needs more evaluations than the old
/// 25 M cap (#1568: real trimmed NURBS faces reach 23-28 M) still
/// divides to the end at the cap the importer runs with.
#[test]
#[ignore = "heavy lane (#1568 cap headroom): divides a smooth patch through 39.3 M evaluations, ~8 s release / ~35 s debug - run at the cut with --release -- --ignored"]
fn a_smooth_face_needing_more_than_the_old_cap_is_divided_not_tripped() {
    // A 12 x 12 bicubic net of alternating heights: an egg crate.
    let n = 12;
    let knots = |n: usize| {
        let mut k = vec![0.0; 4];
        k.extend((1..n - 3).map(|i| i as f64));
        k.extend([(n - 3) as f64; 4]);
        KnotVec::from(k)
    };
    let cps = (0..n)
        .map(|i| (0..n).map(|j| Point3::new(i as f64, j as f64, if (i + j) % 2 == 0 { 1.0 } else { -1.0 })).collect())
        .collect();
    let s = Surface::BSplineSurface(Box::new(BSplineSurface::new((knots(n), knots(n)), cps)));
    let m = Metered::new(s, FACE_EVAL_CAP);
    let top = (n - 3) as f64;
    let (u, v) = m.parameter_division(((0.0, top), (0.0, top)), TOL_PAST_OLD_CAP);
    let used = m.meter.used.load(Ordering::Relaxed);
    assert!(used > 25_000_000, "the fixture must need more than the old cap: {used}");
    assert!(!m.meter.tripped(), "a face that converges at {used} evaluations was tripped");
    assert!(u.len() > 2 && v.len() > 2, "the division is the converged grid, not the trip's bare range");
}

/// #1568: sc_logo's face #836 was divided over u from -2e51 - a
/// trimming bound gone wild. A smooth patch asked outside its own range
/// is flagged, so its trip is never taken for a continuous face's.
#[test]
fn a_division_outside_the_surface_range_is_flagged() {
    // The flag is set before any evaluation; a cap of 1 keeps the
    // division over the wild range from running (it would run away).
    let m = Metered::new(smooth(), 1);
    let _ = m.parameter_division(((0.0, 1.0), (0.0, 1.0)), 0.1);
    assert!(!m.meter.outside.load(Ordering::Relaxed), "inside its own range");
    let _ = m.parameter_division(((-2e51, 1.0), (0.0, 1.0)), 0.1);
    assert!(m.meter.outside.load(Ordering::Relaxed), "the wild range must be flagged");
    m.meter.reset();
    assert!(!m.meter.outside.load(Ordering::Relaxed), "a new pass starts clean");
    assert!(within((-1e-9, 1.0 + 1e-9), (Bound::Included(0.0), Bound::Included(1.0))), "round-off is not wild");
}

/// No work outlives a refused pass: truck's rayon tasks are joined
/// before `robust_triangulation` returns, no thread is spawned, so the
/// runaway face's meter is still once the call is back.
#[test]
fn nothing_keeps_running_after_a_refused_pass() {
    use truck_meshalgo::prelude::RobustMeshableShape;
    let text = include_str!("../../../../../testdata/models/bspline_discontinuous.step");
    let exchange = truck_stepio::r#in::ruststep::parser::parse(text).expect("fixture parses");
    let table = truck_stepio::r#in::Table::from_data_section(&exchange.data[0]);
    let shell = table.shell.values().next().expect("one shell");
    let cshell = table.to_compressed_shell(shell).expect("converts");
    // A cap large enough that the pass takes a while: work left running
    // in the background would still be counting when we look.
    let g = Guarded::new(cshell, 2_000_000);
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| g.shell.robust_triangulation(0.01)));
    assert_eq!(g.tripped(), Some(Trip { index: 5, kind: "B-spline surface", continuous: false }));
    let spent = |g: &Guarded| g.meters.iter().map(|m| m.used.load(Ordering::Relaxed)).sum::<u64>();
    let before = spent(&g);
    std::thread::sleep(std::time::Duration::from_millis(300));
    assert_eq!(spent(&g), before, "a tessellation is still evaluating after the call returned");
}

#[test]
fn the_refusal_names_the_face_its_kind_and_the_cap() {
    let trip = |index, kind, continuous| Trip { index, kind, continuous };
    let e = refusal(trip(36, "NURBS surface", false), 62, Some(1234), Some(17), 99).to_string();
    assert!(e.contains("face #1234 (NURBS surface, face 37 of 62 in shell #17)"), "{e}");
    assert!(e.contains("did not converge within 99 evaluations"), "{e}");
    assert!(e.contains("discontinuous or degenerate"), "{e}");
    let e = refusal(trip(0, "B-spline surface", false), 1, None, None, 5).to_string();
    assert!(e.contains("face 1 of 1 (B-spline surface)"), "{e}");
    // A continuous face is never called broken.
    let e = refusal(trip(0, "NURBS surface", true), 1, None, None, 5).to_string();
    assert!(e.contains("too dense") && !e.contains("discontinuous"), "{e}");
}
