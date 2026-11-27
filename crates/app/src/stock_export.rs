//! What the program leaves, as a mesh (M7.2 D): the traced stock's
//! field iso-extracted (`stock_sim::iso`) and written as binary STL.
//! Provenance: the binary STL layout (80-byte header, u32 count, 50
//! bytes a facet - the 3D Systems format as publicly documented) and
//! the M7.1 field; our own code, nothing from vendor binaries.
//!
//! ## The laws here
//!
//! - **The mesh is a cache of the field, never a second truth.** It is
//!   derived from the same `FieldView` the viewport traces, and keyed by
//!   it (`MeshCache`): a new view - any cut appended, any edit re-armed
//!   - is a new mesh; nothing reads the triangles back as material.
//! - **A hole is a refusal.** An extraction with an edge not on exactly
//!   two facets (a hole, or fins on one edge: #4122) is never written:
//!   the file would claim a solid it is not.

use std::path::Path;
use std::sync::Arc;

use crate::geometry::{repair, Triangle};
use crate::stock_sim::iso;
use crate::stock_view::FieldView;

/// The machined stock's closed surface, extracted at the view's cell.
pub fn machined_mesh(fv: &FieldView) -> Result<Vec<Triangle>, String> {
    let field = fv.stock.field().ok_or("the stock carries no field to extract")?;
    let tris = iso::extract(field).map_err(|e| e.to_string())?;
    if tris.is_empty() {
        return Err("the program leaves no material - nothing to export".into());
    }
    // #4122: a solid's every edge is on two facets; an open edge, or one
    // on four or more facets, is not a solid.
    let unpaired = iso::unpaired_edges(&tris);
    if unpaired > 0 {
        return Err(format!("the extracted surface has {unpaired} edges not on two facets - refused, not written"));
    }
    Ok(tris)
}

/// Binary STL bytes for `tris` (normals from the winding, f32).
#[must_use]
pub fn stl_bytes(tris: &[Triangle]) -> Vec<u8> {
    let mut b = Vec::with_capacity(84 + tris.len() * 50);
    let mut header = [0u8; 80];
    let tag = b"camcraft machined stock";
    header[..tag.len()].copy_from_slice(tag);
    b.extend_from_slice(&header);
    let n = u32::try_from(tris.len()).unwrap_or(u32::MAX);
    b.extend_from_slice(&n.to_le_bytes());
    for t in tris.iter().take(n as usize) {
        let nrm = t.normal();
        for p in [nrm, t.a, t.b, t.c] {
            for c in [p.x, p.y, p.z] {
                b.extend_from_slice(&(c as f32).to_le_bytes());
            }
        }
        b.extend_from_slice(&[0, 0]);
    }
    b
}

/// The binary STL of `tris` as the file will hold it, and its facet
/// count (#4182). A corner is written as `f32`, so corners closer than one
/// `f32` step are one corner in the file: when any facet collapses there
/// or any two distinct corners meet, the corners are welded at their
/// written values (`repair::weld_at_f32`) and the result is checked again,
/// so a file that is not a closed solid is refused by name, never written.
/// Facets whose corners all stay distinct are written byte for byte as
/// `stl_bytes` writes them.
pub fn file_bytes(tris: &[Triangle]) -> Result<(Vec<u8>, usize), String> {
    let welded = repair::weld_at_f32(tris).map_err(|e| format!("the mesh cannot be welded at its written corners: {e}"))?;
    let written: &[Triangle] = match &welded {
        None => tris,
        Some(r) => &r.mesh.facets,
    };
    if written.is_empty() {
        return Err("the mesh collapses to nothing at the corners binary STL writes - nothing to export".into());
    }
    let unpaired = iso::unpaired_edges(written);
    if unpaired > 0 {
        return Err(format!("as written, the surface has {unpaired} edges not on two facets - refused, not written"));
    }
    if u32::try_from(written.len()).is_err() {
        return Err("the mesh has more facets than binary STL can count".into());
    }
    Ok((stl_bytes(written), written.len()))
}

/// Extract and write `fv`'s machined stock to `path`; the facet count.
pub fn write_stl(fv: &FieldView, path: &Path) -> Result<usize, String> {
    let (bytes, n) = file_bytes(&machined_mesh(fv)?)?;
    std::fs::write(path, bytes).map_err(|e| format!("could not write {}: {e}", path.display()))?;
    Ok(n)
}

/// The mesh cache: one extraction per field view, rebuilt when the view
/// changes (identity of the `Arc` the viewport holds - the toolpath
/// cache's keyed law, applied to the field). A refusal is remembered
/// for its view exactly like a mesh: a frame that asks again gets the
/// same answer, never a second extraction (M13.1 owed: the viewer reads
/// this cache every frame).
#[derive(Default)]
pub struct MeshCache {
    key: Option<Arc<FieldView>>,
    mesh: Option<Result<Arc<Vec<Triangle>>, String>>,
    /// The mesh's per-facet deviation classes (#25), judged once by
    /// `classify` and retired with the view they belong to.
    classes: Option<Arc<Vec<f32>>>,
}

impl MeshCache {
    /// The mesh of `fv`, extracting only when `fv` is not the cached view.
    pub fn get(&mut self, fv: &Arc<FieldView>) -> Result<Arc<Vec<Triangle>>, String> {
        if let (Some(k), Some(m)) = (&self.key, &self.mesh)
            && Arc::ptr_eq(k, fv)
        {
            return m.clone();
        }
        self.key = None;
        self.mesh = None;
        self.classes = None;
        let m = machined_mesh(fv).map(Arc::new);
        self.key = Some(Arc::clone(fv));
        self.mesh = Some(m.clone());
        m
    }

    /// The viewer's read (M13.1 owed): the mesh already extracted for
    /// this very view, or `None` - another view, a refusal, nothing
    /// asked yet. Never extracts: the viewport collects its scene every
    /// frame, and an extraction is `get`'s, once per view.
    #[must_use]
    pub fn viewed(&self, fv: &Arc<FieldView>) -> Option<Arc<Vec<Triangle>>> {
        match (&self.key, &self.mesh) {
            (Some(k), Some(Ok(m))) if Arc::ptr_eq(k, fv) => Some(Arc::clone(m)),
            _ => None,
        }
    }

    /// Judge the cached mesh of `fv` against its model, once (#25):
    /// a no-op when the classes already stand, or when `fv` is not the
    /// cached view with a mesh (nothing to judge; never an extraction).
    pub fn classify(&mut self, fv: &Arc<FieldView>) {
        if self.classes.is_some() {
            return;
        }
        if let Some(m) = self.viewed(fv) {
            self.classes = Some(Arc::new(crate::machined_classes::mesh_classes(fv, &m)));
        }
    }

    /// The classes `classify` judged for this very view's mesh, one per
    /// facet; `None` for another view or before the judgement.
    #[must_use]
    pub fn viewed_classes(&self, fv: &Arc<FieldView>) -> Option<Arc<Vec<f32>>> {
        self.viewed(fv)?;
        self.classes.as_ref().map(Arc::clone)
    }

    /// Drop the cached view, mesh and classes.
    pub fn clear(&mut self) {
        self.key = None;
        self.mesh = None;
        self.classes = None;
    }
}
