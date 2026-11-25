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
    /// The checked recipe. The plane, the tool and the entry choice are
    /// checked by their own constructors; this checks the guide, the load
    /// control and the pairs of choices that cannot go together.
    pub fn new(
        plane: WorkPlane,
        axis: AxisControl,
        guide: GuideChoice,
        load: LoadControl,
        order: OrderChoice,
        link: LinkPolicy,
        entry: EntryChoice,
    ) -> Result<Self, Refusal> {
        guide.validate()?;
        load.validate()?;
        let rings = matches!(guide, GuideChoice::Rings { .. });
        if matches!(load, LoadControl::ConstantEngagement { reach: Reach::Rings, .. }) && !rings {
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
    pub fn guide(&self) -> &GuideChoice {
        &self.guide
    }
    #[must_use]
    pub fn load(&self) -> &LoadControl {
        &self.load
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::EntryKind;
    use crate::guide::{BAD_STEPOVER, RingRef};
    use crate::load::BAD_LOAD;

    fn make(guide: GuideChoice, load: LoadControl) -> Result<Recipe, Refusal> {
        let plane = WorkPlane::setup_xy(0.0).unwrap();
        let entry = EntryChoice::new(EntryKind::Helix, 2.0, vec![]).unwrap();
        Recipe::new(plane, AxisControl::Normal, guide, load, OrderChoice::default(), LinkPolicy::default(), entry)
    }

    fn rings(stepover: f64) -> GuideChoice {
        GuideChoice::Rings { reference: RingRef::Boundary, first_offset: 0.0, stepover, circle: None, count: None }
    }

    #[test]
    fn a_recipe_refuses_a_malformed_choice_when_it_is_made() {
        let r = make(rings(2.0), LoadControl::None).unwrap();
        assert_eq!((r.axis(), r.link()), (AxisControl::Normal, LinkPolicy::StayWhereProved));
        assert_eq!(make(rings(-1.0), LoadControl::None).unwrap_err().reason, BAD_STEPOVER);
        let bad_load = LoadControl::ConstantEngagement { load: 2.0, reach: Reach::Stock };
        assert_eq!(make(rings(2.0), bad_load).unwrap_err().reason, BAD_LOAD);
    }

    #[test]
    fn ring_reach_needs_a_ring_guide() {
        let lc = LoadControl::ConstantEngagement { load: 0.4, reach: Reach::Rings };
        assert!(make(rings(2.0), lc).is_ok());
        let rows = GuideChoice::Rows { angle: 0.0, stepover: 2.0, closing_ring: true };
        let e = make(rows.clone(), lc).unwrap_err();
        assert_eq!((e.stage, e.reason), (Stage::Recipe, RINGS_REACH_NEEDS_RINGS));
        assert!(make(rows, LoadControl::ConstantEngagement { load: 0.4, reach: Reach::Stock }).is_ok());
    }
}
