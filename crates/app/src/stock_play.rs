//! The machined volume unfolding with the playhead (M7.2 C): the
//! display field cut move by move up to the transport's position,
//! seeking backward from a checkpoint. Provenance: the M7.1 field
//! engine's order-independent subtraction (a running max of swept-tool
//! distances, so any prefix of the program is cut exactly by its own
//! moves); the checkpoint journal is plain periodic snapshotting.
//! Nothing from vendor binaries.
//!
//! ## The laws here
//!
//! - **Playback is the field's truth unfolding.** The stock shown at
//!   move `k` is the uncut stock minus exactly the program's first `k`
//!   moves - the same cut `stock_view::field_view` makes for all of
//!   them, so the last frame of playback IS the Deviate picture.
//! - **Backward seek re-applies from a checkpoint.** A snapshot of the
//!   cut stock is kept every `stride` moves on the way forward; seeking
//!   back clones the nearest one at or before the target and cuts the
//!   rest. The snapshot count is bounded (`MAX_CHECKPOINTS`), so memory
//!   is a constant multiple of one field, never proportional to moves.
//!   (Dirty-brick deltas instead of whole snapshots are M7.3's cost law.)
//! - **Completed moves cut the kept stock; the move in flight cuts a
//!   copy (#1516).** The stock after `k` moves is exactly the prefix
//!   cut above, and it alone is snapshotted and cut on. The move under
//!   the playhead is shown as far as the tool has travelled, on a copy
//!   of that stock (`stock_play_head`), so a frame costs one copy
//!   plus one partial sweep, never a re-cut; a seek to the end cuts
//!   everything.
//! - **The timeline and the plan agree by name, or nothing plays.**
//!   The transport's script and the posting plan are walked by
//!   different callers; an operation the plan does not hold is a
//!   refusal, never a silent skip that would show material the
//!   machine removes as standing.

use std::collections::HashSet;
use std::sync::Arc;

use crate::stock_sim::Stock;
use crate::stock_view::{build_stock, build_target, FieldView, DEFAULT_TEXELS};

/// Snapshots kept at most - the backward seek's memory bound.
pub const MAX_CHECKPOINTS: usize = 32;

/// The smallest checkpoint stride, in moves.
pub const MIN_STRIDE: usize = 64;

struct PlayOp {
    name: String,
    moves: Vec<crate::cl::Move>,
    radius: f64,
    ball: bool,
    cone: Option<f64>,
    corner: Option<f64>,
    drill: bool,
    /// Global index of this op's first move.
    start: usize,
    /// A rotary op's fitted axis: its slices are chorded about it
    /// (M13.1, `stock_sim::rotary`).
    rotary: Option<crate::verify::RotaryAxis>,
    /// A turning op's nose and spindle (#19, `stock_sim::lathe`).
    lathe: Option<crate::stock_sim::LathePlan>,
    /// An undercut op's head/neck cutter and neck length (#1444).
    undercut: Option<(crate::undercut::Undercut, f64)>,
}

/// The playback field: the uncut stock, the plan, the stock cut to
/// `at` moves, and the checkpoint journal.
pub struct FieldPlay {
    ops: Vec<PlayOp>,
    index: PlayIndex,
    total: usize,
    /// The checkpoints: the stock after each kept move count, ascending;
    /// key 0 is uncut (`camcraft_sim::play_keys`, #2181).
    keys: Keyframes<Stock>,
    cur: Stock,
    at: usize,
    /// A checkpoint replaced `cur` since the last `take_dirty`: its
    /// dirty ledger is not relative to anything uploaded (M13.1).
    restored: bool,
    /// Jump mode (M13.1 leftover): the dirty ledger is off, every
    /// take answers "send the whole layer".
    batched: bool,
    target: Option<Arc<Stock>>,
    target_note: Option<String>,
    cell: f64,
    tol: f64,
    /// The travelled fraction of move `at` the picture shows (#1516).
    frac: f64,
    /// `cur` plus the travelled part of move `at`, when there is one.
    partial: Option<Stock>,
    /// The bricks `partial` changed beyond `cur`.
    partial_bricks: HashSet<[i32; 3]>,
    /// The partial bricks of the last picture taken: the next take
    /// re-sends them, whatever stands there now.
    ghost: HashSet<[i32; 3]>,
    /// Moves cut since the build: the replay law's yardstick (#2181).
    moves_cut: usize,
    /// The layer mode (#2629, S3b): the twin-able ops' cut, off the field.
    layer: Option<cut::LayerMode>,
}

impl FieldPlay {
    /// Build the playback for the document's program at the default
    /// display budget. `judged`: carry the model's field so the eye
    /// classes each hit (the Deviate cell is armed); otherwise the
    /// picture is unjudged material.
    pub fn new(world: &flecs_ecs::prelude::World, judged: bool) -> Result<Self, String> {
        Self::with_budget(world, judged, DEFAULT_TEXELS)
    }

    /// `new` at a display budget of `texels` cells (the
    /// `field_view_with` twin: coarser is cheaper, never less honest).
    pub fn with_budget(world: &flecs_ecs::prelude::World, judged: bool, texels: f64) -> Result<Self, String> {
        Self::from_seed(PlaySeed::gather(world, judged, texels)?)
    }

    /// Build the fields from a gathered seed - no world needed, so a
    /// worker thread can pay for it.
    pub fn from_seed(seed: PlaySeed) -> Result<Self, String> {
        let PlaySeed { stock_facets, planned, cell, part, tol, rest } = seed;
        let index = PlayIndex { rest, ..PlayIndex::of(&planned) };
        let stock = build_stock(&stock_facets, cell)?;
        let mut ops = Vec::with_capacity(planned.len());
        let mut total = 0usize;
        for p in planned {
            let (moves, undercut) = match p.undercut {
                Some(u) => (u.moves, Some((u.cutter, u.neck_len))),
                None => (p.moves, None),
            };
            let n = moves.len();
            ops.push(PlayOp {
                name: p.name,
                moves,
                radius: p.radius,
                ball: p.ball,
                cone: p.cone,
                corner: p.corner,
                drill: p.drill,
                start: total,
                rotary: p.rotary,
                lathe: p.lathe,
                undercut,
            });
            total = total.saturating_add(n);
        }
        let (target, target_note) = if let (Some(_), Err(why)) = (&part, &tol) {
            // #1056: no colours off a mesh the gate refuses to judge.
            (None, Some(why.clone()))
        } else if let Some(part) = &part {
            build_target(part, cell)
        } else {
            (None, Some("playback shows material, not a verdict - arm Deviate to judge".to_string()))
        };
        let stride = total.div_ceil(MAX_CHECKPOINTS).max(MIN_STRIDE);
        Ok(FieldPlay {
            ops,
            index,
            total,
            keys: Keyframes::new(stock.clone(), stride, MAX_CHECKPOINTS),
            cur: stock,
            at: 0,
            restored: true,
            batched: false,
            target: target.map(Arc::new),
            target_note,
            cell,
            tol: tol.unwrap_or(crate::stock_sim::DEV_TOL),
            frac: 0.0,
            partial: None,
            partial_bricks: HashSet::new(),
            ghost: HashSet::new(),
            moves_cut: 0,
            layer: None,
        })
    }

    /// Moves in the whole program.
    #[must_use]
    pub const fn total(&self) -> usize {
        self.total
    }

    /// Moves the shown stock has been cut by.
    #[must_use]
    pub const fn at(&self) -> usize {
        self.at
    }

    /// Whether the model's field rides along (see `new`).
    #[must_use]
    pub const fn judged(&self) -> bool {
        self.target.is_some()
    }

    /// Snapshots held (bounded by `MAX_CHECKPOINTS` + 1).
    #[must_use]
    pub fn checkpoints(&self) -> usize {
        self.keys.len()
    }

    /// `PlayIndex::index_at` on this playback's plan.
    #[must_use]
    pub fn index_at(&self, p: &Progress) -> Option<usize> {
        self.index.index_at(p)
    }

    /// The move table (`PlayIndex`).
    #[must_use]
    pub fn index(&self) -> &PlayIndex {
        &self.index
    }

    /// The newest checkpoint at or before `k`: where a backward seek
    /// to `k` restarts cutting from (0 when none is nearer).
    #[must_use]
    pub fn floor_checkpoint(&self, k: usize) -> usize {
        self.keys.floor_at(k)
    }

    /// The next checkpoint boundary past `at` (the end at the latest):
    /// a jump cuts checkpoint to checkpoint, one batch per stride.
    #[must_use]
    pub fn next_boundary(&self) -> usize {
        self.keys.next_boundary(self.at, self.total)
    }

    /// Jump mode on or off (M13.1 leftover): on, the field keeps no
    /// dirty ledger (a jump's picture travels whole anyway, and the
    /// per-cell bookkeeping is pure cost); off again, the next take is
    /// a whole upload and the ledger starts fresh from there.
    pub fn set_batched(&mut self, on: bool) {
        if self.batched == on {
            return;
        }
        self.batched = on;
        self.cur.set_dirty_tracking(!on);
        if !on {
            self.restored = true;
        }
    }

    /// Whether jump mode is on.
    #[must_use]
    pub const fn batched(&self) -> bool {
        self.batched
    }

    /// Cut (or restore) the shown stock to exactly `k` moves; `Ok(true)`
    /// when the picture changed. `k` past the end clamps to the end.
    pub fn seek(&mut self, k: usize) -> Result<bool, String> {
        let k = k.min(self.total);
        if k == self.at {
            return Ok(false);
        }
        // The move in flight changes: its partial picture goes.
        self.drop_partial();
        // Back, or forward past a whole stride to a checkpoint ahead
        // (a precalculated keyframe, #2181): restore, then cut the rest.
        if self.keys.restores(self.at, k) {
            let (at, snap) = self.keys.floor(k).ok_or("playback lost its uncut stock")?;
            self.cur = snap.clone();
            self.at = *at;
            self.restored = true;
            // The snapshot carries the ledger mode of its own time.
            self.cur.set_dirty_tracking(!self.batched);
        }
        // Forward in stride-sized runs, offering each boundary passed
        // to the store: its count cap and byte budget decide (#2181).
        while self.at < k {
            let boundary = (self.at / self.keys.stride()).saturating_add(1).saturating_mul(self.keys.stride());
            let to = k.min(boundary);
            self.cut(self.at, to)?;
            self.at = to;
            if to == boundary {
                self.keys.offer(to, &self.cur);
            }
        }
        Ok(true)
    }

    /// The shown stock as the eye's view (a snapshot: the eye holds it
    /// while playback moves on). The copy is the cut layers only - the
    /// mesh layer and the model are shared (`Stock` clones share the
    /// field's mesh bricks; the target rides an `Arc`).
    #[must_use]
    pub fn view(&self) -> FieldView {
        FieldView {
            stock: self.partial.as_ref().unwrap_or(&self.cur).clone(),
            target: self.target.clone(),
            target_note: self.target_note.clone(),
            cell: self.cell,
            tol: self.tol,
            packed: None,
        }
    }

    /// The cut bricks changed since the last take, as one contiguous
    /// forward run; `None` when the delta is not provable that way (a
    /// checkpoint restore, the first take, jump mode, a column stock)
    /// and the caller must re-send the whole cut layer.
    /// A partial picture adds its own bricks and the last one's.
    pub fn take_dirty(&mut self) -> Option<HashSet<[i32; 3]>> {
        let d = self.cur.take_dirty_bricks();
        let ghost = std::mem::replace(&mut self.ghost, self.partial_bricks.clone());
        if self.batched {
            return None;
        }
        let restored = std::mem::replace(&mut self.restored, false);
        d.filter(|_| !restored).map(|mut set| {
            set.extend(ghost);
            set.extend(self.partial_bricks.iter().copied());
            set
        })
    }
}

#[path = "stock_play_index.rs"]
mod index;
pub use index::{PlayIndex, PlaySeed};

#[path = "stock_play_head.rs"]
mod head;
pub use head::{progress, settle_frac, Progress};

#[path = "stock_play_cut.rs"]
mod cut;

#[path = "stock_play_precalc.rs"]
mod precalc;
pub use camcraft_sim::play_keys::{Keyframes, Precalc, MAX_KEYFRAMES, PRECALC_BUDGET};
