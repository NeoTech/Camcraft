//! The medial guide (#4176, K2): the centre line of the cutter-centre area,
//! for Slot, Trochoidal slot, Engrave and V-Carve of an area, and the spine
//! of the 2D Adaptive. The line is the medial axis of the area (Blum 1967),
//! computed by `Region::medial_axis` of `camcraft-geometry` (the Voronoi
//! diagram of the sampled boundary, pruned by the object angle).
//! Provenance: Blum, "A transformation for extracting new descriptors of
//! shape" (1967); Brandt and Algazi (1992); Foskey, Lin and Manocha (2003),
//! as cited in `camcraft-geometry`'s `geo2d/medial.rs`; our own design
//! record (`RECOMMENDED.md`, `## #4024`) "Stage 2" and
//! `docs/decisions/4176-kernel-guides-medial-trace-helix.md`. Written new:
//! no code path of the old spine of `camcraft-motion` is ported.

use camcraft_geometry::geo2d::MEDIAL_MAX_SAMPLES;

use super::cover::{ARC_SAG, Ask, Raw, check_tol, finish, geo, judge, lowest, pieces, planar};
use super::rings::length;
use super::{Family, GuideChoice, GuidePlanner, GuideSet};
use crate::region::{KernelRegion, PlanarRegion};
use crate::verdict::{Note, Refusal, Stage, Verdict};
use crate::{Point2, Region};

/// Refusal reason: the medial planner was given another guide choice.
pub const NOT_MEDIAL: &str = "the medial planner plans only a medial choice";
/// Refusal reason: an area with no centre line.
pub const NO_MEDIAL: &str = "the area has no centre line longer than the tolerance: a round or a very small area has a centre point, not a line";
/// Refusal reason: a centre line through a neck narrower than the tolerance.
pub const NARROW_NECK: &str = "the area has a neck narrower than the tolerance, or touches itself: its centre line would run through a point";
/// Refusal reason: more boundary samples than the planner makes.
pub const MEDIAL_TOO_FINE: &str = "the centre line needs more boundary samples than the planner makes (200000)";

/// The least angle (radians) that the two boundary points of a kept medial
/// edge subtend at it: 60 degrees. The centre line of a slot subtends 180
/// degrees, the branch into a corner of 90 degrees subtends 90; a corner
/// flatter than 120 degrees (a flattened arc) has no branch.
pub const MIN_ANGLE: f64 = std::f64::consts::FRAC_PI_3;

/// The medial guide planner. It reads only the region: no start side, no
/// order, no link (laws 2 and 3). Each branch of the medial axis is a curve
/// of family `Medial`, index 0; a branch round a hole is closed.
#[derive(Clone, Copy, Debug, Default)]
pub struct MedialPlanner;

impl GuidePlanner for MedialPlanner {
    fn guide(&self, r: &KernelRegion, g: &GuideChoice, tol: f64) -> Result<Verdict<GuideSet>, Refusal> {
        let GuideChoice::Medial = g else {
            return Err(Refusal::new(Stage::Guide, NOT_MEDIAL));
        };
        plan(planar(r)?, check_tol(tol)?)
    }
}

/// The first place where a piece of the area is narrower than `tol`: its
/// erosion by half of `tol` splits the piece or opens one of its holes.
fn neck(center: &Region, tol: f64) -> Result<Option<Point2>, Refusal> {
    let count = |r: &Region| r.loops().iter().fold((0usize, 0usize), |(o, h), l| if l.area() > 0.0 { (o + 1, h) } else { (o, h + 1) });
    for (piece, _) in pieces(center)? {
        let (outer, holes) = count(&piece);
        let (o, h) = count(&geo(piece.offset(-0.5 * tol))?);
        if o > outer || (o == outer && h < holes) {
            return Ok(Some(lowest(&piece)));
        }
    }
    Ok(None)
}

fn plan(p: &PlanarRegion, tol: f64) -> Result<Verdict<GuideSet>, Refusal> {
    let perimeter: f64 = p.center().loops().iter().map(|l| length(l.points()) + l.points().first().zip(l.points().last()).map_or(0.0, |(a, b)| (a.x - b.x).hypot(a.y - b.y))).sum();
    let samples = perimeter / Region::medial_spacing(tol);
    if !samples.is_finite() || samples >= MEDIAL_MAX_SAMPLES as f64 {
        return Err(Refusal::new(Stage::Guide, MEDIAL_TOO_FINE).fix("use a larger tolerance"));
    }
    if let Some(at) = neck(p.center(), tol)? {
        return Err(Refusal::new(Stage::Guide, NARROW_NECK).at(at).fix("use a smaller tool, or select each side of the neck"));
    }
    let axis = geo(p.center().medial_axis(tol, MIN_ANGLE))?;
    let mut notes = Vec::new();
    let (mut raws, mut short) = (Vec::new(), 0usize);
    for c in &axis.curves {
        let closing = if c.closed { c.points.first().zip(c.points.last()).map_or(0.0, |(a, b)| (a.x - b.x).hypot(a.y - b.y)) } else { 0.0 };
        if length(&c.points) + closing < tol {
            short += 1;
        } else {
            raws.push(Raw { pts: c.points.clone(), closed: c.closed, k: 0 });
        }
    }
    if raws.is_empty() {
        return Err(Refusal::new(Stage::Guide, NO_MEDIAL).fix("use a ring guide for a round area"));
    }
    if short > 0 {
        notes.push(Note::new(Stage::Guide, format!("{short} piece(s) of the medial axis shorter than the tolerance are not curves"))?);
    }
    if axis.spurs > 0 {
        let text = format!("{} side branch(es) of the medial axis shorter than four sample spacings are sampling noise and are not curves", axis.spurs);
        notes.push(Note::new(Stage::Guide, text)?);
    }
    let (curves, marks, more) = finish(&raws, Vec::new(), |_| Family::Medial)?;
    notes.extend(more);
    judge(p, GuideSet::with_marks(curves, marks)?, notes, Ask::Line, tol + ARC_SAG)
}
