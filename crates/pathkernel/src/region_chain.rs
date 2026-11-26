//! Stage 1 for a guide that follows the selection itself (#4176: Trace,
//! Engrave on a line, 2D Contour on an open chain, 2D Chamfer): the picked
//! chains, open or closed, with no material side. The region of an area
//! (`region.rs`) refuses an open chain; this region holds it.
//! Provenance: our own design record `docs/decisions/4024-path-kernel-design.md`
//! "Stage 1: Region" ("a later phase adds a region that is not a planar
//! area as a new variant") and `docs/decisions/4176-kernel-guides-medial-trace-helix.md`.

use crate::frame::{ToolShape, WorkPlane};
use crate::region::KernelRegion;
use crate::selection::{Chain, Selection};
use crate::verdict::{Refusal, Stage};
use crate::{Region, region_finite};

/// Refusal reason: a keep-out area with a point that is not finite.
pub const KEEP_OUT_NOT_FINITE: &str = "the keep-out area has a point that is not a finite number";

/// The picked chains of a trace in the frame of their work plane, the tool
/// and the keep-out area (fixtures, Do not touch). It has no material side
/// and no start side: the side of a trace is its offset in the guide
/// choice. Valid by construction: the chains are a checked [`Selection`].
#[derive(Clone, Debug, PartialEq)]
pub struct ChainRegion {
    plane: WorkPlane,
    chains: Vec<Chain>,
    keep_out: Region,
    tool: ToolShape,
}

impl ChainRegion {
    /// Refused for a keep-out area that is not finite. The selection is
    /// already checked (no crossing, no repeated point).
    pub fn new(plane: WorkPlane, sel: &Selection, keep_out: Region, tool: ToolShape) -> Result<Self, Refusal> {
        if !region_finite(&keep_out) {
            return Err(Refusal::new(Stage::Region, KEEP_OUT_NOT_FINITE));
        }
        Ok(Self { plane, chains: sel.chains().to_vec(), keep_out, tool })
    }
    #[must_use]
    pub fn plane(&self) -> WorkPlane {
        self.plane
    }
    /// The picked chains, in the order of the selection.
    #[must_use]
    pub fn chains(&self) -> &[Chain] {
        &self.chains
    }
    #[must_use]
    pub fn keep_out(&self) -> &Region {
        &self.keep_out
    }
    #[must_use]
    pub fn tool(&self) -> ToolShape {
        self.tool
    }
}

/// The region of a trace: the chains of `sel` as they are. A pure function
/// of its inputs; it takes no material side and no start side (laws 2, 3).
pub fn chain_region(sel: &Selection, plane: WorkPlane, tool: ToolShape, keep_out: Region) -> Result<KernelRegion, Refusal> {
    ChainRegion::new(plane, sel, keep_out, tool).map(KernelRegion::Chains)
}
