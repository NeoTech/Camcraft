//! The guide stage as one planner (#4176): each guide choice goes to the
//! planner of its family, and a family with no planner yet is refused by
//! name, never planned as another family (law 4).
//! Provenance: our own design record `docs/decisions/4024-path-kernel-design.md`
//! "Stage 2: Guide curves" and law 4.

use super::{GuideChoice, GuidePlanner, GuideSet, RingPlanner, RowPlanner};
use crate::region::KernelRegion;
use crate::verdict::{Refusal, Stage, Verdict};

/// Refusal reason: a guide family that has no planner yet.
pub const FAMILY_NOT_YET: &str = "this guide family is not planned yet: the medial line and the trace come with #4176, second part";

/// The guide stage: rings to [`RingPlanner`], rows to [`RowPlanner`]; the
/// medial line and the trace are refused until their planners exist.
#[derive(Clone, Copy, Debug, Default)]
pub struct Guides;

impl GuidePlanner for Guides {
    fn guide(&self, r: &KernelRegion, g: &GuideChoice, tol: f64) -> Result<Verdict<GuideSet>, Refusal> {
        match g {
            GuideChoice::Rings(_) => RingPlanner.guide(r, g, tol),
            GuideChoice::Rows(_) => RowPlanner.guide(r, g, tol),
            GuideChoice::Medial | GuideChoice::Trace(_) => Err(Refusal::new(Stage::Guide, FAMILY_NOT_YET)),
        }
    }
}
