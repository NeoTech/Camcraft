//! The Delaunay triangulation of a point set (#4176), for the medial axis
//! of a sampled boundary (`medial.rs`).
//! Provenance: the incremental insertion of Bowyer, "Computing Dirichlet
//! tessellations" (The Computer Journal 24, 1981) and Watson, "Computing
//! the n-dimensional Delaunay tessellation" (same issue), with the
//! visibility walk of Devillers, Pion and Teillaud, "Walking in a
//! triangulation" (2002) for the point location; the orientation and
//! in-circle predicates as in de Berg et al., "Computational Geometry"
//! (3rd ed., 2008), ch. 9. Written new; no vendor material consulted.
//!
//! The predicates are in `f64`, translated to the inserted point. A cavity
//! that the rounding makes not star-shaped from the point is grown until
//! each of its edges sees the point, so the output is always a valid
//! triangulation; where four points are (nearly) on one circle it is one of
//! the Delaunay triangulations. Every loop is bounded: a failure is a
//! `GeoError`, never a panic or a hang.

use super::{GeoError, Point2};

/// No neighbour (an edge of the enclosing triangle).
const NONE: usize = usize::MAX;

#[derive(Clone, Copy)]
struct Tri {
    v: [usize; 3],
    /// `n[i]` is the neighbour across the edge opposite `v[i]`.
    n: [usize; 3],
    alive: bool,
}

/// Twice the signed area of `a b c`: above zero when counter-clockwise.
fn orient(a: Point2, b: Point2, c: Point2) -> f64 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

/// Above zero when `d` is inside the circle through the counter-clockwise
/// triangle `a b c`.
fn in_circle(a: Point2, b: Point2, c: Point2, d: Point2) -> f64 {
    let (ax, ay, bx, by, cx, cy) = (a.x - d.x, a.y - d.y, b.x - d.x, b.y - d.y, c.x - d.x, c.y - d.y);
    (ax * ax + ay * ay) * (bx * cy - cx * by) - (bx * bx + by * by) * (ax * cy - cx * ay) + (cx * cx + cy * cy) * (ax * by - bx * ay)
}

fn bad(what: &str) -> GeoError {
    GeoError::Invalid(format!("delaunay triangulation: {what}"))
}

struct Mesh {
    p: Vec<Point2>,
    t: Vec<Tri>,
    free: Vec<usize>,
    /// The visit stamp of each triangle, for the cavity search.
    seen: Vec<u32>,
    stamp: u32,
}

impl Mesh {
    fn pt(&self, i: usize) -> Result<Point2, GeoError> {
        self.p.get(i).copied().ok_or_else(|| bad("a vertex index is out of range"))
    }
    fn tri(&self, i: usize) -> Result<Tri, GeoError> {
        self.t.get(i).copied().ok_or_else(|| bad("a triangle index is out of range"))
    }
    fn corners(&self, t: &Tri) -> Result<[Point2; 3], GeoError> {
        Ok([self.pt(t.v[0])?, self.pt(t.v[1])?, self.pt(t.v[2])?])
    }

    /// A live triangle that holds `q` (inside or on its boundary).
    fn locate(&self, start: usize, q: Point2) -> Result<usize, GeoError> {
        let mut at = start;
        let limit = self.t.len().saturating_mul(2).saturating_add(16);
        for step in 0..limit {
            let tr = self.tri(at)?;
            let c = self.corners(&tr)?;
            // The edge tried first turns with the step, so that the walk
            // cannot cycle between two triangles (Devillers et al.).
            let moved = (0..3).map(|j| (j + step) % 3).find(|&j| orient(c[(j + 1) % 3], c[(j + 2) % 3], q) < 0.0);
            match moved {
                None => return Ok(at),
                Some(j) if tr.n[j] != NONE => at = tr.n[j],
                Some(_) => break,
            }
        }
        // The walk did not end (rounding): a scan of the live triangles.
        for (i, tr) in self.t.iter().enumerate().filter(|(_, t)| t.alive) {
            let c = self.corners(tr)?;
            if (0..3).all(|j| orient(c[(j + 1) % 3], c[(j + 2) % 3], q) >= 0.0) {
                return Ok(i);
            }
        }
        Err(bad("a point is outside every triangle"))
    }

    fn mark(&mut self, i: usize) -> Result<bool, GeoError> {
        let s = self.seen.get_mut(i).ok_or_else(|| bad("a triangle index is out of range"))?;
        let fresh = *s != self.stamp;
        *s = self.stamp;
        Ok(fresh)
    }

    /// Inserts point `i`; returns a new triangle that holds it.
    fn insert(&mut self, i: usize, start: usize) -> Result<usize, GeoError> {
        let q = self.pt(i)?;
        let t0 = self.locate(start, q)?;
        self.stamp = self.stamp.wrapping_add(1);
        if self.stamp == 0 {
            self.seen.iter_mut().for_each(|s| *s = u32::MAX);
            self.stamp = 1;
        }
        let mut cavity = vec![t0];
        self.mark(t0)?;
        let mut k = 0;
        while let Some(&t) = cavity.get(k) {
            k += 1;
            for nb in self.tri(t)?.n {
                if nb == NONE || self.seen.get(nb) == Some(&self.stamp) {
                    continue;
                }
                let tr = self.tri(nb)?;
                let c = self.corners(&tr)?;
                if in_circle(c[0], c[1], c[2], q) > 0.0 {
                    self.mark(nb)?;
                    cavity.push(nb);
                }
            }
        }
        let edges = self.border(&mut cavity, q)?;
        for &t in &cavity {
            if let Some(tr) = self.t.get_mut(t) {
                tr.alive = false;
            }
            self.free.push(t);
        }
        self.fan(i, &edges)
    }

    /// The border edges `(a, b, outer)` of the cavity, counter-clockwise.
    /// Grows the cavity across each edge that does not see `q`.
    fn border(&mut self, cavity: &mut Vec<usize>, q: Point2) -> Result<Vec<(usize, usize, usize)>, GeoError> {
        for _ in 0..self.t.len().saturating_add(4) {
            let mut edges = Vec::new();
            let mut grow = None;
            for &t in cavity.iter() {
                let tr = self.tri(t)?;
                for j in 0..3 {
                    let nb = tr.n[j];
                    if nb != NONE && self.seen.get(nb) == Some(&self.stamp) {
                        continue;
                    }
                    let (a, b) = (tr.v[(j + 1) % 3], tr.v[(j + 2) % 3]);
                    if orient(self.pt(a)?, self.pt(b)?, q) <= 0.0 {
                        if nb == NONE {
                            return Err(bad("a point is on the enclosing triangle"));
                        }
                        grow = Some(nb);
                    }
                    edges.push((a, b, nb));
                }
            }
            match grow {
                None => return Ok(edges),
                Some(nb) => {
                    self.mark(nb)?;
                    cavity.push(nb);
                }
            }
        }
        Err(bad("the cavity of a point does not close"))
    }

    /// New triangles `(a, b, i)` on the border edges, linked to each other
    /// and to the outer triangles.
    fn fan(&mut self, i: usize, edges: &[(usize, usize, usize)]) -> Result<usize, GeoError> {
        let mut made = Vec::with_capacity(edges.len());
        for &(a, b, outer) in edges {
            let tr = Tri { v: [a, b, i], n: [NONE, NONE, outer], alive: true };
            let id = match self.free.pop() {
                Some(id) => {
                    *self.t.get_mut(id).ok_or_else(|| bad("a free slot is out of range"))? = tr;
                    id
                }
                None => {
                    self.t.push(tr);
                    self.seen.push(0);
                    self.t.len() - 1
                }
            };
            if let Some(o) = self.t.get_mut(outer) {
                // The outer triangle's slot opposite its vertex that is not a or b.
                if let Some(s) = (0..3).find(|&s| o.v[s] != a && o.v[s] != b) {
                    o.n[s] = id;
                }
            }
            made.push((a, b, id));
        }
        // Across (b, i) is the new triangle that starts at b; across (i, a)
        // is the one that ends at a. The border must be one simple cycle.
        let mut starts: Vec<usize> = made.iter().map(|m| m.0).collect();
        starts.sort_unstable();
        if starts.windows(2).any(|w| w[0] == w[1]) {
            return Err(bad("the cavity border is not one cycle"));
        }
        for &(a, b, id) in &made {
            let next = made.iter().find(|m| m.0 == b).map(|m| m.2).ok_or_else(|| bad("the cavity border is not one cycle"))?;
            let prev = made.iter().find(|m| m.1 == a).map(|m| m.2).ok_or_else(|| bad("the cavity border is not one cycle"))?;
            let tr = self.t.get_mut(id).ok_or_else(|| bad("a new triangle is out of range"))?;
            tr.n[0] = next;
            tr.n[1] = prev;
        }
        made.first().map(|m| m.2).ok_or_else(|| bad("a point made no triangle"))
    }
}

/// The Delaunay triangles of `pts`, each counter-clockwise, as indices into
/// `pts`. The points are inserted in the given order (a caller that sorts
/// them gets an output that depends on the set only). The points must be
/// finite and distinct.
pub(crate) fn triangulate(pts: &[Point2]) -> Result<Vec<[usize; 3]>, GeoError> {
    if pts.len() < 3 {
        return Ok(Vec::new());
    }
    if !pts.iter().all(|p| p.x.is_finite() && p.y.is_finite()) {
        return Err(bad("a point is not finite"));
    }
    let (mut lo, mut hi) = (pts[0], pts[0]);
    for p in pts {
        lo = Point2::new(lo.x.min(p.x), lo.y.min(p.y));
        hi = Point2::new(hi.x.max(p.x), hi.y.max(p.y));
    }
    let d = (hi.x - lo.x).max(hi.y - lo.y).max(1.0);
    let m = Point2::new(0.5 * (lo.x + hi.x), 0.5 * (lo.y + hi.y));
    let n = pts.len();
    let mut p = pts.to_vec();
    p.extend([Point2::new(m.x - 20.0 * d, m.y - d), Point2::new(m.x + 20.0 * d, m.y - d), Point2::new(m.x, m.y + 20.0 * d)]);
    let first = Tri { v: [n, n + 1, n + 2], n: [NONE; 3], alive: true };
    let mut mesh = Mesh { p, t: vec![first], free: Vec::new(), seen: vec![0], stamp: 0 };
    let mut last = 0;
    for i in 0..n {
        last = mesh.insert(i, last)?;
    }
    Ok(mesh.t.iter().filter(|t| t.alive && t.v.iter().all(|&v| v < n)).map(|t| t.v).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_circles(pts: &[Point2], tris: &[[usize; 3]]) -> bool {
        tris.iter().all(|t| {
            let (a, b, c) = (pts[t[0]], pts[t[1]], pts[t[2]]);
            let scale = (b.x - a.x).hypot(b.y - a.y).powi(4).max(1e-30);
            (0..pts.len()).filter(|i| !t.contains(i)).all(|i| in_circle(a, b, c, pts[i]) <= 1e-9 * scale)
        })
    }

    #[test]
    fn a_square_grid_gives_two_triangles_per_cell_and_empty_circles() {
        // Every cell of a grid is four points on one circle: the hard case
        // for the in-circle test.
        let pts: Vec<Point2> = (0..6).flat_map(|i| (0..5).map(move |j| Point2::new(f64::from(i), f64::from(j)))).collect();
        let tris = triangulate(&pts).unwrap();
        assert_eq!(tris.len(), 2 * 5 * 4);
        assert!(tris.iter().all(|t| orient(pts[t[0]], pts[t[1]], pts[t[2]]) > 0.0));
        assert!(empty_circles(&pts, &tris));
        let area: f64 = tris.iter().map(|t| 0.5 * orient(pts[t[0]], pts[t[1]], pts[t[2]])).sum();
        assert!((area - 20.0).abs() < 1e-9, "the hull of the grid is covered: {area}");
    }

    #[test]
    fn scattered_points_give_empty_circles() {
        let mut s = 7u64;
        let mut next = || {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (s >> 11) as f64 / (1u64 << 53) as f64
        };
        let pts: Vec<Point2> = (0..300).map(|_| Point2::new(100.0 * next(), 50.0 * next())).collect();
        let tris = triangulate(&pts).unwrap();
        assert!(tris.iter().all(|t| orient(pts[t[0]], pts[t[1]], pts[t[2]]) > 0.0));
        assert!(empty_circles(&pts, &tris));
    }

    #[test]
    fn too_few_or_bad_points_are_no_triangles_or_refused() {
        assert!(triangulate(&[Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)]).unwrap().is_empty());
        assert!(triangulate(&[Point2::new(0.0, 0.0), Point2::new(1.0, 0.0), Point2::new(f64::NAN, 1.0)]).is_err());
    }
}
