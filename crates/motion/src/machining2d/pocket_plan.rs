//! The ring and row plan of one planar area (split from `pocket.rs` for
//! the 400-line law, #3316): the shape law both the 2-D pocket and the
//! 3-D pocket's per-level plan share. Provenance: concentric offset rings
//! and zigzag rows of published pocketing practice; no vendor material
//! consulted.

use crate::geo2d::{Point2, Region};

use super::face::GenError;
use super::pass_start::PassStart;
use super::pocket_strategy::PocketStrategy;
use super::pocket_zigzag::zigzag_passes;
use super::sided_rings::sided_passes;
use super::waterscan::waterline_passes;

/// Concentric rings marching inward: ring k is the cutter-center area
/// eroded by `k * stepover`, travelled closed. Offsets are taken from
/// `center` at absolute distances (never buffer-of-buffer), so micron
/// quantization cannot compound across rings.
/// The cutter-center passes for one planar area: the shape law both the
/// 2-D pocket and the 3-D pocket's per-level plan share (the strategies
/// band/erode `center` identically; only the numbers come from the
/// operation's params either way).
pub(crate) fn plan_passes(
    center: &Region,
    stepover: f64,
    climb: bool,
    strategy: PocketStrategy,
) -> Result<Vec<Vec<Point2>>, GenError> {
    plan_passes_from(center, stepover, climb, strategy, PassStart::Auto)
}

/// [`plan_passes`] with the side each level starts on (#3316): read by
/// the ring tilings only; zigzag rows have no ring order.
pub(crate) fn plan_passes_from(
    center: &Region,
    stepover: f64,
    climb: bool,
    strategy: PocketStrategy,
    start: PassStart,
) -> Result<Vec<Vec<Point2>>, GenError> {
    match strategy {
        PocketStrategy::Zigzag | PocketStrategy::Auto => zigzag_passes(center, stepover, climb),
        // Offset rings are ring tiling at the caller's step; the HSM
        // plan is `pocket_hsm.rs`'s (the planar pocket's own, and the
        // 3-D ladder's per level through `machining3d::hsm`), which
        // hands a refused area back here as rings.
        // Auto is the core-first ring family; Inside / Outside walk the
        // level curves of the start side (`sided_rings.rs`).
        PocketStrategy::Waterline | PocketStrategy::Adaptive | PocketStrategy::OffsetRings => match start {
            PassStart::Auto => waterline_passes(center, stepover, climb),
            PassStart::Inside | PassStart::Outside => sided_passes(center, stepover, climb, start),
        },
    }
}
