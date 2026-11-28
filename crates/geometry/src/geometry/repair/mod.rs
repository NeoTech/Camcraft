//! Mesh repair (M13.11, the M1.1 "Mesh repair" leftover): vertex welding
//! and hole filling for toolpath-supporting surfaces. Never run on
//! import - the user asks for it by command, and the report says exactly
//! what moved.
//!
//! Provenance: written from published algorithms only. Welding is the
//! classic spatial-hash vertex merge (Teschner et al., "Optimized Spatial
//! Hashing for Collision Detection of Deformable Objects", VMV 2003); the
//! pass shape (weld, drop degenerates, then close loops) follows Botsch,
//! Kobbelt, Pauly, Alliez, Levy, "Polygon Mesh Processing", A K Peters
//! 2010, ch. 8 "Model Repair". Hole filling (`holes`) follows Liepa,
//! "Filling Holes in Meshes", SGP 2003, restricted to the planar case:
//! each boundary loop is triangulated by ear clipping in its best-fit
//! plane (Meisters' two-ears theorem, 1975; Eberly, "Triangulation by Ear
//! Clipping", Geometric Tools 2002; the plane from Newell's method,
//! Sutherland/Sproull/Schumacker 1974). No vendor CAM material consulted.
//!
//! Prime Directive: every entry is total - non-finite input, nonsense
//! tolerances, non-manifold or oversize loops return a named `GeoError`,
//! never a panic. The budget the caller states (`max_edges`) bounds the
//! only superlinear step, the per-loop triangulation.

use std::collections::{HashMap, HashSet};

use super::{Mesh, Point3, Triangle};
use crate::geo2d::GeoError;

mod holes;

pub use holes::fill_holes;

/// Squared facet-normal magnitude under which a triangle counts as
/// zero-area (the `feature` module's degenerate threshold, 1e-12 mm^2 of
/// doubled area).
const ZERO_AREA2: f64 = 1e-24;

/// What a repair pass did, with the mesh it produced. Counts are of
/// distinct vertex positions (bit-exact before welding) and facets.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RepairReport {
    pub vertices_before: usize,
    pub vertices_after: usize,
    pub triangles_before: usize,
    pub triangles_after: usize,
    /// Vertices merged into a neighbour within the weld tolerance.
    pub welded: usize,
    /// Collapsed, zero-area or duplicate triangles removed.
    pub dropped: usize,
    pub holes_filled: usize,
    pub triangles_added: usize,
    pub mesh: Mesh,
}

impl RepairReport {
    /// Did the pass touch anything? `false` is the "nothing to repair"
    /// refusal the command layer voices by name.
    #[must_use]
    pub fn changed(&self) -> bool {
        self.welded > 0 || self.dropped > 0 || self.holes_filled > 0
    }

    /// Fold a later pass (run on `self.mesh`) into this report: the
    /// before-counts stay, the after-counts and the mesh move on.
    #[must_use]
    pub fn then(self, next: RepairReport) -> RepairReport {
        RepairReport {
            vertices_before: self.vertices_before,
            triangles_before: self.triangles_before,
            vertices_after: next.vertices_after,
            triangles_after: next.triangles_after,
            welded: self.welded + next.welded,
            dropped: self.dropped + next.dropped,
            holes_filled: self.holes_filled + next.holes_filled,
            triangles_added: self.triangles_added + next.triangles_added,
            mesh: next.mesh,
        }
    }

    /// One line for a status bar or a model summary.
    #[must_use]
    pub fn describe(&self) -> String {
        format!(
            "welded {} vertex(es), dropped {} triangle(s), filled {} hole(s) (+{} triangle(s)): \
             {} -> {} vertices, {} -> {} triangles",
            self.welded,
            self.dropped,
            self.holes_filled,
            self.triangles_added,
            self.vertices_before,
            self.vertices_after,
            self.triangles_before,
            self.triangles_after
        )
    }
}

/// A shared-vertex view of a facet soup: positions plus index triples.
pub(crate) struct Indexed {
    pub pts: Vec<Point3>,
    pub tris: Vec<[u32; 3]>,
}

impl Indexed {
    /// Back to the facet soup, in the stored facet order.
    pub(crate) fn to_mesh(&self) -> Mesh {
        let facets = self
            .tris
            .iter()
            .map(|t| Triangle::new(self.pts[t[0] as usize], self.pts[t[1] as usize], self.pts[t[2] as usize]))
            .collect();
        Mesh::from_facets(facets)
    }
}

fn check_finite(mesh: &Mesh) -> Result<(), GeoError> {
    if mesh.all_finite() {
        Ok(())
    } else {
        Err(GeoError::Invalid("non-finite vertex coordinate".into()))
    }
}

/// Bit-exact key; -0.0 folds onto +0.0 so the two spellings of zero weld.
fn exact_key(p: Point3) -> [u64; 3] {
    [(p.x + 0.0).to_bits(), (p.y + 0.0).to_bits(), (p.z + 0.0).to_bits()]
}

/// Share bit-equal positions, keep every facet. Refuses non-finite input.
pub(crate) fn index_exact(mesh: &Mesh) -> Result<Indexed, GeoError> {
    check_finite(mesh)?;
    let mut keys: HashMap<[u64; 3], u32> = HashMap::new();
    let mut pts: Vec<Point3> = Vec::new();
    let mut tris = Vec::with_capacity(mesh.len());
    for t in mesh.facets.iter() {
        let mut idx = [0u32; 3];
        for (slot, p) in [t.a, t.b, t.c].into_iter().enumerate() {
            let next = pts.len() as u32;
            idx[slot] = *keys.entry(exact_key(p)).or_insert_with(|| {
                pts.push(p);
                next
            });
        }
        tris.push(idx);
    }
    Ok(Indexed { pts, tris })
}

/// Lattice cell of a coordinate at cell size `cell`, refusing magnitudes
/// the i64 lattice cannot hold (no silent saturation onto one cell).
fn cell_of(x: f64, cell: f64) -> Result<i64, GeoError> {
    let q = (x / cell).floor();
    if q.is_finite() && (-9.0e18..=9.0e18).contains(&q) {
        Ok(q as i64)
    } else {
        Err(GeoError::OutOfRange)
    }
}

/// Merge positions within `tol` by spatial hashing: a vertex joins the
/// first earlier vertex within `tol` found in its own or a neighbouring
/// cell (cell size = `tol`, so a match can only live 1 cell away).
fn index_within(mesh: &Mesh, tol: f64) -> Result<Indexed, GeoError> {
    check_finite(mesh)?;
    let tol2 = tol * tol;
    let mut grid: HashMap<[i64; 3], Vec<u32>> = HashMap::new();
    let mut pts: Vec<Point3> = Vec::new();
    let mut tris = Vec::with_capacity(mesh.len());
    for t in mesh.facets.iter() {
        let mut idx = [0u32; 3];
        for (slot, p) in [t.a, t.b, t.c].into_iter().enumerate() {
            let c = [cell_of(p.x, tol)?, cell_of(p.y, tol)?, cell_of(p.z, tol)?];
            let mut hit = None;
            'cells: for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        let key = [c[0] + dx, c[1] + dy, c[2] + dz];
                        if let Some(bucket) = grid.get(&key) {
                            for &i in bucket {
                                let d = pts[i as usize] - p;
                                if d.dot(d) <= tol2 {
                                    hit = Some(i);
                                    break 'cells;
                                }
                            }
                        }
                    }
                }
            }
            idx[slot] = match hit {
                Some(i) => i,
                None => {
                    let i = pts.len() as u32;
                    pts.push(p);
                    grid.entry(c).or_default().push(i);
                    i
                }
            };
        }
        tris.push(idx);
    }
    Ok(Indexed { pts, tris })
}

/// Doubled area squared of an index triangle; zero for collapsed ones.
fn area2(pts: &[Point3], t: [u32; 3]) -> f64 {
    let (a, b, c) = (pts[t[0] as usize], pts[t[1] as usize], pts[t[2] as usize]);
    let n = (b - a).cross(c - a);
    n.dot(n)
}

/// How `drop_degenerate` settles triangles that share one vertex set
/// (either winding).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Duplicates {
    /// The first copy stays (`weld`): a model's double-sided surface
    /// keeps one side, never none.
    KeepFirst,
    /// By edge parity (`weld_at_f32` only, #4182): see `duplicate_sets`.
    ByParity,
}

/// Drop collapsed and zero-area triangles, then settle duplicates as
/// `duplicates` says. Compacts to the vertices still referenced;
/// returns the dropped count.
fn drop_degenerate(ix: &mut Indexed, duplicates: Duplicates) -> usize {
    let before = ix.tris.len();
    let pts = &ix.pts;
    // A NaN area (overflowing cross product) fails the keep test too.
    ix.tris.retain(|&t| t[0] != t[1] && t[1] != t[2] && t[2] != t[0] && area2(pts, t) > ZERO_AREA2);
    let keep_one = duplicate_sets(&ix.tris, duplicates);
    if !keep_one.is_empty() {
        let mut kept: HashSet<[u32; 3]> = HashSet::with_capacity(keep_one.len());
        ix.tris.retain(|&t| match keep_one.get(&sorted(t)) {
            None => true,
            Some(false) => false,
            Some(true) => kept.insert(sorted(t)),
        });
    }
    let dropped = before - ix.tris.len();
    // Compact: a vertex only degenerate facets used must not linger in
    // the count the report states.
    let mut remap: Vec<Option<u32>> = vec![None; ix.pts.len()];
    let mut pts = Vec::with_capacity(ix.pts.len());
    for t in &mut ix.tris {
        for i in t.iter_mut() {
            let slot = *i as usize;
            *i = match remap.get(slot).copied().flatten() {
                Some(j) => j,
                None => {
                    let j = pts.len() as u32;
                    pts.push(ix.pts[slot]);
                    if let Some(r) = remap.get_mut(slot) {
                        *r = Some(j);
                    }
                    j
                }
            };
        }
    }
    ix.pts = pts;
    dropped
}

fn sorted(t: [u32; 3]) -> [u32; 3] {
    let mut k = t;
    k.sort_unstable();
    k
}

/// Each vertex set that two or more triangles share, and whether one copy
/// stays. `KeepFirst`: always. `ByParity`: unless the triangles outside
/// the set use every one of its three edges an even number of times - a
/// back-to-back pair left where a closed body collapsed at the written
/// corners keeps no copy, a stray copy on a closed solid keeps one.
/// Keeping one of such a pair left its three edges odd, so the weld
/// opened the closed mesh it was meant to check.
fn duplicate_sets(tris: &[[u32; 3]], duplicates: Duplicates) -> HashMap<[u32; 3], bool> {
    let mut copies: HashMap<[u32; 3], u32> = HashMap::new();
    for &t in tris {
        *copies.entry(sorted(t)).or_insert(0) += 1;
    }
    copies.retain(|_, c| *c > 1);
    if duplicates == Duplicates::KeepFirst {
        return copies.into_keys().map(|k| (k, true)).collect();
    }
    if copies.is_empty() {
        return HashMap::new();
    }
    let edges = |k: &[u32; 3]| [(k[0], k[1]), (k[1], k[2]), (k[0], k[2])];
    let mut uses: HashMap<(u32, u32), u32> = copies.keys().flat_map(edges).map(|e| (e, 0)).collect();
    for &t in tris {
        for e in edges(&sorted(t)) {
            if let Some(u) = uses.get_mut(&e) {
                *u += 1;
            }
        }
    }
    copies
        .iter()
        .map(|(k, &c)| {
            let outside_even = edges(k).iter().all(|e| uses.get(e).copied().unwrap_or(0).saturating_sub(c) % 2 == 0);
            (*k, !outside_even)
        })
        .collect()
}

/// The mesh a binary STL holds (#4182): every corner as the `f32` value
/// the file writes. Two corners closer than one `f32` step are distinct
/// in `facets` and one corner in the file, so their facets would be
/// written with a repeated corner, or two facets' corners written as one
/// (two edges become one edge on four facets). When any facet collapses
/// there or any two distinct corners meet, the rounded corners are welded
/// bit-exact (as `weld` at 0): collapsed facets dropped, meeting corners
/// one vertex, and duplicates settled by edge parity, here only: the
/// input is a closed surface, so a duplicate set in it is two faces the
/// rounding laid together. `weld` keeps its first copy (a part model's
/// double-sided surface keeps one side). `None` when no two corners meet:
/// the facets write as they are. Refusals as `weld`'s.
pub fn weld_at_f32(facets: &[Triangle]) -> Result<Option<RepairReport>, GeoError> {
    let q = |p: Point3| Point3::new(f64::from(p.x as f32), f64::from(p.y as f32), f64::from(p.z as f32));
    let collapses = facets.iter().any(|t| {
        let (a, b, c) = (q(t.a), q(t.b), q(t.c));
        a == b || b == c || c == a
    });
    // Adding 0.0 folds -0.0 into 0.0: one position, one key.
    let key = |p: Point3| [(p.x + 0.0).to_bits(), (p.y + 0.0).to_bits(), (p.z + 0.0).to_bits()];
    let mut first: HashMap<[u64; 3], [u64; 3]> = HashMap::new();
    let meet = facets.iter().flat_map(|t| [t.a, t.b, t.c]).any(|p| *first.entry(key(q(p))).or_insert(key(p)) != key(p));
    if !collapses && !meet {
        return Ok(None);
    }
    let rounded = facets.iter().map(|t| Triangle::new(q(t.a), q(t.b), q(t.c))).collect();
    weld_with(&Mesh::from_facets(rounded), 0.0, Duplicates::ByParity).map(Some)
}

/// Weld vertices within `tol` of each other (bit-exact only for `tol`
/// = 0), drop degenerate and duplicate triangles, re-index. The welded
/// mesh keeps each merged vertex at the position of its first
/// occurrence, so nothing moves by more than `tol`.
///
/// Refusals, by name: non-finite coordinates, a negative or non-finite
/// tolerance, coordinates beyond the weld lattice.
pub fn weld(mesh: &Mesh, tol: f64) -> Result<RepairReport, GeoError> {
    weld_with(mesh, tol, Duplicates::KeepFirst)
}

fn weld_with(mesh: &Mesh, tol: f64, duplicates: Duplicates) -> Result<RepairReport, GeoError> {
    if !tol.is_finite() || tol < 0.0 {
        return Err(GeoError::Invalid(format!(
            "weld tolerance must be a finite, non-negative length (got {tol})"
        )));
    }
    let vertices_before = index_exact(mesh)?.pts.len();
    let mut ix = if tol > 0.0 { index_within(mesh, tol)? } else { index_exact(mesh)? };
    let welded = vertices_before.saturating_sub(ix.pts.len());
    let dropped = drop_degenerate(&mut ix, duplicates);
    Ok(RepairReport {
        vertices_before,
        vertices_after: ix.pts.len(),
        triangles_before: mesh.len(),
        triangles_after: ix.tris.len(),
        welded,
        dropped,
        holes_filled: 0,
        triangles_added: 0,
        mesh: ix.to_mesh(),
    })
}

/// Edges used by an odd number of facets after a bit-exact index: zero
/// on a closed surface. The diagnostic `fill_holes` acts on.
#[must_use]
pub fn open_edge_count(mesh: &Mesh) -> usize {
    let Ok(ix) = index_exact(mesh) else {
        return 0;
    };
    let mut count: HashMap<(u32, u32), u32> = HashMap::new();
    for t in &ix.tris {
        for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
            let e = if a <= b { (a, b) } else { (b, a) };
            *count.entry(e).or_insert(0) += 1;
        }
    }
    count.values().filter(|&&c| c % 2 == 1).count()
}
