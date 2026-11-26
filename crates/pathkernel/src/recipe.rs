//! The recipe of a strategy: one choice for each stage, checked when it is
//! made (record "The strategy is a recipe"; the Prime Directive's
//! "validate early"). A malformed choice is refused here, never inside a
//! stage.
//! Provenance: our own design record `docs/decisions/4024-path-kernel-design.md`.

use crate::entry::EntryChoice;
use crate::frame::{AxisControl, WorkPlane};
use crate::guide::GuideChoice;
use crate::link::LinkPolicy;
use crate::load::{LoadControl, Reach};
use crate::order::OrderChoice;
use crate::verdict::{Refusal, Stage};

/// One choice for each stage.
#[derive(Clone, Debug, PartialEq)]
pub struct Recipe {
    plane: WorkPlane,
    axis: AxisControl,
    guide: GuideChoice,
    load: LoadControl,
    order: OrderChoice,
    link: LinkPolicy,
    entry: EntryChoice,
}

/// Refusal reason: ring-reach engagement control on a guide that has no rings.
pub const RINGS_REACH_NEEDS_RINGS: &str = "engagement control that reads the rings before needs a ring guide";

impl Recipe {
    /// The checked recipe. Each choice is valid by its own type (the crate
    /// rule "valid by construction"); this checks the pairs of choices that
    /// cannot go together.
    pub fn new(
        plane: WorkPlane,
        axis: AxisControl,
        guide: GuideChoice,
        load: LoadControl,
        order: OrderChoice,
        link: LinkPolicy,
        entry: EntryChoice,
    ) -> Result<Self, Refusal> {
        let rings = matches!(guide, GuideChoice::Rings(_));
        if matches!(load, LoadControl::ConstantEngagement(c) if c.reach() == Reach::Rings) && !rings {
            return Err(Refusal::new(Stage::Recipe, RINGS_REACH_NEEDS_RINGS));
        }
        Ok(Self { plane, axis, guide, load, order, link, entry })
    }
    #[must_use]
    pub fn plane(&self) -> &WorkPlane {
        &self.plane
    }
    #[must_use]
    pub fn axis(&self) -> AxisControl {
        self.axis
    }
    #[must_use]
    pub fn guide(&self) -> GuideChoice {
        self.guide
    }
    #[must_use]
    pub fn load(&self) -> LoadControl {
        self.load
    }
    #[must_use]
    pub fn order(&self) -> OrderChoice {
        self.order
    }
    #[must_use]
    pub fn link(&self) -> LinkPolicy {
        self.link
    }
    #[must_use]
    pub fn entry(&self) -> &EntryChoice {
        &self.entry
    }
}
