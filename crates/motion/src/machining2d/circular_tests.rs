//! Unit pins of the Circular kernel (split from `circular.rs`, #2898):
//! the vault ladder, the ring chord count and the ring walk.

use super::*;

/// #2004: the rim ring and a flat disk start a stepdown under Top Z;
/// an inner vault ring keeps the previous ring's floor as its first
/// level (its new strip opens there), the same ladder as before.
#[test]
fn an_inner_vault_ring_keeps_its_opening_plane() {
    assert_eq!(levels_from(8.0, 0.0, 5.0).ok(), Some(vec![8.0, 3.0, 0.0]));
    assert_eq!(levels_from(8.0, 0.0, 50.0).ok(), Some(vec![8.0, 0.0]));
    assert_eq!(levels_from(2.0, 2.0, 5.0).ok(), Some(vec![2.0]));
    assert!(levels_from(8.0, 0.0, 0.0).is_err());
    assert_eq!(cut_levels(8.0, 0.0, 5.0).ok(), Some(vec![3.0, 0.0]));
    // The rim ring's floor at Top Z and an opening plane at Top Z go.
    assert_eq!(under_top(vec![13.0], 13.0), Vec::<f64>::new());
    assert_eq!(under_top(vec![13.0, 8.32], 13.0), vec![8.32]);
    assert_eq!(under_top(vec![8.0, 3.0], 13.0), vec![8.0, 3.0]);
}

/// #1558: the polygon's chord count comes from the tolerance, not a
/// fixed 512 - its sagitta stays within it, and it is the fewest
/// power of two that does (capped at the cutter-center disk's 512).
#[test]
fn ring_chords_are_sized_by_the_tolerance() {
    for rho in [0.05, 0.5, 1.0, 3.0, 25.0, 100.0] {
        let n = ring_chords(rho, ARC_FIT_TOL);
        let sag = |n: usize| rho * (1.0 - (PI / n as f64).cos());
        assert!(sag(n) <= ARC_FIT_TOL, "rho {rho}: {n} chords sag {}", sag(n));
        assert!(n == 8 || sag(n / 2) > ARC_FIT_TOL, "rho {rho}: {n} is not the fewest");
        assert!(n.is_power_of_two() && RING_MAX_CHORDS.is_multiple_of(n));
        let ring = ring_loop(Point2::new(0.0, 0.0), rho, true);
        assert_eq!(ring.len(), n + 1, "closed");
    }
    assert_eq!(ring_chords(1.0, ARC_FIT_TOL), 32, "R1 (the issue's ring): 32, not 512");
    assert_eq!(ring_chords(f64::NAN, ARC_FIT_TOL), RING_MAX_CHORDS);
}

/// #1558 review: ring ramps are the circular kernel's opt-in; every
/// other ladder starts without it (and `push_zig_ring` re-proves
/// the ring itself, so a wrong opt-in still posts chords).
#[test]
fn ring_ramps_are_opt_in() {
    assert!(!Motion::new(5.0, None, 100.0, 50.0).ring_zig);
    assert!(Motion::new(5.0, None, 100.0, 50.0).with_ring_zig().ring_zig);
}

/// #1558 review: a ring narrower than MIN_ARC_CHORD walks its
/// polygon, never two half-turn arcs a post could round into full
/// circles; a wider one walks the arcs.
#[test]
fn a_sub_resolution_ring_walks_its_polygon() {
    let c = Point2::new(1.0, 2.0);
    for (rho, arcs) in [(0.004, false), (0.0005, false), (0.006, true), (1.0, true)] {
        let walk = ring_walk(c, &ring_loop(c, rho, true), true);
        assert_eq!(walk.iter().all(|s| matches!(s, Seg::Arc { .. })), arcs, "rho {rho}");
    }
}
