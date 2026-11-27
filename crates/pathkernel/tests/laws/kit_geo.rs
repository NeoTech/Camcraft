//! The plane measures of the law kit (#4027, split out of `kit.rs` by
//! #4406): polylines of curves, point and segment distances, and the gap
//! of two curve sets or sequences. `kit` re-exports each one.
//! Provenance: our own design record (RECOMMENDED.md "#4024 - Path kernel
//! K0", "The laws as checks"); the Hausdorff distance read at the vertices.

use camcraft_pathkernel::guide::Curve;
use camcraft_pathkernel::{Loop, Point2, Region};

/// The points of a curve as a polyline (a closed curve repeats its start).
pub fn poly(c: &Curve) -> Vec<Point2> {
    let mut v: Vec<Point2> = c.pts().iter().map(|p| p.at()).collect();
    if let Some(&p) = v.first().filter(|_| c.closed()) {
        v.push(p);
    }
    v
}

pub fn dist(a: Point2, b: Point2) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

pub fn seg_dist(p: Point2, a: Point2, b: Point2) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len2 = dx * dx + dy * dy;
    let t = if len2 > 0.0 { (((p.x - a.x) * dx + (p.y - a.y) * dy) / len2).clamp(0.0, 1.0) } else { 0.0 };
    dist(p, Point2::new(a.x + t * dx, a.y + t * dy))
}

pub fn poly_dist(p: Point2, line: &[Point2]) -> f64 {
    match line {
        [only] => dist(p, *only),
        _ => line.windows(2).map(|w| seg_dist(p, w[0], w[1])).fold(f64::INFINITY, f64::min),
    }
}

/// The distance of `p` outside `area` (0 inside).
pub fn outside(area: &Region, p: Point2) -> f64 {
    if area.contains(p) {
        return 0.0;
    }
    let ring = |l: &Loop| {
        let mut v = l.points().to_vec();
        v.extend(l.points().first().copied());
        poly_dist(p, &v)
    };
    area.loops().iter().map(ring).fold(f64::INFINITY, f64::min)
}

/// The Hausdorff distance of two curves in the plane, read at the vertices.
pub fn curve_gap(a: &Curve, b: &Curve) -> f64 {
    let (pa, pb) = (poly(a), poly(b));
    let one = |from: &[Point2], to: &[Point2]| from.iter().map(|p| poly_dist(*p, to)).fold(0.0, f64::max);
    one(&pa, &pb).max(one(&pb, &pa))
}

/// The gap of two curve multisets (order and direction free): each curve of
/// `a` matched to its nearest unused curve of `b`. A count that differs is
/// an infinite gap.
pub fn set_gap(a: &[&Curve], b: &[&Curve]) -> (f64, String) {
    if a.len() != b.len() {
        return (f64::INFINITY, format!("{} curves against {}", a.len(), b.len()));
    }
    let (mut used, mut worst) = (vec![false; b.len()], (0.0, String::from("each curve")));
    for (i, ca) in a.iter().enumerate() {
        let best = b.iter().enumerate().filter(|(j, _)| !used[*j]).map(|(j, cb)| (curve_gap(ca, cb), j)).min_by(|x, y| x.0.total_cmp(&y.0));
        let Some((g, j)) = best else {
            return (f64::INFINITY, format!("curve {i} has no partner"));
        };
        used[j] = true;
        if g > worst.0 {
            worst = (g, format!("curve {i} (k {})", ca.k()));
        }
    }
    worst
}

/// The gap of two curve sequences, point by point (order and direction
/// count). A count that differs is an infinite gap.
pub fn seq_gap(a: &[&Curve], b: &[&Curve]) -> (f64, String) {
    if a.len() != b.len() {
        return (f64::INFINITY, format!("{} cuts against {}", a.len(), b.len()));
    }
    let mut worst = (0.0, String::from("each cut"));
    for (i, (ca, cb)) in a.iter().zip(b).enumerate() {
        let g = if ca.pts().len() == cb.pts().len() {
            let d = |(p, q): (&camcraft_pathkernel::PlanePoint, &camcraft_pathkernel::PlanePoint)| {
                dist(p.at(), q.at()).max((p.depth() - q.depth()).abs())
            };
            ca.pts().iter().zip(cb.pts()).map(d).fold(0.0, f64::max)
        } else {
            f64::INFINITY
        };
        if g > worst.0 {
            worst = (g, format!("cut {i} (k {})", ca.k()));
        }
    }
    worst
}
