//! The ring guide (#4176, K2): contour-parallel rings as the level sets of
//! the distance to the walls of the cutter-centre area, spaced by the
//! choice's stepover, with clean-up rings where the regular rings leave an
//! area uncut, and each full-width stretch named (#3788).
//! Provenance: contour-parallel pocketing as level sets of the distance to
//! the boundary, Held, "On the Computational Geometry of Pocket Machining"
//! (1991), computed by the polygon offset of `camcraft-geometry`; our own
//! design record `docs/decisions/4024-path-kernel-design.md` "Stage 2" and
//! `docs/decisions/4176-kernel-guides-rings-rows.md`. Written new: no code
//! path of the old ring planners of `camcraft-motion` is ported.

use camcraft_geometry::geo2d::RingClip;

use super::cover::{
    ARC_SAG, Raw, check_step, check_tol, finish, geo, judge, loop_walks, pieces, planar, uncovered,
};
use super::mark::{Mark, MarkKind};
use super::{Family, GuideChoice, GuidePlanner, GuideSet, RingRef, RingsChoice};
use crate::frame::PlanePoint;
use crate::region::{KernelRegion, PlanarRegion};
use crate::verdict::{Refusal, Stage, Verdict};
use crate::{Loop, Point2, Region, in_area};

/// Refusal reason: the ring planner was given another guide choice.
pub const NOT_RINGS: &str = "the ring planner plans only a rings choice";
/// Refusal reason: exact circle rings (Circular, Bore) are not planned yet.
pub const CIRCLE_NOT_YET: &str = "exact circle rings are not planned yet: they come with the helix guide (#4176, second part)";
/// Refusal reason: the band reference is not planned yet.
pub const BAND_NOT_YET: &str = "the band ring reference is not planned yet: it comes with the order stage (K4, #3934)";
/// Refusal reason: a first ring offset below zero.
pub const OFFSET_BELOW_CENTER: &str = "a ring offset below zero puts the ring outside the cutter-centre area";
/// Refusal reason: the first ring offset leaves no area.
pub const NO_RING: &str = "the first ring offset leaves no area: there is no ring";
/// Refusal reason: more rings than the planner makes.
pub const TOO_MANY_RINGS: &str = "the area needs more rings than the planner makes (10000)";

/// The most regular rings of one plan.
const MAX_RINGS: u32 = 10_000;
/// The most rounds of clean-up rings.
const MAX_FILL_ROUNDS: u32 = 6;
/// The area (mm2) from which an uncovered piece gets a clean-up ring: a
/// square of 10 um. A smaller piece stays, named by the guard.
const FILL_MIN: f64 = 1e-4;

/// The ring guide planner. It reads only the region and the rings choice:
/// no start side, no order, no link (laws 2 and 3).
#[derive(Clone, Copy, Debug, Default)]
pub struct RingPlanner;

impl GuidePlanner for RingPlanner {
    fn guide(&self, r: &KernelRegion, g: &GuideChoice, tol: f64) -> Result<Verdict<GuideSet>, Refusal> {
        let GuideChoice::Rings(c) = g else {
            return Err(Refusal::new(Stage::Guide, NOT_RINGS));
        };
        plan(planar(r)?, c, check_tol(tol)?)
    }
}

/// The distance field whose level sets are the rings.
enum Field {
    /// The distance to every edge of the area: the erosion of the area.
    Boundary,
    /// The distance to the part walls only; the stretches on the stock edge
    /// (`edge`, a thin band round it) are not rings.
    Part { walls: Vec<Vec<Point2>>, edge: Region },
}

impl Field {
    fn new(p: &PlanarRegion, reference: RingRef, tol: f64) -> Result<Self, Refusal> {
        if reference == RingRef::Boundary || p.stock_edge().is_empty() {
            return Ok(Self::Boundary);
        }
        Ok(Self::Part { walls: loop_walks(p.part_walls()), edge: geo(Region::sweep(&loop_walks(p.stock_edge()), tol))? })
    }

    /// The area at distance `d` or more from the reference walls.
    fn level(&self, center: &Region, d: f64) -> Result<Region, Refusal> {
        if d <= 0.0 {
            return Ok(center.clone());
        }
        match self {
            Self::Boundary => geo(center.offset(-d)),
            Self::Part { walls, .. } => geo(center.difference(&geo(Region::sweep(walls, d))?)),
        }
    }

    /// The rings of index `k` on the boundary of a level area.
    fn rings(&self, level: &Region, k: u32) -> Result<Vec<Raw>, Refusal> {
        let mut out = Vec::new();
        for l in level.loops() {
            let pts = l.points().to_vec();
            match self {
                Self::Boundary => out.push(Raw { pts, closed: true, k }),
                Self::Part { edge, .. } => match geo(edge.clip_ring(&pts, false))? {
                    RingClip::Whole => out.push(Raw { pts, closed: true, k }),
                    RingClip::Pieces(ps) => out.extend(ps.into_iter().filter(|s| s.len() >= 2).map(|s| Raw { pts: s, closed: false, k })),
                },
            }
        }
        Ok(out)
    }
}

fn plan(p: &PlanarRegion, c: &RingsChoice, tol: f64) -> Result<Verdict<GuideSet>, Refusal> {
    if c.circle().is_some() {
        return Err(Refusal::new(Stage::Guide, CIRCLE_NOT_YET));
    }
    if c.reference() == RingRef::Band {
        return Err(Refusal::new(Stage::Guide, BAND_NOT_YET));
    }
    let rad = p.tool().radius();
    let step = check_step(c.stepover(), rad)?;
    let d0 = c.first_offset();
    if d0 < 0.0 {
        return Err(Refusal::new(Stage::Guide, OFFSET_BELOW_CENTER));
    }
    let reach = p.center().bounds().map_or(0.0, |(lo, hi)| (hi.x - lo.x).hypot(hi.y - lo.y));
    if (reach - d0) / step > f64::from(MAX_RINGS) {
        return Err(Refusal::new(Stage::Guide, TOO_MANY_RINGS).fix("use a larger stepover"));
    }
    let margin = tol + ARC_SAG;
    let field = Field::new(p, c.reference(), tol)?;
    let (mut raws, levels) = regular(p.center(), &field, d0, step, c.count())?;
    if raws.is_empty() {
        return Err(Refusal::new(Stage::Guide, NO_RING).fix("use a smaller first offset"));
    }
    let mut marks = Vec::new();
    // A ring count asks for that many rings, not for a cleared area: the
    // guard and the clean-up rings apply to a choice without a count.
    let left = match c.count() {
        None => {
            let target = if d0 > rad { geo(p.center().offset(rad - d0))? } else { p.center().clone() };
            fill(&target, &levels, &mut raws, &mut marks, rad.min(step), (rad, margin))?
        }
        Some(_) => Region::empty(),
    };
    full_width(p, &raws, &mut marks, rad, (margin, tol))?;
    let (curves, marks, notes) = finish(&raws, marks, |_| Family::Ring)?;
    judge(GuideSet::with_marks(curves, marks)?, &left, notes)
}

/// The regular rings, level by level from the first offset, and the level
/// areas (nested, the first one outermost).
fn regular(center: &Region, field: &Field, d0: f64, step: f64, count: Option<u32>) -> Result<(Vec<Raw>, Vec<Region>), Refusal> {
    let (mut raws, mut levels) = (Vec::new(), Vec::new());
    for k in 0u32.. {
        if count.is_some_and(|n| k >= n) {
            break;
        }
        if k >= MAX_RINGS {
            return Err(Refusal::new(Stage::Guide, TOO_MANY_RINGS).fix("use a larger stepover"));
        }
        let level = field.level(center, d0 + f64::from(k) * step)?;
        if level.is_empty() {
            break;
        }
        raws.extend(field.rings(&level, k)?);
        levels.push(level);
    }
    Ok((raws, levels))
}

/// Adds clean-up rings until the discs cover `target`, at most
/// [`MAX_FILL_ROUNDS`] rounds; returns the area that stays uncovered. A
/// clean-up ring of an uncovered piece is the boundary of the piece eroded
/// by half of `inset` (the piece itself when it is thinner).
fn fill(target: &Region, levels: &[Region], raws: &mut Vec<Raw>, marks: &mut Vec<(usize, Mark)>, inset: f64, (rad, margin): (f64, f64)) -> Result<Region, Refusal> {
    for _ in 0..MAX_FILL_ROUNDS {
        let left = uncovered(target, raws, rad, margin)?;
        let big: Vec<Region> = pieces(&left)?.into_iter().filter(|(_, a)| *a >= FILL_MIN).map(|(g, _)| g).collect();
        if big.is_empty() {
            return Ok(left);
        }
        for g in big {
            let core = geo(g.offset(-0.5 * inset))?;
            let src = if core.is_empty() { &g } else { &core };
            for l in src.loops() {
                marks.push((raws.len(), Mark::whole(0, MarkKind::Fill)));
                raws.push(Raw { pts: l.points().to_vec(), closed: true, k: fill_index(l, levels) });
            }
        }
    }
    uncovered(target, raws, rad, margin)
}

/// The index of a clean-up ring: one more than the deepest level area that
/// holds all of it, so that it sorts with the next deeper rings.
fn fill_index(l: &Loop, levels: &[Region]) -> u32 {
    let deepest = levels.iter().rposition(|lv| l.points().iter().all(|p| in_area(lv, *p)));
    deepest.map_or(0, |j| u32::try_from(j).map_or(u32::MAX, |j| j.saturating_add(1)))
}

/// Names each full-width stretch: a stretch of a curve of index `k` whose
/// tool disc overlaps no disc of a curve of index `k + 1` (the next deeper
/// curves; a deeper curve is farther, behind them) and no air beyond the
/// stock edge. When the deeper curves are cut first, the tool cuts there
/// at full width (#3788: the strands of a ring through the neck between two
/// lobes). A crumb of `tol` or less where the test is tangent is no stretch.
fn full_width(p: &PlanarRegion, raws: &[Raw], marks: &mut Vec<(usize, Mark)>, rad: f64, (margin, tol): (f64, f64)) -> Result<(), Refusal> {
    let air = if p.stock_edge().is_empty() { Region::empty() } else { geo(Region::sweep(&loop_walks(p.stock_edge()), rad))? };
    let Some(top) = raws.iter().map(|r| r.k).max() else {
        return Ok(());
    };
    for k in 0..=top {
        let deeper: Vec<Vec<Point2>> = raws.iter().filter(|r| r.k == k.saturating_add(1)).map(Raw::walk).collect();
        let near = if deeper.is_empty() { Region::empty() } else { geo(Region::sweep(&deeper, 2.0 * rad + margin))? };
        let reach = if air.is_empty() { near } else { geo(near.union(&air))? };
        for (i, raw) in raws.iter().enumerate().filter(|(_, r)| r.k == k) {
            let stretches = if raw.closed {
                match geo(reach.clip_ring(&raw.pts, false))? {
                    RingClip::Whole => {
                        marks.push((i, Mark::whole(0, MarkKind::FullWidth)));
                        continue;
                    }
                    RingClip::Pieces(ps) => ps,
                }
            } else {
                geo(reach.clip_polyline(&raw.pts, false))?
            };
            for s in stretches.into_iter().filter(|s| length(s) > tol) {
                let pts: Result<Vec<PlanePoint>, Refusal> = s.into_iter().map(PlanePoint::flat).collect();
                marks.push((i, Mark::part(0, MarkKind::FullWidth, pts?)?));
            }
        }
    }
    Ok(())
}

/// The length of a polyline.
pub(crate) fn length(pts: &[Point2]) -> f64 {
    pts.windows(2).map(|w| (w[1].x - w[0].x).hypot(w[1].y - w[0].y)).sum()
}
