//! The user's start and exit points on the zigzag raster (#3330, the
//! 5-Axis Parallel slice of #1448). The raster's passes are kept as
//! they are; a pick only chooses the corner where the cut starts:
//!
//! - **Start:** of the four raster corners (each end of the first and
//!   of the last pass), the one nearest the pick. The pass order is
//!   reversed when that corner is on the last pass, and every pass runs
//!   the other way when that corner is the far end of its pass. The
//!   zigzag alternation stays.
//! - **Exit:** a zigzag that starts at a corner ends at one corner
//!   only. An exit pick alone chooses the order whose last pass ends
//!   nearest it. With a start pick, the exit must be near the end
//!   that the start's order gives.
//!
//! The geometry of each pass does not change: the same samples are
//! cut, only the order and the direction. The raster cuts both ways by
//! its own zigzag law (the climb flag is not read here), so no order
//! asks for a cut direction the operation forbids. The links between
//! passes are planned on the new order by the raster's own link laws
//! (`march::emit_runs`), and every pass end is a sample whose plunge
//! column the raster's column law already cleared
//! (`support::plunge_support`). What a pick cannot keep is refused by
//! name, never moved: a pick farther than its reach from every corner,
//! and an exit that the start's order cannot end at.
//!
//! Provenance: first principles - the four end corners of a serpentine
//! raster and their reversal symmetries; the entry and exit position
//! is the common vocabulary of published CAM practice. No vendor
//! material consulted.

use crate::cl::{PathBuilder, Toolpath};
use crate::geo2d::Point2;
use crate::geometry::{Point3, Triangle};
use crate::machining2d::GenError;
use crate::path_picks::{PathPick, PathPicks, PickRole};

use super::params::{validate, Finish3DParams, FinishPattern};
use super::raster::{raster_pass_seamed, rings_and_tail};
use super::support::build_sups;

/// The start and exit points a raster reads, in the generator's frame.
/// Both `None` is the automatic order of every earlier path, byte for
/// byte.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RasterSeam {
    /// The cut starts at the raster corner nearest this pick.
    pub start: Option<PathPick>,
    /// The last pass ends at the raster corner nearest this pick.
    pub exit: Option<PathPick>,
}

impl RasterSeam {
    /// The seam of an operation's picks: the first start and the first
    /// exit. A pick with a Z is read in 3D, a pick without one in XY.
    #[must_use]
    pub fn of(picks: &PathPicks) -> Self {
        Self { start: picks.first(PickRole::Start), exit: picks.first(PickRole::Exit) }
    }

    /// No pick of either role.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.start.is_none() && self.exit.is_none()
    }
}

/// Where the ordered raster starts and ends: the tool tips of its first
/// and of its last sample.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RasterEnds {
    /// The tip where the first pass plunges.
    pub start: Point3,
    /// The tip where the last pass ends.
    pub exit: Point3,
}

/// One pass: its pieces, each a run of `(centre XY, centre Z)` samples.
pub(super) type Pass = Vec<Vec<(Point2, f64)>>;

/// The farthest a pick may sit from the corner it chooses: one and a
/// half tool diameters, the contour's slack for a click beside it.
fn reach(tool_diameter: f64) -> f64 {
    tool_diameter * 1.5
}

/// "start point (x, y)" or "exit point (x, y, z)", the refusals' lead.
fn named(role: PickRole, p: &PathPick) -> String {
    match p.z {
        Some(z) => format!("{} point ({:.3}, {:.3}, {:.3})", role.word(), p.xy.x, p.xy.y, z),
        None => format!("{} point ({:.3}, {:.3})", role.word(), p.xy.x, p.xy.y),
    }
}

/// The distance of tip `q` from the pick: in 3D when the pick states a
/// Z, in XY when it does not.
fn gap(p: &PathPick, q: Point3) -> f64 {
    let dz = p.z.map_or(0.0, |z| q.z - z);
    ((q.x - p.xy.x).powi(2) + (q.y - p.xy.y).powi(2) + dz * dz).sqrt()
}

/// The four orders, `(reverse the pass order, run each pass the other
/// way)`, the automatic one first: a tie keeps the earlier order.
const ORDERS: [(bool, bool); 4] = [(false, false), (false, true), (true, false), (true, true)];

/// The first and the last tip of the passes `ps` cut in `order`.
fn corners(ps: &[&Pass], order: (bool, bool), r: f64) -> Option<(Point3, Point3)> {
    let head = |p: &Pass| p.first().and_then(|pc| pc.first()).copied();
    let tail = |p: &Pass| p.last().and_then(|pc| pc.last()).copied();
    let (first, last) = (*ps.first()?, *ps.last()?);
    let (a, b) = match order {
        (false, false) => (head(first), tail(last)),
        (false, true) => (tail(first), head(last)),
        (true, false) => (head(last), tail(first)),
        (true, true) => (tail(last), head(first)),
    };
    let tip = |(q, cz): (Point2, f64)| Point3::new(q.x, q.y, cz - r);
    Some((tip(a?), tip(b?)))
}

/// Orders the passes for the seam's picks and gives the ordered ends.
/// An empty seam leaves `passes` as they are. `None` when no pass has a
/// sample.
pub(super) fn order(seam: &RasterSeam, passes: &mut [Pass], r: f64, tool_d: f64) -> Result<Option<RasterEnds>, GenError> {
    let live: Vec<&Pass> = passes.iter().filter(|p| p.iter().any(|pc| !pc.is_empty())).collect();
    let mut cands: Vec<((bool, bool), Point3, Point3)> = Vec::with_capacity(4);
    for o in ORDERS {
        if let Some((a, b)) = corners(&live, o, r) {
            cands.push((o, a, b));
        }
    }
    let Some(&(_, a0, b0)) = cands.first() else {
        if let Some((role, p)) = seam.start.map(|p| (PickRole::Start, p)).or(seam.exit.map(|p| (PickRole::Exit, p))) {
            return Err(GenError::Invalid(format!("{}: the raster has no pass to read it on", named(role, &p))));
        }
        return Ok(None);
    };
    if seam.is_empty() {
        return Ok(Some(RasterEnds { start: a0, exit: b0 }));
    }
    let lim = reach(tool_d);
    let (s, e) = (seam.start, seam.exit);
    if let Some(p) = s {
        cands.retain(|c| gap(&p, c.1) <= lim);
        if cands.is_empty() {
            return Err(far(PickRole::Start, &p, &live, r, lim, |c| c.1));
        }
    }
    if let Some(p) = e {
        let before = cands.clone();
        cands.retain(|c| gap(&p, c.2) <= lim);
        if cands.is_empty() {
            if s.is_some() {
                return Err(inconsistent(&p, &before, lim));
            }
            return Err(far(PickRole::Exit, &p, &live, r, lim, |c| c.2));
        }
    }
    let score = |c: &((bool, bool), Point3, Point3)| s.map_or(0.0, |p| gap(&p, c.1)) + e.map_or(0.0, |p| gap(&p, c.2));
    let mut best = cands[0];
    for c in &cands[1..] {
        if score(c) < score(&best) {
            best = *c;
        }
    }
    let ((rev, flip), start, exit) = best;
    if rev {
        passes.reverse();
    }
    if flip {
        for p in passes.iter_mut() {
            p.reverse();
            for pc in p.iter_mut() {
                pc.reverse();
            }
        }
    }
    Ok(Some(RasterEnds { start, exit }))
}

/// A pick farther than the reach from every corner of its role.
fn far(role: PickRole, p: &PathPick, live: &[&Pass], r: f64, lim: f64, at: impl Fn(&((bool, bool), Point3, Point3)) -> Point3) -> GenError {
    let near = ORDERS
        .iter()
        .filter_map(|&o| corners(live, o, r).map(|(a, b)| at(&(o, a, b))))
        .min_by(|x, y| gap(p, *x).total_cmp(&gap(p, *y)));
    let (q, d) = near.map_or((Point3::ZERO, f64::INFINITY), |q| (q, gap(p, q)));
    GenError::Invalid(format!(
        "{}: the raster can {} only at a corner (an end of its first or its last pass); the nearest \
         corner ({:.3}, {:.3}, {:.3}) stands {d:.3} mm away, farther than {lim:.3} mm (1.5 tool \
         diameters) - pick near a corner, or clear the {} point",
        named(role, p),
        if role == PickRole::Start { "start" } else { "end" },
        q.x,
        q.y,
        q.z,
        role.word()
    ))
}

/// An exit pick that no order starting at the start pick ends near.
fn inconsistent(p: &PathPick, starts: &[((bool, bool), Point3, Point3)], lim: f64) -> GenError {
    let ends: Vec<String> = starts.iter().map(|c| format!("({:.3}, {:.3}, {:.3})", c.2.x, c.2.y, c.2.z)).collect();
    GenError::Invalid(format!(
        "{}: a raster that starts at the start point ends at {} - farther than {lim:.3} mm from the \
         exit point; the zigzag cannot end there without a change of its passes - pick near that \
         end, or clear the exit point",
        named(PickRole::Exit, p),
        ends.join(" or ")
    ))
}

/// The raster finish ([`super::parallel`], direction +X) with the
/// user's start and exit points: the raster's passes ordered for the
/// picks, then the waterline rings and the clearance exit, as the
/// automatic path. Returns the path and where the raster starts and
/// ends (`None` when no shallow ground gives a pass). An empty `seam`
/// is [`super::parallel`], byte for byte.
pub fn parallel_seamed(facets: &[Triangle], p: &Finish3DParams, seam: &RasterSeam) -> Result<(Toolpath, Option<RasterEnds>), GenError> {
    if !matches!(p.pattern, FinishPattern::Parallel) {
        if seam.is_empty() {
            return Ok((super::raster::parallel(facets, p)?, None));
        }
        return Err(GenError::Invalid(
            "a start or exit point is read on the one-direction Parallel raster only - clear the picks".into(),
        ));
    }
    validate(p, facets)?;
    let r = p.tool_diameter * 0.5;
    let sups = build_sups(facets, r);
    let mut b = PathBuilder::approached(Point3::new(0.0, 0.0, p.clearance_z));
    let ends = raster_pass_seamed(&mut b, p, r, &sups, (1.0, 0.0), false, p.tolerance, seam)?;
    if ends.is_none()
        && let Some((role, q)) = seam.start.map(|q| (PickRole::Start, q)).or(seam.exit.map(|q| (PickRole::Exit, q)))
    {
        return Err(GenError::Invalid(format!(
            "{}: the raster has no pass to read it on - no shallow ground stands inside the depths",
            named(role, &q)
        )));
    }
    Ok((rings_and_tail(b, p, r, facets, ends.is_some())?, ends))
}
