//! The playback's world half (#2268): the inputs read from the document
//! (`PlaySeed::gather`) and the world-built playback (`FieldPlayWorld`).
//! The field, its move table, the playhead and the cut are camcraft-sim's
//! (`camcraft_sim::play`); this module re-exports them, so its callers
//! keep their paths. Provenance: our own playback design; nothing from
//! vendor binaries.

use crate::stock_sim::Planned;
use crate::stock_view::display_inputs_of;

pub use camcraft_sim::play::{
    progress, settle_frac, Clock, FieldPlay, Keyframes, PlayIndex, PlayInputs, PlayOpPlan, Precalc, Progress,
    MAX_CHECKPOINTS, MAX_KEYFRAMES, MIN_STRIDE, PRECALC_BUDGET,
};

/// Everything playback reads from the world, gathered on the world's
/// thread: plain data (`PlayInputs`), so the fields can be built (and
/// cut) elsewhere. `FieldPlay::from_seed` and the `PlayWorker` spawns
/// take it as it is.
#[derive(Clone, Debug)]
pub struct PlaySeed(PlayInputs);

impl PlaySeed {
    /// Read the document's program and stock at a display budget of
    /// `texels` cells. `judged`: carry the model's field so the eye
    /// classes each hit (the Deviate cell is armed); otherwise the
    /// picture is unjudged material.
    pub fn gather(world: &flecs_ecs::prelude::World, judged: bool, texels: f64) -> Result<Self, String> {
        // #2280: the program plays up to the first op with no toolpath.
        let (planned, rest) = crate::stock_sim::plan_program_until_ungenerated(world)?;
        let setup = rest.first().and_then(|(_, e)| crate::model::operation_of(world, e)).and_then(|op| op.setup);
        let (stock_facets, planned, cell) = display_inputs_of(world, texels, planned, setup)?;
        let part = if judged { Some(crate::stock_sim::part_facets(world)?) } else { None };
        let tol = crate::model_fidelity::program_tol(world);
        let rest = rest.into_iter().map(|(name, _)| name).collect();
        let ops = planned.into_iter().map(op_plan).collect();
        Ok(PlaySeed(PlayInputs { stock_facets, ops, cell, part, tol, rest, clock: crate::clock::monotonic }))
    }

    /// The move table the transport maps onto (cheap; no field built).
    #[must_use]
    pub fn index(&self) -> PlayIndex {
        self.0.index()
    }
}

impl From<PlaySeed> for PlayInputs {
    fn from(seed: PlaySeed) -> Self {
        seed.0
    }
}

/// One planned op as playback cuts it: an undercut op's own moves and
/// its head/neck cutter (#1444), else the op's moves.
fn op_plan(p: Planned) -> PlayOpPlan {
    let (moves, undercut) = match p.undercut {
        Some(u) => (u.moves, Some((u.cutter, u.neck_len))),
        None => (p.moves, None),
    };
    PlayOpPlan {
        name: p.name,
        moves,
        radius: p.radius,
        ball: p.ball,
        cone: p.cone,
        corner: p.corner,
        drill: p.drill,
        rotary: p.rotary,
        lathe: p.lathe,
        undercut,
    }
}

/// The playback built straight from the world (`PlaySeed::gather`, then
/// `FieldPlay::from_seed`).
pub trait FieldPlayWorld: Sized {
    /// Build the playback for the document's program at the default
    /// display budget (see `PlaySeed::gather` for `judged`).
    fn new(world: &flecs_ecs::prelude::World, judged: bool) -> Result<Self, String>;

    /// `new` at a display budget of `texels` cells (the
    /// `field_view_with` twin: coarser is cheaper, never less honest).
    fn with_budget(world: &flecs_ecs::prelude::World, judged: bool, texels: f64) -> Result<Self, String>;
}

impl FieldPlayWorld for FieldPlay {
    fn new(world: &flecs_ecs::prelude::World, judged: bool) -> Result<Self, String> {
        Self::with_budget(world, judged, crate::stock_view::DEFAULT_TEXELS)
    }

    fn with_budget(world: &flecs_ecs::prelude::World, judged: bool, texels: f64) -> Result<Self, String> {
        FieldPlay::from_seed(PlaySeed::gather(world, judged, texels)?)
    }
}
