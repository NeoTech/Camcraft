//! The playback's move table and its plain inputs: where each op's
//! moves sit on the global move count (`PlayIndex`), and the data the
//! host reads from its document on its own thread (`PlayInputs`). Split
//! from `play.rs` (#2528); moved from app (#2268). App's
//! `stock_play::PlaySeed::gather` fills the inputs from the world.
//! Provenance: our own playback design (see `play`).

use std::time::Duration;

use super::Progress;
use crate::cl::Move;
use crate::geometry::Triangle;
use crate::stock_sim::LathePlan;
use crate::verify::RotaryAxis;
use camcraft_tools::undercut::Undercut;

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

impl PlayIndex {
    pub(super) fn of(ops: &[PlayOpPlan]) -> Self {
        let mut ix = PlayIndex::default();
        for p in ops {
            let n = p.moves.len();
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

/// One op of the program as playback cuts it, in posting order.
#[derive(Clone, Debug)]
pub struct PlayOpPlan {
    /// The operation's name - the timeline's key for it.
    pub name: String,
    /// The moves the field cuts (an undercut op's own moves).
    pub moves: Vec<Move>,
    pub radius: f64,
    pub ball: bool,
    pub cone: Option<f64>,
    pub corner: Option<f64>,
    pub drill: bool,
    /// A rotary op's fitted axis: its slices are chorded about it
    /// (M13.1, `stock_sim::simulate_rotary_slice`).
    pub rotary: Option<RotaryAxis>,
    /// A turning op's nose and spindle (#19).
    pub lathe: Option<LathePlan>,
    /// An undercut op's head/neck cutter and neck length (#1444).
    pub undercut: Option<(Undercut, f64)>,
}

/// The host's monotonic clock: the time since a fixed origin of its
/// choice. The worker times its chunks and its publications with it,
/// so this crate reads no clock of its own (the #31 wasm32 seam is the
/// host's: app passes `camcraft_app::clock`).
pub type Clock = fn() -> Duration;

/// Everything playback reads from the document, gathered on the
/// document's thread: plain data, so the fields can be built (and cut)
/// elsewhere.
#[derive(Clone, Debug)]
pub struct PlayInputs {
    /// The stock's world facets.
    pub stock_facets: Vec<Triangle>,
    /// The program, op by op.
    pub ops: Vec<PlayOpPlan>,
    /// The display cell.
    pub cell: f64,
    /// The model's world facets when the playback is judged.
    pub part: Option<Vec<Triangle>>,
    /// The verdict tolerance the colours are gated at, or why the
    /// program is not judged (#1056, `model_fidelity::program_tol`).
    pub tol: Result<f64, String>,
    /// See [`PlayIndex::stop`].
    pub rest: Vec<String>,
    /// The host's clock (see [`Clock`]).
    pub clock: Clock,
}

impl PlayInputs {
    /// The move table the transport maps onto (cheap; no field built).
    #[must_use]
    pub fn index(&self) -> PlayIndex {
        PlayIndex { rest: self.rest.clone(), ..PlayIndex::of(&self.ops) }
    }
}
