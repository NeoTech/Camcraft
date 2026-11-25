//! The common result types of the stages: a refusal, a fallback, a gap and
//! the verdict that carries them (record "Common result types").
//! Provenance: our own design record `docs/decisions/4024-path-kernel-design.md`;
//! the Prime Directive of AGENTS.md (a result is correct or visibly refused).

use crate::{Point2, finite};

/// The stage that wrote a refusal, a fallback or a note.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Stage {
    Recipe,
    Region,
    Guide,
    Order,
    Load,
    Link,
    Entry,
    Exit,
}

/// A stage that cannot meet its law refuses with a named reason. Each
/// `reason` is a named `const` of the stage that writes it.
#[derive(Clone, Debug, PartialEq)]
pub struct Refusal {
    pub stage: Stage,
    pub reason: &'static str,
    pub at: Option<Point2>,
    pub fix: Option<&'static str>,
}

impl Refusal {
    #[must_use]
    pub fn new(stage: Stage, reason: &'static str) -> Self {
        Self { stage, reason, at: None, fix: None }
    }

    /// The refusal at one spot. A spot that is not finite is dropped: the
    /// refusal still stands, without a place.
    #[must_use]
    pub fn at(mut self, p: Point2) -> Self {
        self.at = finite(&[p.x, p.y]).then_some(p);
        self
    }

    #[must_use]
    pub fn fix(mut self, fix: &'static str) -> Self {
        self.fix = Some(fix);
        self
    }
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.stage, self.reason)?;
        if let Some(fix) = self.fix {
            write!(f, " ({fix})")?;
        }
        Ok(())
    }
}

/// A stage ran another behaviour than the one asked. The status line names
/// it (law 4): a fallback is never silent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fallback {
    stage: Stage,
    asked: &'static str,
    used: &'static str,
    reason: &'static str,
}

/// Refusal reason: a fallback whose used behaviour is the asked one.
pub const FALLBACK_SAME: &str = "a fallback must use another behaviour than the one asked";
/// Refusal reason: a fallback or a note with no reason text.
pub const NO_REASON: &str = "a fallback or a note must name its reason";

impl Fallback {
    /// A fallback from `asked` to `used`, for `reason`. Refused when the two
    /// are the same, or when a name is empty.
    pub fn new(stage: Stage, asked: &'static str, used: &'static str, reason: &'static str) -> Result<Self, Refusal> {
        if reason.trim().is_empty() || asked.trim().is_empty() || used.trim().is_empty() {
            return Err(Refusal::new(stage, NO_REASON));
        }
        if asked == used {
            return Err(Refusal::new(stage, FALLBACK_SAME));
        }
        Ok(Self { stage, asked, used, reason })
    }
    #[must_use]
    pub fn stage(&self) -> Stage {
        self.stage
    }
    #[must_use]
    pub fn asked(&self) -> &'static str {
        self.asked
    }
    #[must_use]
    pub fn used(&self) -> &'static str {
        self.used
    }
    #[must_use]
    pub fn reason(&self) -> &'static str {
        self.reason
    }
}

/// An area of the cutter-centre region that no swept disc covers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gap {
    area_mm2: f64,
    at: Point2,
}

/// Refusal reason: a gap with no positive, finite area or no finite place.
pub const BAD_GAP: &str = "a gap must have a positive finite area and a finite place";

impl Gap {
    pub fn new(area_mm2: f64, at: Point2) -> Result<Self, Refusal> {
        if !finite(&[area_mm2, at.x, at.y]) || area_mm2 <= 0.0 {
            return Err(Refusal::new(Stage::Guide, BAD_GAP));
        }
        Ok(Self { area_mm2, at })
    }
    #[must_use]
    pub fn area_mm2(&self) -> f64 {
        self.area_mm2
    }
    #[must_use]
    pub fn at(&self) -> Point2 {
        self.at
    }
}

/// A note for the status line, from one stage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Note {
    pub stage: Stage,
    pub text: String,
}

/// The result of a stage: done (with each fallback named), incomplete
/// (with each gap named; never posted silently), or refused.
#[derive(Clone, Debug, PartialEq)]
pub enum Verdict<T> {
    Done { out: T, fallbacks: Vec<Fallback>, notes: Vec<Note> },
    Incomplete { out: T, gaps: Vec<Gap>, notes: Vec<Note> },
    Refused(Refusal),
}

/// Refusal reason: an incomplete verdict that names no gap.
pub const INCOMPLETE_NO_GAP: &str = "an incomplete result must name at least one gap";

impl<T> Verdict<T> {
    #[must_use]
    pub fn done(out: T) -> Self {
        Self::Done { out, fallbacks: Vec::new(), notes: Vec::new() }
    }

    /// An incomplete result. Refused when `gaps` is empty: "incomplete"
    /// without a named gap would hide what is missing.
    pub fn incomplete(stage: Stage, out: T, gaps: Vec<Gap>) -> Result<Self, Refusal> {
        if gaps.is_empty() {
            return Err(Refusal::new(stage, INCOMPLETE_NO_GAP));
        }
        Ok(Self::Incomplete { out, gaps, notes: Vec::new() })
    }

    /// Adds a fallback. An incomplete or refused verdict keeps its state;
    /// the fallback goes in as a note so that it is still named.
    #[must_use]
    pub fn with_fallback(self, fb: Fallback) -> Self {
        match self {
            Self::Done { out, mut fallbacks, notes } => {
                fallbacks.push(fb);
                Self::Done { out, fallbacks, notes }
            }
            Self::Incomplete { out, gaps, mut notes } => {
                notes.push(Note { stage: fb.stage, text: format!("{} instead of {}: {}", fb.used, fb.asked, fb.reason) });
                Self::Incomplete { out, gaps, notes }
            }
            r @ Self::Refused(_) => r,
        }
    }

    /// The output, if the stage gave one (done or incomplete).
    #[must_use]
    pub fn out(&self) -> Option<&T> {
        match self {
            Self::Done { out, .. } | Self::Incomplete { out, .. } => Some(out),
            Self::Refused(_) => None,
        }
    }

    /// Only a done verdict can reach the post.
    #[must_use]
    pub fn postable(&self) -> bool {
        matches!(self, Self::Done { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fallback_to_the_same_behaviour_is_refused() {
        let e = Fallback::new(Stage::Order, "inside", "inside", "x").unwrap_err();
        assert_eq!(e.reason, FALLBACK_SAME);
        assert_eq!(Fallback::new(Stage::Order, "inside", "zigzag", " ").unwrap_err().reason, NO_REASON);
        let fb = Fallback::new(Stage::Order, "inside", "zigzag", "rows have no start side").unwrap();
        assert_eq!((fb.asked(), fb.used(), fb.stage()), ("inside", "zigzag", Stage::Order));
    }

    #[test]
    fn a_gap_needs_a_positive_finite_area() {
        let p = Point2::new(1.0, 2.0);
        assert!(Gap::new(0.0, p).is_err());
        assert!(Gap::new(-1.0, p).is_err());
        assert!(Gap::new(f64::NAN, p).is_err());
        assert!(Gap::new(1.0, Point2::new(f64::INFINITY, 0.0)).is_err());
        assert_eq!(Gap::new(0.5, p).unwrap().area_mm2(), 0.5);
    }

    #[test]
    fn incomplete_names_a_gap_and_is_not_postable() {
        assert_eq!(Verdict::incomplete(Stage::Guide, 1, vec![]).unwrap_err().reason, INCOMPLETE_NO_GAP);
        let gap = Gap::new(0.2, Point2::new(0.0, 0.0)).unwrap();
        let v = Verdict::incomplete(Stage::Guide, 1, vec![gap]).unwrap();
        assert!(!v.postable());
        assert_eq!(v.out(), Some(&1));
        assert!(Verdict::done(1).postable());
        assert_eq!(Verdict::<i32>::Refused(Refusal::new(Stage::Link, "x")).out(), None);
    }

    #[test]
    fn a_fallback_on_an_incomplete_verdict_stays_named() {
        let gap = Gap::new(0.2, Point2::new(0.0, 0.0)).unwrap();
        let fb = Fallback::new(Stage::Entry, "helix", "ramp", "no room").unwrap();
        let v = Verdict::incomplete(Stage::Guide, (), vec![gap]).unwrap().with_fallback(fb.clone());
        let Verdict::Incomplete { notes, .. } = v else { panic!("still incomplete") };
        assert!(notes[0].text.contains("ramp instead of helix"));
        let Verdict::Done { fallbacks, .. } = Verdict::done(()).with_fallback(fb) else { panic!("done") };
        assert_eq!(fallbacks.len(), 1);
    }

    #[test]
    fn a_refusal_drops_a_place_that_is_not_finite() {
        let r = Refusal::new(Stage::Region, "empty").at(Point2::new(f64::NAN, 0.0)).fix("select a closed loop");
        assert_eq!(r.at, None);
        assert_eq!(r.to_string(), "Region: empty (select a closed loop)");
    }
}
