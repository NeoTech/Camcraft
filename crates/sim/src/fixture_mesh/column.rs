//! The steel a closed fixture mesh puts on one vertical line (#1530):
//! the Z spans where the line `x = x0, y = y0` runs inside the solid.
//! The carried-workholding gate reads it per carried-stock column, so a
//! vise is judged by its jaws, not by the box that spans the part they
//! grip, and a nest pocket by its walls and floor.
//!
//! Provenance: ray-crossing inclusion with signed crossings along one
//! axis (the winding number of a closed oriented surface; Haines, "Point
//! in Polygon Strategies", Graphics Gems IV, 1994; O'Rourke,
//! "Computational Geometry in C", 2nd ed., ch. 7) - the same reading as
//! [`super::ray`], cast along +Z through every crossing instead of from
//! one point. Barycentric point-in-triangle in the XY projection is
//! textbook (Ericson, "Real-Time Collision Detection", 2005, 3.4). Our
//! own code.
//!
//! ## Why the answer is conservative
//!
//! A line through facet interiors only, transversally, reads the
//! winding exactly. A line through a facet edge or vertex, or running
//! in a facet's plane (a vertical wall face), is AMBIGUOUS: the line is
//! nudged a millionth of the mesh's scale and read again. When every nudge is
//! ambiguous the answer is `None` and the caller judges by the box.

use super::bvh::tri_box;
use super::FixtureMesh;
use super::gjk::{V, cross, dot, sub};

/// Barycentric margin under which a crossing is "at an edge".
const EDGE_EPS: f64 = 1e-7;
/// Sine of the angle under which a facet stands "vertical" to the line.
const GRAZE_SIN: f64 = 1e-6;
/// The nudges tried when a line is ambiguous, in units of
/// [`nudge_unit`]: deliberately off-axis, so a line on a facet diagonal
/// (an axis-aligned grid meets them all) moves off it.
const NUDGES: [(f64, f64); 5] = [(0.0, 0.0), (3.7, 1.3), (-2.9, 4.1), (1.7, -5.3), (-4.3, -2.3)];

/// One nudge step: a millionth of the mesh's coordinate scale (at least
/// a nanometre) - large enough to clear [`EDGE_EPS`] on a facet as long
/// as the mesh, and microns at most on a machine-table-sized mesh, far
/// below any contact tolerance a caller judges with.
fn nudge_unit(mesh: &FixtureMesh) -> f64 {
    (1e3 * mesh.pad).max(1e-6)
}

/// Distance from `q` to segment `ab` in the XY plane.
fn seg_dist(q: (f64, f64), a: V, b: V) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 0.0 { (((q.0 - a[0]) * dx + (q.1 - a[1]) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
    let (px, py) = (a[0] + t * dx - q.0, a[1] + t * dy - q.1);
    (px * px + py * py).sqrt()
}

/// One line's crossings `(z, +1 entering / -1 leaving)` or None when the
/// line is ambiguous.
fn crossings(mesh: &FixtureMesh, x: f64, y: f64, budget: &mut u64) -> Option<Vec<(f64, i32)>> {
    let pad = mesh.pad;
    let mut out: Vec<(f64, i32)> = Vec::new();
    let mut ambiguous = false;
    mesh.bvh.walk(
        |n| n.lo[0] - pad <= x && x <= n.hi[0] + pad && n.lo[1] - pad <= y && y <= n.hi[1] + pad,
        |t| {
            // Out of budget: unread, never a guess.
            let Some(left) = budget.checked_sub(1) else {
                ambiguous = true;
                return true;
            };
            *budget = left;
            let Some(tri) = mesh.tris.get(t) else { return false };
            if mesh.flat.get(t).copied().unwrap_or(false) {
                return false; // no interior to cross
            }
            let [a, b, c] = *tri;
            let (lo, hi) = tri_box(tri);
            if x < lo[0] - pad || x > hi[0] + pad || y < lo[1] - pad || y > hi[1] + pad {
                return false;
            }
            let n = cross(sub(b, a), sub(c, a));
            let nn = dot(n, n).sqrt();
            if !(n[2].is_finite() && nn.is_finite()) {
                ambiguous = true;
                return true;
            }
            if n[2].abs() <= GRAZE_SIN * nn {
                // A (near-)vertical facet: the line crosses it only by
                // running in its plane - ambiguous when on it.
                let q = (x, y);
                if seg_dist(q, a, b).min(seg_dist(q, b, c)).min(seg_dist(q, c, a)) <= pad {
                    ambiguous = true;
                    return true;
                }
                return false;
            }
            // Barycentric coordinates in the XY projection.
            let det = n[2];
            let u = ((b[0] - x) * (c[1] - y) - (c[0] - x) * (b[1] - y)) / det;
            let v = ((c[0] - x) * (a[1] - y) - (a[0] - x) * (c[1] - y)) / det;
            let w = 1.0 - u - v;
            if !(u.is_finite() && v.is_finite()) {
                ambiguous = true;
                return true;
            }
            if u < -EDGE_EPS || v < -EDGE_EPS || w < -EDGE_EPS {
                return false;
            }
            if u <= EDGE_EPS || v <= EDGE_EPS || w <= EDGE_EPS {
                ambiguous = true; // through an edge or a vertex
                return true;
            }
            let z = u * a[2] + v * b[2] + w * c[2];
            // An outward normal pointing down is where an upward line enters.
            out.push((z, if n[2] < 0.0 { 1 } else { -1 }));
            false
        },
    );
    (!ambiguous).then_some(out)
}

/// The Z spans inside the solid along one line: nonzero winding or odd
/// crossing count (overlapping shells, an inside-out shell), as
/// [`super::ray`] reads a point.
fn spans_of(mut xs: Vec<(f64, i32)>) -> Vec<(f64, f64)> {
    xs.sort_by(|p, q| p.0.total_cmp(&q.0));
    let mut out: Vec<(f64, f64)> = Vec::new();
    let (mut wind, mut count) = (0i64, 0u64);
    for pair in xs.windows(2) {
        let (Some(&(z0, s)), Some(&(z1, _))) = (pair.first(), pair.get(1)) else { continue };
        wind += i64::from(s);
        count += 1;
        if (wind != 0 || count % 2 == 1) && z1 > z0 {
            match out.last_mut() {
                Some(last) if last.1 >= z0 => last.1 = z1,
                _ => out.push((z0, z1)),
            }
        }
    }
    out
}

impl FixtureMesh {
    /// The Z spans, ascending and disjoint, where the vertical line
    /// through `(x, y)` runs inside the solid; empty where it misses the
    /// steel. An ambiguous line is nudged a millionth of the mesh's
    /// scale and read again; `None` when every nudge is ambiguous, the
    /// input is not finite, or `budget` runs out - the caller then judges
    /// by the box. `budget` is the facets the caller lets the query
    /// visit, shared across calls (each visit takes one): a fan of slim
    /// facets that every line meets stays bounded in time.
    #[must_use]
    pub fn steel_on_line(&self, x: f64, y: f64, budget: &mut u64) -> Option<Vec<(f64, f64)>> {
        if !(x.is_finite() && y.is_finite()) {
            return None;
        }
        let u = nudge_unit(self);
        for (dx, dy) in NUDGES {
            if *budget == 0 {
                return None;
            }
            if let Some(xs) = crossings(self, x + u * dx, y + u * dy, budget) {
                return Some(spans_of(xs));
            }
        }
        None
    }
}
