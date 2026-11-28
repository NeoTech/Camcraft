//! The playback's move table and its gathered inputs: where each op's
//! moves sit on the global move count (`PlayIndex`), and the plain data
//! read from the world on its own thread (`PlaySeed`). Split from
//! `stock_play.rs` (#2528); structure only.
//! Provenance: our own playback design (see `stock_play`).

use super::Progress;
use crate::geometry::Triangle;
use crate::stock_sim::Planned;
use crate::stock_view::display_inputs_of;

/// Where each op's moves sit on the global move count: what the UI
/// needs to turn a playhead into a move index, without the moves.
#[derive(Clone, Debug, Default)]
pub struct PlayIndex {
    /// (op name, global index of its first move, its move count).
    pub(super) ops: Vec<(String, usize, usize)>,
    pub(super) total: usize,
    /// The op the playback stops before and every op after it (#2280,
    /// `PlaySeed::gather`); empty when the whole program plays.
    pub(super) rest: Vec<String>,
}

fn planned_moves(p: &Planned) -> &[crate::cl::Move] {
    p.undercut.as_ref().map_or(p.moves.as_slice(), |u| u.moves.as_slice())
}

impl PlayIndex {
    pub(super) fn of(planned: &[Planned]) -> Self {
        let mut ix = PlayIndex::default();
        for p in planned {
            let n = planned_moves(p).len();
            ix.ops.push((p.name.clone(), ix.total, n));
            ix.total = ix.total.saturating_add(n);
        }
        ix
    }

    /// Moves in the whole program.
    #[must_use]
    pub const fn total(&self) -> usize {
        self.total
    }

    /// The global move index the playhead `p` stands at, or `None`
    /// when the plan holds no op of that name. An op's timeline is its
    /// moves, one segment each, behind at most one connector rapid (the
    /// air between programs, cut by nothing): a span one longer than
    /// the op's moves carries that connector, and it is not a move.
    #[must_use]
    pub fn index_at(&self, p: &Progress) -> Option<usize> {
        let (_, start, len) = self.ops.iter().find(|o| o.0 == p.op)?;
        let connector = usize::from(p.span == len + 1);
        let done = p.segs_done.saturating_sub(connector).min(*len);
        Some(start.saturating_add(done))
    }
}

/// Everything playback reads from the world, gathered on the world's
/// thread: plain data, so the fields can be built (and cut) elsewhere.
pub struct PlaySeed {
    pub(super) stock_facets: Vec<Triangle>,
    pub(super) planned: Vec<Planned>,
    pub(super) cell: f64,
    /// The model's world facets when the playback is judged.
    pub(super) part: Option<Vec<Triangle>>,
    /// The verdict tolerance the colours are gated at, or why the
    /// program is not judged (#1056, `model_fidelity::program_tol`).
    pub(super) tol: Result<f64, String>,
    /// See [`PlayIndex::stop`].
    pub(super) rest: Vec<String>,
}

impl PlaySeed {
    /// Read the document's program and stock at a display budget of
    /// `texels` cells (see `FieldPlay::new` for `judged`).
    pub fn gather(world: &flecs_ecs::prelude::World, judged: bool, texels: f64) -> Result<Self, String> {
        // #2280: the program plays up to the first op with no toolpath.
        let (planned, rest) = crate::stock_sim::plan_program_until_ungenerated(world)?;
        let setup = rest.first().and_then(|(_, e)| crate::model::operation_of(world, e)).and_then(|op| op.setup);
        let (stock_facets, planned, cell) = display_inputs_of(world, texels, planned, setup)?;
        let part = if judged { Some(crate::stock_sim::part_facets(world)?) } else { None };
        let tol = crate::model_fidelity::program_tol(world);
        let rest = rest.into_iter().map(|(name, _)| name).collect();
        Ok(PlaySeed { stock_facets, planned, cell, part, tol, rest })
    }

    /// The move table the transport maps onto (cheap; no field built).
    #[must_use]
    pub fn index(&self) -> PlayIndex {
        PlayIndex { rest: self.rest.clone(), ..PlayIndex::of(&self.planned) }
    }
}
