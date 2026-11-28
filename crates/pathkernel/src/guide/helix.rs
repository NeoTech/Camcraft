//! The helix guide and the exact circle rings of a hole (#4176, K2):
//! Circular and Bore get concentric circles at the choice's stepover, each
//! an exact [`Circle`] with a carrier polygon whose chord count follows
//! the tolerance (#3756: no fixed 256 chords); Thread and a helical bore
//! get one helix with its circle, pitch and hand.
//! Provenance: the circular pocket as concentric rings (the level sets of
//! the distance to the wall of a disc, Held, "On the Computational Geometry
//! of Pocket Machining", 1991); the chord count of a circle from its sagitta
//! (`r (1 - cos(pi / n)) <= tol`, elementary geometry); the helix of a
//! thread mill as in the machine-tool handbooks (one turn for each pitch);
//! our own design record (`RECOMMENDED.md`, `## #4024`)
//! "Stage 2" and `docs/decisions/4176-kernel-guides-medial-trace-helix.md`.
//! Written new: no code path of the old Bore, Circular or Thread planners
//! of `camcraft-motion` is ported.

use std::f64::consts::{PI, TAU};

use super::cover::{ARC_SAG, Ask, Raw, check_step, check_tol, finish_arcs, geo, judge, planar};
use super::mark::{Mark, MarkKind};
use super::rings::{MAX_RINGS, NO_RING, OFFSET_BELOW_CENTER, TOO_MANY_RINGS, WALL_STRIP_UNCUT, full_width};
use super::{Circle, Curve, Family, GuideChoice, GuidePlanner, GuideSet, Hand, HelixChoice, Lead, RingsChoice};
use crate::frame::PlanePoint;
use crate::region::{KernelRegion, PlanarRegion};
use crate::verdict::{Refusal, Stage, Verdict};
use crate::{KERNEL_TOL, Point2, in_area};

/// Refusal reason: the helix planner was given another guide choice.
pub const NOT_HELIX: &str = "the helix planner plans only a helix choice";
/// Refusal reason: exact circle rings on an area that is not the disc of
/// the circle.
pub const CIRCLE_NOT_REGION: &str = "exact circle rings need a cutter-centre area that is the disc of the circle";
/// Refusal reason: a helix that leaves the cutter-centre area.
pub const HELIX_OUTSIDE: &str = "the helix leaves the cutter-centre area: the tool would cut the wall";
/// Refusal reason: a helix of more points than the planner makes.
pub const HELIX_TOO_FINE: &str = "the helix needs more points than the planner makes (1000000)";

/// The most chords of one turn.
const MAX_CHORDS: f64 = 65_536.0;
/// The most points of one helix.
const MAX_HELIX_POINTS: f64 = 1_000_000.0;

/// The helix guide planner. It reads only the region and the helix choice:
/// no start side, no order, no link (laws 2 and 3). The direction of travel
/// is stage 4; the hand is the shape of the curve.
#[derive(Clone, Copy, Debug, Default)]
pub struct HelixPlanner;

impl GuidePlanner for HelixPlanner {
    fn guide(&self, r: &KernelRegion, g: &GuideChoice, tol: f64) -> Result<Verdict<GuideSet>, Refusal> {
        let GuideChoice::Helix(h) = g else {
            return Err(Refusal::new(Stage::Guide, NOT_HELIX));
        };
        helix(planar(r)?, h, check_tol(tol)?)
    }
}

/// The chords of one turn of a circle of radius `rho`: the least count of
/// eight or more whose sagitta is within `tol`.
pub(crate) fn chords(rho: f64, tol: f64) -> f64 {
    if tol >= rho {
        return 8.0;
    }
    let n = (PI / (1.0 - tol / rho).acos()).ceil();
    if n.is_finite() { n.clamp(8.0, MAX_CHORDS) } else { MAX_CHORDS }
}

/// The carrier polygon of a circle: `n` points from the angle 0 (the first
/// axis of the plane), counter-clockwise.
fn polygon(c: Point2, rho: f64, n: f64) -> Vec<Point2> {
    let count = n as u32;
    (0..count).map(|i| f64::from(i) * TAU / n).map(|a| Point2::new(c.x + rho * a.cos(), c.y + rho * a.sin())).collect()
}

/// Checks that the cutter-centre area is one loop whose vertices and edge
/// midpoints are within `tol` and the offset flattening of the circle.
fn fits(p: &PlanarRegion, circle: Circle, tol: f64) -> Result<(), Refusal> {
    let (c, r) = (circle.center(), circle.radius());
    let refuse = |at: Point2| Err(Refusal::new(Stage::Guide, CIRCLE_NOT_REGION).at(at).fix("give the circle of the cutter-centre area of the hole"));
    let [only] = p.center().loops() else {
        return refuse(c);
    };
    let v = only.points();
    for (i, a) in v.iter().enumerate() {
        let b = v[(i + 1) % v.len()];
        let m = Point2::new(0.5 * (a.x + b.x), 0.5 * (a.y + b.y));
        for q in [*a, m] {
            if ((q.x - c.x).hypot(q.y - c.y) - r).abs() > tol + ARC_SAG {
                return refuse(q);
            }
        }
    }
    Ok(())
}

/// The exact circle rings of a hole. Ring `k` has the radius
/// `circle - first_offset - k * stepover` while it is above the tolerance;
/// a choice without a count gets a clean-up circle at half the radius of
/// the last ring when the last ring's disc does not reach the centre. The
/// single guard ([`judge`]) decides done or incomplete.
pub(super) fn circle_rings(p: &PlanarRegion, c: &RingsChoice, circle: Circle, tol: f64) -> Result<Verdict<GuideSet>, Refusal> {
    let rad = p.tool().radius();
    let step = check_step(c.stepover(), rad)?;
    let d0 = c.first_offset();
    if d0 < 0.0 {
        return Err(Refusal::new(Stage::Guide, OFFSET_BELOW_CENTER));
    }
    if c.count().is_none() && d0 > rad + KERNEL_TOL {
        return Err(Refusal::new(Stage::Guide, WALL_STRIP_UNCUT).fix("use a first offset of the tool radius or less, or a stock to leave on the region"));
    }
    fits(p, circle, tol)?;
    let top = circle.radius() - d0;
    if top <= tol {
        return Err(Refusal::new(Stage::Guide, NO_RING).fix("use a smaller first offset"));
    }
    if top / step > f64::from(MAX_RINGS) {
        return Err(Refusal::new(Stage::Guide, TOO_MANY_RINGS).fix("use a larger stepover"));
    }
    let (mut raws, mut arcs, mut marks) = (Vec::new(), Vec::new(), Vec::new());
    let mut push = |raws: &mut Vec<Raw>, rho: f64, k: u32| -> Result<(), Refusal> {
        raws.push(Raw { pts: polygon(circle.center(), rho, chords(rho, tol)), closed: true, k });
        arcs.push(Some(Circle::new(circle.center(), rho)?));
        Ok(())
    };
    let mut last = None;
    for k in 0..MAX_RINGS {
        let rho = top - f64::from(k) * step;
        if c.count().is_some_and(|n| k >= n) || rho <= tol {
            break;
        }
        push(&mut raws, rho, k)?;
        last = Some((rho, k));
    }
    if let (None, Some((rho, k))) = (c.count(), last)
        && rho > rad
    {
        // The disc of the last ring leaves the centre of the hole uncut.
        marks.push((raws.len(), Mark::whole(0, MarkKind::Fill)));
        push(&mut raws, 0.5 * rho, k.saturating_add(1))?;
    }
    let margin = tol + ARC_SAG;
    full_width(p, &raws, &mut marks, rad, (margin, tol))?;
    let (curves, marks, notes) = finish_arcs(&raws, &arcs, marks, |_| Family::Ring)?;
    let ask = c.count().map_or(Ask::Clear, Ask::Count);
    judge(p, GuideSet::with_marks(curves, marks)?, notes, ask, margin)
}

/// One helix from the level of the plan (depth 0) down by the drop, from
/// the angle 0 (the first axis of the plane). A right hand turns clockwise
/// as it descends, a left hand counter-clockwise.
fn helix(p: &PlanarRegion, h: &HelixChoice, tol: f64) -> Result<Verdict<GuideSet>, Refusal> {
    let circle = h.circle();
    let (c, rho) = (circle.center(), circle.radius());
    let n = chords(rho, tol);
    let inside = geo(p.center().offset(tol + ARC_SAG))?;
    if let Some(q) = polygon(c, rho, n).into_iter().find(|q| !in_area(&inside, *q)) {
        return Err(Refusal::new(Stage::Guide, HELIX_OUTSIDE).at(q).fix("use a smaller helix radius or a smaller tool"));
    }
    let turns = h.drop() / h.pitch();
    let m = (n * turns).ceil().max(2.0);
    if !m.is_finite() || m > MAX_HELIX_POINTS {
        return Err(Refusal::new(Stage::Guide, HELIX_TOO_FINE).fix("use a larger tolerance"));
    }
    let way = match h.hand() {
        Hand::Right => -1.0,
        Hand::Left => 1.0,
    };
    let count = m as u32;
    let pts = (0..=count)
        .map(|i| {
            let f = f64::from(i) / m;
            let a = way * TAU * turns * f;
            PlanePoint::new(Point2::new(c.x + rho * a.cos(), c.y + rho * a.sin()), -h.drop() * f)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let curve = Curve::helix(circle, Lead::new(h.pitch(), h.hand())?, pts, 0)?;
    Ok(Verdict::done(GuideSet::new(vec![curve])))
}
