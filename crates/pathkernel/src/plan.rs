//! The plan output of one level: a list of steps - cut, link, entry, exit
//! (record "What phase 1 keeps open", point 5). The constructor checks the
//! step law: each cut follows an entry or a stay-down link, a lift is
//! followed by an entry, and the plan ends with an exit.
//! Provenance: our own design record `docs/decisions/4024-path-kernel-design.md`.

use crate::entry::{Entry, Exit};
use crate::frame::unit;
use crate::guide::Curve;
use crate::link::Hop;
use crate::verdict::{Refusal, Stage};

/// One cut along a curve, with an optional tool axis (none = the plane
/// normal of the recipe).
#[derive(Clone, Debug, PartialEq)]
pub struct Cut {
    curve: Curve,
    axis: Option<[f64; 3]>,
}

/// Refusal reason: a tool axis that is not a finite unit vector.
pub const BAD_AXIS: &str = "the tool axis of a cut must be a finite unit vector";

impl Cut {
    pub fn new(curve: Curve, axis: Option<[f64; 3]>) -> Result<Self, Refusal> {
        if axis.is_some_and(|a| !unit(a)) {
            return Err(Refusal::new(Stage::Recipe, BAD_AXIS));
        }
        Ok(Self { curve, axis })
    }
    #[must_use]
    pub fn curve(&self) -> &Curve {
        &self.curve
    }
    #[must_use]
    pub fn axis(&self) -> Option<[f64; 3]> {
        self.axis
    }
}

/// One step of a plan. A later phase adds a canned cycle with no path.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Step {
    Entry(Entry),
    Cut(Cut),
    Link(Hop),
    Exit(Exit),
}

/// The checked steps of one level.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Plan {
    steps: Vec<Step>,
}

/// Refusal reason: a cut with no entry or stay-down link before it.
pub const CUT_WITHOUT_ENTRY: &str = "a cut must follow an entry or a stay-down link";
/// Refusal reason: a plan that does not end with an exit.
pub const NO_EXIT: &str = "a plan with cuts must end with an exit";
/// Refusal reason: a link that is not between two cuts.
pub const STRAY_LINK: &str = "a link must follow a cut";

impl Plan {
    /// An empty plan is legal (nothing to cut at this level).
    pub fn new(steps: Vec<Step>) -> Result<Self, Refusal> {
        // `ready`: the tool is at depth with a proof (after an entry or a stay link).
        let (mut ready, mut after_cut) = (false, false);
        for s in &steps {
            match s {
                Step::Entry(_) => (ready, after_cut) = (true, false),
                Step::Cut(_) if !ready => return Err(Refusal::new(Stage::Entry, CUT_WITHOUT_ENTRY)),
                Step::Cut(_) => after_cut = true,
                Step::Link(_) if !after_cut => return Err(Refusal::new(Stage::Link, STRAY_LINK)),
                Step::Link(h) => (ready, after_cut) = (matches!(h, Hop::Stay(_)), false),
                Step::Exit(_) => (ready, after_cut) = (false, false),
            }
        }
        let cuts = steps.iter().any(|s| matches!(s, Step::Cut(_)));
        if cuts && !matches!(steps.last(), Some(Step::Exit(_))) {
            return Err(Refusal::new(Stage::Exit, NO_EXIT));
        }
        Ok(Self { steps })
    }
    #[must_use]
    pub fn steps(&self) -> &[Step] {
        &self.steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point2;
    use crate::entry::{EntryChoice, EntryKind, Proof};
    use crate::frame::{PlanePoint, ToolShape};
    use crate::guide::Family;

    fn cut() -> Step {
        let pts = vec![PlanePoint::flat(Point2::new(0.0, 0.0)), PlanePoint::flat(Point2::new(5.0, 0.0))];
        Step::Cut(Cut::new(Curve::new(pts, None, false, 0, Family::Row).unwrap(), None).unwrap())
    }
    fn entry() -> Step {
        let c = EntryChoice::new(EntryKind::Air, 2.0, vec![]).unwrap();
        let t = ToolShape::disc(3.0, 0.0, false, 10.0).unwrap();
        Step::Entry(Entry::new(EntryKind::Air, vec![PlanePoint::default()], Proof::InAir, &c, t).unwrap())
    }
    fn exit() -> Step {
        Step::Exit(Exit::new(None, 5.0).unwrap())
    }
    fn stay() -> Step {
        Step::Link(Hop::stay(vec![Point2::new(5.0, 1.0)]).unwrap())
    }
    fn lift() -> Step {
        Step::Link(Hop::lift("crosses a planned cut").unwrap())
    }

    #[test]
    fn a_cut_needs_an_entry_or_a_stay_link() {
        assert!(Plan::new(vec![]).is_ok());
        assert!(Plan::new(vec![entry(), cut(), stay(), cut(), exit()]).is_ok());
        assert!(Plan::new(vec![entry(), cut(), lift(), entry(), cut(), exit()]).is_ok());
        assert_eq!(Plan::new(vec![cut(), exit()]).unwrap_err().reason, CUT_WITHOUT_ENTRY);
        assert_eq!(Plan::new(vec![entry(), cut(), lift(), cut(), exit()]).unwrap_err().reason, CUT_WITHOUT_ENTRY);
        assert_eq!(Plan::new(vec![entry(), stay(), cut(), exit()]).unwrap_err().reason, STRAY_LINK);
        assert_eq!(Plan::new(vec![entry(), cut()]).unwrap_err().reason, NO_EXIT);
    }

    #[test]
    fn a_cut_axis_is_a_unit_vector() {
        let pts = vec![PlanePoint::default(), PlanePoint::flat(Point2::new(1.0, 0.0))];
        let c = Curve::new(pts, None, false, 0, Family::Trace).unwrap();
        assert!(Cut::new(c.clone(), Some([0.0, 0.0, 1.0])).is_ok());
        assert_eq!(Cut::new(c.clone(), Some([0.0, 0.0, 2.0])).unwrap_err().reason, BAD_AXIS);
        assert!(Cut::new(c, Some([f64::NAN, 0.0, 1.0])).is_err());
    }
}
