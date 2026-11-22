//! Which kinds take a user start / exit pick (#1448, #3324): one capability
//! per registry row, the one accessor that reads the picks of any kind, and
//! the one refusal of a pick that a kind does not take - in the kind's own
//! name. The design is `docs/decisions/1448-start-exit-shared.md`; each
//! family slice of #1448 changes its own arm here when its planner reads a
//! pick.
//!
//! Provenance: our own parameter model over the general machining practice
//! of a chosen entry and exit position; no vendor material.

use crate::geo2d::Point2;
use crate::path_picks::{PathPick, PathPicks, PickRole};

use super::kinds::KindInfo;
use super::types::{OpParams, OpType};

/// What a pick means on a kind, and so how the viewport snaps it. Every
/// capability except [`Self::NotTaken`] takes a start pick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickCapability {
    /// A point on a closed ring: the ring is re-sequenced to start there
    /// (the 2D contour). Snaps to a wall. Takes a start and an exit.
    Ring,
    /// A point inside the region: the descent place of the first pass
    /// and the end of the last pass. Snaps to a point in the region.
    Region,
    /// The first hole of the order and the last one. Snaps to a hole.
    HoleOrder,
    /// One end of an open chain or a slot, and the other end. Snaps to
    /// an end.
    Ends,
    /// The corner of the first raster row and the end of the last row.
    /// Snaps to a corner.
    Raster,
    /// The wire's threading point (`WireContourOp.start`): a start only.
    Threading,
    /// The kind does not take a pick; the reason, said by name in the
    /// refusal and on the operation page.
    NotTaken(&'static str),
}

/// The reason of a cutting kind whose family slice of #1448 has not shipped.
pub const PICK_OWED: &str =
    "the planner chooses the entry and the exit of this kind until its slice of #1448 ships";

/// The reason of the probe: it has no path to start.
pub const PICK_NO_PATH: &str = "a probe touches the part and cuts no path";

impl PickCapability {
    /// Does this capability take a pick of `role`?
    #[must_use]
    pub fn takes(self, role: PickRole) -> bool {
        match self {
            Self::NotTaken(_) => false,
            Self::Threading => role == PickRole::Start,
            Self::Ring | Self::Region | Self::HoleOrder | Self::Ends | Self::Raster => true,
        }
    }
}

impl KindInfo {
    /// The kind's pick capability. Exhaustive: a new kind fails to compile
    /// until it states one (the registry's own law).
    #[must_use]
    pub fn pick(&self) -> PickCapability {
        match self.ty {
            OpType::Contour => PickCapability::Ring,
            OpType::WireContour => PickCapability::Threading,
            OpType::Probe => PickCapability::NotTaken(PICK_NO_PATH),
            OpType::Face
            | OpType::Pocket
            | OpType::Slot
            | OpType::Trace
            | OpType::Adaptive2D
            | OpType::Offset2D
            | OpType::Engrave
            | OpType::Chamfer2D
            | OpType::Circular
            | OpType::Thread
            | OpType::Drill
            | OpType::Bore
            | OpType::Pocket3D
            | OpType::Finish3D
            | OpType::Deburr
            | OpType::Contour5D
            | OpType::Parallel5D
            | OpType::Swarf
            | OpType::AdvancedSwarf
            | OpType::RotaryContour
            | OpType::RotaryParallel
            | OpType::RotaryOffset
            | OpType::MultiAxisRough
            | OpType::Flow5D
            | OpType::MultiAxisContour
            | OpType::Facing
            | OpType::ProfileRough
            | OpType::ProfileFinish
            | OpType::GrooveRough
            | OpType::GrooveFinish
            | OpType::Part
            | OpType::Threading
            | OpType::Chamfering
            | OpType::TaperTurning
            | OpType::MultiPassTurning
            | OpType::FormTurning
            | OpType::BoringTurning
            | OpType::PlasmaProfile
            | OpType::TrochoidalSlot
            | OpType::SinkerBurn
            | OpType::SinkerSideBurn
            | OpType::SlotUndercut
            | OpType::VCarve => PickCapability::NotTaken(PICK_OWED),
        }
    }
}

impl OpParams {
    /// The start and exit picks of this operation, for every kind: the
    /// contour family's `links.approach` points and the wire's threading
    /// point, read into the one shared shape. A kind without a pick field
    /// answers empty lists.
    #[must_use]
    pub fn path_picks(&self) -> PathPicks {
        let one = |p: Option<Point2>| p.map(PathPick::xy).into_iter().collect::<Vec<_>>();
        if let OpParams::WireContour(w) = self {
            return PathPicks { start: one(w.start.map(|[x, y]| Point2::new(x, y))), exit: Vec::new() };
        }
        self.links().map_or_else(PathPicks::default, |l| PathPicks {
            start: one(l.approach.start_point),
            exit: one(l.approach.exit_point),
        })
    }

    /// The by-name refusal of a pick that this kind does not take (#1448):
    /// it names the kind, the pick and the reason. A tilted (3+2) contour
    /// keeps its own refusal. `None` when every pick is taken.
    pub(super) fn pick_refusal(&self) -> Option<String> {
        let picks = self.path_picks();
        if picks.is_empty() {
            return None;
        }
        let info = self.kind().info();
        let cap = info.pick();
        for role in [PickRole::Start, PickRole::Exit] {
            if picks.of(role).is_empty() {
                continue;
            }
            let what = role.word();
            if matches!(self, OpParams::Contour { tilt: Some(_), .. }) {
                return Some(format!("a {what} point is not taken on a tilted (3+2) contour yet - clear it or the tilt"));
            }
            if !cap.takes(role) {
                let why = match cap {
                    PickCapability::NotTaken(why) => why,
                    _ => "this kind takes a start point only",
                };
                return Some(format!("{}: a {what} point is not taken - {why}; clear it on this operation", info.label));
            }
        }
        None
    }
}
