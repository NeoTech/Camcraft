//! The eye's cut of a rotary slice (M13.1; moved from app's
//! `stock_sim::rotary`, #2268, so the playback in this crate reaches
//! it). Provenance: our own engines and the M13 radial law; nothing
//! from vendor binaries.
//!
//! ## The laws here
//!
//! - **The eye's pose tolerance is its chord law.** The field chords
//!   every arc at a quarter cell (`backend.rs`' `chord_tol`); the turn
//!   is chorded at the same sag ([`rotary_pose_tol`]), so the picture
//!   is one chord class throughout.
//! - **A slice turns from where the head stood.** `rotary_chords`
//!   reads a program's first move as standing on its own pose (no
//!   approach is invented); a playback slice starting mid-program
//!   would then skip its first move's turn. [`simulate_rotary_slice`]
//!   prepends a standing rapid at the pose the program held before the
//!   slice, so the turn is chorded, and the rapid cuts nothing.

use super::{RotaryChordStats, SimOp, Stock, StockError};
use crate::cl::{Move, MoveKind};
use crate::verify::RotaryAxis;

/// The chord sag the eye allows at the tool's farthest point from the
/// rotary axis: a quarter cell, the same sagitta law the field chords
/// every arc at - the picture is one chord class throughout.
#[must_use]
pub fn rotary_pose_tol(cell: f64) -> f64 {
    (cell * 0.25).max(1e-9)
}

/// Cut `op`'s moves `[a, a + moves.len())` of a rotary program through
/// the chord pre-pass, turning from the pose held before `a` (see the
/// module law). `all` is the op's whole program, `moves` the slice.
pub fn simulate_rotary_slice(
    stock: &mut Stock,
    op: &SimOp,
    all: &[Move],
    a: usize,
    ax: &RotaryAxis,
    pose_tol: f64,
) -> Result<RotaryChordStats, StockError> {
    let held = all.get(..a).and_then(|before| before.iter().rev().find_map(|m| m.axis));
    let first = op.moves.first();
    match (held, first) {
        (Some(axis), Some(m0)) if m0.axis != Some(axis) => {
            let mut v: Vec<Move> = Vec::with_capacity(op.moves.len() + 1);
            v.push(Move { rotary: None, kind: MoveKind::Rapid, from: m0.from, to: m0.from, feed: None, arc: None, axis: Some(axis) });
            v.extend_from_slice(op.moves);
            stock.simulate_rotary(&[SimOp { moves: &v, ..*op }], ax, pose_tol)
        }
        _ => stock.simulate_rotary(std::slice::from_ref(op), ax, pose_tol),
    }
}
