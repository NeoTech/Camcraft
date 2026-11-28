//! Does a triangle overlap the inside of a simple ring, with a slack band
//! on the ring's boundary (#2440)? A triangle that only touches the ring
//! (a shared vertex, an edge along the ring's edge) does not overlap it.
//! Used to find material above a floor inside the floor's inner ring.
//!
//! Provenance: our own code; the orientation predicate, the even-odd
//! point-in-polygon rule and the proper segment-crossing test of standard
//! computational geometry (O'Rourke, "Computational Geometry in C",
//! ch. 1 and 7). No vendor material consulted.

use super::Point2;

/// The axis-aligned bounds of `pts`, `(min, max)`. Infinite for an empty
/// slice, so nothing falls inside them.
#[must_use]
pub fn bounds(pts: &[Point2]) -> (Point2, Point2) {
    let mut lo = Point2::new(f64::INFINITY, f64::INFINITY);
    let mut hi = Point2::new(f64::NEG_INFINITY, f64::NEG_INFINITY);
    for p in pts {
        lo = Point2::new(lo.x.min(p.x), lo.y.min(p.y));
        hi = Point2::new(hi.x.max(p.x), hi.y.max(p.y));
    }
    (lo, hi)
}

/// True when triangle `tri` overlaps the inside of the simple closed ring
/// `ring` by more than the slack `eps` (mm): a triangle corner is inside
/// the ring and more than `eps` from its boundary, a ring vertex is inside
/// the triangle by more than `eps`, the triangle's centroid is inside the
/// ring by more than `eps`, or an edge of each crosses the other properly.
/// A ring of fewer than three points or a non-finite input overlaps
/// nothing; the caller decides what that means.
#[must_use]
pub fn triangle_overlaps_ring(tri: &[Point2; 3], ring: &[Point2], eps: f64) -> bool {
    let n = ring.len();
    if n < 3 || !eps.is_finite() || !tri.iter().chain(ring).all(|p| p.is_finite()) {
        return false;
    }
    let c = Point2::new((tri[0].x + tri[1].x + tri[2].x) / 3.0, (tri[0].y + tri[1].y + tri[2].y) / 3.0);
    if tri.iter().chain(std::iter::once(&c)).any(|&p| strictly_in_ring(p, ring, eps)) {
        return true;
    }
    if ring.iter().any(|&p| strictly_in_triangle(p, tri, eps)) {
        return true;
    }
    for i in 0..3 {
        let (a, b) = (tri[i], tri[(i + 1) % 3]);
        for j in 0..n {
            let (Some(&p), Some(&q)) = (ring.get(j), ring.get((j + 1) % n)) else { continue };
            if crosses(a, b, p, q, eps) {
                return true;
            }
        }
    }
    false
}

/// Twice the signed area of `a b c`, positive counter-clockwise.
fn orient(a: Point2, b: Point2, c: Point2) -> f64 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

/// Signed distance of `c` from the line `a b`; 0 for a zero-length span.
fn side(a: Point2, b: Point2, c: Point2) -> f64 {
    let len = (b.x - a.x).hypot(b.y - a.y);
    if len > 0.0 && len.is_finite() { orient(a, b, c) / len } else { 0.0 }
}

/// Distance from `p` to the segment `a b`.
fn seg_dist(p: Point2, a: Point2, b: Point2) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 0.0 { (((p.x - a.x) * dx + (p.y - a.y) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
    (p.x - (a.x + t * dx)).hypot(p.y - (a.y + t * dy))
}

/// `p` is inside `ring` (even-odd) and more than `eps` from each edge.
fn strictly_in_ring(p: Point2, ring: &[Point2], eps: f64) -> bool {
    let n = ring.len();
    let mut is_in = false;
    for i in 0..n {
        let (Some(&a), Some(&b)) = (ring.get(i), ring.get((i + 1) % n)) else { continue };
        if seg_dist(p, a, b) <= eps {
            return false;
        }
        if (a.y > p.y) != (b.y > p.y) && p.x < (p.y - a.y) * (b.x - a.x) / (b.y - a.y) + a.x {
            is_in = !is_in;
        }
    }
    is_in
}

/// `p` is inside the triangle by more than `eps` from each of its edges.
/// A triangle with no area (a vertical facet seen from above) holds
/// nothing.
fn strictly_in_triangle(p: Point2, t: &[Point2; 3], eps: f64) -> bool {
    let s = orient(t[0], t[1], t[2]).signum();
    if s == 0.0 {
        return false;
    }
    (0..3).all(|i| s * side(t[i], t[(i + 1) % 3], p) > eps)
}

/// The segments `a b` and `p q` cross at one point inside both, each end
/// more than `eps` off the other segment's line.
fn crosses(a: Point2, b: Point2, p: Point2, q: Point2, eps: f64) -> bool {
    let (d1, d2) = (side(a, b, p), side(a, b, q));
    let (d3, d4) = (side(p, q, a), side(p, q, b));
    ((d1 > eps && d2 < -eps) || (d1 < -eps && d2 > eps)) && ((d3 > eps && d4 < -eps) || (d3 < -eps && d4 > eps))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: f64, y: f64) -> Point2 {
        Point2::new(x, y)
    }

    /// A diamond, half diagonal 3, centre (12, 0); its corner (15, 0)
    /// touches the line x = 15.
    fn diamond() -> Vec<Point2> {
        vec![p(15.0, 0.0), p(12.0, 3.0), p(9.0, 0.0), p(12.0, -3.0)]
    }

    #[test]
    fn a_triangle_inside_the_ring_overlaps_it() {
        assert!(triangle_overlaps_ring(&[p(11.0, -0.5), p(13.0, -0.5), p(12.0, 0.5)], &diamond(), 1e-5));
    }

    #[test]
    fn a_triangle_round_the_whole_ring_overlaps_it() {
        assert!(triangle_overlaps_ring(&[p(0.0, -10.0), p(30.0, -10.0), p(12.0, 20.0)], &diamond(), 1e-5));
    }

    #[test]
    fn a_strip_across_the_ring_overlaps_it() {
        // No corner of either is inside the other; the edges cross.
        assert!(triangle_overlaps_ring(&[p(0.0, -0.5), p(30.0, -0.5), p(0.0, 0.5)], &diamond(), 1e-5));
    }

    #[test]
    fn a_wall_that_only_touches_the_ring_does_not_overlap_it() {
        // A land beside the line x = 15, which meets the diamond at its
        // corner only, and a vertical wall seen from above.
        assert!(!triangle_overlaps_ring(&[p(15.0, -20.0), p(25.0, 20.0), p(15.0, 20.0)], &diamond(), 1e-5));
        assert!(!triangle_overlaps_ring(&[p(15.0, 0.0), p(15.0, 20.0), p(15.0, 20.0)], &diamond(), 1e-5));
        assert!(!triangle_overlaps_ring(&[p(15.0, -20.0), p(15.0, 20.0), p(25.0, 0.0)], &diamond(), 1e-5));
    }

    #[test]
    fn a_triangle_off_the_ring_or_bad_input_overlaps_nothing() {
        assert!(!triangle_overlaps_ring(&[p(20.0, 5.0), p(25.0, 5.0), p(22.0, 9.0)], &diamond(), 1e-5));
        assert!(!triangle_overlaps_ring(&[p(f64::NAN, 0.0), p(13.0, 0.0), p(12.0, 1.0)], &diamond(), 1e-5));
        assert!(!triangle_overlaps_ring(&[p(11.0, 0.0), p(13.0, 0.0), p(12.0, 1.0)], &diamond()[..2], 1e-5));
    }

    #[test]
    fn bounds_cover_the_points() {
        let (lo, hi) = bounds(&diamond());
        assert_eq!((lo, hi), (p(9.0, -3.0), p(15.0, 3.0)));
    }
}
