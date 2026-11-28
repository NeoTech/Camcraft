//! The exact gap between a tool column swept straight up Z and the
//! part's facets (#4315), for the swarf exit rise's proof
//! (`super::stamp_rise`).
//!
//! A ball end mill posed along `a` is the radius-`r` neighbourhood of
//! its axis segment (the ball about the centre, the flute about the
//! rest). Swept straight up by `dz`, that segment sweeps a
//! parallelogram, so the swept tool is the parallelogram's radius-`r`
//! neighbourhood, and its depth into the part is `r` less the
//! parallelogram's distance to the nearest facet. The distance is the
//! triangle-triangle distance over the parallelogram's two triangles:
//! zero where an edge of one triangle crosses the other, else the
//! least of the six vertex-to-triangle and nine edge-to-edge
//! distances. No sample spacing, so no edge or vertex of the part can
//! hide between samples.
//!
//! Provenance: the triangle-triangle distance by vertex/edge
//! candidates and the segment-triangle crossing test (Moller and
//! Trumbore, 1997) are published closed forms (Ericson, "Real-Time
//! Collision Detection", 2005, ch. 5). Our own design notes; no vendor
//! material consulted.

use crate::finishing3d::closest_pt_tri;
use crate::geometry::{Point3, Triangle};

use super::rotary_flow_set::seg_seg_at;

/// `p + s·v`.
fn add(p: Point3, v: Point3, s: f64) -> Point3 {
    Point3::new(p.x + v.x * s, p.y + v.y * s, p.z + v.z * s)
}

fn dist(a: Point3, b: Point3) -> f64 {
    let d = a - b;
    d.dot(d).max(0.0).sqrt()
}

/// Does the segment `p -> q` cross triangle `t` (Moller-Trumbore,
/// both ends included)? A segment in the triangle's plane does not
/// count: the vertex and edge candidates of [`tri_tri_gap`] see it.
fn crosses(p: Point3, q: Point3, t: &[Point3; 3]) -> bool {
    let (e1, e2, d) = (t[1] - t[0], t[2] - t[0], q - p);
    let h = d.cross(e2);
    let det = e1.dot(h);
    if det == 0.0 || !det.is_finite() {
        return false;
    }
    let s = p - t[0];
    let u = s.dot(h) / det;
    if !(0.0..=1.0).contains(&u) {
        return false;
    }
    let w = s.cross(e1);
    let v = d.dot(w) / det;
    if v < 0.0 || u + v > 1.0 {
        return false;
    }
    (0.0..=1.0).contains(&(e2.dot(w) / det))
}

/// The distance between triangles `t` and `u` (zero where they meet).
/// A degenerate triangle (a segment or a point) is read as itself.
pub(super) fn tri_tri_gap(t: &[Point3; 3], u: &[Point3; 3]) -> f64 {
    let edges = |x: &[Point3; 3]| [(x[0], x[1]), (x[1], x[2]), (x[2], x[0])];
    let (et, eu) = (edges(t), edges(u));
    if et.iter().any(|&(p, q)| crosses(p, q, u)) || eu.iter().any(|&(p, q)| crosses(p, q, t)) {
        return 0.0;
    }
    let mut best = f64::INFINITY;
    for &v in t {
        best = best.min(dist(v, closest_pt_tri(v, u)));
    }
    for &v in u {
        best = best.min(dist(v, closest_pt_tri(v, t)));
    }
    for &(p, q) in &et {
        for &(a, b) in &eu {
            best = best.min(seg_seg_at(p, q, a, b).0);
        }
    }
    best
}

/// The distance from the parallelogram `o + s·u + t·v` (`s`, `t` in
/// `[0, 1]`) to the nearest facet, no more than `cap`: a facet whose
/// box stands `cap` or more away is not read. Returns as soon as a
/// facet nearer than `stop` is found (a caller asking "at least
/// `stop`?" needs no more). `v` zero reads the segment `o -> o + u`.
pub(super) fn quad_gap(
    o: Point3,
    u: Point3,
    v: Point3,
    facets: &[Triangle],
    boxes: &[[f64; 6]],
    cap: f64,
    stop: f64,
) -> f64 {
    let (ou, ouv, ov) = (add(o, u, 1.0), add(add(o, u, 1.0), v, 1.0), add(o, v, 1.0));
    let halves = [[o, ou, ouv], [o, ouv, ov]];
    let mut lo = [f64::INFINITY; 3];
    let mut hi = [f64::NEG_INFINITY; 3];
    for c in [o, ou, ouv, ov] {
        for (i, x) in [c.x, c.y, c.z].into_iter().enumerate() {
            lo[i] = lo[i].min(x);
            hi[i] = hi[i].max(x);
        }
    }
    let mut best = cap;
    for (f, bx) in facets.iter().zip(boxes) {
        let g = (0..3).map(|i| (bx[i] - hi[i]).max(lo[i] - bx[i + 3]).max(0.0).powi(2)).sum::<f64>().sqrt();
        if g >= best {
            continue;
        }
        let tri = [f.a, f.b, f.c];
        for h in &halves {
            best = best.min(tri_tri_gap(h, &tri));
        }
        if best < stop {
            return best;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: f64, y: f64, z: f64) -> Point3 {
        Point3::new(x, y, z)
    }

    fn boxed(f: &[Triangle]) -> Vec<[f64; 6]> {
        f.iter()
            .map(|t| {
                [
                    t.a.x.min(t.b.x.min(t.c.x)),
                    t.a.y.min(t.b.y.min(t.c.y)),
                    t.a.z.min(t.b.z.min(t.c.z)),
                    t.a.x.max(t.b.x.max(t.c.x)),
                    t.a.y.max(t.b.y.max(t.c.y)),
                    t.a.z.max(t.b.z.max(t.c.z)),
                ]
            })
            .collect()
    }

    #[test]
    fn the_gap_is_exact_for_a_vertex_an_edge_and_a_face() {
        // The square X 0..10, Y 0, Z 0..10 (a lying axis swept up).
        let (o, u, v) = (p(0.0, 0.0, 0.0), p(10.0, 0.0, 0.0), p(0.0, 0.0, 10.0));
        let gap = |t: Triangle| quad_gap(o, u, v, &[t], &boxed(&[t]), f64::INFINITY, 0.0);
        // A spike's tip 2.95 off the middle of the square's face.
        let tip = Triangle::new(p(4.3, 2.95, 6.1), p(0.0, 9.0, 6.1), p(9.0, 9.0, 6.1));
        assert!((gap(tip) - 2.95).abs() < 1e-12, "{}", gap(tip));
        // A knife edge across the square's top, 1 above it.
        let knife = Triangle::new(p(5.0, -3.0, 11.0), p(5.0, 3.0, 11.0), p(5.0, 0.0, 20.0));
        assert!((gap(knife) - 1.0).abs() < 1e-12, "{}", gap(knife));
        // A facet that pierces the square meets it.
        let pierce = Triangle::new(p(5.0, -3.0, 5.0), p(5.0, 3.0, 5.0), p(5.0, 0.0, 8.0));
        assert_eq!(gap(pierce), 0.0);
    }

    #[test]
    fn a_degenerate_sweep_reads_its_segment_and_far_facets_are_capped() {
        let (o, u) = (p(0.0, 0.0, 0.0), p(10.0, 0.0, 0.0));
        let t = Triangle::new(p(5.0, 2.0, -1.0), p(5.0, 2.0, 1.0), p(5.0, 9.0, 0.0));
        let b = boxed(&[t]);
        let zero = p(0.0, 0.0, 0.0);
        assert!((quad_gap(o, u, zero, &[t], &b, f64::INFINITY, 0.0) - 2.0).abs() < 1e-12);
        assert_eq!(quad_gap(o, u, zero, &[t], &b, 1.5, 0.0), 1.5, "a facet past the cap is not read");
        assert_eq!(quad_gap(o, u, zero, &[], &[], 3.0, 0.0), 3.0, "no facet: the cap");
    }
}
