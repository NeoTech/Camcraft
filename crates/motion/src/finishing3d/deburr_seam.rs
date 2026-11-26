//! The user's start and exit points on Deburr's edge chains (#3329, the
//! Deburr slice of #1448). The chains and their cut geometry stay the
//! automatic ride's; a pick only chooses the order:
//!
//! - **Start:** the chain nearest the start pick is cut first. An open
//!   chain starts at its end nearest the pick (it runs the other way
//!   when that is its far end). A closed chain is re-seated: it starts
//!   and ends at its point nearest the pick.
//! - **Exit:** the chain nearest the exit pick is cut last. An open
//!   chain ends at its end nearest the pick. A closed chain is re-seated
//!   at its point nearest the pick, so its lap ends there. When the start
//!   and the exit are on one closed chain, the chain starts at the start
//!   point, cuts its whole lap and runs on along its own edges to the
//!   exit point (a re-trace of cut material, never a new cut).
//! - The other chains keep the automatic order between them.
//!
//! What a pick cannot keep is refused by name before any motion, never
//! moved: a pick farther than its reach from every chain, an exit that
//! an open chain cannot end at (its start end), a start and an exit on
//! one chain while other chains remain (the operation cannot both
//! start and end on that chain), and a plunge or retract
//! column at a moved chain end that comes within the tool radius of the
//! part. The column check is exact: the distance from the column's
//! segment to each facet (`stock_sim::dist_seg_tri`), no sampling. Each
//! end that a pick moves is checked: the new start and the new end of a
//! reversed or re-seated chain.
//!
//! Provenance: first principles - the closest point on a segment
//! (orthogonal projection clamped to the segment), the rotation of a
//! closed polyline's vertex list and the reversal of an open one, as
//! the 5-axis contour's ring seam (`ring_seam`). The entry and exit
//! position is the common vocabulary of published CAM practice. No
//! vendor material consulted.

use crate::geometry::{Point3, Triangle};
use crate::machining2d::GenError;
use crate::path_picks::{PathPick, PathPicks, PickRole};
use crate::stock_sim::dist_seg_tri;

/// The start and exit picks a Deburr ride reads, in model mm, with the
/// chamfer mill's diameter. No pick is the automatic order of every
/// earlier path, byte for byte.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ChainSeam {
    /// The chain nearest this pick is cut first, from its point nearest it.
    pub start: Option<PathPick>,
    /// The chain nearest this pick is cut last, to its point nearest it.
    pub exit: Option<PathPick>,
    /// The chamfer mill's largest diameter, mm: it sets the reach of a
    /// pick and the radius that a moved plunge or retract column clears.
    pub tool_diameter: f64,
}

impl ChainSeam {
    /// The seam of an operation's picks: the first start and the first
    /// exit. A pick with a Z is measured in 3D, one without in XY.
    #[must_use]
    pub fn of(picks: &PathPicks, tool_diameter: f64) -> Self {
        Self { start: picks.first(PickRole::Start), exit: picks.first(PickRole::Exit), tool_diameter }
    }

    /// No pick of either role.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.start.is_none() && self.exit.is_none()
    }
}

/// One chain of the ride and its tip drop.
pub(super) type Run = (Vec<Point3>, f64);

/// The farthest a pick may sit from the nearest chain: one and a half
/// tool diameters, the contour's slack for a click beside the edge.
fn reach(tool_diameter: f64) -> f64 {
    tool_diameter * 1.5
}

/// "start point (x, y)" or "start point (x, y, z)", the refusals' lead.
fn named(role: PickRole, q: PathPick) -> String {
    match q.z {
        Some(z) => format!("{} point ({:.3}, {:.3}, {z:.3})", role.word(), q.xy.x, q.xy.y),
        None => format!("{} point ({:.3}, {:.3})", role.word(), q.xy.x, q.xy.y),
    }
}

/// The distance from the pick to `p`: in 3D when the pick has a Z,
/// else in XY.
fn dist(q: PathPick, p: Point3) -> f64 {
    let (dx, dy) = (p.x - q.xy.x, p.y - q.xy.y);
    let dz = q.z.map_or(0.0, |z| p.z - z);
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// The closest point to the pick on the polyline `run`: `(segment
/// index, parameter on it, point, distance)`; the first segment wins a
/// tie. `None` for a run with no segment or no finite answer.
fn nearest(run: &[Point3], q: PathPick) -> Option<(usize, f64, Point3, f64)> {
    let mut best: Option<(usize, f64, Point3, f64)> = None;
    for (i, w) in run.windows(2).enumerate() {
        let (a, b) = (w[0], w[1]);
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let dz = if q.z.is_some() { b.z - a.z } else { 0.0 };
        let (px, py, pz) = (q.xy.x - a.x, q.xy.y - a.y, q.z.map_or(0.0, |z| z - a.z));
        let dd = dx * dx + dy * dy + dz * dz;
        let t = if dd > 0.0 { ((px * dx + py * dy + pz * dz) / dd).clamp(0.0, 1.0) } else { 0.0 };
        let f = Point3::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t, a.z + (b.z - a.z) * t);
        let d = dist(q, f);
        if d.is_finite() && best.is_none_or(|b| d < b.3) {
            best = Some((i, t, f, d));
        }
    }
    best
}

/// Drops each point that repeats its predecessor (a foot that lands on
/// a vertex), so no zero-length move is cut.
fn dedupe(pts: &mut Vec<Point3>) {
    pts.dedup_by(|b, a| (a.x - b.x).abs() <= 1e-12 && (a.y - b.y).abs() <= 1e-12 && (a.z - b.z).abs() <= 1e-12);
}

/// Where a pick lands: the chain, and the closest point on it.
#[derive(Clone, Copy)]
struct Landing {
    run: usize,
    seg: usize,
    t: f64,
    foot: Point3,
}

/// The chain nearest the pick of `role`, or the by-name refusal of a
/// pick beyond its reach.
fn land(runs: &[Run], q: PathPick, role: PickRole, tool_diameter: f64) -> Result<Landing, GenError> {
    let mut best: Option<(Landing, f64)> = None;
    for (k, (run, _)) in runs.iter().enumerate() {
        let Some((seg, t, foot, d)) = nearest(run, q) else { continue };
        if best.as_ref().is_none_or(|b| d < b.1) {
            best = Some((Landing { run: k, seg, t, foot }, d));
        }
    }
    let lim = reach(tool_diameter);
    match best {
        Some((l, d)) if d <= lim => Ok(l),
        Some((_, d)) => Err(GenError::Invalid(format!(
            "{} is {d:.3} mm from the nearest edge chain of the Deburr, farther than the {lim:.3} mm a {} point may sit \
             from it - pick on or beside an edge to break",
            named(role, q),
            role.word()
        ))),
        None => Err(GenError::Invalid(format!(
            "{}: the Deburr has no edge chain to {} on - clear the {} point",
            named(role, q),
            role.word(),
            role.word()
        ))),
    }
}

/// The closed chain `run` (`run[m] == run[0]`) re-seated to start and
/// end at the foot `l`, then, with `on`, carried on along its own
/// edges to the foot `on`.
fn seat(run: &[Point3], l: Landing, on: Option<Landing>) -> Vec<Point3> {
    let m = run.len().saturating_sub(1);
    let mut out = Vec::with_capacity(m + 3);
    out.push(l.foot);
    out.extend((1..=m).map(|k| run[(l.seg + k) % m]));
    out.push(l.foot);
    if let Some(g) = on {
        let mut steps = (g.seg + m - l.seg) % m;
        if steps == 0 && g.t < l.t {
            steps = m;
        }
        out.extend((1..=steps).map(|k| run[(l.seg + k) % m]));
        out.push(g.foot);
    }
    dedupe(&mut out);
    out
}

/// Does the open chain `run` run the other way so that it starts at
/// the end nearest the start pick (or ends at the end nearest the exit
/// pick)? A pick as near to one end as the other keeps the automatic
/// direction.
fn far_end(run: &[Point3], q: PathPick) -> bool {
    match (run.first(), run.last()) {
        (Some(&a), Some(&b)) => dist(q, b) < dist(q, a),
        _ => false,
    }
}

impl ChainSeam {
    /// The runs in the order and the direction the picks give, or `None`
    /// with no pick (the automatic order). `closed[k]` says that run `k`
    /// closes on itself. `facets` and `clearance_z` judge the columns of
    /// each moved chain end. Refused by name, before any motion.
    pub(super) fn order(&self, runs: &[Run], closed: &[bool], facets: &[Triangle], clearance_z: f64) -> Result<Option<Vec<Run>>, GenError> {
        if self.is_empty() {
            return Ok(None);
        }
        let d = self.tool_diameter;
        if !(d.is_finite() && d > 0.0) {
            return Err(GenError::Invalid(format!("Deburr start and exit points need a positive tool diameter, not {d}")));
        }
        let s = self.start.map(|q| land(runs, q, PickRole::Start, d).map(|l| (q, l))).transpose()?;
        let e = self.exit.map(|q| land(runs, q, PickRole::Exit, d).map(|l| (q, l))).transpose()?;
        let mut out: Vec<Run> = runs.to_vec();
        // The pick that answers for each moved end of each chain:
        // (plunge, retract).
        let mut owner: Vec<Option<(PathPick, PickRole, PathPick, PickRole)>> = vec![None; runs.len()];
        let is_closed = |k: usize| closed.get(k).copied().unwrap_or(false) && runs[k].0.len() >= 3;
        match (s, e) {
            (Some((sq, sl)), Some((eq, el))) if sl.run == el.run => {
                let k = sl.run;
                let run = &runs[k].0;
                // One chain cannot be cut first and last while other
                // chains are cut between: the operation would end on
                // another chain, a silently moved exit (#4168 review).
                let others = runs.iter().enumerate().filter(|(j, (r, _))| *j != k && r.len() >= 2).count();
                if others > 0 {
                    return Err(GenError::Invalid(format!(
                        "{}: the start point and the exit point pick one edge chain, but the Deburr cuts {others} other \
                         edge chain(s) after the start chain - pick the exit on another chain or clear one of the two points",
                        named(PickRole::Exit, eq)
                    )));
                }
                if is_closed(k) {
                    out[k].0 = seat(run, sl, Some(el));
                } else {
                    let flip = far_end(run, sq);
                    // The exit must be nearest the end the start leaves free.
                    if far_end(run, eq) == flip {
                        return Err(GenError::Invalid(format!(
                            "{}: the open edge chain that starts at the start point ends at its other end - pick near \
                             that end or clear the exit point",
                            named(PickRole::Exit, eq)
                        )));
                    }
                    if flip {
                        out[k].0.reverse();
                    }
                }
                owner[k] = Some((sq, PickRole::Start, eq, PickRole::Exit));
            }
            (s, e) => {
                if let Some((q, l)) = s {
                    let k = l.run;
                    if is_closed(k) {
                        out[k].0 = seat(&runs[k].0, l, None);
                    } else if far_end(&runs[k].0, q) {
                        out[k].0.reverse();
                    }
                    owner[k] = Some((q, PickRole::Start, q, PickRole::Start));
                }
                if let Some((q, l)) = e {
                    let k = l.run;
                    if is_closed(k) {
                        out[k].0 = seat(&runs[k].0, l, None);
                    } else if near_first(&runs[k].0, q) {
                        out[k].0.reverse();
                    }
                    owner[k] = Some((q, PickRole::Exit, q, PickRole::Exit));
                }
            }
        }
        columns(runs, &out, &owner, 0.5 * d, facets, clearance_z)?;
        // The start chain first, the exit chain last, the rest between.
        let first = s.map(|(_, l)| l.run);
        let last = e.map(|(_, l)| l.run).filter(|k| Some(*k) != first);
        let mut seq: Vec<usize> = first.into_iter().collect();
        seq.extend((0..out.len()).filter(|k| Some(*k) != first && Some(*k) != last));
        seq.extend(last);
        Ok(Some(seq.into_iter().map(|k| out[k].clone()).collect()))
    }
}

/// Is the first point of the open chain `run` strictly nearer the pick
/// than its last? The exit chain then runs the other way.
fn near_first(run: &[Point3], q: PathPick) -> bool {
    match (run.first(), run.last()) {
        (Some(&a), Some(&b)) => dist(q, a) < dist(q, b),
        _ => false,
    }
}

/// The exact column law at every chain end that a pick moved: the
/// plunge column at a new first point and the retract column at a new
/// last point. The refusal names the pick that moved the end.
fn columns(
    auto: &[Run],
    out: &[Run],
    owner: &[Option<(PathPick, PickRole, PathPick, PickRole)>],
    r: f64,
    facets: &[Triangle],
    clearance_z: f64,
) -> Result<(), GenError> {
    let same = |a: Option<&Point3>, b: Option<&Point3>| match (a, b) {
        (Some(a), Some(b)) => a.x == b.x && a.y == b.y && a.z == b.z,
        _ => a.is_none() && b.is_none(),
    };
    for (k, (run, _)) in out.iter().enumerate() {
        let Some((sq, sr, eq, er)) = owner.get(k).copied().flatten() else { continue };
        let was = &auto[k].0;
        let ends = [
            (run.first(), !same(run.first(), was.first()), sq, sr, "plunge column to"),
            (run.last(), !same(run.last(), was.last()), eq, er, "retract column from"),
        ];
        for (p, moved, q, role, what) in ends {
            let Some(&p) = p else { continue };
            if moved && !column_clear(p, r, clearance_z, facets) {
                return Err(GenError::Invalid(format!(
                    "{}: the {what} the edge at ({:.3}, {:.3}, {:.3}) comes within the tool radius {r:.3} mm of the \
                     part - pick another point or clear the {} point",
                    named(role, q),
                    p.x,
                    p.y,
                    p.z,
                    role.word()
                )));
            }
        }
    }
    Ok(())
}

/// Does the vertical column above the edge point `p` keep a full tool
/// radius `r` from every facet, from one radius above the edge up to
/// the clearance plane? Below that the cone works across the break,
/// which the ride's contact law sizes. Exact: the distance from the
/// column's segment to each facet, no sampling.
pub(super) fn column_clear(p: Point3, r: f64, clearance_z: f64, facets: &[Triangle]) -> bool {
    let top = clearance_z.max(p.z);
    let a = Point3::new(p.x, p.y, (p.z + r).min(top));
    let b = Point3::new(p.x, p.y, top);
    let limit = r - 1e-6;
    facets.iter().all(|t| dist_seg_tri(a, b, t) >= limit)
}
