//! The closed checks on an extracted mesh (#4122): which edges are
//! not where a solid's edges must be. Split from `iso` along the
//! extraction / checking seam; `iso` re-exports both counts.
//! Provenance: `open_edges` and its 1e-9 weld moved unchanged from
//! `iso.rs`; `unpaired_edges` is our own, from #4122.

use std::collections::HashMap;

use crate::geometry::{Point3, Triangle};

/// Edges used by an odd number of triangles: zero on a closed surface.
/// Vertices are welded on the 1e-9 lattice (`extract` interpolates each
/// shared edge from the same two samples, so shared points are equal).
#[must_use]
pub fn open_edges(tris: &[Triangle]) -> usize {
    edge_uses(tris).values().filter(|&&c| c % 2 == 1).count()
}

/// Edges not on exactly two triangles, after the same weld as
/// [`open_edges`] (#4122): zero when every edge has two facets, as a
/// solid's must. An edge on four or six facets (fins, two solids on one
/// edge) passes `open_edges` and counts here. A triangle whose corners
/// meet at the weld leaves an edge from a corner to itself, which counts
/// however many facets it is on.
#[must_use]
pub fn unpaired_edges(tris: &[Triangle]) -> usize {
    edge_uses(tris).iter().filter(|&(&(a, b), &c)| a == b || c != 2).count()
}

/// Each edge's facet count, its corners welded on the 1e-9 lattice.
fn edge_uses(tris: &[Triangle]) -> HashMap<([i64; 3], [i64; 3]), u32> {
    let key = |p: Point3| [(p.x * 1e9).round() as i64, (p.y * 1e9).round() as i64, (p.z * 1e9).round() as i64];
    let mut count: HashMap<([i64; 3], [i64; 3]), u32> = HashMap::new();
    for t in tris {
        for (a, b) in [(t.a, t.b), (t.b, t.c), (t.c, t.a)] {
            let (ka, kb) = (key(a), key(b));
            let e = if ka <= kb { (ka, kb) } else { (kb, ka) };
            *count.entry(e).or_insert(0) += 1;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::cube;

    #[test]
    fn two_cubes_on_one_edge_pass_open_edges_and_count_as_unpaired() {
        // #4122: the shared edge is on four facets, an even count.
        let a = cube(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));
        let b = cube(Point3::new(1.0, 1.0, 0.0), Point3::new(2.0, 2.0, 1.0));
        assert_eq!(unpaired_edges(&a.facets), 0, "one cube is a solid");
        let both: Vec<Triangle> = a.facets.iter().chain(b.facets.iter()).copied().collect();
        assert_eq!(open_edges(&both), 0);
        assert_eq!(unpaired_edges(&both), 1, "the shared edge");
    }

    #[test]
    fn a_facet_collapsed_at_the_weld_counts_as_unpaired() {
        let mut tris = cube(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0)).facets.to_vec();
        // Two facets whose first two corners meet at the weld, their third
        // corners apart: after the weld each is the edge v-v once and its
        // long edge twice, so every edge but v-v is on exactly two facets,
        // and v-v is on two as well. Only the corner-to-itself rule counts it.
        let (v, w) = (Point3::new(0.5, 0.5, 0.5), Point3::new(0.5 + 1e-12, 0.5, 0.5));
        tris.push(Triangle::new(v, w, Point3::new(0.25, 0.5, 0.5)));
        tris.push(Triangle::new(v, w, Point3::new(0.5, 0.25, 0.5)));
        assert_eq!(open_edges(&tris), 0);
        assert_eq!(unpaired_edges(&tris), 1, "the edge from v to itself");
    }
}
