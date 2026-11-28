//! The stock as the eye sees it, as plain data (M7.2 B3; moved from
//! app's `stock_view`, #2268): the view the sphere-traced viewport
//! draws, its packed lattice, and the display field built from world
//! facets. The world walk that gathers the facets and the plan stays in
//! app (`stock_view`). Provenance: the M7.1 field engine; the
//! subdivision is plain midpoint refinement. Nothing from vendor
//! binaries.

use std::sync::Arc;

use crate::geometry::{Point3, Triangle};
use crate::stock_sim::preview::pack::MAX_TEXELS;
use crate::stock_sim::preview::{pack_field, pack_target, CutPatch, DistanceLattice, LatticeMeta};
use crate::stock_sim::{Backend, Stock, FIELD_BAND};

/// The texels the stock's display cell is sized for by default: a
/// thirty-second of the dense pack budget. Finer cells cost the field
/// subtraction cubically (the flat tool's spanning flank sweeps the
/// whole stock height per move), and the first fixture run showed a
/// quarter of the budget taking minutes per program - the per-brick
/// cost law that would buy it back is M7.3's.
pub const DEFAULT_TEXELS: f64 = (MAX_TEXELS / 32) as f64;

/// A facet's longest edge after subdivision, in cells: well inside the
/// field builder's per-facet candidate limit with the band dilated in.
const EDGE_CELLS: f64 = 32.0;

/// Subdivision rounds are capped: 6 rounds is 4096 children a facet.
const MAX_ROUNDS: u32 = 6;

/// What the viewport traces: the stock after the program, and the
/// model it is judged against when the model can be signed.
#[derive(Debug)]
pub struct FieldView {
    pub stock: Stock,
    /// Shared, never re-cloned: the model does not move while a
    /// program plays.
    pub target: Option<Arc<Stock>>,
    /// Why `target` is empty, said so the status line can repeat it.
    pub target_note: Option<String>,
    pub cell: f64,
    /// The verdict tolerance the eye's colours are gated at (#1056): the
    /// program sim's own (`model_fidelity::program_tol`), so a reduced-
    /// fidelity model's picture is tightened as its verdict is.
    pub tol: f64,
    /// The GPU lattice already packed off the UI thread (playback):
    /// `None` and the eye packs `stock` itself.
    pub packed: Option<Packed>,
}

/// A view's lattice, packed where the cutting ran: the session's
/// constant layers behind one `Arc` (an eye holding the same `base`
/// re-sends only `cut`), and this frame's cut layer - whole, or as a
/// patch over an earlier frame of the session (M13.1).
#[derive(Clone, Debug)]
pub struct Packed {
    pub base: Arc<PackedBase>,
    /// This frame's number in its session (ascending, from 1).
    pub seq: u64,
    pub cut: CutLayer,
}

/// A frame's cut layer over `base.meta`'s extent.
#[derive(Clone, Debug)]
pub enum CutLayer {
    /// The whole layer.
    Full(Arc<Vec<f32>>),
    /// Only what changed since frame `from`: valid on top of ANY frame
    /// `m` of the session with `from <= m < seq` (cuts only grow, and
    /// the patch covers every brick that moved from `from` to here).
    /// An eye holding anything else repacks the layer whole.
    Patch { from: u64, patch: Arc<CutPatch> },
}

impl Packed {
    /// Bytes this frame sends to the GPU as a cut-layer rewrite.
    #[must_use]
    pub fn cut_bytes(&self) -> usize {
        match &self.cut {
            CutLayer::Full(c) => c.len().saturating_mul(4),
            CutLayer::Patch { patch, .. } => patch.bytes(),
        }
    }

    /// True when the patch applies over a resident frame `held`.
    #[must_use]
    pub fn patches(&self, held: u64) -> bool {
        matches!(&self.cut, CutLayer::Patch { from, .. } if *from <= held && held < self.seq)
    }
}

/// What a playback session never re-packs: the stock's mesh layer
/// (parity-signed) and the model's distance lattice.
#[derive(Debug)]
pub struct PackedBase {
    pub meta: LatticeMeta,
    pub mesh: Vec<f32>,
    pub target: Option<DistanceLattice>,
}

impl PackedBase {
    /// Pack the session's constant layers; `None` when the stock is
    /// not a field or will not pack (the eye then packs, or marches
    /// on the CPU, as before).
    #[must_use]
    pub fn of(stock: &Stock, target: Option<&Stock>) -> Option<(Self, Vec<f32>)> {
        let mut lat = stock.field().and_then(pack_field)?;
        // #22: one eikonal sweep re-normalizes the band before the
        // render pass (display only; sign and zero set kept, and later
        // playback cut layers only lift cells over it). A refusal keeps
        // the packed lattice as it was - a picture, never a verdict.
        if let Err(e) = lat.renormalize() {
            tracing::warn!("stock view kept the un-normalized band: {e}");
        }
        let target = target.and_then(Stock::field).and_then(pack_target);
        Some((PackedBase { meta: lat.meta, mesh: lat.mesh, target }, lat.cut))
    }
}

/// The uncut display field from the stock's world facets (no world).
pub fn build_stock(stock_facets: &[Triangle], cell: f64) -> Result<Stock, String> {
    let backend = Backend::Field { band: FIELD_BAND };
    Stock::from_facets(backend, &subdivide(stock_facets, cell), cell)
        .map_err(|e| format!("the field view could not build the stock: {e}"))
}

/// The model as a field at `cell` from its world facets, or why it
/// cannot be one.
pub fn build_target(part: &[Triangle], cell: f64) -> (Option<Stock>, Option<String>) {
    if part.is_empty() {
        return (None, Some("no model to judge against".to_string()));
    }
    match Stock::from_facets(Backend::Field { band: FIELD_BAND }, &subdivide(part, cell), cell) {
        Ok(t) => (Some(t), None),
        Err(e) => (None, Some(format!("the model cannot be signed as a field: {e}"))),
    }
}

/// Uniform 4-way midpoint subdivision until the longest edge spans
/// at most `EDGE_CELLS` cells. Every shared edge splits at the same
/// midpoint on both sides, so a watertight shell stays watertight -
/// the field's parity law needs exactly that.
pub fn subdivide(facets: &[Triangle], cell: f64) -> Vec<Triangle> {
    let longest = facets
        .iter()
        .flat_map(|t| [(t.a, t.b), (t.b, t.c), (t.c, t.a)])
        .map(|(p, q)| dist(p, q))
        .fold(0.0f64, f64::max);
    let ratio = longest / (EDGE_CELLS * cell);
    let rounds = if ratio.is_finite() && ratio > 1.0 { ratio.log2().ceil() as u32 } else { 0 };
    let mut out = facets.to_vec();
    for _ in 0..rounds.min(MAX_ROUNDS) {
        let mut next = Vec::with_capacity(out.len() * 4);
        for t in &out {
            let (ab, bc, ca) = (mid(t.a, t.b), mid(t.b, t.c), mid(t.c, t.a));
            next.push(Triangle::new(t.a, ab, ca));
            next.push(Triangle::new(ab, t.b, bc));
            next.push(Triangle::new(ca, bc, t.c));
            next.push(Triangle::new(ab, bc, ca));
        }
        out = next;
    }
    out
}

fn mid(p: Point3, q: Point3) -> Point3 {
    Point3::new((p.x + q.x) * 0.5, (p.y + q.y) * 0.5, (p.z + q.z) * 0.5)
}

fn dist(p: Point3, q: Point3) -> f64 {
    ((p.x - q.x).powi(2) + (p.y - q.y).powi(2) + (p.z - q.z).powi(2)).sqrt()
}
