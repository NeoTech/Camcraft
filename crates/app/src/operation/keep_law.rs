//! The keep law (#1603): the pocket family (2D Pocket, 2D Adaptive, 2D
//! Offset) clears material AROUND a part it must keep, and answers for
//! that part on the operation itself - whole-model source or stock
//! source alike - so the verdict never rides on the program sim alone.
//!
//! Provenance: our own design (`docs/decisions/1595-engrave-mark-law.md`,
//! the #1603 section) over the sim crate's solid law
//! (`verify::solid_hits`). No vendor material consulted.
//!
//! ## The law
//!
//! - **Whole-model source** (`DepthLaw::Keep`): the part's sections are
//!   islands (#12), so with the default area controls every move stands
//!   outside the part. An area offset or extension moves the boundary
//!   INTO those islands, and nothing on the operation judged that. The
//!   solid law, nothing excused, now runs against the source's own mesh.
//! - **Stock source** (or a pick off the setup's stock): the source is
//!   the material to remove, but the parts standing inside it are not
//!   (#1554). The same solid law runs against the setup's parts
//!   ([`parts_in_stock`]). A document with no part judges nothing.
//!
//! The cutter's own tip and stack are swept against the mesh, so a legal
//! pocket - its rings tangent to the part - stays clean, and a cut that
//! enters the part is a blocking `Gouge` on its move.

use crate::geometry::Triangle;

use super::OpParams;

/// Whether `params` is a pocket-family kind: it clears an area and
/// keeps the part.
#[must_use]
pub fn is_keep_kind(params: &OpParams) -> bool {
    matches!(params, OpParams::Pocket { .. } | OpParams::Adaptive2D { .. } | OpParams::Offset2D { .. })
}

/// The parts a pocket-family operation whose source is NOT the part
/// (the stock, or picks off it) must still keep: the setup's parts in
/// its frame - what the program sim judges against
/// (`setup_align::parts_facets_in`: fixtures and a stock-only model
/// left out). Empty for every other kind, and for a part-solid source
/// (whose own mesh `DepthLaw::Keep` or `Section` reads).
#[must_use]
pub(super) fn parts_in_stock(
    world: &flecs_ecs::prelude::World,
    setup: Option<&str>,
    params: &OpParams,
    part_solid: bool,
) -> Vec<Triangle> {
    if part_solid || !is_keep_kind(params) {
        return Vec::new();
    }
    crate::setup_align::parts_facets_in(world, setup)
}
