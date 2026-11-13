//! STEP closed-shell law (#1573): a shell the file declares closed must
//! mesh closed, and a face the kernel returns no mesh for is refused by
//! name - never imported as a model with a wall missing.
//!
//! Provenance: ISO 10303-42 `closed_shell` (and `manifold_solid_brep`,
//! whose outer shell is closed by the schema); watertightness as edge
//! manifoldness - every edge used by exactly two facets, once each way.
//! Our own code; nothing from any CAM vendor.
//!
//! Why: truck skips a face whose boundary it cannot place (no polygon) and
//! trims wrongly a face whose boundary it places wrongly, with no error
//! either way. The mesh then has holes, and the stock simulation and every
//! verdict judge the program against a part with walls missing.

use std::collections::{HashMap, HashSet};

use ruststep::ast::{DataSection, Name, Parameter};

use super::ImportError;
use crate::geometry::{Point3, Triangle};

/// Vertex weld for the edge count, in the file's length unit: truck's own
/// `TOLERANCE`. Two faces evaluate a shared boundary point on their own
/// surfaces, so its copies differ by up to about 1e-7 (#1573).
pub(super) const WELD: f64 = 1e-6;

/// The ids of every shell the file declares closed: each `CLOSED_SHELL`,
/// and each shell a `MANIFOLD_SOLID_BREP` names as its outer.
pub(super) fn closed_shells(section: &DataSection) -> HashSet<u64> {
    let mut closed = HashSet::new();
    for (id, recs) in super::step_units::by_id(section) {
        for r in recs {
            if r.name == "CLOSED_SHELL" {
                closed.insert(id);
            } else if r.name == "MANIFOLD_SOLID_BREP"
                && let Some(Parameter::Ref(Name::Entity(outer))) = super::step_units::args(r).get(1)
            {
                closed.insert(*outer);
            }
        }
    }
    closed
}

/// Edges of `tris` not shared by exactly two facets running opposite
/// ways, and the first such edge's ends. Vertices are welded by
/// neighbourhood within `weld` (the 27 cells around each point), so two
/// copies either side of a rounding step still meet; a facet with two
/// welded corners has collapsed onto an edge and bounds nothing.
pub(super) fn open_edges(tris: &[Triangle], weld: f64) -> (usize, Option<(Point3, Point3)>) {
    let cell = |p: Point3| {
        let c = |v: f64| (v / weld).floor().clamp(-9.0e15, 9.0e15) as i64;
        (c(p.x), c(p.y), c(p.z))
    };
    let mut ids: Vec<Point3> = Vec::new();
    let mut grid: HashMap<(i64, i64, i64), Vec<usize>> = HashMap::new();
    let mut id_of = |p: Point3| -> usize {
        let (cx, cy, cz) = cell(p);
        for dx in -1..=1i64 {
            for dy in -1..=1i64 {
                for dz in -1..=1i64 {
                    let key = (cx.saturating_add(dx), cy.saturating_add(dy), cz.saturating_add(dz));
                    for &i in grid.get(&key).map(Vec::as_slice).unwrap_or(&[]) {
                        if let Some(q) = ids.get(i)
                            && (q.x - p.x).abs() <= weld
                            && (q.y - p.y).abs() <= weld
                            && (q.z - p.z).abs() <= weld
                        {
                            return i;
                        }
                    }
                }
            }
        }
        ids.push(p);
        grid.entry((cx, cy, cz)).or_default().push(ids.len() - 1);
        ids.len() - 1
    };
    // (sum of directions, uses, first seen) per undirected edge.
    let mut balance: HashMap<(usize, usize), (i64, u32, usize)> = HashMap::new();
    let mut order = 0usize;
    for t in tris {
        let k = [id_of(t.a), id_of(t.b), id_of(t.c)];
        if k[0] == k[1] || k[1] == k[2] || k[2] == k[0] {
            continue;
        }
        for i in 0..3 {
            let (a, b) = (k[i], k[(i + 1) % 3]);
            let e = balance.entry((a.min(b), a.max(b))).or_insert((0, 0, order));
            e.0 += if a < b { 1 } else { -1 };
            e.1 = e.1.saturating_add(1);
            order += 1;
        }
    }
    let open: Vec<(&(usize, usize), usize)> = balance
        .iter()
        .filter(|(_, (sum, uses, _))| *sum != 0 || *uses != 2)
        .map(|(e, (_, _, seen))| (e, *seen))
        .collect();
    let first = open
        .iter()
        .min_by_key(|(_, seen)| *seen)
        .and_then(|((a, b), _)| Some((*ids.get(*a)?, *ids.get(*b)?)));
    (open.len(), first)
}

/// The refusal for closed shell `#shell` of `faces` faces meshing with
/// `edges` unmatched edges, the first from `a` to `b` (mm). `dropped`
/// names a face the kernel returned no mesh for, when one is known.
pub(super) fn open_refusal(
    shell: u64,
    faces: usize,
    edges: usize,
    first: Option<(Point3, Point3)>,
    dropped: Option<String>,
) -> ImportError {
    let at = first
        .map(|(a, b)| format!(", the first from ({:.4}, {:.4}, {:.4}) to ({:.4}, {:.4}, {:.4}) mm", a.x, a.y, a.z, b.x, b.y, b.z))
        .unwrap_or_default();
    let cause = dropped.map(|f| format!("; {f} got no mesh from the kernel")).unwrap_or_default();
    ImportError::Step(format!(
        "closed shell #{shell} ({faces} faces) meshes open: {edges} unmatched edges{at}{cause} - \
         the model would import with walls missing; re-export the model, or import an STL"
    ))
}

/// The refusal for a face the kernel returned no mesh for, in a shell the
/// file does not declare closed.
pub(super) fn dropped_refusal(face: &str, shell: u64) -> ImportError {
    ImportError::Step(format!(
        "{face} in shell #{shell} could not be tessellated: the kernel could not place its boundary on its surface \
         and returned no mesh for it - re-export the model, or import an STL"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::cube;

    #[test]
    fn a_cube_is_closed_and_a_cube_less_one_facet_is_not() {
        let m = cube(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 2.0, 3.0));
        assert_eq!(open_edges(&m.facets, WELD).0, 0);
        let (n, first) = open_edges(&m.facets[1..], WELD);
        assert_eq!(n, 3, "one facet gone leaves its three edges unmatched");
        assert!(first.is_some());
    }

    #[test]
    fn copies_of_a_vertex_within_the_weld_still_meet() {
        let m = cube(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));
        let nudge = |p: Point3| Point3::new(p.x + 4e-7, p.y - 4e-7, p.z);
        let mut tris = m.facets.to_vec();
        if let Some(t) = tris.first_mut() {
            *t = Triangle::new(nudge(t.a), nudge(t.b), nudge(t.c));
        }
        assert_eq!(open_edges(&tris, WELD).0, 0);
    }
}
