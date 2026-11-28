//! Unit pins of the medial axis (#4176): a slot, a rectangle, an annulus,
//! the order of the input, and the refusals.

use super::*;
use crate::geo2d::Loop;

const TOL: f64 = 1e-3;
const ANGLE: f64 = std::f64::consts::FRAC_PI_3;

fn region(loops: Vec<Vec<Point2>>) -> Region {
    Region::new(loops.into_iter().map(|l| Loop::new(l).unwrap()).collect()).unwrap()
}

fn arc(c: Point2, r: f64, a0: f64, a1: f64, n: usize) -> Vec<Point2> {
    (0..=n).map(|i| a0 + (a1 - a0) * i as f64 / n as f64).map(|a| Point2::new(c.x + r * a.cos(), c.y + r * a.sin())).collect()
}

/// A slot of half-width `w` round the segment (0, 0)-(len, 0).
fn stadium(len: f64, w: f64) -> Vec<Point2> {
    use std::f64::consts::FRAC_PI_2;
    let mut v = arc(Point2::new(len, 0.0), w, -FRAC_PI_2, FRAC_PI_2, 64);
    v.extend(arc(Point2::new(0.0, 0.0), w, FRAC_PI_2, 3.0 * FRAC_PI_2, 64));
    v
}

fn circle(r: f64, n: usize, ccw: bool) -> Vec<Point2> {
    let mut v: Vec<Point2> =
        (0..n).map(|i| std::f64::consts::TAU * i as f64 / n as f64).map(|a| Point2::new(r * a.cos(), r * a.sin())).collect();
    if !ccw {
        v.reverse();
    }
    v
}

fn length(c: &MedialCurve) -> f64 {
    c.points.windows(2).map(|w| dist(w[0], w[1])).sum()
}

#[test]
fn the_axis_of_a_slot_is_its_centre_segment() {
    let ax = region(vec![stadium(40.0, 3.0)]).medial_axis(TOL, ANGLE).unwrap();
    let long: Vec<&MedialCurve> = ax.curves.iter().filter(|c| length(c) > 1.0).collect();
    assert_eq!(long.len(), 1, "{ax:?}");
    let c = long[0];
    assert!(!c.closed);
    assert!(c.points.iter().all(|p| p.y.abs() <= 2.0 * TOL), "{:?}", c.points);
    assert!(c.radii.iter().all(|r| (r - 3.0).abs() <= 0.01), "{:?}", c.radii);
    let (x0, x1) = c.points.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), p| (a.min(p.x), b.max(p.x)));
    assert!(x0.abs() <= 0.05 && (x1 - 40.0).abs() <= 0.05, "ends at {x0} and {x1}");
    // Ramer-Douglas-Peucker leaves a straight axis with few points.
    assert!(c.points.len() <= 8, "{} points", c.points.len());
}

#[test]
fn a_rectangle_has_a_centre_segment_and_four_corner_branches() {
    let r = region(vec![vec![Point2::new(0.0, 0.0), Point2::new(40.0, 0.0), Point2::new(40.0, 10.0), Point2::new(0.0, 10.0)]]);
    let ax = r.medial_axis(TOL, ANGLE).unwrap();
    assert_eq!(ax.curves.len(), 5, "{ax:?}");
    let mid: Vec<&MedialCurve> = ax.curves.iter().filter(|c| c.points.iter().all(|p| (p.y - 5.0).abs() <= 2.0 * TOL)).collect();
    assert_eq!(mid.len(), 1, "{ax:?}");
    let xs: Vec<f64> = mid[0].points.iter().map(|p| p.x).collect();
    assert!(xs.iter().any(|x| (x - 5.0).abs() <= 0.01) && xs.iter().any(|x| (x - 35.0).abs() <= 0.01), "{xs:?}");
    // Each corner branch runs from a fork to near its corner on the bisector.
    for c in ax.curves.iter().filter(|c| !std::ptr::eq(*c, mid[0])) {
        assert!(c.points.iter().all(|p| (p.y - p.x).abs() <= 0.01 || (p.y + p.x - 10.0).abs() <= 0.01
            || (p.y - (p.x - 30.0)).abs() <= 0.01 || (p.y + p.x - 40.0).abs() <= 0.01), "{c:?}");
        assert!(length(c) > 6.5, "{c:?}");
    }
}

#[test]
fn the_axis_of_an_annulus_is_one_closed_circle_midway() {
    let r = region(vec![circle(10.0, 256, true), circle(6.0, 256, false)]);
    let ax = r.medial_axis(TOL, ANGLE).unwrap();
    let ring: Vec<&MedialCurve> = ax.curves.iter().filter(|c| c.closed).collect();
    assert_eq!(ring.len(), 1, "{ax:?}");
    assert!(ring[0].points.iter().all(|p| (p.x.hypot(p.y) - 8.0).abs() <= 0.01), "{:?}", ring[0].points);
    assert!(ax.curves.iter().filter(|c| !c.closed).all(|c| length(c) < 0.1), "{ax:?}");
}

#[test]
fn the_axis_does_not_depend_on_the_start_vertex_or_the_loop_order() {
    let mut a = stadium(30.0, 4.0);
    let one = region(vec![a.clone(), circle(1.0, 32, false).into_iter().map(|p| Point2::new(p.x + 15.0, p.y)).collect()]);
    a.rotate_left(17);
    let two = region(vec![circle(1.0, 32, false).into_iter().map(|p| Point2::new(p.x + 15.0, p.y)).collect(), a]);
    let (x, y) = (one.medial_axis(TOL, ANGLE).unwrap(), two.medial_axis(TOL, ANGLE).unwrap());
    assert_eq!(x.curves.len(), y.curves.len());
    for (c, d) in x.curves.iter().zip(&y.curves) {
        assert_eq!(c.points.len(), d.points.len());
        assert!(c.points.iter().zip(&d.points).all(|(p, q)| dist(*p, *q) <= 1e-9));
    }
}

#[test]
fn bad_input_is_refused_not_guessed() {
    let r = region(vec![stadium(10.0, 2.0)]);
    assert!(r.medial_axis(0.0, ANGLE).is_err());
    assert!(r.medial_axis(f64::NAN, ANGLE).is_err());
    assert!(r.medial_axis(TOL, 0.0).is_err());
    assert!(Region::empty().medial_axis(TOL, ANGLE).is_err());
    let big = region(vec![circle(5000.0, 64, true)]);
    let e = big.medial_axis(1e-6, ANGLE).unwrap_err();
    assert!(format!("{e}").contains("boundary samples"), "{e}");
}

#[test]
fn a_neck_that_touches_gives_an_axis_with_no_crash() {
    // Two squares that share one corner: the area touches itself there.
    let r = region(vec![
        vec![Point2::new(0.0, 0.0), Point2::new(4.0, 0.0), Point2::new(4.0, 4.0), Point2::new(0.0, 4.0)],
        vec![Point2::new(4.0, 4.0), Point2::new(8.0, 4.0), Point2::new(8.0, 8.0), Point2::new(4.0, 8.0)],
    ]);
    let ax = r.medial_axis(TOL, ANGLE).unwrap();
    assert!(!ax.curves.is_empty());
}

/// #4407: the circumcentre checks its denominator before the division: a
/// flat triangle or a value that is not finite has no centre.
#[test]
fn a_flat_or_not_finite_triangle_has_no_circumcentre() {
    let p = Point2::new;
    assert_eq!(circumcenter(p(0.0, 0.0), p(1.0, 0.0), p(2.0, 0.0)), None, "three points on a line");
    assert_eq!(circumcenter(p(0.0, 0.0), p(f64::NAN, 0.0), p(0.0, 1.0)), None, "a point that is not finite");
    assert_eq!(circumcenter(p(0.0, 0.0), p(f64::INFINITY, 0.0), p(0.0, 1.0)), None, "a point at infinity");
    let c = circumcenter(p(0.0, 0.0), p(2.0, 0.0), p(0.0, 2.0)).unwrap();
    assert!((c.x - 1.0).abs() < 1e-12 && (c.y - 1.0).abs() < 1e-12, "{c:?}");
}
