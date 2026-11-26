//! The user's start and exit points on the 3-D rough's levels (#3329,
//! the 3D Pocket slice of #1448). The level plan stays the strategy's:
//! a pick never re-orders the passes (the ring order is the engagement
//! law, core-first or from the side that Pass starts at chooses). A pick
//! only seats the passes that the plan already has:
//!
//! - **Start:** on each level, the first ring of the first pass starts
//!   at its point nearest the start pick, cuts its whole lap back to
//!   that point, then runs on along its own cut to its own old start.
//!   From there the level is the automatic path, move for move: no link
//!   changes. An open zigzag row chain starts at its end nearest the
//!   pick instead (rows cut both ways, so a reversed chain cuts the same
//!   mix). The descent onto the level then goes through the ladder's own
//!   entry rule (helix, ramp, arc or plunge, with their stand-ins),
//!   never a motion that the rule would refuse.
//! - **Exit:** on each level, the last ring of the last pass laps from
//!   its own start as before, then runs on along its own cut to its
//!   point nearest the exit pick. An open zigzag chain ends at its end
//!   nearest the pick instead. The operation's exit is the end of the
//!   last pass of the last level that cuts.
//! - A run-on re-traces a cut at its own depth, in its own direction:
//!   it is never a new cut. A level of one ring takes both.
//!
//! What a pick cannot keep is refused by name, never moved: a start
//! point farther than its reach from the first pass of a level, a
//! descent that the entry rule puts farther than that reach from the
//! start point, an exit point farther than its reach from the end of
//! the operation, a level that the Adaptive (HSM) plan spirals (it has
//! its own entry bore), and a plunge or retract column at a point that a
//! pick moved that comes within the tool radius of the part. The column
//! check is exact: the distance from the column's segment to each facet
//! above the column's foot (`stock_sim::dist_seg_tri`), no sampling.
//!
//! Provenance: first principles - the closest point on a segment
//! (orthogonal projection clamped to the segment), the rotation of a
//! closed polyline's vertex list and the reversal of an open one, the
//! clip of a triangle by a half-space (Sutherland and Hodgman 1974), as
//! the Deburr's chain seam. The entry and exit position is the common
//! vocabulary of published CAM practice. No vendor material consulted.

use super::hsm::LevelPlan;
use super::params::Pocket3DParams;

use crate::cl::{PathBuilder, lattice};
use crate::geo2d::Point2;
use crate::geometry::Point3;
use crate::machining2d::{GenError, PocketStrategy};
use crate::path_picks::{PathPicks, PickRole};

/// The start and exit points a 3-D rough reads, in model XY. No pick is
/// the automatic ladder of every earlier path, byte for byte.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LevelSeam {
    /// Each level's first pass starts at its point nearest this point.
    pub start: Option<Point2>,
    /// Each level's last pass ends at its point nearest this point.
    pub exit: Option<Point2>,
    /// A pick stated a Z, which the ladder does not read: each level
    /// descends at the pick's X and Y. Said once, by name.
    pub z_unread: bool,
}

/// The note of a pick whose Z the ladder does not read.
pub const PICK_Z_UNREAD: &str =
    "start / exit point: the 3D Pocket reads the X and Y of a pick - each level descends at the start point and leaves at the exit point, so the pick's Z is not read";

impl LevelSeam {
    /// The seam of an operation's picks: the first start and the first
    /// exit, in XY.
    #[must_use]
    pub fn of(picks: &PathPicks) -> Self {
        let s = picks.first(PickRole::Start);
        let e = picks.first(PickRole::Exit);
        Self {
            start: s.map(|q| q.xy),
            exit: e.map(|q| q.xy),
            z_unread: s.is_some_and(|q| q.z.is_some()) || e.is_some_and(|q| q.z.is_some()),
        }
    }

    /// No pick of either role.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.start.is_none() && self.exit.is_none()
    }
}

/// The farthest a pick may sit from the point of the pass it seats:
/// one and a half tool diameters, the Deburr's and the contour's slack
/// for a click beside the cut.
pub(super) fn reach(tool_diameter: f64) -> f64 {
    tool_diameter * 1.5
}

pub(super) use super::level_columns::columns;

/// The tolerance of the exact column law, mm: the region offsets
/// flatten a round join to 5 um and the lattice snaps to 1 um, so a
/// ring tangent to a wall sits a few microns inside the tool radius.
/// The sim's chord budget (`CHORD_TOL`) covers both.
pub(super) const COLUMN_TOL: f64 = super::params::CHORD_TOL;

fn named(role: PickRole, q: Point2) -> String {
    format!("{} point ({:.3}, {:.3})", role.word(), q.x, q.y)
}

fn d2(a: Point2, b: Point2) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

fn same(a: Point2, b: Point2) -> bool {
    a.x == b.x && a.y == b.y
}

/// The index that closes the first ring of a pass (a ring, or a chain
/// of rings: each ring repeats its own start): the first later point on
/// the pass's first point, at least three segments on. `None`: an open
/// pass.
fn first_ring_end(pass: &[Point2]) -> Option<usize> {
    let a = *pass.first()?;
    pass.iter().enumerate().skip(3).find(|(_, q)| same(**q, a)).map(|(k, _)| k)
}

/// The index that opens the last ring of a pass: the last earlier point
/// on the pass's last point, at least three segments back.
fn last_ring_start(pass: &[Point2]) -> Option<usize> {
    let b = *pass.last()?;
    let n = pass.len();
    (0..n.checked_sub(3)?).rev().find(|&j| same(pass[j], b))
}

/// The closest point to `q` on the polyline: `(segment, parameter,
/// lattice foot)`; the first segment wins a tie.
fn nearest(pass: &[Point2], q: Point2) -> Option<(usize, f64, Point2)> {
    let mut best: Option<(usize, f64, Point2, f64)> = None;
    for (i, w) in pass.windows(2).enumerate() {
        let (a, b) = (w[0], w[1]);
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let dd = dx * dx + dy * dy;
        let t = if dd > 0.0 { (((q.x - a.x) * dx + (q.y - a.y) * dy) / dd).clamp(0.0, 1.0) } else { 0.0 };
        let f = Point2::new(lattice(a.x + dx * t), lattice(a.y + dy * t));
        let d = d2(f, q);
        if d.is_finite() && best.is_none_or(|b| d < b.3) {
            best = Some((i, t, f, d));
        }
    }
    best.map(|(i, t, f, _)| (i, t, f))
}

/// One lap of the closed ring (`ring[m] == ring[0]`) from its point
/// nearest `s` back to that point, and the segment of that point.
fn lap_from(ring: &[Point2], s: Point2) -> Option<(Vec<Point2>, usize)> {
    let m = ring.len().checked_sub(1)?;
    let (seg, _, foot) = nearest(ring, s)?;
    let mut out = Vec::with_capacity(m + 2);
    out.push(foot);
    out.extend((1..=m).map(|k| ring[(seg + k) % m]));
    out.push(foot);
    out.dedup_by(|b, a| same(*a, *b));
    Some((out, seg))
}

/// The ring re-seated at its point nearest `s` ([`lap_from`]), then run
/// on along its own cut, in its own direction, to its own start
/// `ring[0]`, so that what follows the ring follows it unchanged.
fn seat_start(ring: &[Point2], s: Point2) -> Option<Vec<Point2>> {
    let m = ring.len().checked_sub(1)?;
    let (mut out, seg) = lap_from(ring, s)?;
    out.extend((seg + 1..=m).map(|k| ring[k % m]));
    out.dedup_by(|b, a| same(*a, *b));
    Some(out)
}

/// The run-on of the closed ring from its own start, after its lap,
/// along its own cut to its point nearest `e` (the points after
/// `ring[0]`, the foot last).
fn run_on(ring: &[Point2], e: Point2) -> Option<Vec<Point2>> {
    let (seg, _, foot) = nearest(ring, e)?;
    let mut out: Vec<Point2> = ring.get(1..=seg)?.to_vec();
    out.push(foot);
    Some(out)
}

/// Seat one level's plan under the picks (the module's laws; passes
/// are cutter-centre XY walks, first to last). Returns the points that a
/// pick moved: a new start, a new end, both ends of a reversed chain.
/// The start's reach is the level's law (every level descends at it);
/// the exit's is the operation's ([`LevelSeam::exit_at`]).
pub(super) fn seat_level(sm: &LevelSeam, passes: &mut [Vec<Point2>], zigzag: bool, z: f64, d: f64) -> Result<Vec<Point2>, GenError> {
    let mut moved = Vec::new();
    let n = passes.len();
    if n == 0 {
        return Ok(moved);
    }
    let flip = |pass: &mut Vec<Point2>, moved: &mut Vec<Point2>| {
        pass.reverse();
        moved.extend(pass.first().copied());
        moved.extend(pass.last().copied());
    };
    // A level of one ring and both picks: the lap from the start point
    // runs on straight to the exit point (no run back to the old start).
    if let (Some(s), Some(e), [only]) = (sm.start, sm.exit, &mut *passes)
        && first_ring_end(only).is_some_and(|k| k + 1 == only.len())
        && let Some((mut lap, _)) = lap_from(only, s)
        && let Some(tail) = run_on(&lap, e)
    {
        moved.extend(lap.first().copied());
        lap.extend(tail);
        lap.dedup_by(|b, a| same(*a, *b));
        moved.extend(lap.last().copied());
        *only = lap;
        return start_reach(passes, s, z, d).map(|()| moved);
    }
    if let Some(e) = sm.exit {
        let last = &mut passes[n - 1];
        if let Some(j) = last_ring_start(last) {
            let home = last[j];
            if let Some(tail) = run_on(&last[j..], e)
                && tail.last().is_some_and(|f| !same(*f, home))
            {
                moved.extend(tail.last().copied());
                last.extend(tail);
                last.dedup_by(|b, a| same(*a, *b));
            }
        } else if zigzag && !(n == 1 && sm.start.is_some()) {
            let (a, b) = (last.first().copied(), last.last().copied());
            if let (Some(a), Some(b)) = (a, b)
                && d2(a, e) < d2(b, e)
            {
                flip(last, &mut moved);
            }
        }
    }
    if let Some(s) = sm.start {
        let first = &mut passes[0];
        if let Some(k) = first_ring_end(first) {
            if let Some(mut head) = seat_start(&first[..=k], s)
                && head.first().is_some_and(|h| first.first().is_some_and(|f| !same(*h, *f)))
            {
                moved.extend(head.first().copied());
                head.extend_from_slice(&first[k + 1..]);
                head.dedup_by(|b, a| same(*a, *b));
                *first = head;
            }
        } else if zigzag {
            let (a, b) = (first.first().copied(), first.last().copied());
            if let (Some(a), Some(b)) = (a, b)
                && d2(b, s) < d2(a, s)
            {
                flip(first, &mut moved);
            }
        }
        start_reach(passes, s, z, d)?;
    }
    Ok(moved)
}

/// The start's reach at one level: the first pass starts within
/// [`reach`] of the start point, else the refusal by name.
fn start_reach(passes: &[Vec<Point2>], s: Point2, z: f64, d: f64) -> Result<(), GenError> {
    let at = passes.first().and_then(|p| p.first()).copied().unwrap_or(s);
    let lim = reach(d);
    let dist = d2(at, s);
    if dist <= lim {
        return Ok(());
    }
    Err(GenError::Invalid(format!(
        "{}: the first pass of the 3D Pocket at Z {z:.3} starts at ({:.3}, {:.3}), {dist:.3} mm from it, farther than \
         the {lim:.3} mm a start point may sit from it - each level descends on the first ring of its first pass (the \
         core ring under Pass starts at Auto, the island ring under Inside, the outer ring under Outside) or at an end \
         of its first zigzag rows; pick on or beside it or clear the start point",
        named(PickRole::Start, s),
        at.x,
        at.y
    )))
}

/// Say once that a pick's Z is not read.
pub(super) fn say_z(b: &mut PathBuilder, picks: Option<&LevelSeam>) {
    if picks.is_some_and(|s| s.z_unread) {
        b.note_once(PICK_Z_UNREAD);
    }
}

/// Seat the level plan `lp` at plane `z` ([`seat_level`]). A level the
/// Adaptive (HSM) plan spirals is refused by name: its passes start in
/// its own entry bore and ride proved arcs, which a re-seat would break.
pub(super) fn seat_plan(sm: &LevelSeam, lp: &mut LevelPlan, p: &Pocket3DParams, z: f64) -> Result<Vec<Point2>, GenError> {
    if lp.hsm || lp.arcs.is_some() {
        let role = if sm.start.is_some() { PickRole::Start } else { PickRole::Exit };
        return Err(GenError::Invalid(format!(
            "3D Pocket: a {} point is not taken on the Adaptive (HSM) level at Z {z:.3} - the plan spirals out of its own              entry bore; pick Zigzag, Waterline or Offset rings, or clear the point",
            role.word()
        )));
    }
    let zigzag = matches!(p.strategy, PocketStrategy::Zigzag | PocketStrategy::Auto);
    seat_level(sm, &mut lp.passes, zigzag, z, p.tool_diameter)
}

impl LevelSeam {
    /// The descent's own law: the entry rule placed the dive at `dove`
    /// (`None`: on the pass start itself). A dive farther than the reach
    /// from the start point is refused by name, never moved there.
    pub(super) fn descent(&self, dove: Option<Point2>, z: f64, d: f64) -> Result<(), GenError> {
        let (Some(s), Some(at)) = (self.start, dove) else { return Ok(()) };
        let lim = reach(d);
        let dist = d2(at, s);
        if dist <= lim {
            return Ok(());
        }
        Err(GenError::Invalid(format!(
            "{}: the entry rule at Z {z:.3} finds no room for its motion at the start point and descends at \
             ({:.3}, {:.3}), {dist:.3} mm from it, farther than the {lim:.3} mm a start point may sit from the descent - \
             pick where the entry fits, change the entry, or clear the start point",
            named(PickRole::Start, s),
            at.x,
            at.y
        )))
    }

    /// The operation's exit law: the last pass of the last level that
    /// cut (plane `z`) ends at `end`. Farther than the reach from the
    /// exit point is refused by name.
    pub(super) fn exit_at(&self, end: Point3, z: Option<f64>, d: f64) -> Result<(), GenError> {
        let (Some(e), Some(z)) = (self.exit, z) else { return Ok(()) };
        let lim = reach(d);
        let dist = d2(Point2::new(end.x, end.y), e);
        if dist <= lim {
            return Ok(());
        }
        Err(GenError::Invalid(format!(
            "{}: the last pass of the 3D Pocket at Z {z:.3} ends at ({:.3}, {:.3}), {dist:.3} mm from it, farther than \
             the {lim:.3} mm an exit point may sit from it - the operation leaves at the end of the last pass of its \
             deepest level (the outer ring under Pass starts at Auto or Inside, the boundary rings of the zigzag rows); \
             pick on or beside that pass or clear the exit point",
            named(PickRole::Exit, e),
            end.x,
            end.y
        )))
    }
}

#[cfg(test)]
#[path = "level_seam_tests.rs"]
mod tests;
