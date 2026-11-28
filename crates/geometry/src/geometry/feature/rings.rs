//! The outer ring and the holes of a planar face, also when the boundary
//! walk met a knot (#2440): a hole tangent to the outer ring shares one
//! vertex with it, and [`super::build_loops`] reports each ring through
//! that vertex as [`LoopKind::Open`]. Such a split is accepted only when
//! it is proved: each ring is simple, each of its spans is a feature edge
//! between the face and another face, each inner ring has a vertex off the
//! outer ring and inside it, and the largest ring's area less the inner
//! rings' areas is the face's facet area. Anything else stays refused.
//! The walk itself does not change, so no [`super::FaceId`] moves.
//!
//! Provenance: our own code; the area identity of a polygon with holes
//! (shoelace / Newell's vector area, Sutherland, Sproull & Schumacker
//! 1974) and the even-odd point-in-polygon rule. No vendor material
//! consulted.

use super::{Face, Features, Loop, LoopKind};
use std::collections::HashSet;

/// Relative slack of the area identity (tessellation round-off).
const AREA_REL: f64 = 1e-3;
/// Absolute slack of the area identity, mm².
const AREA_ABS: f64 = 1e-6;

/// The outer ring and the inner rings of face `fi`: the walk's own when
/// it closed every ring, else the proved split of its knotted rings.
/// None when the face has no ring that can be trusted as its outline.
pub fn face_rings(f: &Features, fi: usize) -> Option<(Loop, Vec<Loop>)> {
    let face = f.faces.get(fi)?;
    if !face.loops.iter().any(|l| matches!(l.kind, LoopKind::Open)) {
        let outer = face.loops.iter().find(|l| matches!(l.kind, LoopKind::Outer))?.clone();
        let inner = face.loops.iter().filter(|l| matches!(l.kind, LoopKind::Hole)).cloned().collect();
        return Some((outer, inner));
    }
    proved(f, fi, face)
}

fn proved(f: &Features, fi: usize, face: &Face) -> Option<(Loop, Vec<Loop>)> {
    let mut areas = Vec::with_capacity(face.loops.len());
    for l in &face.loops {
        let n = l.verts.len();
        if n < 3 || l.verts.iter().collect::<HashSet<_>>().len() != n {
            return None;
        }
        for i in 0..n {
            let (Some(&a), Some(&b)) = (l.verts.get(i), l.verts.get((i + 1) % n)) else { return None };
            if !bounds_face(f, fi, a, b) {
                return None;
            }
        }
        areas.push(vector_area(f, &l.verts)?);
    }
    let big = (0..areas.len()).max_by(|&i, &j| norm(areas[i]).total_cmp(&norm(areas[j])))?;
    let outer_area = norm(*areas.get(big)?);
    let inner_area: f64 = areas.iter().enumerate().filter(|(i, _)| *i != big).map(|(_, a)| norm(*a)).sum();
    let net = outer_area - inner_area;
    if !(net.is_finite() && face.area.is_finite() && (net - face.area).abs() <= AREA_REL * face.area + AREA_ABS) {
        return None;
    }
    let outer = face.loops.get(big)?;
    let axis = dominant(*areas.get(big)?);
    let poly: Vec<(f64, f64)> = outer.verts.iter().map(|&v| project(f, v, axis)).collect::<Option<_>>()?;
    let on_outer: HashSet<u32> = outer.verts.iter().copied().collect();
    let mut inner = Vec::with_capacity(face.loops.len().saturating_sub(1));
    for (i, l) in face.loops.iter().enumerate() {
        if i == big {
            continue;
        }
        let free: Vec<u32> = l.verts.iter().copied().filter(|v| !on_outer.contains(v)).collect();
        if free.is_empty() {
            return None;
        }
        for v in free {
            let (x, y) = project(f, v, axis)?;
            if !inside(x, y, &poly) {
                return None;
            }
        }
        inner.push(Loop { kind: LoopKind::Hole, verts: l.verts.clone() });
    }
    Some((Loop { kind: LoopKind::Outer, verts: outer.verts.clone() }, inner))
}

/// The welded vertices `a` and `b` span a feature edge of face `fi`: one
/// of its facets is in the face and one is in another face.
fn bounds_face(f: &Features, fi: usize, a: u32, b: u32) -> bool {
    let key = if a < b { (a, b) } else { (b, a) };
    let Ok(at) = f.edges.binary_search_by_key(&key, |e| (e.a, e.b)) else {
        return false;
    };
    let Some(e) = f.edges.get(at) else { return false };
    let on = e.facets.iter().take((e.n_facets as usize).min(3));
    let mine = |t: &u32| f.facet_face.get(*t as usize).is_some_and(|&g| g as usize == fi);
    let (mut any_mine, mut any_other) = (false, false);
    for t in on {
        if mine(t) {
            any_mine = true;
        } else {
            any_other = true;
        }
    }
    any_mine && any_other
}

fn norm(a: [f64; 3]) -> f64 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

/// Half Newell's normal of a ring: its vector area, any plane.
fn vector_area(f: &Features, ring: &[u32]) -> Option<[f64; 3]> {
    let n = ring.len();
    let mut s = [0.0; 3];
    for i in 0..n {
        let p = f.verts.get(*ring.get(i)? as usize)?;
        let q = f.verts.get(*ring.get((i + 1) % n)? as usize)?;
        s[0] += p[1] * q[2] - p[2] * q[1];
        s[1] += p[2] * q[0] - p[0] * q[2];
        s[2] += p[0] * q[1] - p[1] * q[0];
    }
    let a = s.map(|c| 0.5 * c);
    a.iter().all(|c| c.is_finite()).then_some(a)
}

/// The axis the ring's plane is most nearly normal to (0 = X).
fn dominant(a: [f64; 3]) -> usize {
    let m = a.map(f64::abs);
    if m[0] >= m[1] && m[0] >= m[2] {
        0
    } else if m[1] >= m[2] {
        1
    } else {
        2
    }
}

/// Vertex `v` with axis `drop` left out.
fn project(f: &Features, v: u32, drop: usize) -> Option<(f64, f64)> {
    let p = f.verts.get(v as usize)?;
    Some(match drop {
        0 => (p[1], p[2]),
        1 => (p[2], p[0]),
        _ => (p[0], p[1]),
    })
}

/// Even-odd point-in-polygon.
fn inside(x: f64, y: f64, poly: &[(f64, f64)]) -> bool {
    let n = poly.len();
    let mut is_in = false;
    for i in 0..n {
        let (Some(&(x1, y1)), Some(&(x2, y2))) = (poly.get(i), poly.get((i + 1) % n)) else { continue };
        if (y1 > y) != (y2 > y) && x < (y - y1) * (x2 - x1) / (y2 - y1) + x1 {
            is_in = !is_in;
        }
    }
    is_in
}

#[cfg(test)]
mod tests {
    use super::super::features_of_facets;
    use super::*;
    use crate::geometry::{Point3, Triangle};

    type P = (f64, f64);

    /// A level floor at z 0 on [0, 10] x [-20, 20], with a wall down to
    /// z -1 under each boundary edge. With `tangent`, a diamond hole
    /// (centre (7, 0), half diagonal 3) touches the edge x = 10 at
    /// (10, 0). `drop_one` leaves one floor facet out.
    fn floor(tangent: bool, drop_one: bool) -> Vec<Triangle> {
        let mut out = Vec::new();
        let mut tri = |a: Point3, b: Point3, c: Point3, n: [f64; 3]| {
            let (u, v) = (b - a, c - a);
            let k = [u.y * v.z - u.z * v.y, u.z * v.x - u.x * v.z, u.x * v.y - u.y * v.x];
            out.push(if k[0] * n[0] + k[1] * n[1] + k[2] * n[2] > 0.0 { Triangle::new(a, b, c) } else { Triangle::new(a, c, b) });
        };
        let at = |p: P, z: f64| Point3::new(p.0, p.1, z);
        let (r, t, l, b) = ((10.0, 0.0), (7.0, 3.0), (4.0, 0.0), (7.0, -3.0));
        let mut outer = vec![(0.0, -20.0), (10.0, -20.0), (10.0, 0.0), (10.0, 20.0), (0.0, 20.0), (0.0, 0.0)];
        let mut floor_tris: Vec<[P; 3]> = Vec::new();
        if tangent {
            // Two fans, each star-shaped from its apex.
            for (apex, ring) in [((0.0, -20.0), vec![(10.0, -20.0), r, b, l, (0.0, 0.0)]), ((0.0, 20.0), vec![(0.0, 0.0), l, t, r, (10.0, 20.0)])] {
                for w in ring.windows(2) {
                    floor_tris.push([apex, w[0], w[1]]);
                }
            }
        } else {
            outer = vec![(0.0, -20.0), (10.0, -20.0), (10.0, 20.0), (0.0, 20.0)];
            floor_tris = vec![[outer[0], outer[1], outer[2]], [outer[0], outer[2], outer[3]]];
        }
        if drop_one {
            floor_tris.pop();
        }
        for f in &floor_tris {
            tri(at(f[0], 0.0), at(f[1], 0.0), at(f[2], 0.0), [0.0, 0.0, 1.0]);
        }
        let mut wall = |p: P, q: P, n: [f64; 3]| {
            tri(at(p, 0.0), at(q, 0.0), at(q, -1.0), n);
            tri(at(p, 0.0), at(q, -1.0), at(p, -1.0), n);
        };
        let m = outer.len();
        for i in 0..m {
            let (p, q) = (outer[i], outer[(i + 1) % m]);
            wall(p, q, [q.1 - p.1, p.0 - q.0, 0.0]);
        }
        if tangent {
            let c = (7.0, 0.0);
            for (p, q) in [(r, b), (b, l), (l, t), (t, r)] {
                wall(p, q, [c.0 - 0.5 * (p.0 + q.0), c.1 - 0.5 * (p.1 + q.1), 0.0]);
            }
        }
        out
    }

    fn floor_index(f: &Features) -> usize {
        f.faces.iter().position(|face| face.normal[2].abs() > 0.999 &&face.offset.abs() < 1e-9).expect("the floor face")
    }

    #[test]
    fn a_closed_face_keeps_the_walk_rings() {
        let f = features_of_facets(&floor(false, false));
        let (outer, inner) = face_rings(&f, floor_index(&f)).expect("rings");
        assert_eq!(outer.verts.len(), 4);
        assert!(inner.is_empty());
    }

    #[test]
    fn a_tangent_hole_splits_into_the_outer_ring_and_one_hole() {
        let f = features_of_facets(&floor(true, false));
        let fi = floor_index(&f);
        assert!(f.faces[fi].loops.iter().any(|l| matches!(l.kind, LoopKind::Open)), "the walk meets the knot: {:?}", f.faces[fi].loops);
        let (outer, inner) = face_rings(&f, fi).expect("the split is proved");
        assert_eq!(outer.kind, LoopKind::Outer);
        assert_eq!(outer.verts.len(), 6, "{outer:?}");
        assert_eq!(inner.len(), 1);
        assert_eq!(inner[0].verts.len(), 4, "{inner:?}");
        assert!((f.faces[fi].area - (400.0 - 18.0)).abs() < 1e-9);
    }

    #[test]
    fn a_knotted_face_that_fails_the_proof_is_refused() {
        // One floor facet is gone: its edges bound no other face and the
        // area identity fails, so the open rings stay refused.
        let f = features_of_facets(&floor(true, true));
        assert!(face_rings(&f, floor_index(&f)).is_none());
        assert!(face_rings(&f, usize::MAX).is_none());
    }
}
