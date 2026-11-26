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

/// Refusal reason: a fallback, a note, a lift or a record with no reason text.
pub const NO_REASON: &str = "a fallback, a note, a lift or a refusal must name its reason";

/// A stage that cannot meet its law refuses with a named reason. Each
/// `reason` is a named `const` of the stage that writes it.
#[derive(Clone, Debug, PartialEq)]
pub struct Refusal {
    stage: Stage,
    reason: &'static str,
    at: Option<Point2>,
    fix: Option<&'static str>,
}

impl Refusal {
    /// A refusal. A blank reason becomes [`NO_REASON`]: a refusal is never
    /// silent about why.
    #[must_use]
    pub fn new(stage: Stage, reason: &'static str) -> Self {
        let reason = if reason.trim().is_empty() { NO_REASON } else { reason };
        Self { stage, reason, at: None, fix: None }
    }

    /// The refusal at one spot. A spot that is not finite is dropped: the
    /// refusal still stands, without a place.
    #[must_use]
    pub fn at(mut self, p: Point2) -> Self {
        self.at = finite(&[p.x, p.y]).then_some(p);
        self
    }

    /// The fix to show. A blank fix is dropped.
    #[must_use]
    pub fn fix(mut self, fix: &'static str) -> Self {
        self.fix = (!fix.trim().is_empty()).then_some(fix);
        self
    }
    #[must_use]
    pub fn stage(&self) -> Stage {
        self.stage
    }
    #[must_use]
    pub fn reason(&self) -> &'static str {
        self.reason
    }
    #[must_use]
    pub fn place(&self) -> Option<Point2> {
        self.at
    }
    #[must_use]
    pub fn fix_text(&self) -> Option<&'static str> {
        self.fix
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

impl Fallback {
    /// A fallback from `asked` to `used`, for `reason`. Refused when the two
    /// are the same, or when a name is blank.
    pub fn new(stage: Stage, asked: &'static str, used: &'static str, reason: &'static str) -> Result<Self, Refusal> {
        if [reason, asked, used].iter().any(|s| s.trim().is_empty()) {
            return Err(Refusal::new(stage, NO_REASON));
        }
        if asked.trim() == used.trim() {
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

/// The gaps of an incomplete result: one or more.
#[derive(Clone, Debug, PartialEq)]
pub struct Gaps(Vec<Gap>);

/// Refusal reason: an incomplete verdict that names no gap.
pub const INCOMPLETE_NO_GAP: &str = "an incomplete result must name at least one gap";

impl Gaps {
    pub fn new(stage: Stage, gaps: Vec<Gap>) -> Result<Self, Refusal> {
        if gaps.is_empty() { Err(Refusal::new(stage, INCOMPLETE_NO_GAP)) } else { Ok(Self(gaps)) }
    }
    #[must_use]
    pub fn list(&self) -> &[Gap] {
        &self.0
    }
}

/// A note for the status line, from one stage. Its text is not blank.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Note {
    stage: Stage,
    text: String,
}

impl Note {
    pub fn new(stage: Stage, text: impl Into<String>) -> Result<Self, Refusal> {
        let text = text.into();
        if text.trim().is_empty() { Err(Refusal::new(stage, NO_REASON)) } else { Ok(Self { stage, text }) }
    }
    #[must_use]
    pub fn stage(&self) -> Stage {
        self.stage
    }
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// The result of a stage: done (with each fallback named), incomplete
/// (with each gap named; never posted silently), or refused. Each variant
/// holds only validated records.
#[derive(Clone, Debug, PartialEq)]
pub enum Verdict<T> {
    Done { out: T, fallbacks: Vec<Fallback>, notes: Vec<Note> },
    Incomplete { out: T, gaps: Gaps, notes: Vec<Note> },
    Refused(Refusal),
}

impl<T> Verdict<T> {
    #[must_use]
    pub fn done(out: T) -> Self {
        Self::Done { out, fallbacks: Vec::new(), notes: Vec::new() }
    }

    /// An incomplete result. Refused when `gaps` is empty: "incomplete"
    /// without a named gap would hide what is missing.
    pub fn incomplete(stage: Stage, out: T, gaps: Vec<Gap>) -> Result<Self, Refusal> {
        Ok(Self::Incomplete { out, gaps: Gaps::new(stage, gaps)?, notes: Vec::new() })
    }

    /// Adds a fallback. An incomplete verdict keeps its state, and the
    /// fallback goes in as a note so that it is still named. A refused
    /// verdict stays refused.
    #[must_use]
    pub fn with_fallback(self, fb: Fallback) -> Self {
        match self {
            Self::Done { out, mut fallbacks, notes } => {
                fallbacks.push(fb);
                Self::Done { out, fallbacks, notes }
            }
            Self::Incomplete { out, gaps, mut notes } => {
                let text = format!("{} instead of {}: {}", fb.used, fb.asked, fb.reason);
                notes.push(Note { stage: fb.stage, text });
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
