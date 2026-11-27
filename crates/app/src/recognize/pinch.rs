//! The rings of a level face whose boundary walk met a knot (#2440): a
//! hole tangent to the outer ring shares one vertex with it, and the
//! feature walk reports every ring through that vertex as open, so the
//! face had no outer ring and no pocket or slot was read off it. The
//! split is taken only when it is proved: each ring is simple, each of
//! its spans is a boundary edge of the face, the inner rings stand inside
//! the largest ring, and the largest ring's area less the inner rings'
//! areas is the face's facet area. Anything else stays refused.
//!
//! Provenance: our own code; the area identity of a polygon with holes
//! (shoelace / Newell's vector area, Sutherland, Sproull & Schumacker
//! 1974) and the even-odd point-in-polygon rule. No vendor material
//! consulted.

use super::walls::Mesh;
use crate::geometry::feature::{Face, Features, Loop, LoopKind};
use std::collections::HashSet;

/// Relative slack of the area identity (tessellation round-off).
const AREA_REL: f64 = 1e-3;
/// Absolute slack of the area identity, mm².
const AREA_ABS: f64 = 1e-6;

/// The outer ring and the inner rings of face `fi`: the walk's own when
/// it closed every ring, else the proved split of its knotted rings.
/// None when the face has no ring that can be trusted as its outline.
pub(super) fn rings(mesh: &Mesh, f: &Features, fi: usize, face: &Face) -> Option<(Loop, Vec<Loop>)> {
    if !face.loops.iter().any(|l| matches!(l.kind, LoopKind::Open)) {
        let outer = face.loops.iter().find(|l| matches!(l.kind, LoopKind::Outer))?.clone();
        let inner = face.loops.iter().filter(|l| matches!(l.kind, LoopKind::Hole)).cloned().collect();
        return Some((outer, inner));
    }
    proved(mesh, f, fi, face)
}

fn proved(mesh: &Mesh, f: &Features, fi: usize, face: &Face) -> Option<(Loop, Vec<Loop>)> {
    let mut areas = Vec::with_capacity(face.loops.len());
    for l in &face.loops {
        let n = l.verts.len();
        if n < 3 || l.verts.iter().collect::<HashSet<_>>().len() != n {
            return None;
        }
        for i in 0..n {
            let (Some(&a), Some(&b)) = (l.verts.get(i), l.verts.get((i + 1) % n)) else { return None };
            if !mesh.bounds_face(f, fi, a, b) {
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
