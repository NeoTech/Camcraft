//! The engagement check of `adaptive_troch`'s clean-up passes (#3849):
//! the planned chains are measured against a post-lane stock model, the
//! peel step steps down while they exceed the limit, and a pass that
//! still exceeds it gets a named note that the operation turns into a
//! warning on the clean-up move. Radial engagement by the stock model of
//! `crate::quality` (our own law, after Otkur & Lazoglu 2007).

use crate::cl::{MoveKind, PathBuilder, Toolpath, lattice};
use crate::geo2d::{Point2, Region};
use crate::geometry::Point3;

use super::adaptive_troch::AdaptiveTrochParams;
use super::adaptive_troch_clean::{CleanChain, emit_cleanup, plan_cleanup_step};
use super::face::GenError;
use super::pocket::PocketParams;
use super::trochoid_level::emit_level;
use super::trochoid_slot::LaneLayout;

/// The peel-step scales tried, in order, until the clean-up holds the limit.
pub(super) const CLEANUP_LADDER: &[f64] = &[1.0, 0.75, 0.5, 0.35, 0.25];

/// Note prefix emitted when trochoidal clean-up passes exceed the engagement limit (#3849).
pub const CLEANUP_ENGAGEMENT_NOTE_PREFIX: &str = "2D Adaptive (Trochoidal): clean-up engagement measured ";
const CLEANUP_ENGAGEMENT_NOTE_PREFIX_3D: &str = "3D Adaptive (Trochoidal): clean-up engagement measured ";

/// Checked clean-up plan for a level: chains, swept region, and engagement status.
pub(super) struct CleanPlan {
    pub(super) chains: Vec<CleanChain>,
    pub(super) cleared: Region,
    pub(super) max_deg: f64,
    pub(super) step_reduced: bool,
    /// The measured engagement and the cutter centre where it peaked,
    /// at the level's Z (`p.bottom_z`), when no ladder step held the limit.
    pub(super) exceeded: Option<(f64, Point3)>,
}

/// Named note for a clean-up pass exceeding `limit_deg` at `at` (#3849).
#[must_use]
pub fn cleanup_engagement_note(measured_deg: f64, at: Point3, limit_deg: f64) -> String {
    format!(
        "{CLEANUP_ENGAGEMENT_NOTE_PREFIX}{measured_deg:.1} deg at ({:.3}, {:.3}, {:.3}), above the {limit_deg:.1} deg limit",
        at.x, at.y, at.z
    )
}

/// `(measured_deg, limit_deg, at)` of the first note that reports an
/// over-limit clean-up pass.
fn parse_note(notes: &[String]) -> Option<(f64, f64, Option<Point3>)> {
    for note in notes {
        let rest = note
            .strip_prefix(CLEANUP_ENGAGEMENT_NOTE_PREFIX)
            .or_else(|| note.strip_prefix(CLEANUP_ENGAGEMENT_NOTE_PREFIX_3D));
        let Some(rest) = rest else { continue };
        let Some((m_str, tail)) = rest.split_once(" deg") else { continue };
        let Some((at_str, l_tail)) = tail.split_once("above the ") else { continue };
        let Some((l_str, _)) = l_tail.split_once(" deg limit") else { continue };
        if let (Ok(measured), Ok(limit)) = (m_str.trim().parse::<f64>(), l_str.trim().parse::<f64>())
            && measured.is_finite()
            && limit.is_finite()
        {
            return Some((measured, limit, parse_point(at_str)));
        }
    }
    None
}

/// The `(x, y, z)` of the note's ` at (x, y, z), ` tail.
fn parse_point(s: &str) -> Option<Point3> {
    let inner = s.split_once('(')?.1.split_once(')')?.0;
    let v: Vec<f64> = inner.split(',').map(|c| c.trim().parse::<f64>()).collect::<Result<_, _>>().ok()?;
    match v[..] {
        [x, y, z] if x.is_finite() && y.is_finite() && z.is_finite() => Some(Point3::new(x, y, z)),
        _ => None,
    }
}

/// Extract `(measured_deg, limit_deg)` when any note reports an over-limit clean-up pass (#3849).
#[must_use]
pub fn cleanup_engagement_exceeded(notes: &[String]) -> Option<(f64, f64)> {
    parse_note(notes).map(|(m, l, _)| (m, l))
}

/// `(move, measured_deg, limit_deg)` of an over-limit clean-up pass
/// (#3849): the move is the feed at the noted level's Z that passes
/// nearest the noted cutter centre; of equal ones the last, because the
/// clean-up is cut after the lanes of its level.
#[must_use]
pub fn cleanup_engagement_move(path: &Toolpath) -> Option<(usize, f64, f64)> {
    let (measured, limit, at) = parse_note(&path.notes)?;
    let at = at.and_then(|q| nearest_feed(path, q)).unwrap_or(0);
    Some((at, measured, limit))
}

fn nearest_feed(path: &Toolpath, q: Point3) -> Option<usize> {
    let mut best: Option<(usize, f64)> = None;
    for (i, m) in path.moves.iter().enumerate() {
        let level = (m.from.z - q.z).abs() <= 1e-3 && (m.to.z - q.z).abs() <= 1e-3;
        if m.kind != MoveKind::Feed || !level {
            continue;
        }
        let d = m.segments3(0.01).iter().map(|&(a, b)| seg_dist(q, a, b)).fold(f64::INFINITY, f64::min);
        match best {
            _ if !d.is_finite() => {}
            None => best = Some((i, d)),
            // Within 1 um of the nearest so far: the later move wins.
            Some((_, bd)) if d <= bd + 1e-3 => best = Some((i, d.min(bd))),
            Some(_) => {}
        }
    }
    best.map(|(i, _)| i)
}

/// XY distance from `q` to the segment `a`-`b`.
fn seg_dist(q: Point3, a: Point3, b: Point3) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len2 = dx * dx + dy * dy;
    let t = if len2 > 1e-18 { (((q.x - a.x) * dx + (q.y - a.y) * dy) / len2).clamp(0.0, 1.0) } else { 0.0 };
    (q.x - (a.x + t * dx)).hypot(q.y - (a.y + t * dy))
}

/// Plan clean-up passes and measure their radial engagement against a single-level
/// post-lane stock model. If clean-up exceeds `t.max_engagement`, steps down along
/// [`CLEANUP_LADDER`] and, if still above the limit, records `exceeded` so callers
/// surface a named diagnostic rather than overloading silently (#3849).
#[allow(clippy::too_many_arguments)]
pub(super) fn plan_cleanup_checked(
    region: &Region,
    center: &Region,
    reach: &Region,
    layouts: &[LaneLayout],
    used_center: &Region,
    cleared: &Region,
    p: &PocketParams,
    t: &AdaptiveTrochParams,
) -> Result<CleanPlan, GenError> {
    plan_cleanup_on(CLEANUP_LADDER, region, center, reach, layouts, used_center, cleared, p, t)
}

/// One measured ladder step.
struct Candidate {
    chains: Vec<CleanChain>,
    cleared: Region,
    deg: f64,
    at: Point3,
    reduced: bool,
}

/// [`plan_cleanup_checked`] over the scales of `ladder` (the first is the
/// baseline).
#[allow(clippy::too_many_arguments)]
pub(super) fn plan_cleanup_on(
    ladder: &[f64],
    region: &Region,
    center: &Region,
    reach: &Region,
    layouts: &[LaneLayout],
    used_center: &Region,
    cleared: &Region,
    p: &PocketParams,
    t: &AdaptiveTrochParams,
) -> Result<CleanPlan, GenError> {
    let plan = |scale: f64| {
        plan_cleanup_step(center, reach, used_center, cleared, p.tool_diameter, p.optimal_load, p.climb, t, scale)
    };
    let (chains, base_cleared) = plan(ladder.first().copied().unwrap_or(1.0))?;
    let Some(start) = first_point(&chains) else {
        return Ok(CleanPlan { chains, cleared: base_cleared, max_deg: 0.0, step_reduced: false, exceeded: None });
    };
    let spec = crate::quality::QualitySpec::new(p.tool_diameter);
    let lane_path = lane_model(layouts)?;
    let measure = |chains: &[CleanChain], start: Point2| -> Result<(f64, Point3), GenError> {
        let err = |e| GenError::Invalid(format!("2D Adaptive (Trochoidal): measuring clean-up stock: {e}"));
        let mut stock = crate::quality::stock_for(region, 1.0, 0.0, p.tool_diameter, &spec).map_err(err)?;
        crate::quality::measure(&lane_path, &mut stock, &spec).map_err(err)?;
        let one_p = PocketParams { top_z: 1.0, bottom_z: 0.0, stepdown: 1.0, clearance_z: 3.0, retract_z: Some(2.0), ..*p };
        let mut cb = PathBuilder::approached(Point3::new(lattice(start.x), lattice(start.y), 2.0));
        emit_cleanup(&mut cb, chains, &one_p, 0.0, 1.0, 2.0);
        let q = crate::quality::measure(&cb.finish(), &mut stock, &spec).map_err(err)?;
        Ok((q.max_deg, Point3::new(q.max_at.x, q.max_at.y, p.bottom_z)))
    };
    let (deg, at) = measure(&chains, start)?;
    let mut best = Candidate { chains, cleared: base_cleared, deg, at, reduced: false };
    if best.deg <= t.max_engagement {
        return Ok(CleanPlan { chains: best.chains, cleared: best.cleared, max_deg: deg, step_reduced: false, exceeded: None });
    }
    for &scale in ladder.iter().skip(1) {
        let (chains, cur_cleared) = plan(scale)?;
        let Some(start) = first_point(&chains) else { continue };
        let (deg, at) = measure(&chains, start)?;
        if deg <= t.max_engagement {
            return Ok(CleanPlan { chains, cleared: cur_cleared, max_deg: deg, step_reduced: true, exceeded: None });
        }
        if deg < best.deg {
            best = Candidate { chains, cleared: cur_cleared, deg, at, reduced: true };
        }
    }
    Ok(CleanPlan {
        chains: best.chains,
        cleared: best.cleared,
        max_deg: best.deg,
        step_reduced: best.reduced,
        exceeded: Some((best.deg, best.at)),
    })
}

/// The first point of the first chain; `None` when there is none to cut.
fn first_point(chains: &[CleanChain]) -> Option<Point2> {
    chains.iter().find_map(|c| c.pts.first().copied())
}

/// The level's lanes alone, on the unit-level model of the measurement.
fn lane_model(layouts: &[LaneLayout]) -> Result<Toolpath, GenError> {
    let first = layouts.first().map_or(Point2::new(0.0, 0.0), |l| l.entry);
    let mut bld = PathBuilder::approached(Point3::new(first.x, first.y, 3.0));
    if layouts.is_empty() {
        return Ok(bld.finish());
    }
    let p3 = |q: Point2, zz: f64| Point3::new(lattice(q.x), lattice(q.y), lattice(zz));
    let (hop, above) = (2.0, 1.0);
    let from = (above + crate::approach::DEFAULT_PLUNGE_HEIGHT).min(hop).max(above);
    for (li, lay) in layouts.iter().enumerate() {
        if li > 0 {
            let at = bld.at();
            if (at.z - hop).abs() > 1e-6 {
                bld.push(MoveKind::Rapid, Point3::new(at.x, at.y, hop), None);
            }
            bld.push(MoveKind::Rapid, p3(lay.entry, hop), None);
        }
        bld.push(MoveKind::Rapid, p3(lay.entry, from), None);
        emit_level(&mut bld, &lay.lay, &lay.tp, 0.0, from, true, lay.open)?;
    }
    let at = bld.at();
    bld.push(MoveKind::Rapid, Point3::new(at.x, at.y, hop), None);
    Ok(bld.finish())
}
