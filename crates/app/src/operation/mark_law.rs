//! The mark law (#1595): the depth law an Engrave or a Trace on the
//! finished part carries itself, so its verdict never depends on
//! whether the program sim can model the stock.
//!
//! Provenance: our own design (`docs/decisions/1595-engrave-mark-law.md`)
//! over the sim crate's solid law (`verify::solid_hits`, the tool's own
//! tip and stack swept against the part's mesh); the "wider than deep"
//! reading of a mark is general machining practice. No vendor material
//! consulted.
//!
//! ## The law
//!
//! Both kinds ride the cutter's centre ON a line of the part, so they
//! enter the part by design and the tangency law stands down
//! (`Mode::Engrave`). What they may remove is a MARK: under the cutter,
//! the part's own surface lowered by at most the mark budget:
//!
//! `budget = min(Top - Bottom, cutter radius)`
//!
//! That is the depth the operation states, and never deeper than half
//! the kerf is wide. Anything deeper is a channel, and a channel the
//! model does not show is a gouge: cut it shallower, or model it (a
//! modelled groove is the part's own surface, and the law reads it as
//! such).
//!
//! The test is the solid law run against the part LOWERED by the
//! budget: a cutter that still enters the lowered part stands more than
//! the budget under the part's surface somewhere beneath it. That
//! convicts the three things a typed depth can hide - a cut deeper than
//! a mark, a Top typed below the surface the line lies on, and a wall
//! standing beside the line above it (the flank cuts it full height) -
//! and reads the mesh itself, never a lattice. The program sim credits
//! the same budget under the same cutter ([`budget_of`],
//! `stock_sim::credit_marks`), so the two judges agree where both speak.
//!
//! ## Bounded by the part under the cutter
//!
//! Lowering alone lets a part thinner than the budget drop clear of the
//! cutter - a 2 mm plate under a 2.9 mm "mark" is cut through (review of
//! PR #1615). So a second reading stands beside it: the cutting edge
//! may not come within [`MARK_FLOOR`] of an underside of the part - its
//! bottom, or the roof of a void in it (`verify::seat_skin`, the
//! undersides raised into a solid the edge may not enter). A mark is
//! thus never deeper than `min(budget, thickness under the cutter -
//! MARK_FLOOR)`; past that it is a breakthrough, said by that name.
//!
//! ## Only the edge gets the budget
//!
//! The budget is the cutting edge's (the fluted length). The shank and
//! the holder on a cutting move, and the whole assembly on a rapid,
//! are judged against the part as it stands, not lowered
//! (`verify::stack_hits`): steel that is not an edge has no mark to
//! make.

use crate::cl::Toolpath;
use crate::geometry::{Point3, Triangle};
use crate::schema::Operation;
use crate::units::{self, Unit};
use crate::verify::{self, Warning, MARK_FLOOR};

use super::material::GOUGE_TOL;
use super::{OpParams, SrcRef};

/// The solid law's sample spacing along a vertical-plane arc, mm (the
/// containment kinds' number; a trace has none, every move is swept
/// exactly).
const STEP: f64 = 0.25;

/// The mark budget of a cut from `top` to `bottom` with a cutter of
/// radius `cutter_r`: the lesser of the two, never negative; zero (no
/// entry at all is a mark) when a number is not finite.
#[must_use]
pub fn budget(top: f64, bottom: f64, cutter_r: f64) -> f64 {
    if !(top.is_finite() && bottom.is_finite() && cutter_r.is_finite()) {
        return 0.0;
    }
    (top - bottom).min(cutter_r).max(0.0)
}

/// Whether `params` is a kind the mark law speaks for.
#[must_use]
pub fn is_mark_kind(params: &OpParams) -> bool {
    matches!(params, OpParams::Engrave { .. } | OpParams::Trace { .. })
}

/// The mark budget operation `op` declares, for the program sim's
/// credit: `Some` only for an Engrave or Trace whose source is the
/// finished part (the operations that carry the law themselves) with
/// heights that resolve. Anything else declares no mark and is judged
/// against the sharp model.
#[must_use]
pub fn budget_of(world: &flecs_ecs::prelude::World, op: &Operation, cutter_r: f64) -> Option<f64> {
    if !is_mark_kind(&op.params) || !names_part(world, op.setup.as_deref(), &op.src) {
        return None;
    }
    let h = super::resolve_params(world, op.setup.as_deref(), Some(&op.src), &op.params).ok()?;
    let b = budget(h.top?, h.bottom?, cutter_r);
    (b > 0.0).then_some(b)
}

/// Whether `src` names a model that is not the setup's stock - the
/// name half of `part_solid_of`, for a caller with no material in hand.
fn names_part(world: &flecs_ecs::prelude::World, setup: Option<&str>, src: &SrcRef) -> bool {
    let stock = crate::setup::stock_entity_name(world, setup);
    match src {
        SrcRef::Stock => false,
        SrcRef::Entity(n) => *n != stock,
        SrcRef::Selection(items) => !items.is_empty() && items.iter().all(|g| g.entity != stock),
    }
}

/// `facets` lowered by `by` along Z: the part as a mark of that depth
/// may leave it, read conservatively (every surface lowered, so a wall
/// keeps its place and only heights give way).
fn lowered(facets: &[Triangle], by: f64) -> Vec<Triangle> {
    let down = |p: Point3| Point3::new(p.x, p.y, p.z - by);
    facets.iter().map(|t| Triangle::new(down(t.a), down(t.b), down(t.c))).collect()
}

/// What the mark law added to a path's warnings, by reading.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Marked {
    /// Cutting moves whose edge stands more than the budget under the
    /// part's surface.
    pub deep: usize,
    /// Cutting moves whose edge comes within [`MARK_FLOOR`] of an
    /// underside of the part: through it, or all but.
    pub through: usize,
    /// Moves whose shank or holder (or, on a rapid, any of the
    /// assembly) enters the part as it stands.
    pub stack: usize,
}

/// Append the mark law's convictions of `path` (in the frame of
/// `facets`) that the laws before it did not already report: a cutting
/// move whose edge stands more than `budget` under the part's surface,
/// or within [`MARK_FLOOR`] of its underside, is a `Gouge`; one whose
/// shank or holder enters the part as it stands a `ShankHit` on the
/// part; a rapid that does a `RapidInStock`. Returns what each reading
/// found (a move two readings convict is counted under both and
/// reported once).
pub(super) fn mark_into(
    warns: &mut Vec<Warning>,
    path: &Toolpath,
    facets: &[Triangle],
    cutter: &verify::Cutter,
    budget: f64,
    excess: f64,
) -> Marked {
    let law = verify::SolidLaw {
        step: STEP,
        // #864: less a coarse model's excess, while that is honest.
        allow: verify::excess_floor(GOUGE_TOL, excess).unwrap_or(GOUGE_TOL),
        hole_walls: false,
    };
    let budget = if budget.is_finite() { budget.max(0.0) } else { 0.0 };
    let is_gouge = |w: &Warning| matches!(w, Warning::Gouge { .. });
    // The edge against the part lowered by the budget. The stack's
    // findings there are dropped: it is judged below, unlowered.
    let deep: Vec<Warning> = verify::solid_hits(path, &lowered(facets, budget), cutter, &law).into_iter().filter(is_gouge).collect();
    // The edge against the floor the mark must leave on every underside.
    // A mesh whose undersides cannot be read fails closed in the reading
    // above (the same vertices), so nothing is lost by an empty skin.
    let skin = verify::seat_skin(facets, MARK_FLOOR).unwrap_or_default();
    let through: Vec<Warning> = verify::solid_hits(path, &skin, cutter, &law).into_iter().filter(is_gouge).collect();
    // Shank, holder and rapids against the part as it stands.
    let stack = verify::stack_hits(path, facets, cutter, &law);
    let found = Marked { deep: deep.len(), through: through.len(), stack: stack.len() };
    for w in deep.into_iter().chain(through).chain(stack) {
        if !warns.contains(&w) {
            warns.push(w);
        }
    }
    found
}

/// The notes a path carries for what the mark law convicted: the
/// budget and the floor by name and what to do about each (the warnings
/// say where). A stack finding carries its own message and no note.
#[must_use]
pub(super) fn notes(kind: &str, found: Marked, budget: f64, unit: Unit) -> Vec<String> {
    let mut out = Vec::new();
    if found.deep > 0 {
        out.push(note(kind, budget, unit));
    }
    if found.through > 0 {
        out.push(through_note(kind, unit));
    }
    out
}

/// The note for a cut deeper than a mark: the budget by name.
#[must_use]
pub fn note(kind: &str, budget: f64, unit: Unit) -> String {
    format!(
        "{kind} cuts deeper than a mark: the cutter stands more than {} {} under the part's surface \
         (the mark budget: the lesser of Top - Bottom and the cutter radius) - cut shallower, raise Top \
         to the surface the line lies on, or model the groove",
        units::len(budget, unit, 3),
        unit.suffix()
    )
}

/// The note for a breakthrough: the floor by name.
#[must_use]
pub fn through_note(kind: &str, unit: Unit) -> String {
    format!(
        "{kind} cuts through the part: under the cutter less than {} {} of the part is left on its underside \
         (a mark is bounded by the part under it, less that floor) - cut shallower; a cut through the part is \
         not a mark",
        units::len(MARK_FLOOR, unit, 3),
        unit.suffix()
    )
}

#[cfg(test)]
#[path = "mark_law_tests.rs"]
mod tests;
