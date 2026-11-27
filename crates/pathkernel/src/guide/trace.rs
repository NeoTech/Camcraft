//! The trace guide (#4176, K2): the picked chains themselves, offset by the
//! choice to their left, their right or not at all, open or closed, for
//! Trace, 2D Contour on a chain, Engrave on a line and 2D Chamfer; with the
//! stretches in the keep-out area left out and named.
//! Provenance: the offset of a closed curve as the polygon offset of its
//! area (Held, "On the Computational Geometry of Pocket Machining", 1991),
//! the offset of an open chain as the trimmed boundary of its Minkowski sum
//! with a disc (Tiller and Hanson, 1984), both in `camcraft-geometry`; our
//! own design record (`RECOMMENDED.md`, `## #4024`) "Stage 2"
//! and `docs/decisions/4176-kernel-guides-medial-trace-helix.md`. Written
//! new: no code path of the old Trace, Contour or Chamfer planners of
//! `camcraft-motion` is ported.

use camcraft_geometry::geo2d::{RingClip, offset_side};

use super::cover::{Raw, check_tol, finish, geo};
use super::rings::length;
use super::{Family, GuideChoice, GuidePlanner, GuideSet};
use crate::region::{ChainRegion, KernelRegion};
use crate::verdict::{Note, Refusal, Stage, Verdict};
use crate::{KERNEL_TOL, Loop, Point2, Region};

/// Refusal reason: the trace planner was given another guide choice.
pub const NOT_TRACE: &str = "the trace planner plans only a trace choice";
/// Refusal reason: the trace planner was given an area.
pub const NOT_A_CHAIN: &str = "the trace planner plans the chains of a trace: build the region with chain_region";
/// Refusal reason: no chain has an offset curve.
pub const TRACE_COLLAPSES: &str = "the offset of each chain collapses: there is no trace curve";
/// Refusal reason: the whole trace is in the keep-out area.
pub const TRACE_KEPT_OUT: &str = "the whole trace is in the keep-out area";

/// The trace guide planner. It reads only the chain region and the trace
/// offset: no start side, no order, no link (laws 2 and 3). Each curve is
/// of family `Trace`, index 0.
#[derive(Clone, Copy, Debug, Default)]
pub struct TracePlanner;

impl GuidePlanner for TracePlanner {
    fn guide(&self, r: &KernelRegion, g: &GuideChoice, tol: f64) -> Result<Verdict<GuideSet>, Refusal> {
        let GuideChoice::Trace(t) = g else {
            return Err(Refusal::new(Stage::Guide, NOT_TRACE));
        };
        let Some(c) = r.chains() else {
            return Err(Refusal::new(Stage::Guide, NOT_A_CHAIN));
        };
        plan(c, t.get(), check_tol(tol)?)
    }
}

fn note(notes: &mut Vec<Note>, text: String) -> Result<(), Refusal> {
    notes.push(Note::new(Stage::Guide, text)?);
    Ok(())
}

/// The offset curves of one closed chain, counter-clockwise: its area
/// shrunk by `d` (to its left, the inside) or grown by `-d`.
fn closed(pts: &[Point2], d: f64, tol: f64) -> Result<Vec<Raw>, Refusal> {
    let mut ccw = pts.to_vec();
    let l = geo(Loop::new(ccw.clone()))?;
    if l.area() < 0.0 {
        ccw.reverse();
    }
    if d.abs() <= KERNEL_TOL {
        return Ok(vec![Raw { pts: ccw, closed: true, k: 0 }]);
    }
    let area = geo(Region::new(vec![geo(Loop::new(ccw))?]))?;
    let off = geo(area.offset_fine(-d, tol))?;
    Ok(off.loops().iter().map(|l| Raw { pts: l.points().to_vec(), closed: true, k: 0 }).collect())
}

fn plan(c: &ChainRegion, d: f64, tol: f64) -> Result<Verdict<GuideSet>, Refusal> {
    let (mut raws, mut notes) = (Vec::new(), Vec::new());
    for chain in c.chains() {
        let pts = chain.points();
        let at = pts.first().copied().unwrap_or_default();
        let made = if chain.is_closed() {
            closed(pts, d, tol)?
        } else if d.abs() <= KERNEL_TOL {
            vec![Raw { pts: pts.to_vec(), closed: false, k: 0 }]
        } else {
            let pieces = geo(offset_side(pts, d))?;
            if pieces.len() > 1 {
                let text = format!("the offset of the open chain at ({:.3}, {:.3}) is {} pieces: the chain comes nearer to itself than the offset", at.x, at.y, pieces.len());
                note(&mut notes, text)?;
            }
            pieces.into_iter().map(|p| Raw { pts: p.points, closed: p.closed, k: 0 }).collect()
        };
        if made.is_empty() {
            note(&mut notes, format!("the offset of {d:.3} mm of the chain at ({:.3}, {:.3}) collapses: it has no trace curve", at.x, at.y))?;
        }
        raws.extend(made);
    }
    if raws.is_empty() {
        return Err(Refusal::new(Stage::Guide, TRACE_COLLAPSES).fix("use a smaller offset"));
    }
    let raws = keep_out(c, raws, &mut notes)?;
    if raws.is_empty() {
        return Err(Refusal::new(Stage::Guide, TRACE_KEPT_OUT).fix("move the keep-out area or pick another chain"));
    }
    let (curves, marks, more) = finish(&raws, Vec::new(), |_| Family::Trace)?;
    notes.extend(more);
    Ok(Verdict::Done { out: GuideSet::with_marks(curves, marks)?, fallbacks: Vec::new(), notes })
}

/// The curves less their stretches where the tool disc meets the keep-out
/// area; the length left out is named in a note.
fn keep_out(c: &ChainRegion, raws: Vec<Raw>, notes: &mut Vec<Note>) -> Result<Vec<Raw>, Refusal> {
    if c.keep_out().is_empty() {
        return Ok(raws);
    }
    let zone = geo(c.keep_out().offset(c.tool().radius()))?;
    let (mut out, mut lost) = (Vec::new(), 0.0);
    for r in raws {
        let walk = r.walk();
        let before = length(&walk);
        let kept: Vec<Raw> = if r.closed {
            match geo(zone.clip_ring(&r.pts, false))? {
                RingClip::Whole => vec![r],
                RingClip::Pieces(ps) => ps.into_iter().map(|s| Raw { pts: s, closed: false, k: 0 }).collect(),
            }
        } else {
            geo(zone.clip_polyline(&r.pts, false))?.into_iter().map(|s| Raw { pts: s, closed: false, k: 0 }).collect()
        };
        let after: f64 = kept.iter().map(|k| length(&k.walk())).sum();
        lost += (before - after).max(0.0);
        out.extend(kept.into_iter().filter(|k| k.pts.len() >= 2));
    }
    if lost > KERNEL_TOL {
        note(notes, format!("the trace leaves out {lost:.3} mm of its curves where the tool meets the keep-out area"))?;
    }
    Ok(out)
}
