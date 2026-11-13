//! 2D Adaptive's trochoidal method (issue #1509): the region is cleared
//! by trochoid loops along its medial line, confined to the region.
//! Trochoidal milling - circular loops whose centre advances along the
//! cut, so the contact angle stays bounded where a straight pass would
//! slot - is Otkur & Lazoglu 2007, "Trochoidal milling", Int. J. Mach.
//! Tools Manuf. 47, and Rauch, Duc & Hascoet 2009, "Improving trochoidal
//! tool paths generation and implementation using process constraints
//! modelling", IJMTM 49; the medial line as the loops' guide is the
//! grassfire skeleton (Blum 1967; Held 1991, "On the Computational
//! Geometry of Pocket Machining"), taken by `adaptive::medial`. The loop
//! layout and the measured-engagement step ladder are `trochoid_slot`'s
//! (#220), the opened bore `trochoid_bore`'s. The lane law here is ours:
//!
//! - the cutter-centre area is the region eroded by the tool radius and
//!   the stock to leave (the pocket's own law);
//! - the lane is that area's medial line, straightened within
//!   [`SPINE_TOL`] of its half-width; its loop band is the least clearance along it, so
//!   every cutter-centre point lies inside the area by construction;
//! - the finished path is then CHECKED against the area, move by move
//!   ([`confined`]) - construction is the plan, the check is the law;
//! - what the lane cannot reach (corners, wider parts) is said in the
//!   notes with its size; past [`REST_MAX`] of the region the method is
//!   refused by name (the region is no single channel).
//!
//! Prime Directive: a region narrower than the tool, one too narrow to
//! loop in, a ring round an island, a region with no single medial
//! line, a lane that would leave most of the region standing and any
//! loop outside the region are each refused by name. No vendor material.

use crate::cl::{MoveKind, Toolpath};
use crate::geo2d::{Point2, Region};
use crate::geometry::Point3;
use crate::load::load_step;

use super::adaptive::{Spine, boundary_dist, medial_spine, widest_point};
use super::face::GenError;
use super::pocket::{PocketParams, PocketStrategy};
use super::pocket_hsm::SMOOTH_PER_R;
use super::trochoid_slot::{DEFAULT_MAX_ENGAGEMENT, DEFAULT_MICROLIFT, MIN_BAND_PER_D, TrochoidParams, TrochoidReturn, plan_lane};

/// The name every refusal and note of the method starts with.
const WHO: &str = "2D Adaptive (Trochoidal)";

/// The medial line is straightened within this share of the area's
/// half-width: the sampled line of a straight channel becomes its two
/// ends (its trimmed ends curl toward the corners by less). What the
/// straightening costs in clearance is read on the straightened line.
const SPINE_TOL: f64 = 0.05;

/// The loops stay this far (mm) inside the cutter-centre area's walls.
const WALL_MARGIN: f64 = 0.01;

/// The lane's clearance is sampled this often along it (mm).
const SAMPLE: f64 = 0.25;

/// A cutter-centre point this far (mm) outside the area is outside it
/// (the path's own lattice and the area's offset quantisation).
const CONFINE_TOL: f64 = 2e-3;

/// The share of the region the lane may leave out of reach before the
/// method is refused; under it the rest is said in the notes ...
pub const REST_MAX: f64 = 0.25;

/// ... when it is more than this share.
const REST_NOTE: f64 = 0.005;

/// Clear `region` with trochoidal loops (see the module law). `p` is the
/// 2D Adaptive's own parameter set: the optimal load is the most the
/// loops may advance a turn, the stepover and the entry are not read
/// (the lane bores its own helix at its start).
pub fn adaptive_trochoidal(region: &Region, p: &PocketParams) -> Result<Toolpath, GenError> {
    // The Adaptive doors (finite numbers, the load in (0, 50] % of D).
    let p = &PocketParams { strategy: PocketStrategy::Adaptive, ..*p };
    p.validate()?;
    if region.is_empty() {
        return Err(GenError::Empty("empty pocket region".into()));
    }
    let r = 0.5 * p.tool_diameter;
    let center = region.offset(-(r + p.stock_to_leave))?;
    if center.is_empty() {
        return Err(GenError::Invalid(format!(
            "{WHO}: the region is narrower than the {:.3} mm tool (with {:.3} mm left on each wall)",
            p.tool_diameter, p.stock_to_leave
        )));
    }
    let (chain, b) = lane(region, &center, p)?;
    cut_lane(&center, &chain, b, p)
}

/// The lane of `center`: its straightened medial line and the loop
/// band's half-width, or the refusal that names why there is none.
fn lane(region: &Region, center: &Region, p: &PocketParams) -> Result<(Vec<Point2>, f64), GenError> {
    let (d, r) = (p.tool_diameter, 0.5 * p.tool_diameter);
    let narrow = |clear: f64| {
        GenError::Invalid(format!(
            "{WHO}: the region is {:.3} mm wide where the loops would run, too narrow for a {d:.3} mm tool to loop in (it needs more than {:.3} mm) - use a smaller tool, or cut it as a Slot",
            2.0 * (clear.max(0.0) + r + p.stock_to_leave),
            d * (1.0 + 2.0 * MIN_BAND_PER_D) + 2.0 * (p.stock_to_leave + WALL_MARGIN)
        ))
    };
    let other = |why: &str| GenError::Invalid(format!("{WHO}: {why} - the trochoidal method loops along one channel; use the Spiral method for this region"));
    let Some((_, widest)) = widest_point(center) else {
        return Err(narrow(0.0));
    };
    if widest - WALL_MARGIN <= MIN_BAND_PER_D * d {
        return Err(narrow(widest));
    }
    let (pts, half) = match medial_spine(center)? {
        Ok((Spine::Open(pts), half)) => (pts, half),
        Ok((Spine::Closed(_), _)) => return Err(other("the region closes round an island")),
        Err(why) => return Err(other(&why)),
    };
    // A line the straightening leaves no length, or a band it leaves no
    // room, is the planner's to refuse (two distinct points, a band
    // wider than `MIN_BAND_PER_D`), under this method's name.
    let chain = straightened(&pts, SPINE_TOL * half);
    let b = clearance(center, &chain) - WALL_MARGIN;
    // What the lane reaches: the cutter's footprint along it.
    let reach = if p.stock_to_leave > 0.0 { region.offset(-p.stock_to_leave)? } else { region.clone() };
    let swept = Region::sweep(std::slice::from_ref(&chain), r + b)?;
    let rest = rest_share(&reach, &swept);
    if rest > REST_MAX {
        return Err(other(&format!(
            "loops along its medial line would leave {:.0} % of the region standing (at most {:.0} %)",
            rest * 100.0,
            REST_MAX * 100.0
        )));
    }
    Ok((chain, b))
}

/// The share of `reach`'s area outside `swept`.
fn rest_share(reach: &Region, swept: &Region) -> f64 {
    let all = reach.area();
    if !(all.is_finite() && all > 0.0) {
        return 0.0;
    }
    ((all - swept.area()) / all).clamp(0.0, 1.0)
}

/// The least distance from the chain to `center`'s boundary (0 where a
/// sample is outside it).
fn clearance(center: &Region, chain: &[Point2]) -> f64 {
    let mut least = f64::INFINITY;
    for w in chain.windows(2) {
        let len = (w[1].x - w[0].x).hypot(w[1].y - w[0].y);
        let n = ((len / SAMPLE).ceil() as usize).clamp(1, 100_000);
        for k in 0..=n {
            let t = k as f64 / n as f64;
            let q = Point2::new(w[0].x + (w[1].x - w[0].x) * t, w[0].y + (w[1].y - w[0].y) * t);
            least = least.min(if center.contains(q) { boundary_dist(center, q) } else { 0.0 });
        }
    }
    if least.is_finite() { least } else { 0.0 }
}

/// `pts` with every point within `tol` of the line through its kept
/// neighbours dropped (Douglas & Peucker 1973), ends kept.
fn straightened(pts: &[Point2], tol: f64) -> Vec<Point2> {
    if pts.len() < 3 {
        return pts.to_vec();
    }
    let mut keep = vec![false; pts.len()];
    keep[0] = true;
    keep[pts.len() - 1] = true;
    let mut todo = vec![(0usize, pts.len() - 1)];
    while let Some((i, j)) = todo.pop() {
        let (a, b) = (pts[i], pts[j]);
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let l = dx.hypot(dy);
        let off = |q: Point2| if l > 1e-12 { ((q.x - a.x) * dy - (q.y - a.y) * dx).abs() / l } else { (q.x - a.x).hypot(q.y - a.y) };
        let far = (i + 1..j).map(|k| (off(pts[k]), k)).max_by(|x, y| x.0.total_cmp(&y.0));
        if let Some((dist, k)) = far
            && dist > tol
        {
            keep[k] = true;
            todo.push((i, k));
            todo.push((k, j));
        }
    }
    let mut out: Vec<Point2> = Vec::new();
    for (q, _) in pts.iter().zip(&keep).filter(|(_, k)| **k) {
        if out.last().is_none_or(|l| (q.x - l.x).hypot(q.y - l.y) > 1e-6) {
            out.push(*q);
        }
    }
    out
}

/// The lane cut: loops of band half-width `b` along `chain`, planned by
/// the measured-engagement ladder, then held to `center` move by move.
pub(super) fn cut_lane(center: &Region, chain: &[Point2], b: f64, p: &PocketParams) -> Result<Toolpath, GenError> {
    let r = 0.5 * p.tool_diameter;
    // A lifted return is a feed: it needs the room under the clearance
    // plane, else the returns hop by the retract plane.
    let ret = if p.clearance_z - p.top_z >= DEFAULT_MICROLIFT {
        TrochoidReturn::Microlift { height: DEFAULT_MICROLIFT }
    } else {
        TrochoidReturn::Retract
    };
    let tp = TrochoidParams {
        tool_diameter: p.tool_diameter,
        width: 2.0 * (b + r + p.stock_to_leave),
        loop_width: None,
        step: load_step(p.tool_diameter, p.optimal_load),
        max_engagement: DEFAULT_MAX_ENGAGEMENT,
        ret,
        smoothing: r * SMOOTH_PER_R,
        stock_to_leave: p.stock_to_leave,
        clearance_z: p.clearance_z,
        retract_z: p.retract_z,
        top_z: p.top_z,
        bottom_z: p.bottom_z,
        stepdown: p.stepdown,
        feed: p.feed,
        return_feed: p.feed,
        plunge_feed: p.plunge_feed,
        climb: p.climb,
    };
    let mut path = plan_lane(chain, &tp, true).map_err(renamed)?.path;
    if let Some(at) = confined(&path, center)? {
        return Err(GenError::invalid_at(
            format!("{WHO}: a loop at ({:.3}, {:.3}) would leave the region - nothing is cut", at.x, at.y),
            at,
        )
        .with_ghost(&path));
    }
    for n in &mut path.notes {
        *n = rename(n);
    }
    let reach = center.offset(r)?;
    let rest = rest_share(&reach, &Region::sweep(&[chain.to_vec()], r + b)?);
    if rest > REST_NOTE {
        path.notes.push(format!(
            "{WHO}: about {:.1} mm2 ({:.0} %) of the region is outside the loops' reach (its corners and wider parts) - a following operation must clear it",
            rest * reach.area(),
            rest * 100.0
        ));
    }
    if p.entry != crate::entry::Entry3D::Helix {
        path.notes.push(format!("{WHO}: the loops enter by their own bore helix at the lane's start - the Entry setting is not read"));
    }
    Ok(path)
}

/// The first cutter-centre point of a cutting move of `path` outside
/// `center` (grown by [`CONFINE_TOL`]), if any.
fn confined(path: &Toolpath, center: &Region) -> Result<Option<Point3>, GenError> {
    let grown = center.offset(CONFINE_TOL)?;
    for m in path.moves.iter().filter(|m| m.kind != MoveKind::Rapid) {
        for (a, b) in m.segments3(0.01) {
            for q in [a, b] {
                if !grown.contains(Point2::new(q.x, q.y)) {
                    return Ok(Some(q));
                }
            }
        }
    }
    Ok(None)
}

/// The slot planner's words under the method's name.
fn rename(msg: &str) -> String {
    msg.replacen("trochoidal slot:", &format!("{WHO}:"), 1)
}

fn renamed(e: GenError) -> GenError {
    match e {
        GenError::Invalid(msg) => GenError::Invalid(rename(&msg)),
        GenError::At { msg, at } => GenError::At { msg: rename(&msg), at },
        GenError::Ghosted { msg, at, ghost } => GenError::Ghosted { msg: rename(&msg), at, ghost },
        other => other,
    }
}

#[cfg(test)]
#[path = "adaptive_troch_tests.rs"]
mod tests;
