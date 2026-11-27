//! The converter from a path kernel plan to a toolpath (#4269, part of
//! #4023): it emits the steps of each level's `Plan` in the plan's order -
//! the entry, the cuts, the links, the lifts and the exit - and makes no
//! decision of its own. A step that it cannot emit is refused by name; no
//! step is dropped and no cut is added. No old planner calls it, so the
//! output with the default planner switch does not change.
//!
//! The move rules (the safety floor of this code: it writes the moves that
//! are posted):
//! - A lateral rapid only at or above the clearance height. A rapid along
//!   the tool axis goes only up (the retract of a lift or an exit).
//! - Below the stock top only the moves of a step that the plan proved: a
//!   cut, a stay-down link, the descent and lead of an entry, the lead-out
//!   of an exit. Each step must start where the tool stands.
//! - A lift is a retract to the clearance height. Its named reason goes to
//!   the notes in the text form of the old links (`links.rs`).
//! - A cut whose curve holds an exact circle becomes arcs (G2/G3); each other
//!   cut, lead and descent is the plan's chords, as feeds.
//!
//! Provenance: our own design record `docs/decisions/4024-path-kernel-design.md`
//! (K6, the converter in motion) and `docs/decisions/4269-kernel-plan-converter.md`;
//! the G0/G1/G2/G3 vocabulary of RS-274 / ISO 6983. No vendor material consulted.

mod arcs;
mod pen;
#[cfg(test)]
mod tests;

use std::ops::Range;

use crate::cl::Toolpath;
use camcraft_pathkernel::WorkPlane;
use camcraft_pathkernel::plan::{Plan, Step};

use pen::Pen;

/// Refusal reason: heights that are not finite, or a clearance under the stock top.
pub const BAD_HEIGHTS: &str = "the converter needs a finite stock top and a finite clearance height at or above it";
/// Refusal reason: a feed rate that is not a positive finite number.
pub const BAD_FEEDS: &str = "the converter needs a positive finite cut feed and plunge feed";
/// Refusal reason: a work plane other than the setup's XY plane.
pub const NOT_SETUP_XY: &str =
    "the converter emits only plans in the setup's XY plane with the tool along +Z (a tilted or lathe plane is not supported yet)";
/// Refusal reason: a cut with a tool axis other than the plane normal.
pub const AXIS_NOT_NORMAL: &str = "the converter emits only cuts with the tool along the plane normal";
/// Refusal reason: a step kind that this converter does not know.
pub const UNKNOWN_STEP: &str = "the plan has a step kind that the converter cannot emit";
/// Refusal reason: an entry while the tool is not at the clearance height.
pub const ENTRY_NOT_CLEAR: &str = "an entry must start with the tool at the clearance height";
/// Refusal reason: an entry with an approach point under the stock top.
pub const APPROACH_IN_MATERIAL: &str = "the approach of an entry must stay at or above the stock top";
/// Refusal reason: a cut, a link or an exit that does not start where the tool stands.
pub const STEP_OFF_TOOL: &str = "a cut, a link or a lead-out must start where the tool stands: the move between them is not proved";
/// Refusal reason: a cut, a link or an exit while the tool is not at depth.
pub const STEP_NOT_DOWN: &str = "a cut, a link or an exit must follow an entry, a cut or a stay-down link";
/// Refusal reason: a plan that does not end with the tool clear.
pub const ENDS_DOWN: &str = "a plan must end with the tool at the clearance height";
/// Refusal reason: a cut with a circle whose points are not on that circle at one depth.
pub const ARC_OFF_CIRCLE: &str = "a cut with an exact circle must have its points on that circle, at one depth";
/// Refusal reason: a cut with a circle whose points do not turn one way round it.
pub const ARC_NOT_MONOTONE: &str = "a cut with an exact circle must turn one way round its centre";
/// Refusal reason: a cut that gave no move.
pub const CUT_DROPPED: &str = "a cut gave no move: the converter never drops a cut";
/// Refusal reason: a point that is not a finite number after the mapping to XYZ.
pub const NOT_FINITE: &str = "a point of the plan is not a finite number in XYZ";

/// The heights that the converter reads, in setup Z: the clearance height
/// (the plane of each lateral rapid and of each lift) and the stock top.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EmitHeights {
    clearance_z: f64,
    top_z: f64,
}

impl EmitHeights {
    pub fn new(clearance_z: f64, top_z: f64) -> Result<Self, EmitRefusal> {
        if !(clearance_z.is_finite() && top_z.is_finite() && clearance_z >= top_z) {
            return Err(EmitRefusal::new(BAD_HEIGHTS));
        }
        Ok(Self { clearance_z, top_z })
    }
    #[must_use]
    pub fn clearance_z(&self) -> f64 {
        self.clearance_z
    }
    #[must_use]
    pub fn top_z(&self) -> f64 {
        self.top_z
    }
}

/// The feed rates (mm/min): the cut feed of the cuts, the links and the
/// leads, and the plunge feed of each move down below the clearance height.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EmitFeeds {
    cut: f64,
    plunge: f64,
}

impl EmitFeeds {
    pub fn new(cut: f64, plunge: f64) -> Result<Self, EmitRefusal> {
        let ok = |v: f64| v.is_finite() && v > 0.0;
        if !(ok(cut) && ok(plunge)) {
            return Err(EmitRefusal::new(BAD_FEEDS));
        }
        Ok(Self { cut, plunge })
    }
    #[must_use]
    pub fn cut(&self) -> f64 {
        self.cut
    }
    #[must_use]
    pub fn plunge(&self) -> f64 {
        self.plunge
    }
}

/// One level: the plan that the caller made for it and the work plane of
/// that level (its origin is on the level; a plan depth of 0 is the level).
#[derive(Clone, Copy, Debug)]
pub struct LevelPlan<'a> {
    pub plan: &'a Plan,
    pub plane: WorkPlane,
}

/// Why the converter refused, with the level and the step where it did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EmitRefusal {
    reason: &'static str,
    level: Option<usize>,
    step: Option<usize>,
}

impl EmitRefusal {
    fn new(reason: &'static str) -> Self {
        Self { reason, level: None, step: None }
    }
    fn at(self, level: usize, step: Option<usize>) -> Self {
        Self { level: Some(level), step, ..self }
    }
    /// The named reason (one of the constants of this module).
    #[must_use]
    pub fn reason(&self) -> &'static str {
        self.reason
    }
    #[must_use]
    pub fn level(&self) -> Option<usize> {
        self.level
    }
    #[must_use]
    pub fn step(&self) -> Option<usize> {
        self.step
    }
}

impl std::fmt::Display for EmitRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "kernel plan to toolpath: {}", self.reason)?;
        if let Some(l) = self.level {
            write!(f, " (level {}", l.saturating_add(1))?;
            if let Some(s) = self.step {
                write!(f, ", step {}", s.saturating_add(1))?;
            }
            write!(f, ")")?;
        }
        Ok(())
    }
}

impl std::error::Error for EmitRefusal {}

/// The toolpath of the plans, and what the converter emitted for them.
#[derive(Clone, Debug, Default)]
pub struct Emitted {
    pub path: Toolpath,
    /// The move range of each cut, in plan order: one range for each cut
    /// step of the plans, never empty.
    pub cut_moves: Vec<Range<usize>>,
    /// The count of entries, stay-down links, lifts and exits emitted.
    pub entries: usize,
    pub stays: usize,
    pub lifts: usize,
    pub exits: usize,
}

/// The toolpath of one level's plan. See [`emit_levels`].
pub fn emit_plan(plan: &Plan, plane: WorkPlane, h: EmitHeights, f: EmitFeeds) -> Result<Emitted, EmitRefusal> {
    emit_levels(&[LevelPlan { plan, plane }], h, f)
}

/// The toolpath of the plans of each level, in the order given (the level
/// order of the caller: the converter does not sort, merge or re-plan).
/// Each plan starts with its own entry and ends with its own exit, so the
/// move from one level to the next is the exit of the one and the entry of
/// the next, through the clearance height.
pub fn emit_levels(levels: &[LevelPlan<'_>], h: EmitHeights, f: EmitFeeds) -> Result<Emitted, EmitRefusal> {
    let mut pen = Pen::new(h, f);
    for (li, level) in levels.iter().enumerate() {
        pen.set_plane(level.plane).map_err(|e| e.at(li, None))?;
        for (si, step) in level.plan.steps().iter().enumerate() {
            let done = match step {
                Step::Entry(e) => pen.entry(e),
                Step::Cut(c) => pen.cut(c),
                Step::Link(hop) => pen.link(hop),
                Step::Exit(x) => pen.exit(x),
                _ => Err(EmitRefusal::new(UNKNOWN_STEP)),
            };
            done.map_err(|e| e.at(li, Some(si)))?;
        }
        pen.end_level().map_err(|e| e.at(li, None))?;
    }
    Ok(pen.finish())
}
