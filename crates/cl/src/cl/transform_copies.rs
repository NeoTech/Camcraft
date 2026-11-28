//! Copies of one path joined into one path (#3012): each copy is the
//! path shifted by [`super::translate`], and a copy's end is linked to
//! the next copy's start by rapids over a safe plane.
//!
//! Provenance: the RS-274 / ISO 6983 rapid (G0) and the common CAM
//! practice to move between parts at the clearance plane. Our own
//! model; no vendor material consulted.
//!
//! The link law: rise straight up from the copy's end to the link
//! plane, cross in XY at that plane, then go straight down to the next
//! copy's start. The link plane is the highest of the asked plane, the
//! end and the start, so a link never goes down through a path's own
//! approach height. Index records (cycle `first_move`, compensated
//! spans) shift with their copy's moves. Records that a join cannot
//! carry are refused by name, never guessed.

use crate::geometry::Point3;

use crate::cl::moves::{lattice, Move, MoveKind};
use crate::cl::path::Toolpath;
use crate::cl::pose::is_vertical;

use super::{translate, TransformError};

/// The most copies one join makes: a larger pattern is an input error.
pub const MAX_COPIES: usize = 10_000;

/// `tp` once for each offset in `offsets`, in that order, as one path.
/// Copy `k` is `translate(tp, offsets[k])`; the copies are linked by
/// rapids at the plane `link_z` (or higher, see the module law).
///
/// Refused: no offset, too many offsets, a non-finite offset or plane,
/// a plasma statement (its kerf records index one cut), and a tilted
/// move (a straight-up rise is not a safe retract for a tilted tool).
/// The records that [`translate`] refuses are refused too.
pub fn copies(tp: &Toolpath, offsets: &[[f64; 3]], link_z: f64) -> Result<Toolpath, TransformError> {
    if offsets.is_empty() || offsets.len() > MAX_COPIES {
        return Err(TransformError::BadInput("the copy count"));
    }
    if !link_z.is_finite() {
        return Err(TransformError::BadInput("the link plane"));
    }
    if tp.plasma.is_some() {
        return Err(TransformError::PlasmaCut);
    }
    if tp.moves.iter().any(|m| m.axis.is_some_and(|a| !is_vertical(&a))) {
        return Err(TransformError::TiltedMove);
    }
    let mut out: Option<Toolpath> = None;
    for offset in offsets {
        let copy = translate(tp, *offset)?;
        out = Some(match out {
            None => copy,
            Some(joined) => append(joined, copy, lattice(link_z)),
        });
    }
    out.ok_or(TransformError::BadInput("the copy count"))
}

/// `next` after `joined`, with the rapid link between them.
fn append(mut joined: Toolpath, next: Toolpath, link_z: f64) -> Toolpath {
    if let (Some(end), Some(start)) = (joined.end(), next.start()) {
        let z = link_z.max(end.z).max(start.z);
        let mut at = end;
        for to in [Point3::new(end.x, end.y, z), Point3::new(start.x, start.y, z), start] {
            if to != at {
                joined.moves.push(Move::new(MoveKind::Rapid, at, to, None));
                at = to;
            }
        }
    }
    let base = joined.moves.len();
    joined.cycles.extend(next.cycles.into_iter().map(|mut c| {
        c.first_move = c.first_move.saturating_add(base);
        c
    }));
    joined.comps.extend(next.comps.into_iter().map(|mut c| {
        c.first = c.first.saturating_add(base);
        c.last = c.last.saturating_add(base);
        c
    }));
    joined.breaks.extend(next.breaks);
    for note in next.notes {
        if !joined.notes.contains(&note) {
            joined.notes.push(note);
        }
    }
    joined.moves.extend(next.moves);
    joined
}

#[cfg(test)]
#[path = "transform_copies_tests.rs"]
mod tests;
