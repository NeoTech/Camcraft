//! The straight-chord law of the planar generators: does a chord stay
//! inside (or on) a closed region? Split from `slots.rs` by #1620, which
//! also closed its collinear hole (below).
//!
//! Provenance: segment/polygon intersection and point-in-polygon tests
//! (O'Rourke, "Computational Geometry in C", 2nd ed., ch. 7); our own
//! arrangement of them. No vendor material consulted.

use crate::geo2d::{Point2, Region};

use super::ring::on_boundary;

/// Parameter slack along the chord: a touch closer than this to either
/// end is the end itself.
const EPS: f64 = 1e-9;

/// True when the straight chord `a` -> `b` never passes outside the
/// closed region: both ends in it, no loop edge properly crossing the
/// open chord, and every piece of the chord between two points where it
/// touches the boundary inside or on it. A chord running along a
/// boundary edge is exactly the tangency a wall pass is, so grazing
/// reads as clear.
///
/// Why pieces and not one midpoint (#1620): without a proper crossing
/// the chord can still leave the region where it runs along an edge and
/// slips off its end at a vertex whose other edge turns away - two
/// pieces of one area that meet the chord line only along their own
/// walls (the cutter-centre pieces either side of a part that pinches
/// the stock's band shut). Every edge touch splits the chord, and the
/// midpoint of each piece decides for that piece: between two touches
/// it meets no boundary, so it lies wholly inside or wholly outside.
pub fn chord_clears(region: &Region, a: Point2, b: Point2) -> bool {
    if (!region.contains(a) && !on_boundary(region, a))
        || (!region.contains(b) && !on_boundary(region, b))
    {
        return false;
    }
    if a == b {
        return true;
    }
    let d = Point2::new(b.x - a.x, b.y - a.y);
    let dd = d.x * d.x + d.y * d.y;
    let mut cuts: Vec<f64> = vec![0.0, 1.0];
    let along = |q: Point2| ((q.x - a.x) * d.x + (q.y - a.y) * d.y) / dd;
    for l in region.loops() {
        let pts = l.points();
        let n = pts.len();
        for i in 0..n {
            let c = pts[i];
            let e = pts[(i + 1) % n];
            let g = Point2::new(e.x - c.x, e.y - c.y);
            let ca = Point2::new(c.x - a.x, c.y - a.y);
            let den = d.x * g.y - d.y * g.x;
            if den == 0.0 {
                // Parallel to this edge. On the chord's line (within
                // the boundary tolerance) it is a graze: its ends split
                // the chord, so the piece beyond the edge is judged on
                // its own.
                let off = ca.x * d.y - ca.y * d.x;
                if off * off <= 1e-18 * dd {
                    cuts.extend([along(c), along(e)].into_iter().filter(|t| EPS < *t && *t < 1.0 - EPS));
                }
                continue;
            }
            let t = (ca.x * g.y - ca.y * g.x) / den;
            let u = (ca.x * d.y - ca.y * d.x) / den;
            if !(EPS < t && t < 1.0 - EPS) {
                continue;
            }
            if u > EPS && u < 1.0 - EPS {
                return false; // crosses the open edge: a real crossing
            }
            if u >= -EPS && u <= 1.0 + EPS {
                // The chord goes through a loop vertex. That cuts the
                // boundary only if the two edges meeting there are on
                // opposite sides of the chord line; both on one side is
                // tangency (a fillet touching the straight it joins).
                let j = if u < 0.5 { i } else { (i + 1) % n };
                let p0 = pts[(j + n - 1) % n];
                let p2 = pts[(j + 1) % n];
                let h0 = (p0.x - a.x) * d.y - (p0.y - a.y) * d.x;
                let h2 = (p2.x - a.x) * d.y - (p2.y - a.y) * d.x;
                if h0 * h2 < 0.0 {
                    return false;
                }
                cuts.push(t);
            }
        }
    }
    cuts.sort_by(f64::total_cmp);
    cuts.dedup_by(|x, y| (*x - *y).abs() <= EPS);
    cuts.windows(2).all(|w| {
        let s = (w[0] + w[1]) * 0.5;
        let m = Point2::new(a.x + d.x * s, a.y + d.y * s);
        region.contains(m) || on_boundary(region, m)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geo2d::Loop;

    fn sq(x0: f64, y0: f64, x1: f64, y1: f64) -> Loop {
        let p = Point2::new;
        Loop::new(vec![p(x0, y0), p(x1, y0), p(x1, y1), p(x0, y1)]).expect("loop")
    }

    /// #1620's shape: two pieces whose bottoms lie on one line with a
    /// gap between them. The chord from one piece's inner corner along
    /// the shared line to the far corner of the other grazes the second
    /// piece's wall and leaves the area across the gap.
    #[test]
    fn a_chord_along_two_pieces_walls_across_the_gap_between_them_does_not_clear() {
        let region = Region::new(vec![sq(-32.0, -32.0, -8.0, -8.0), sq(8.0, -32.0, 32.0, -8.0)]).expect("region");
        let (a, b) = (Point2::new(-8.0, -32.0), Point2::new(32.0, -32.0));
        assert!(!chord_clears(&region, a, b), "the gap x in (-8, 8) is outside the area");
        assert!(!chord_clears(&region, b, a), "either way along it");
        // Its two grazing halves on each piece's own wall still clear.
        assert!(chord_clears(&region, Point2::new(-32.0, -32.0), a));
        assert!(chord_clears(&region, Point2::new(8.0, -32.0), b));
    }

    /// The graze law the wall passes ride stays: a chord along one edge,
    /// past a convex corner tangent to the chord, inside one piece.
    #[test]
    fn a_chord_grazing_one_pieces_wall_still_clears() {
        let region = Region::new(vec![sq(0.0, 0.0, 30.0, 30.0)]).expect("region");
        assert!(chord_clears(&region, Point2::new(0.0, 0.0), Point2::new(30.0, 0.0)));
        assert!(chord_clears(&region, Point2::new(5.0, 0.0), Point2::new(25.0, 0.0)));
        assert!(chord_clears(&region, Point2::new(0.0, 0.0), Point2::new(30.0, 30.0)));
        assert!(!chord_clears(&region, Point2::new(0.0, 0.0), Point2::new(31.0, 0.0)));
    }
}
