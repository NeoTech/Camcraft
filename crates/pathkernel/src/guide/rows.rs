//! The row guide (#4176, K2): parallel rows at the choice's angle, spaced
//! evenly at the stepover or less across the cutter-centre area and
//! clipped to it, for Face and Zigzag; with the area's boundary as a
//! closing ring when the choice asks for one.
//! Provenance: direction-parallel (zigzag) area clearing as in Held, "On
//! the Computational Geometry of Pocket Machining" (1991); the clip of a
//! line by a polygon from the published computer-graphics literature
//! (`Region::clip_polyline` of `camcraft-geometry`); our own design record
//! (`RECOMMENDED.md`, `## #4024`) "Stage 2" and
//! `docs/decisions/4176-kernel-guides-rings-rows.md`. Written new: no code
//! path of the old Face and Zigzag planners of `camcraft-motion` is ported.

use super::cover::{ARC_SAG, Ask, Raw, check_step, check_tol, finish, geo, judge, planar};
use super::mark::{Mark, MarkKind};
use super::rings::length;
use super::{Family, GuideChoice, GuidePlanner, GuideSet, RowsChoice};
use crate::region::{KernelRegion, PlanarRegion};
use crate::verdict::{Note, Refusal, Stage, Verdict};
use crate::Point2;

/// Refusal reason: the row planner was given another guide choice.
pub const NOT_ROWS: &str = "the row planner plans only a rows choice";
/// Refusal reason: more rows than the planner makes.
pub const TOO_MANY_ROWS: &str = "the area needs more rows than the planner makes (100000)";

/// The most rows of one plan.
const MAX_ROWS: f64 = 100_000.0;

/// The row guide planner. It reads only the region and the rows choice: no
/// start side, no order, no link (laws 2 and 3). Each row piece is an open
/// curve of family `Row` with the index of its row; the closing ring is of
/// family `Ring`, index 0, marked [`MarkKind::Closing`].
#[derive(Clone, Copy, Debug, Default)]
pub struct RowPlanner;

impl GuidePlanner for RowPlanner {
    fn guide(&self, r: &KernelRegion, g: &GuideChoice, tol: f64) -> Result<Verdict<GuideSet>, Refusal> {
        let GuideChoice::Rows(c) = g else {
            return Err(Refusal::new(Stage::Guide, NOT_ROWS));
        };
        plan(planar(r)?, c, check_tol(tol)?)
    }
}

fn plan(p: &PlanarRegion, c: &RowsChoice, tol: f64) -> Result<Verdict<GuideSet>, Refusal> {
    let rad = p.tool().radius();
    let step = check_step(c.stepover(), rad)?;
    let (s, co) = c.angle().to_radians().sin_cos();
    let (u, n) = ((co, s), (-s, co));
    let center = p.center();
    let pts: Vec<Point2> = center.loops().iter().flat_map(|l| l.points()).copied().collect();
    let Some((lo, hi)) = center.bounds() else {
        return Err(Refusal::new(Stage::Guide, crate::region::EMPTY_CENTER));
    };
    // Coordinates about the middle of the area keep the row math small.
    let mid = Point2::new(0.5 * (lo.x + hi.x), 0.5 * (lo.y + hi.y));
    let along = |q: &Point2, d: (f64, f64)| (q.x - mid.x) * d.0 + (q.y - mid.y) * d.1;
    let (vmin, vmax) = pts.iter().map(|q| along(q, n)).fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), v| (a.min(v), b.max(v)));
    let half = 1.0 + (hi.x - lo.x).hypot(hi.y - lo.y);
    let span = vmax - vmin;
    let gaps = (span / step).ceil().max(1.0);
    if !gaps.is_finite() || gaps > MAX_ROWS {
        return Err(Refusal::new(Stage::Guide, TOO_MANY_ROWS).fix("use a larger stepover"));
    }
    // The rows run on the boundary at the two extremes, so that a wall
    // parallel to the rows is cut; the clip area is the centre grown by a
    // quarter of the tolerance, so that a row on the boundary is kept.
    let clip = geo(center.offset(0.25 * tol))?;
    let mut raws = Vec::new();
    let mut crumbs = 0usize;
    let count = gaps as u32;
    for i in 0..=count {
        let v = if i == count { vmax } else { vmin + f64::from(i) * span / gaps };
        let at = |t: f64| Point2::new(mid.x + v * n.0 + t * u.0, mid.y + v * n.1 + t * u.1);
        for piece in geo(clip.clip_polyline(&[at(-half), at(half)], true))? {
            if length(&piece) < tol {
                crumbs += 1;
            } else {
                raws.push(Raw { pts: piece, closed: false, k: i });
            }
        }
    }
    let mut marks = Vec::new();
    if c.closing_ring() {
        for l in center.loops() {
            marks.push((raws.len(), Mark::whole(0, MarkKind::Closing)));
            raws.push(Raw { pts: l.points().to_vec(), closed: true, k: 0 });
        }
    }
    let (curves, marks, mut notes) = finish(&raws, marks, |r| if r.closed { Family::Ring } else { Family::Row })?;
    if crumbs > 0 {
        let text = format!("{crumbs} row piece(s) shorter than the tolerance touch the area at one point and are not rows");
        notes.push(Note::new(Stage::Guide, text)?);
    }
    judge(p, GuideSet::with_marks(curves, marks)?, notes, Ask::Clear, tol + ARC_SAG)
}
