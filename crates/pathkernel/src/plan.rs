//! The plan output of one level: a list of steps - cut, link, entry, exit
//! (record "What phase 1 keeps open", point 5). The constructor checks the
//! step law as a transition table (see [`Plan::new`]).
//! Provenance: our own design record (`RECOMMENDED.md`, `## #4024`).

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

/// Refusal reason: a cut with no entry or stay-down link just before it.
pub const CUT_WITHOUT_ENTRY: &str = "each cut must follow an entry or a stay-down link";
/// Refusal reason: a plan that does not end with an exit after its last cut.
pub const NO_EXIT: &str = "the last cut of a plan must be followed by an exit";
/// Refusal reason: a link that does not follow a cut.
pub const STRAY_LINK: &str = "a link must follow a cut";
/// Refusal reason: an entry while the tool is at depth after a cut, or two entries in a row.
pub const STRAY_ENTRY: &str = "an entry must start the plan, or follow an exit or a lift";
/// Refusal reason: an exit that does not follow a cut.
pub const STRAY_EXIT: &str = "an exit must follow a cut";
/// Refusal reason: a plan that ends after an entry, a stay-down link or a lift.
pub const DANGLING: &str = "a plan must not end after an entry or a link: a cut and an exit must follow";

/// Where the tool is between two steps.
#[derive(Clone, Copy, PartialEq, Eq)]
enum At {
    /// Above the material: the start, or after an exit.
    Clear,
    /// Above the material after a lift: an entry must follow.
    Lifted,
    /// At depth with a proof (after an entry or a stay-down link): one cut may follow.
    Ready,
    /// At depth at the end of a cut: a link or an exit must follow.
    CutEnd,
}

impl Plan {
    /// The checked steps. The transition table (any other step is refused):
    ///
    /// | at       | entry  | cut     | stay link | lift   | exit  |
    /// |----------|--------|---------|-----------|--------|-------|
    /// | clear    | ready  | -       | -         | -      | -     |
    /// | lifted   | ready  | -       | -         | -      | -     |
    /// | ready    | -      | cut end | -         | -      | -     |
    /// | cut end  | -      | -       | ready     | lifted | clear |
    ///
    /// A cut consumes the readiness: each next cut needs its own entry or
    /// stay-down link. The plan ends clear. An empty plan is legal (nothing
    /// to cut at this level).
    pub fn new(steps: Vec<Step>) -> Result<Self, Refusal> {
        let mut at = At::Clear;
        for s in &steps {
            at = match (at, s) {
                (At::Clear | At::Lifted, Step::Entry(_)) => At::Ready,
                (_, Step::Entry(_)) => return Err(Refusal::new(Stage::Entry, STRAY_ENTRY)),
                (At::Ready, Step::Cut(_)) => At::CutEnd,
                (_, Step::Cut(_)) => return Err(Refusal::new(Stage::Entry, CUT_WITHOUT_ENTRY)),
                (At::CutEnd, Step::Link(Hop::Stay(_))) => At::Ready,
                (At::CutEnd, Step::Link(Hop::Lift(_))) => At::Lifted,
                (_, Step::Link(_)) => return Err(Refusal::new(Stage::Link, STRAY_LINK)),
                (At::CutEnd, Step::Exit(_)) => At::Clear,
                (_, Step::Exit(_)) => return Err(Refusal::new(Stage::Exit, STRAY_EXIT)),
            };
        }
        match at {
            At::Clear => Ok(Self { steps }),
            At::CutEnd => Err(Refusal::new(Stage::Exit, NO_EXIT)),
            At::Ready | At::Lifted => Err(Refusal::new(Stage::Exit, DANGLING)),
        }
    }
    #[must_use]
    pub fn steps(&self) -> &[Step] {
        &self.steps
    }
}
