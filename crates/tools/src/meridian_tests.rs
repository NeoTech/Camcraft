//! #3367: the tool meridian - shapes from each mill kind, the outer and
//! inner bounds of a concave arc, and the named refusals.

use super::{Bound, Meridian, MeridianAtom, MeridianError};
use crate::spun::SpunPoint;
use crate::tools::{Tool, ToolKind};
use crate::undercut::cutter;

fn mill(kind: ToolKind, d: f64, corner_r: f64, tip_angle: f64, fl: f64) -> Tool {
    Tool {
        corner_r,
        ..cutter(kind, d, fl, d * 0.5, 30.0, tip_angle)
    }
}

fn arc(z: f64, r: f64, a: f64) -> SpunPoint {
    SpunPoint { z, r, arc: a }
}

/// The largest atom radius at `z` (the union's radius).
fn union_rho(atoms: &[MeridianAtom], z: f64) -> f64 {
    atoms
        .iter()
        .filter(|a| a.z0 - 1e-12 <= z && z <= a.z1 + 1e-12)
        .fold(0.0, |m, a| m.max(a.rho(z)))
}

#[test]
fn meridian_agrees_with_profile_on_the_cutting_band() {
    let tools = [
        mill(ToolKind::EndMill, 10.0, 0.0, 0.0, 20.0),
        mill(ToolKind::BallNose, 10.0, 5.0, 0.0, 20.0),
        mill(ToolKind::BallNose, 10.0, 5.0, 0.0, 3.0),
        mill(ToolKind::BullNose, 10.0, 2.0, 0.0, 20.0),
        mill(ToolKind::Drill, 8.0, 0.0, 118.0, 30.0),
        mill(ToolKind::Taper, 10.0, 0.0, 90.0, 20.0),
        mill(ToolKind::Taper, 10.0, 0.0, 30.0, 5.0),
        cutter(ToolKind::TSlot, 20.0, 5.0, 10.0, 10.0, 0.0),
        cutter(ToolKind::Keyseat, 25.0, 3.0, 10.0, 15.0, 0.0),
        cutter(ToolKind::Dovetail, 20.0, 5.0, 8.0, 10.0, 60.0),
        cutter(ToolKind::Lollipop, 10.0, 6.0, 4.0, 20.0, 0.0),
    ];
    for t in tools {
        let m = t.meridian().unwrap_or_else(|e| panic!("{}: {e}", t.kind));
        assert!(
            m.cutting().iter().all(|c| *c),
            "{}: every segment of a mill cuts",
            t.kind
        );
        let atoms = m
            .atoms(Bound::Outer, 1e-3)
            .unwrap_or_else(|e| panic!("{}: {e}", t.kind));
        let prof = t.profile();
        for &(z, _) in prof.iter().filter(|p| p.0 > 0.0 && p.0 <= t.flute_len) {
            let want = union_rho(&atoms, z);
            let hit = prof
                .iter()
                .any(|q| (q.0 - z).abs() <= 1e-12 && (q.1 - want).abs() <= 1e-9);
            assert!(
                hit,
                "{}: the meridian reads {want} at z {z}, the profile does not",
                t.kind
            );
        }
    }
}

/// A roundover-style meridian: a non-cutting bearing band, then a
/// concave quarter arc (radius 6, centre (4, 9.5)) that cuts.
fn roundover() -> Meridian {
    let pts = vec![
        SpunPoint::line(0.0, 0.0),
        SpunPoint::line(0.0, 3.5),
        arc(4.0, 3.5, -6.0),
        SpunPoint::line(10.0, 9.5),
    ];
    Meridian::new(pts, vec![false, false, true]).expect("a sound roundover")
}

fn arc_rho(z: f64) -> f64 {
    9.5 - (36.0 - (z - 4.0) * (z - 4.0)).max(0.0).sqrt()
}

/// The nearest and farthest distance from the arc centre (4, 9.5) to a
/// frustum atom's generator segment.
fn centre_distances(a: &MeridianAtom) -> (f64, f64) {
    let (p, q) = ((a.z0, a.rho(a.z0)), (a.z1, a.rho(a.z1)));
    let d = (q.0 - p.0, q.1 - p.1);
    let w = (4.0 - p.0, 9.5 - p.1);
    let t = ((w.0 * d.0 + w.1 * d.1) / (d.0 * d.0 + d.1 * d.1)).clamp(0.0, 1.0);
    let near = (w.0 - t * d.0).hypot(w.1 - t * d.1);
    let far = w.0.hypot(w.1).max((4.0 - q.0).hypot(9.5 - q.1));
    (near, far)
}

#[test]
fn concave_arc_outer_bound_contains_and_inner_bound_stays_inside() {
    let m = roundover();
    for lim in [0.1, 0.01, 0.001] {
        let outer = m.atoms(Bound::Outer, lim).expect("outer");
        let inner = m.atoms(Bound::Inner, lim).expect("inner");
        let (oa, ia): (Vec<_>, Vec<_>) = (
            outer.iter().filter(|a| a.cutting).copied().collect(),
            inner.iter().filter(|a| a.cutting).copied().collect(),
        );
        assert!(oa.len() >= 2 && ia.len() >= 2, "the arc splits");
        // The bearing band is the one non-cutting atom.
        assert_eq!(outer.iter().filter(|a| !a.cutting).count(), 1);
        assert_eq!(inner.iter().filter(|a| !a.cutting).count(), 1);
        for i in 0..=600 {
            let z = 4.0 + 6.0 * f64::from(i) / 600.0;
            let want = arc_rho(z);
            assert!(
                union_rho(&oa, z) >= want - 1e-12,
                "outer under the arc at z {z} (sag {lim})"
            );
            assert!(
                union_rho(&ia, z) <= want + 1e-12,
                "inner over the arc at z {z} (sag {lim})"
            );
        }
        for a in &oa {
            let (near, _) = centre_distances(a);
            assert!(
                6.0 - near <= lim + 1e-12,
                "a chord sags {} over {lim}",
                6.0 - near
            );
        }
        for a in &ia {
            let (_, far) = centre_distances(a);
            assert!(
                far - 6.0 <= lim + 1e-12,
                "a tangent stands off {} over {lim}",
                far - 6.0
            );
        }
    }
    let coarse = m.atoms(Bound::Outer, 0.1).expect("coarse").len();
    let fine = m.atoms(Bound::Outer, 0.001).expect("fine").len();
    assert!(
        fine > coarse,
        "a finer sag limit splits the arc into more atoms"
    );
}

#[test]
fn degenerate_meridians_are_refused_by_name() {
    let l = SpunPoint::line;
    let new = |p: Vec<SpunPoint>| {
        let n = p.len().saturating_sub(1);
        Meridian::new(p, vec![true; n])
    };
    assert_eq!(new(vec![l(0.0, 0.0)]), Err(MeridianError::TooFew));
    assert_eq!(
        new(vec![l(0.0, 0.0), l(0.0, 0.0), l(5.0, 1.0)]),
        Err(MeridianError::Short(1))
    );
    assert_eq!(
        new(vec![l(0.0, 0.0), l(0.0, -1.0), l(5.0, 1.0)]),
        Err(MeridianError::Negative(2))
    );
    assert_eq!(
        new(vec![l(0.0, 0.0), l(f64::NAN, 1.0)]),
        Err(MeridianError::NonFinite(2))
    );
    assert_eq!(
        new(vec![l(0.0, 0.0), arc(0.0, 1.0, f64::INFINITY), l(5.0, 1.0)]),
        Err(MeridianError::NonFinite(2))
    );
    assert_eq!(
        new(vec![l(0.0, 0.0), arc(0.0, 1.0, 1.0), l(5.0, 1.0)]),
        Err(MeridianError::ArcTooTight(2))
    );
    assert_eq!(
        new(vec![l(0.0, 0.0), l(5.0, 1.0), l(4.0, 1.0)]),
        Err(MeridianError::RunsBack(2))
    );
    // A convex arc over a flat chord bulges below the tip.
    assert_eq!(
        new(vec![arc(0.0, 0.0, 3.0), l(0.0, 5.0)]),
        Err(MeridianError::ArcFolds(1))
    );
    // A concave arc that dips through the axis.
    assert_eq!(
        new(vec![arc(0.0, 1.0, -5.0), l(8.0, 1.0)]),
        Err(MeridianError::Negative(1))
    );
    assert_eq!(
        Meridian::new(vec![l(0.0, 0.0), l(5.0, 1.0)], vec![]),
        Err(MeridianError::FlagCount)
    );
    let m = roundover();
    assert_eq!(m.atoms(Bound::Outer, 0.0), Err(MeridianError::BadSag));
    assert_eq!(m.atoms(Bound::Inner, f64::NAN), Err(MeridianError::BadSag));
    assert_eq!(
        m.atoms(Bound::Outer, 1e-12),
        Err(MeridianError::SagTooFine(3))
    );
}

#[test]
fn tool_meridian_refuses_bad_tools_by_name() {
    let tap = mill(ToolKind::Tap, 6.0, 0.0, 0.0, 20.0);
    assert_eq!(tap.meridian(), Err(MeridianError::NotAMill(ToolKind::Tap)));
    let nan = mill(ToolKind::EndMill, f64::NAN, 0.0, 0.0, 20.0);
    assert!(matches!(nan.meridian(), Err(MeridianError::NonFinite(_))));
    let neg = mill(ToolKind::EndMill, -6.0, 0.0, 0.0, 20.0);
    assert!(matches!(neg.meridian(), Err(MeridianError::Negative(_))));
    let flat = mill(ToolKind::EndMill, 0.0, 0.0, 0.0, 20.0);
    assert!(matches!(flat.meridian(), Err(MeridianError::Short(_))));
    let fat = mill(ToolKind::BullNose, 10.0, 6.0, 0.0, 20.0);
    assert!(matches!(fat.meridian(), Err(MeridianError::Unsound(_))));
    let narrow = cutter(ToolKind::Dovetail, 20.0, 5.0, 19.0, 10.0, 60.0);
    assert!(matches!(narrow.meridian(), Err(MeridianError::Unsound(_))));
}

#[test]
fn tool_meridian_checks_inputs_before_shape_fallbacks() {
    for kind in [ToolKind::Drill, ToolKind::Taper] {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for t in [
                mill(kind, bad, 0.0, 118.0, 20.0),
                mill(kind, 10.0, 0.0, 118.0, bad),
                mill(kind, 10.0, 0.0, bad, 20.0),
            ] {
                assert!(
                    matches!(t.meridian(), Err(MeridianError::NonFinite(_))),
                    "{t:?}"
                );
            }
        }
    }
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let t = mill(ToolKind::BullNose, 10.0, bad, 0.0, 20.0);
        assert!(matches!(t.meridian(), Err(MeridianError::NonFinite(_))));
    }
    let t = mill(ToolKind::BullNose, 10.0, -1.0, 0.0, 20.0);
    assert!(matches!(t.meridian(), Err(MeridianError::Negative(_))));
}

#[test]
fn accepted_extreme_meridians_have_finite_correct_radii() {
    let m = Meridian::new(
        vec![SpunPoint::line(0.0, 1.0), SpunPoint::line(1e308, 10.0)],
        vec![true],
    )
    .expect("the segment length is finite");
    let atoms = m.atoms(Bound::Outer, 0.001).expect("line atoms");
    for (z, want) in [(0.0, 1.0), (2.5e307, 3.25), (5e307, 5.5), (1e308, 10.0)] {
        let rho = atoms[0].rho(z);
        assert!(rho.is_finite());
        assert_eq!(rho, want);
    }
    let rad = super::MAX_ARC_COORD;
    let m = Meridian::new(
        vec![arc(0.0, 0.0, rad), SpunPoint::line(rad, rad)],
        vec![true],
    )
    .expect("the circle arithmetic stays in range");
    let atoms = m.atoms(Bound::Outer, 0.001).expect("circle atom");
    for (z, want) in [(0.0, 0.0), (rad * 0.5, 0.75_f64.sqrt()), (rad, 1.0)] {
        let rho = atoms[0].rho(z);
        assert!(rho.is_finite());
        assert!((rho / rad - want).abs() < 1e-12);
    }
}

#[test]
fn meridian_refuses_overflowing_lengths_and_circle_ranges() {
    let l = SpunPoint::line;
    for pts in [
        vec![l(-1e308, 1.0), l(1e308, 2.0)],
        vec![l(0.0, 0.0), l(1.3e308, 1.3e308)],
        vec![arc(0.0, 0.0, 1e200), l(1e200, 1e200)],
        vec![arc(1e200, 1.0, 1.0), l(1e200, 2.0)],
    ] {
        assert_eq!(
            Meridian::new(pts, vec![true]),
            Err(MeridianError::OutOfRange(1))
        );
    }
    let ball = mill(ToolKind::BallNose, 1e200, 0.0, 0.0, 1e199);
    assert_eq!(ball.meridian(), Err(MeridianError::OutOfRange(1)));
}
