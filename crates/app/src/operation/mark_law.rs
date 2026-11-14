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

use crate::cl::Toolpath;
use crate::geometry::{Point3, Triangle};
use crate::schema::Operation;
use crate::units::{self, Unit};
use crate::verify::{self, Warning};

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

/// Append the mark law's convictions of `path` (in the frame of
/// `facets`) that the laws before it did not already report, in move
/// order: a cutting move whose edge stands more than `budget` under the
/// part is a `Gouge`, one whose shank or holder does a `ShankHit` on
/// the part, a rapid a `RapidInStock`. Returns how many it added.
pub(super) fn mark_into(
    warns: &mut Vec<Warning>,
    path: &Toolpath,
    facets: &[Triangle],
    cutter: &verify::Cutter,
    budget: f64,
    excess: f64,
) -> usize {
    let law = verify::SolidLaw {
        step: STEP,
        // #864: less a coarse model's excess, while that is honest.
        allow: verify::excess_floor(GOUGE_TOL, excess).unwrap_or(GOUGE_TOL),
        hole_walls: false,
    };
    let budget = if budget.is_finite() { budget.max(0.0) } else { 0.0 };
    let mut added = 0;
    for w in verify::solid_hits(path, &lowered(facets, budget), cutter, &law) {
        if !warns.contains(&w) {
            warns.push(w);
            added += 1;
        }
    }
    added
}

/// The note a path carries when the mark law convicted it: the budget
/// by name and what to do about it (the warnings say where).
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

#[cfg(test)]
#[path = "mark_law_tests.rs"]
mod tests;
