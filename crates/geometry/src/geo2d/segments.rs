//! The distance between two straight segments in the plane (#4177): zero
//! where they cross or touch, else the least distance of an end point to
//! the other segment. The link stage of the path kernel reads it to prove
//! that a link crosses no cut.
//! Provenance: the orientation test of two segments and the point-segment
//! distance, as in O'Rourke, "Computational Geometry in C" (2nd ed., 1998),
//! ch. 1 and 7; no vendor material consulted.

use super::Point2;

/// The signed area of the triangle `a`, `b`, `c` (twice): positive when
/// `c` is to the left of the line from `a` to `b`.
fn orient(a: Point2, b: Point2, c: Point2) -> f64 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

/// The distance from `p` to the closed segment `a`-`b` (a point when `a`
/// and `b` are the same).
#[must_use]
pub fn point_segment_distance(p: Point2, a: Point2, b: Point2) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len2 = dx * dx + dy * dy;
    let t = if len2 > 0.0 { (((p.x - a.x) * dx + (p.y - a.y) * dy) / len2).clamp(0.0, 1.0) } else { 0.0 };
    (p.x - (a.x + t * dx)).hypot(p.y - (a.y + t * dy))
}

/// True when the segments `a0`-`a1` and `b0`-`b1` cross properly: each one
/// has the ends of the other strictly on its two sides. A touch, a shared
/// end or a collinear overlap is not a crossing (the crossing law of the
/// simulator's verifier reads the same test).
#[must_use]
pub fn segments_cross(a0: Point2, a1: Point2, b0: Point2, b1: Point2) -> bool {
    let (oa, ob) = (orient(b0, b1, a0), orient(b0, b1, a1));
    let (oc, od) = (orient(a0, a1, b0), orient(a0, a1, b1));
    ((oa > 0.0 && ob < 0.0) || (oa < 0.0 && ob > 0.0)) && ((oc > 0.0 && od < 0.0) || (oc < 0.0 && od > 0.0))
}

/// The least distance between the closed segments `a0`-`a1` and `b0`-`b1`:
/// zero when they cross properly, else the least distance of an end point
/// of one to the other segment (a touch or a collinear overlap gives zero
/// that way). A value that is not finite comes from a point that is not
/// finite; the caller treats it as "not proved apart".
#[must_use]
pub fn segment_gap(a0: Point2, a1: Point2, b0: Point2, b1: Point2) -> f64 {
    if segments_cross(a0, a1, b0, b1) {
        return 0.0;
    }
    point_segment_distance(a0, b0, b1)
        .min(point_segment_distance(a1, b0, b1))
        .min(point_segment_distance(b0, a0, a1))
        .min(point_segment_distance(b1, a0, a1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: f64, y: f64) -> Point2 {
        Point2::new(x, y)
    }

    #[test]
    fn crossing_segments_have_no_gap() {
        assert_eq!(segment_gap(p(0.0, 0.0), p(2.0, 2.0), p(0.0, 2.0), p(2.0, 0.0)), 0.0);
        assert!(segments_cross(p(0.0, 0.0), p(2.0, 2.0), p(0.0, 2.0), p(2.0, 0.0)));
    }

    #[test]
    fn a_touch_a_shared_end_and_an_overlap_do_not_cross() {
        assert!(!segments_cross(p(0.0, 0.0), p(2.0, 0.0), p(1.0, 0.0), p(1.0, 3.0)));
        assert!(!segments_cross(p(0.0, 0.0), p(2.0, 0.0), p(2.0, 0.0), p(3.0, 1.0)));
        assert!(!segments_cross(p(0.0, 0.0), p(2.0, 0.0), p(1.0, 0.0), p(3.0, 0.0)));
        assert!(!segments_cross(p(0.0, 0.0), p(1.0, 0.0), p(2.0, -1.0), p(2.0, 1.0)));
    }

    #[test]
    fn apart_segments_have_the_least_end_distance() {
        let g = segment_gap(p(0.0, 0.0), p(2.0, 0.0), p(1.0, 0.5), p(1.0, 3.0));
        assert!((g - 0.5).abs() < 1e-12, "{g}");
        let g = segment_gap(p(0.0, 0.0), p(1.0, 0.0), p(3.0, 0.0), p(4.0, 0.0));
        assert!((g - 2.0).abs() < 1e-12, "{g}");
    }

    #[test]
    fn a_touch_and_an_overlap_have_no_gap() {
        assert_eq!(segment_gap(p(0.0, 0.0), p(2.0, 0.0), p(1.0, 0.0), p(1.0, 3.0)), 0.0);
        assert_eq!(segment_gap(p(0.0, 0.0), p(2.0, 0.0), p(1.0, 0.0), p(3.0, 0.0)), 0.0);
        assert_eq!(segment_gap(p(0.0, 0.0), p(2.0, 0.0), p(2.0, 0.0), p(3.0, 1.0)), 0.0);
    }

    #[test]
    fn a_point_segment_and_a_non_finite_point_are_read() {
        assert!((point_segment_distance(p(1.0, 1.0), p(0.0, 0.0), p(0.0, 0.0)) - 2f64.sqrt()).abs() < 1e-12);
        assert!(!segment_gap(p(f64::NAN, 0.0), p(1.0, 0.0), p(0.0, 1.0), p(1.0, 1.0)).is_finite());
    }
}
