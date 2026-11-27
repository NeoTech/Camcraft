//! The guide stage as one planner (#4176): each guide choice goes to the
//! planner of its family, never to the planner of another family (law 4).
//! Provenance: our own design record (`RECOMMENDED.md`, `## #4024`)
//! "Stage 2: Guide curves" and law 4.

use super::{GuideChoice, GuidePlanner, GuideSet, HelixPlanner, MedialPlanner, RingPlanner, RowPlanner, TracePlanner};
use crate::region::KernelRegion;
use crate::verdict::{Refusal, Verdict};

/// The guide stage: rings (exact circles included) to [`RingPlanner`], rows
/// to [`RowPlanner`], the medial line to [`MedialPlanner`], the trace to
/// [`TracePlanner`] and the helix to [`HelixPlanner`].
#[derive(Clone, Copy, Debug, Default)]
pub struct Guides;

impl GuidePlanner for Guides {
    fn guide(&self, r: &KernelRegion, g: &GuideChoice, tol: f64) -> Result<Verdict<GuideSet>, Refusal> {
        match g {
            GuideChoice::Rings(_) => RingPlanner.guide(r, g, tol),
            GuideChoice::Rows(_) => RowPlanner.guide(r, g, tol),
            GuideChoice::Medial => MedialPlanner.guide(r, g, tol),
            GuideChoice::Trace(_) => TracePlanner.guide(r, g, tol),
            GuideChoice::Helix(_) => HelixPlanner.guide(r, g, tol),
        }
    }
}
