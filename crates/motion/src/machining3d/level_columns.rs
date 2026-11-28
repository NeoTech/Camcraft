//! The exact column law of the 3D Pocket's picks (#3329, split from
//! `level_seam`): the tool's cylinder at a point that a pick moved, from
//! the cut height up to the clearance plane, keeps the tool radius from
//! every facet above its foot. Exact: each facet is clipped to the
//! half-space above the foot, then the distance from the column's axis
//! segment to what is left is measured (`stock_sim::dist_seg_tri`).
//!
//! Provenance: the closest distance of a segment and a triangle
//! (Ericson, "Real-Time Collision Detection" 5.1, through `stock_sim`)
//! and the clip of a polygon by a half-space (Sutherland and Hodgman
//! 1974). No vendor material consulted.

use super::level_seam::COLUMN_TOL;

use crate::geo2d::Point2;
use crate::geometry::{Point3, Triangle};
use crate::machining2d::GenError;
use crate::stock_sim::dist_seg_tri;

/// The exact column law at each point a pick moved: the tool's
/// cylinder of radius `r` from the cut height `tip(q)` up to the
/// clearance plane keeps the radius (less [`COLUMN_TOL`]) from every
/// facet. Material at or under the cut height is the floor the tool
/// stands on, so each facet is first clipped to the half-space above
/// the foot; the distance from the column's axis segment to what is
/// left is then exactly the clearance of the cylinder. No sampling.
pub(super) fn columns(points: &[Point2], tip: impl Fn(Point2) -> f64, r: f64, clearance_z: f64, facets: &[Triangle]) -> Result<(), GenError> {
    for &q in points {
        let z0 = tip(q);
        if let Some(z) = column_hit(q, z0, r, clearance_z, facets) {
            return Err(GenError::Invalid(format!(
                "start / exit point: the plunge or retract column at ({:.3}, {:.3}) from Z {z0:.3} comes within the tool \
                 radius {r:.3} mm of the part at Z {z:.3} - pick another point or clear the start or exit point",
                q.x, q.y
            )));
        }
    }
    Ok(())
}

/// The lowest height at which the column at `q` from `z0` up to the
/// clearance plane meets the part within `r - COLUMN_TOL`, or `None`.
pub(super) fn column_hit(q: Point2, z0: f64, r: f64, clearance_z: f64, facets: &[Triangle]) -> Option<f64> {
    let top = clearance_z.max(z0);
    let (a, b) = (Point3::new(q.x, q.y, z0), Point3::new(q.x, q.y, top));
    let limit = r - COLUMN_TOL;
    // A shelf within one lattice micron of the foot is the floor under
    // the tool (the islands law's own nudge, `rest::EPS`).
    let floor = z0 + super::rest::EPS;
    let far = |t: &Triangle| {
        let (xs, ys, zs) = ([t.a.x, t.b.x, t.c.x], [t.a.y, t.b.y, t.c.y], [t.a.z, t.b.z, t.c.z]);
        let lo = |v: [f64; 3]| v.iter().copied().fold(f64::INFINITY, f64::min);
        let hi = |v: [f64; 3]| v.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        hi(zs) <= floor || lo(xs) > q.x + r || hi(xs) < q.x - r || lo(ys) > q.y + r || hi(ys) < q.y - r
    };
    let mut hit: Option<f64> = None;
    for t in facets.iter().filter(|t| !far(t)) {
        for piece in clip_above(t, floor) {
            // A distance that is not a number is a hit: fail safe.
            let d = dist_seg_tri(a, b, &piece);
            if d.is_nan() || d < limit {
                let z = piece.a.z.min(piece.b.z).min(piece.c.z);
                hit = Some(hit.map_or(z, |h| h.min(z)));
            }
        }
    }
    hit
}

/// The part of triangle `t` at or above the plane `z0`, as up to two
/// triangles (Sutherland-Hodgman against one half-space).
fn clip_above(t: &Triangle, z0: f64) -> Vec<Triangle> {
    let v = [t.a, t.b, t.c];
    let mut poly: Vec<Point3> = Vec::with_capacity(4);
    for i in 0..3 {
        let (p, q) = (v[i], v[(i + 1) % 3]);
        let (pin, qin) = (p.z >= z0, q.z >= z0);
        if pin {
            poly.push(p);
        }
        if pin != qin {
            let s = (z0 - p.z) / (q.z - p.z);
            poly.push(Point3::new(p.x + (q.x - p.x) * s, p.y + (q.y - p.y) * s, z0));
        }
    }
    match poly.len() {
        3 => vec![Triangle::new(poly[0], poly[1], poly[2])],
        4 => vec![Triangle::new(poly[0], poly[1], poly[2]), Triangle::new(poly[0], poly[2], poly[3])],
        _ => Vec::new(),
    }
}

