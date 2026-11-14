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
//!   Only where #12's reading applies (`GenJob::around_part`): with no
//!   stock set up the model is the volume to remove, and nothing is kept.
//! - **Stock source** (or a pick off the setup's stock): the source is
//!   the material to remove, but the parts standing inside it are not
//!   (#1554). The same solid law runs against the setup's parts
//!   ([`parts_in_stock`]). A document with no part judges nothing.
//!   #1607 widened this from the pocket family to every 2D kind that
//!   answers for the part on a part source ([`is_stock_kept_kind`]):
//!   the typed kinds (Slot, Trochoidal slot, Circular), Contour and
//!   Face take the same solid law; Engrave and Trace take their own
//!   mark law (`mark_law`) against the parts instead, so a mark on the
//!   part's surface reads the same whether the line came off the part
//!   or off the stock. #1631 added the hole kinds (Drill without
//!   cross holes, Bore, Thread), which take their own containment law
//!   against the parts - the round wall of a hole the part models where
//!   the op declares one is excused, nothing else - and the 2D Chamfer,
//!   which takes the solid law: a stock edge is not a part edge.
//!
//! - **Part source, neighbouring parts** (#1621): a part-source
//!   operation's own law reads its source's mesh only, so a cut on part
//!   B that enters part A beside it had no finding on the operation -
//!   and with the stock turned (the sim refuses to judge) nothing judged
//!   A at all. Every part-source operation that carries a part law
//!   (`depth_law_of` not `None`) now also answers to the plain solid law
//!   against the setup's OTHER parts ([`neighbour_parts`]): no mark
//!   budget there (a mark is on the part it marks), no hole excused (the
//!   holes are the source's), nothing kept around. Its own source keeps
//!   its own law.
//!
//! The cutter's own tip and stack are swept against the mesh, so a legal
//! pocket - its rings tangent to the part - stays clean, and a cut that
//! enters the part is a blocking `Gouge` on its move.

use crate::geometry::Triangle;

use super::{OpParams, SrcRef};

/// Whether `params` is a pocket-family kind: it clears an area and
/// keeps the part.
#[must_use]
pub fn is_keep_kind(params: &OpParams) -> bool {
    matches!(params, OpParams::Pocket { .. } | OpParams::Adaptive2D { .. } | OpParams::Offset2D { .. })
}

/// Whether `params` answers for the parts standing in its stock when
/// its source is the stock (#1603, #1607, #1631): every 2D kind that
/// carries a part law on a part source - the pocket family (`Keep`),
/// Slot, Trochoidal slot and Circular (`authored_law`), Contour and Face
/// (`Section`), Engrave and Trace (`Mark`), the hole kinds (`Contain`,
/// [`is_hole_kind`]) - and the 2D Chamfer (#1631: on a part source its
/// break is the part's own edge; off the stock, the solid law).
#[must_use]
pub fn is_stock_kept_kind(params: &OpParams) -> bool {
    is_keep_kind(params)
        || super::mark_law::is_mark_kind(params)
        || is_hole_kind(params)
        || matches!(
            params,
            OpParams::Slot { .. }
                | OpParams::TrochoidalSlot { .. }
                | OpParams::Circular { .. }
                | OpParams::Contour { .. }
                | OpParams::Face { .. }
                | OpParams::Chamfer2D { .. }
        )
}

/// Whether `params` is a hole kind that answers to the containment law
/// (`DepthLaw::Contain`) on a part source: a Drill with no cross holes
/// (a cross chain answers to `cross_part` against its source), a Bore,
/// a Thread. Off the stock (#1631) the same law reads the setup's
/// parts: the drill, bar or mill may enter a part only through a hole
/// the part models there - its declared hole's round wall excused, to
/// its diameter and no deeper than its floor - and anywhere else it is
/// a gouge.
#[must_use]
pub fn is_hole_kind(params: &OpParams) -> bool {
    match params {
        OpParams::Drill { cross, .. } => cross.is_empty(),
        OpParams::Bore { .. } | OpParams::Thread { .. } => true,
        _ => false,
    }
}

/// The parts an operation whose source is NOT the part (the stock, or
/// picks off it) must still keep: the setup's parts in its frame - what
/// the program sim judges against (`setup_align::parts_facets_in`:
/// fixtures and a stock-only model left out). Empty for a kind outside
/// [`is_stock_kept_kind`], and for a part-solid source (whose own mesh
/// its `DepthLaw` reads).
#[must_use]
pub(super) fn parts_in_stock(
    world: &flecs_ecs::prelude::World,
    setup: Option<&str>,
    params: &OpParams,
    part_solid: bool,
) -> Vec<Triangle> {
    if part_solid || !is_stock_kept_kind(params) {
        return Vec::new();
    }
    crate::setup_align::parts_facets_in(world, setup)
}

/// The parts a part-source operation must keep beside its own (#1621):
/// every part the setup holds (`setup_align::parts_facets_in`'s set -
/// fixtures and a stock-only model left out) but the entities its
/// source names, in the setup's frame. Empty for a source that is not
/// the part (whose parts [`parts_in_stock`] reads), for a kind that
/// carries no part law on a part source, and for a one-part document.
#[must_use]
pub(super) fn neighbour_parts(
    world: &flecs_ecs::prelude::World,
    setup: Option<&str>,
    params: &OpParams,
    src: &SrcRef,
    part_solid: bool,
) -> Vec<Triangle> {
    if !part_solid {
        return Vec::new();
    }
    kept_models(world, params, src, true)
        .into_iter()
        .filter_map(|(_, e)| crate::setup_align::facets_in(world, &e, setup))
        .flatten()
        .collect()
}

/// The models whose meshes an operation's part laws read beyond its own
/// source: on a part source ([`neighbour_parts`]) the setup's other
/// parts, on a stock source ([`parts_in_stock`]) every part. Empty for a
/// kind that carries no such law.
fn kept_models(
    world: &flecs_ecs::prelude::World,
    params: &OpParams,
    src: &SrcRef,
    part_solid: bool,
) -> Vec<(String, flecs_ecs::prelude::Entity)> {
    let own: Vec<&str> = match src {
        _ if !part_solid => {
            if !is_stock_kept_kind(params) {
                return Vec::new();
            }
            Vec::new()
        }
        _ if super::generate::depth_law_of(params, src, true) == super::generate::DepthLaw::None => return Vec::new(),
        SrcRef::Stock => return Vec::new(),
        SrcRef::Entity(n) => vec![n.as_str()],
        SrcRef::Selection(items) => items.iter().map(|g| g.entity.as_str()).collect(),
    };
    let skip = crate::stock_source::stock_only_model(world);
    crate::model::model_entities(world)
        .into_iter()
        .filter(|(name, e)| !own.contains(&name.as_str()) && crate::stock_source::judged(world, e, skip))
        .collect()
}
