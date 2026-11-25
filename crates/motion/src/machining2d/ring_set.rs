//! The one ring planner of a planar area (#3743 step 1, #3750): the
//! cutter-centre area, a reference edge ([`RingRef`]), the first offset,
//! the stepover and the cut sense in; the passes of the ring set out, in
//! the order of the start side, with the notes they owe the op page. The
//! 2D Offset (rings round the selected contour), the 2D Pocket (rings of
//! the area's own walls), the Adaptive ring fallback and the 3-D level
//! all plan their rings here. The start side gives only the ring order
//! and the start point; Auto round a part is resolved before the plan by
//! the #3744 rule (`ring_side::resolve`). The level curves of one side are
//! built in `level_rings`, the links in `ring_links`, the entry in
//! `ring_entry`.
//!
//! Provenance: contour-parallel pocketing as level sets of the distance
//! to the boundary (Held, "On the Computational Geometry of Pocket
//! Machining", Springer LNCS 500, 1991); the reference split, the
//! completeness guard and the fallback are ours. No vendor material
//! consulted.

use crate::geo2d::{Point2, Region};

use super::face::GenError;
use super::level_order::order;
use super::level_rings::level_pieces;
use super::pass_start::{PassStart, StartSide};
use super::pocket::PocketParams;
use super::pocket_plan::plan_passes;
use super::pocket_strategy::PocketStrategy;

/// The edge that the rings are measured from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RingRef {
    /// The selected contour: the area surrounds the part (2D Offset). The
    /// rings are the distance from the part, one set for every start side
    /// (#3744).
    Part,
    /// The walls of the area itself (2D Pocket, the 3-D level). Inside
    /// measures from the holes and islands, Outside from the outer edge
    /// (#3316).
    Boundary,
}

impl RingRef {
    /// The reference of a start side: [`RingRef::Part`] for an area round
    /// a part.
    pub(super) fn of(side: &StartSide) -> Self {
        if side.around_part { Self::Part } else { Self::Boundary }
    }
}

/// The input of one ring plan.
pub(super) struct RingSet<'a> {
    /// The material area whose edges the rings are measured from.
    pub(super) region: &'a Region,
    /// The cutter-centre area that clips each ring.
    pub(super) center: &'a Region,
    /// The edge the rings are measured from.
    pub(super) reference: RingRef,
    /// The first offset (mm) from `region` to the first ring: the tool
    /// radius and the stock to leave (0: `region` is its own outline).
    pub(super) first: f64,
    /// The tool radius (mm), for the order and the completeness guard.
    pub(super) tool_r: f64,
    /// The stepover (mm) from ring to ring.
    pub(super) step: f64,
    /// Climb (true) or conventional cut sense.
    pub(super) climb: bool,
}

impl RingSet<'_> {
    /// The passes of the ring set for `side` (resolved), with the notes
    /// they owe the op page. Auto, a row strategy, and an Inside with no
    /// hole to start from keep the planner's old plan byte for byte. The
    /// completeness guard: a start-side plan whose passes leave part of
    /// the cutter-centre area unswept is never posted; the area takes the
    /// Auto plan, named.
    pub(super) fn plan(&self, strategy: PocketStrategy, side: &StartSide) -> Result<(Vec<Vec<Point2>>, Vec<String>), GenError> {
        let rings = matches!(strategy, PocketStrategy::Waterline | PocketStrategy::OffsetRings | PocketStrategy::Adaptive);
        if side.start != PassStart::Auto
            && rings
            && let Some(pieces) = level_pieces(self, side.start)?
        {
            let plan = order(pieces, self.center, self.tool_r, side)?;
            let left = unswept(self.center, &plan.passes, self.tool_r)?;
            if left <= LEFT_TOL {
                return Ok((plan.passes, plan.notes));
            }
            let auto = plan_passes(self.center, self.step, self.climb, strategy)?;
            return Ok((auto, vec![side.fallback_note(left)]));
        }
        Ok((plan_passes(self.center, self.step, self.climb, strategy)?, Vec::new()))
    }
}

/// The ring plan of one pocket area for `side` (resolved): the rings of
/// `region` eroded by the tool radius and the stock to leave, clipped to
/// `center`.
pub(super) fn ring_plan(
    region: &Region,
    center: &Region,
    p: &PocketParams,
    strategy: PocketStrategy,
    side: &StartSide,
) -> Result<(Vec<Vec<Point2>>, Vec<String>), GenError> {
    let set = RingSet {
        region,
        center,
        reference: RingRef::of(side),
        first: p.tool_diameter * 0.5 + p.stock_to_leave,
        tool_r: p.tool_diameter * 0.5,
        step: p.tiling_step(),
        climb: p.climb,
    };
    set.plan(strategy, side)
}

/// The 3-D rough's level (#3361): [`ring_plan`] over a cutter-centre
/// area that is its own outline (the level law erodes the target once,
/// so the start side is read off `center`'s own edges). Auto, a row
/// strategy and an Inside with no hole keep `plan_passes` byte for byte.
pub fn level_ring_plan(
    center: &Region,
    tool_r: f64,
    step: f64,
    climb: bool,
    strategy: PocketStrategy,
    side: &StartSide,
) -> Result<(Vec<Vec<Point2>>, Vec<String>), GenError> {
    let set = RingSet { region: center, center, reference: RingRef::of(side), first: 0.0, tool_r, step, climb };
    set.plan(strategy, side)
}

/// The most area (mm2) of the cutter-centre area that the passes may leave
/// unswept before the plan counts as incomplete.
const LEFT_TOL: f64 = 0.1;

/// The area of `center` that no disc of radius `r` on `passes` reaches,
/// read after a 50 um erosion: the arc-flat and ring-end slivers of a
/// complete plan vanish, a standing column (any patch wider than 0.1 mm)
/// stays. Zero for a complete plan.
pub(super) fn unswept(center: &Region, passes: &[Vec<Point2>], r: f64) -> Result<f64, GenError> {
    let swept = Region::sweep(passes, r * (1.0 - 1e-3))?;
    Ok(center.difference(&swept)?.offset(-0.05)?.area().abs())
}

#[cfg(test)]
#[path = "ring_set_tests.rs"]
mod tests;
