//! Finding 1 of the #4123 review: the order stage may only permute the
//! curves, restart a closed curve and reverse a curve (law 3). Any other
//! change of a curve - another vertex order, a moved point, a changed
//! circle, index or family - is refused.

use super::common::{curve, pts};
use camcraft_pathkernel::Point2;
use camcraft_pathkernel::guide::{Circle, Curve, Family, GuideSet};
use camcraft_pathkernel::order::{BAD_START, NOT_A_PERMUTATION, OrderedSet, START_OFF_CURVE, StartSide, UNRESOLVED};

const SQ: [(f64, f64); 4] = [(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0)];

fn ring(xy: &[(f64, f64)]) -> Curve {
    curve(xy, true, 0, Family::Ring)
}

fn order(input: &[Curve], out: Vec<Curve>) -> Result<OrderedSet, &'static str> {
    let g = GuideSet::new(input.to_vec());
    let start = out.first().map_or(Point2::new(0.0, 0.0), |c| c.pts()[0].at());
    OrderedSet::new(&g, out, start, StartSide::Inside).map_err(|e| e.reason())
}

#[test]
fn a_restart_or_a_reversal_of_a_closed_curve_is_accepted() {
    let a = ring(&SQ);
    for shift in 0..4 {
        let mut fwd = SQ.to_vec();
        fwd.rotate_left(shift);
        let mut rev = fwd.clone();
        rev.reverse();
        assert!(order(std::slice::from_ref(&a), vec![ring(&fwd)]).is_ok(), "shift {shift}");
        assert!(order(std::slice::from_ref(&a), vec![ring(&rev)]).is_ok(), "reversed shift {shift}");
    }
}

#[test]
fn another_vertex_order_of_a_closed_curve_is_refused() {
    // The same four vertices, crossing: (0,0) (4,4) (4,0) (0,4) is a bow tie, not the square.
    let crossing = ring(&[(0.0, 0.0), (4.0, 4.0), (4.0, 0.0), (0.0, 4.0)]);
    assert_eq!(order(&[ring(&SQ)], vec![crossing]).unwrap_err(), NOT_A_PERMUTATION);
    let swapped = ring(&[(0.0, 0.0), (4.0, 0.0), (0.0, 4.0), (4.0, 4.0)]);
    assert_eq!(order(&[ring(&SQ)], vec![swapped]).unwrap_err(), NOT_A_PERMUTATION);
}

#[test]
fn an_open_curve_may_only_be_reversed() {
    let a = curve(&[(0.0, 0.0), (1.0, 0.0), (2.0, 1.0)], false, 0, Family::Row);
    assert!(order(std::slice::from_ref(&a), vec![curve(&[(2.0, 1.0), (1.0, 0.0), (0.0, 0.0)], false, 0, Family::Row)]).is_ok());
    // A cyclic shift of an open curve is another path.
    let shifted = curve(&[(1.0, 0.0), (2.0, 1.0), (0.0, 0.0)], false, 0, Family::Row);
    assert_eq!(order(&[a], vec![shifted]).unwrap_err(), NOT_A_PERMUTATION);
}

#[test]
fn a_changed_circle_index_family_or_point_is_refused() {
    let c1 = Circle::new(Point2::new(2.0, 2.0), 2.0).unwrap();
    let c2 = Circle::new(Point2::new(2.0, 2.0), 2.5).unwrap();
    let with = |c: Option<Circle>| Curve::new(pts(&SQ), c, true, 0, Family::Ring).unwrap();
    assert!(order(&[with(Some(c1))], vec![with(Some(c1))]).is_ok());
    assert_eq!(order(&[with(Some(c1))], vec![with(Some(c2))]).unwrap_err(), NOT_A_PERMUTATION);
    assert_eq!(order(&[with(Some(c1))], vec![with(None)]).unwrap_err(), NOT_A_PERMUTATION);
    assert_eq!(order(&[ring(&SQ)], vec![curve(&SQ, true, 7, Family::Ring)]).unwrap_err(), NOT_A_PERMUTATION);
    assert_eq!(order(&[ring(&SQ)], vec![curve(&SQ, true, 0, Family::Trace)]).unwrap_err(), NOT_A_PERMUTATION);
    let moved = ring(&[(0.0, 0.0), (4.0, 0.0), (4.0, 4.5), (0.0, 4.0)]);
    assert_eq!(order(&[ring(&SQ)], vec![moved]).unwrap_err(), NOT_A_PERMUTATION);
}

#[test]
fn a_dropped_or_doubled_curve_is_refused_and_a_permutation_accepted() {
    let a = ring(&SQ);
    let b = curve(&[(1.0, 1.0), (3.0, 1.0), (3.0, 3.0)], true, 1, Family::Ring);
    assert_eq!(order(&[a.clone(), b.clone()], vec![a.clone()]).unwrap_err(), NOT_A_PERMUTATION);
    assert_eq!(order(&[a.clone(), b.clone()], vec![a.clone(), a.clone()]).unwrap_err(), NOT_A_PERMUTATION);
    assert!(order(&[a.clone(), b.clone()], vec![b, a]).is_ok());
}

#[test]
fn the_start_is_resolved_finite_and_on_the_first_curve() {
    let g = GuideSet::new(vec![ring(&SQ)]);
    let p = Point2::new(0.0, 0.0);
    assert_eq!(OrderedSet::new(&g, vec![ring(&SQ)], p, StartSide::Auto).unwrap_err().reason(), UNRESOLVED);
    assert_eq!(OrderedSet::new(&g, vec![ring(&SQ)], Point2::new(f64::NAN, 0.0), StartSide::Inside).unwrap_err().reason(), BAD_START);
    let off = OrderedSet::new(&g, vec![ring(&SQ)], Point2::new(4.0, 4.0), StartSide::Inside).unwrap_err();
    assert_eq!(off.reason(), START_OFF_CURVE);
    assert_eq!(OrderedSet::new(&g, vec![ring(&SQ)], p, StartSide::Outside).unwrap().resolved(), StartSide::Outside);
}
