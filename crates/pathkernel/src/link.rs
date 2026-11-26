//! Stage 5, Link (K4): a hop between two cuts stays at depth only with a
//! proof, else it lifts with a named reason that the tally counts. K4 adds
//! `link()` and the proofs.
//! Provenance: our own design record `docs/decisions/4024-path-kernel-design.md`
//! "Stage 5: Link"; stay-down linking as in the published high-speed
//! machining literature (Held & Spielberger 2009).

use std::collections::BTreeMap;

use crate::Point2;
use crate::frame::Spot;
use crate::plan::Cut;
use crate::region::KernelRegion;
use crate::verdict::{NO_REASON, Refusal, Stage};

/// The user's link policy, as the stages read it. Motion converts its
/// `Links` into this value.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LinkPolicy {
    /// Stay at depth where a proof holds, else lift with a reason.
    #[default]
    StayWhereProved,
    /// The user asked for a retract between cuts: always lift.
    Retract,
}

/// The finite points of a stay-down hop: one or more.
#[derive(Clone, Debug, PartialEq)]
pub struct StayPath(Vec<Spot>);

impl StayPath {
    #[must_use]
    pub fn points(&self) -> Vec<Point2> {
        self.0.iter().map(|s| s.get()).collect()
    }
}

/// The reason of a lift: not blank.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LiftReason(&'static str);

impl LiftReason {
    #[must_use]
    pub fn get(self) -> &'static str {
        self.0
    }
}

/// One hop between two cuts. Each variant holds a checked value.
#[derive(Clone, Debug, PartialEq)]
pub enum Hop {
    /// Stay at depth along these points (a proof holds for them).
    Stay(StayPath),
    /// Lift along the tool axis, for a named reason.
    Lift(LiftReason),
}

/// Refusal reason: a stay hop with no point.
pub const BAD_STAY: &str = "a stay-down hop needs one or more finite points";
/// The lift reason when the user asked for a retract.
pub const USER_RETRACT: &str = "the user asked for a retract between cuts";

impl Hop {
    pub fn stay(pts: Vec<Point2>) -> Result<Self, Refusal> {
        if pts.is_empty() {
            return Err(Refusal::new(Stage::Link, BAD_STAY));
        }
        let spots = pts.into_iter().map(|p| Spot::new(Stage::Link, p)).collect::<Result<Vec<_>, _>>()?;
        Ok(Self::Stay(StayPath(spots)))
    }
    pub fn lift(reason: &'static str) -> Result<Self, Refusal> {
        if reason.trim().is_empty() {
            return Err(Refusal::new(Stage::Link, NO_REASON));
        }
        Ok(Self::Lift(LiftReason(reason)))
    }
}

/// The count of lifts by reason. The status line prints it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LinkTally {
    lifts: BTreeMap<&'static str, u32>,
}

impl LinkTally {
    /// Counts `hop` if it is a lift; returns the hop unchanged.
    pub fn record(&mut self, hop: Hop) -> Hop {
        if let Hop::Lift(reason) = &hop {
            let n = self.lifts.entry(reason.get()).or_insert(0);
            *n = n.saturating_add(1);
        }
        hop
    }
    #[must_use]
    pub fn lifts(&self) -> u32 {
        self.lifts.values().fold(0u32, |a, b| a.saturating_add(*b))
    }
    #[must_use]
    pub fn by_reason(&self) -> &BTreeMap<&'static str, u32> {
        &self.lifts
    }
}

/// A link proof: a hop between two cuts, always counted in the tally.
pub trait LinkProof {
    fn link(&self, from: &Cut, to: &Cut, r: &KernelRegion, planned: &[Cut], policy: LinkPolicy, tally: &mut LinkTally) -> Hop;
}
