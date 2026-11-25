//! The one entry rule of a ring pass (#3743 step 1, #3750), in this
//! order:
//!
//! 1. An entry in air outside the stock box, by the approach distance
//!    (#3745), then a G17 arc lead in the cut plane (#3746): the air room
//!    that this module plans ([`RingEntry::plan`], `air_entry`).
//! 2. A proved pre-drilled hole (`predrilled`, through the ladder's
//!    holes).
//! 3. A cleared column that an earlier cut opened (the ladder's cleared
//!    links and solid restart).
//! 4. A helix or a ramp into material, only where the tool must enter
//!    material (the ladder's selected entry).
//! 5. Else a refusal with a named reason: a selected Arc on an Outside
//!    ring pass with no air room is refused here (#3884), and each
//!    stand-in entry of the ladder is named (`entry_warn`). Never a
//!    silent plunge.
//!
//! Steps 2 to 4 are the shared entry chain of the pocket ladder
//! (`levels`, `pocket_emit`), which the passes that are not rings use too;
//! this module decides what is particular to a ring pass. The 2D Offset,
//! the 2D Pocket and the avoid pocket take their ring entry here.
//!
//! Provenance: our own entry order (the maintainer's rule in #3741, the
//! shared design of #3743); no vendor material consulted.

use crate::geo2d::Region;

use super::air_entry::AirRoom;
use super::entry_warn::refuse_outside_arc;
use super::face::GenError;
use super::pass_start::{PassStart, StartSide, entry_advice_note};
use super::pocket::PocketParams;
use super::pocket_strategy::PocketStrategy;

/// The entry plan of the ring passes of one area.
pub(super) struct RingEntry {
    /// The room in air outside the stock box (step 1), if the pass has
    /// one.
    pub(super) air: Option<AirRoom>,
    /// The note that names the user's lead-in where it is not the Arc
    /// (#3748), once for each path.
    pub(super) advice: Option<String>,
}

impl RingEntry {
    /// The entry of the passes of `strategy` over `center`, for the start
    /// side that the user `asked` and the `side` that it resolved to.
    /// `keepout` (the fixtures and the clamps of an avoid level, in
    /// cutter-centre terms) is taken out of the air room (#3810). A
    /// selected Arc on an Outside ring pass with no air room is refused by
    /// name (#3884).
    pub(super) fn plan(
        asked: PassStart,
        side: &StartSide,
        strategy: PocketStrategy,
        center: &Region,
        p: &PocketParams,
        keepout: Option<(&Region, &Region)>,
    ) -> Result<Self, GenError> {
        let tool_r = p.tool_diameter * 0.5;
        let air = AirRoom::for_side(side, strategy, center, tool_r, p.links.approach.approach_distance);
        let air = match keepout {
            Some((a, b)) => air.and_then(|r| r.without(&a.union(b).ok()?)),
            None => air,
        };
        // An Adaptive op on its ring fallback too (#3884 review).
        let rings = matches!(strategy, PocketStrategy::Waterline | PocketStrategy::OffsetRings);
        refuse_outside_arc(asked, p.entry, rings, air.is_some())?;
        // #3748: the user's Inside or Outside with a lead-in other than
        // Arc: used as selected, named. #3884: the air room is not the
        // gate. Not for an Adaptive op on its ring fallback: its page has
        // no advice.
        let advice = if rings && p.strategy != PocketStrategy::Adaptive { entry_advice_note(asked, p.entry) } else { None };
        Ok(Self { air, advice })
    }
}
