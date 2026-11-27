//! #3035: the rest grid in two halves - the world reads (`stage`) and
//! the stock simulation (`StagedPrior::simulate`), which reads no world
//! and so runs in the generation job, off the session's thread.
//! Provenance: our own design (the #2257 / M5.3 B collect-run seam).

use std::cell::Cell;

use super::ahead::{Ahead, digest_of};
use super::{PriorState, carried_grid, chain_of, prior_digest, simulate_ahead};
use crate::machining2d::GenError;
use crate::stock_sim::carry::{self, Held};
use crate::stock_sim::SimOp;
use flecs_ecs::prelude::World;

thread_local! {
    static SIMULATIONS: Cell<u64> = const { Cell::new(0) };
}

/// How many same-setup rest grids this thread has simulated - the pin
/// that staging a rest operation simulates none (#3035).
#[must_use]
pub fn prior_simulations() -> u64 {
    SIMULATIONS.with(Cell::get)
}

/// A rest grid whose world reads are done: ready, or the stock and the
/// cached moves ahead, still to be cut.
pub struct StagedPrior {
    ready: Option<PriorState>,
    ahead: Option<(Held, Vec<Ahead>, u64)>,
    keep_grid: bool,
}

impl StagedPrior {
    fn ready(p: PriorState) -> Self {
        Self { ready: Some(p), ahead: None, keep_grid: true }
    }

    /// #319: declared material past a carried grid - no mask. The
    /// simulation still runs, so its refusal still refuses.
    #[must_use]
    pub fn without_grid(self) -> Self {
        Self { keep_grid: false, ..self }
    }

    /// Cut the moves ahead into the stock; no world is read here.
    pub fn simulate(self) -> Result<PriorState, GenError> {
        let mut state = match (self.ready, self.ahead) {
            (Some(p), _) => p,
            (None, Some((mut held, ahead, digest))) => {
                SIMULATIONS.with(|n| n.set(n.get().saturating_add(1)));
                let refused = |e: String| GenError::Invalid(format!("rest from previous operations: {e}"));
                let runs: Vec<(SimOp, bool)> = ahead
                    .iter()
                    .filter(|a| a.cut)
                    .map(|a| (SimOp { moves: &a.moves, radius: a.radius, ball: a.ball, cone: a.cone, corner: a.corner }, a.drill))
                    .collect();
                simulate_ahead(&mut held, &runs).map_err(refused)?;
                PriorState { grid: Some(held.into_tops().map_err(refused)?), digest: Some(digest) }
            }
            (None, None) => return Err(GenError::Invalid("internal: a staged rest grid holds nothing".into())),
        };
        if !self.keep_grid {
            state.grid = None;
        }
        Ok(state)
    }
}

/// Every world read of [`super::prior_state`], in its order and with its
/// refusals; the same-setup simulation is left to [`StagedPrior::simulate`].
/// A carried chain (#318) stays whole here: its body is cached per thread.
pub fn stage(world: &World, name: &str, setup: Option<&str>, tool_r: f64) -> Result<StagedPrior, GenError> {
    let chain = chain_of(world, name, setup, true).map_err(GenError::Invalid)?;
    if chain.crosses() {
        // The stock this setup starts from is what the earlier setups
        // left, carried (#318) - a mask even when nothing ahead cut.
        let digest = prior_digest(world, name, setup);
        let min_r = chain.ops().filter(|a| a.cut && a.radius > 0.0).map(|a| a.radius).fold(tool_r, f64::min);
        let grid = carried_grid(world, &chain, min_r, true)
            .map_err(|e| GenError::Invalid(format!("rest from previous setups: {e}")))?;
        return Ok(StagedPrior::ready(PriorState { grid: Some(grid), digest }));
    }
    let ahead = chain.into_ops();
    if ahead.is_empty() {
        return Ok(StagedPrior::ready(PriorState { grid: None, digest: None }));
    }
    let digest = digest_of(&ahead);
    if !ahead.iter().any(|a| a.cut) {
        // Nothing cut before us: the stock as it came is the rest as
        // it stands, and no mask is a cheaper way to say that.
        return Ok(StagedPrior::ready(PriorState { grid: None, digest: Some(digest) }));
    }
    let min_r = ahead.iter().filter(|a| a.cut && a.radius > 0.0).map(|a| a.radius).fold(tool_r, f64::min);
    let refused = |e: String| GenError::Invalid(format!("rest from previous operations: {e}"));
    // #637: the stock on this thread's engine (the column grid by default).
    let held = carry::first_held(world, setup, min_r).map_err(refused)?;
    Ok(StagedPrior { ready: None, ahead: Some((held, ahead, digest)), keep_grid: true })
}
