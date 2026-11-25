//! Stage 5, Link (K4): a hop between two cuts stays at depth only with a
//! proof, else it lifts with a named reason that the tally counts. K4 adds
//! `link()` and the proofs.
//! Provenance: our own design record `docs/decisions/4024-path-kernel-design.md`
//! "Stage 5: Link"; stay-down linking as in the published high-speed
//! machining literature (Held & Spielberger 2009).

use std::collections::BTreeMap;

use crate::plan::Cut;
use crate::region::KernelRegion;
use crate::verdict::{Refusal, Stage};
use crate::{Point2, finite};

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

/// One hop between two cuts.
#[derive(Clone, Debug, PartialEq)]
pub enum Hop {
    /// Stay at depth along these points (a proof holds for them).
    Stay(Vec<Point2>),
    /// Lift along the tool axis, for a named reason.
    Lift { reason: &'static str },
}

/// Refusal reason: a stay hop with no point or a point that is not finite.
pub const BAD_STAY: &str = "a stay-down hop needs one or more finite points";
/// Refusal reason: a lift with no reason.
pub const LIFT_NO_REASON: &str = "a lift must name its reason";
/// The lift reason when the user asked for a retract.
pub const USER_RETRACT: &str = "the user asked for a retract between cuts";

impl Hop {
    pub fn stay(pts: Vec<Point2>) -> Result<Self, Refusal> {
        if pts.is_empty() || !pts.iter().all(|p| finite(&[p.x, p.y])) {
            return Err(Refusal::new(Stage::Link, BAD_STAY));
        }
        Ok(Self::Stay(pts))
    }
    pub fn lift(reason: &'static str) -> Result<Self, Refusal> {
        if reason.trim().is_empty() {
            return Err(Refusal::new(Stage::Link, LIFT_NO_REASON));
        }
        Ok(Self::Lift { reason })
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
        if let Hop::Lift { reason } = hop {
            let n = self.lifts.entry(reason).or_insert(0);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hop_is_checked() {
        assert_eq!(Hop::stay(vec![]).unwrap_err().reason, BAD_STAY);
        assert!(Hop::stay(vec![Point2::new(f64::NAN, 0.0)]).is_err());
        assert!(Hop::stay(vec![Point2::new(1.0, 0.0)]).is_ok());
        assert_eq!(Hop::lift("").unwrap_err().reason, LIFT_NO_REASON);
    }

    #[test]
    fn the_tally_counts_each_lift_by_reason() {
        let mut t = LinkTally::default();
        t.record(Hop::lift(USER_RETRACT).unwrap());
        t.record(Hop::lift("crosses a planned cut").unwrap());
        t.record(Hop::lift(USER_RETRACT).unwrap());
        t.record(Hop::stay(vec![Point2::new(0.0, 0.0)]).unwrap());
        assert_eq!(t.lifts(), 3);
        assert_eq!(t.by_reason()[USER_RETRACT], 2);
        assert_eq!(LinkPolicy::default(), LinkPolicy::StayWhereProved);
    }
}
