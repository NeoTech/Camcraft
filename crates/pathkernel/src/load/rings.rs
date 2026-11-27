//! Constant engagement on rings (K3, #4026): where a ring reads above the
//! load on the stock that the rings before it leave, a ring of the same
//! distance field is inserted between it and the nearest ring cut before
//! it (the mid level), and the plan is read again. An insertion that does
//! not lower the load (the classic sharp corner of offset rings) is taken
//! out again and the cut is named; so is a cut with no earlier pass beside
//! it, a curve that is not a ring of a distance field, and a gap at the
//! smallest step.
//! Provenance: the engagement of a circular path and the closer spacing
//! that it needs round a small core (Stori & Wright 2000, "Constant
//! engagement tool path generation for convex geometries", J. Manuf.
//! Systems 19(3)); rings as level sets of the distance to the walls (Held
//! 1991); the floor of 15 % of the load width from the held #3933 design
//! (`docs/decisions/3933-pocket-offset-rings-spiral.md`).

use std::collections::{HashMap, HashSet};

use super::law::{Read, Stock, read};
use super::stage::{FRESH_CUT, Outcome, beside, finish, on_span, poly_dist};
use super::{Controlled, OrderedSet, PLAN_TOL};
use crate::frame::PlanePoint;
use crate::guide::{Curve, Family, MarkKind};
use crate::region::PlanarRegion;
use crate::verdict::{Refusal, Stage, Verdict};
use crate::{Loop, Point2, Region, to_loops};

/// Over-limit reason: an inserted pass did not lower the load.
pub const NOT_FIXED: &str = "more passes do not bring this cut under the load limit (a sharp corner of the rings): use Adaptive or round the corner";
/// Over-limit reason: the cut is not a ring of a distance field.
pub const NOT_A_RING: &str = "this cut is not a ring of the wall distance: no pass can be inserted before it";
/// Over-limit reason: the gap to the pass before is at the smallest step.
pub const AT_FLOOR: &str = "the passes are at the smallest step and this cut is still above the load limit";
/// Over-limit reason: the most passes were inserted.
pub const TOO_MANY_PASSES: &str = "the load stage inserted the most passes that it makes; this cut is still above the load limit";
/// Refusal reason: a polygon operation of the stage failed.
pub const LOAD_GEOMETRY: &str = "the load stage geometry cannot be computed: a value is out of range";

/// The most read-and-insert rounds.
const MAX_ROUNDS: usize = 12;
/// The most curves: this many times the curves of the guide, and 64 more.
const GROWTH: usize = 8;
/// The smallest gap between two rings: this share of the load width
/// (the floor of the held #3933 design).
const FLOOR_SHARE: f64 = 0.15;

/// The distance field of a ring.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Field {
    /// The distance to every wall of the cutter-centre area.
    Wall,
    /// The distance to the part walls only (a region with a stock edge).
    Part,
}

fn geo<T, E>(r: Result<T, E>) -> Result<T, Refusal> {
    r.map_err(|_| Refusal::new(Stage::Load, LOAD_GEOMETRY))
}

fn refs(p: &PlanarRegion, f: Field) -> &[Loop] {
    match f {
        Field::Wall => p.center().loops(),
        Field::Part => p.part_walls(),
    }
}

/// The level of curve `c` in field `f`: the distance of each point to the
/// field's walls, when they agree within `tol`.
fn depth(c: &Curve, p: &PlanarRegion, f: Field, tol: f64) -> Option<f64> {
    let walls = refs(p, f);
    if walls.is_empty() {
        return None;
    }
    let ds: Vec<f64> = c.pts().iter().map(|q| to_loops(q.at(), walls)).collect();
    let (lo, hi) = ds.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), d| (a.min(*d), b.max(*d)));
    (hi - lo <= tol && lo.is_finite()).then(|| (lo + hi) / 2.0)
}

/// The fields to try: the part walls first where the region has a stock edge.
fn fields(p: &PlanarRegion) -> Vec<Field> {
    if p.stock_edge().is_empty() || p.part_walls().is_empty() { vec![Field::Wall] } else { vec![Field::Part, Field::Wall] }
}

/// The pieces of the level `d` of field `f` that lie within `gap` of
/// `near`, each in the direction of `near`, as ring curves.
fn between(p: &PlanarRegion, f: Field, d: f64, near: &Curve, gap: f64, tol: f64) -> Result<Vec<Curve>, Refusal> {
    let area = match f {
        Field::Wall => geo(p.center().offset(-d))?,
        Field::Part => {
            let walks: Vec<Vec<Point2>> = p.part_walls().iter().map(|l| l.points().iter().chain(l.points().first()).copied().collect()).collect();
            geo(p.center().difference(&geo(Region::sweep(&walks, d))?))?
        }
    };
    let walls = refs(p, f);
    let z = near.pts().first().map_or(0.0, PlanePoint::depth);
    let mut out = Vec::new();
    for l in area.loops() {
        let pts = l.points();
        let on: Vec<bool> = pts.iter().map(|q| (to_loops(*q, walls) - d).abs() <= tol).collect();
        for (mut piece, closed) in runs(pts, &on) {
            if piece.iter().any(|q| poly_dist(*q, near) > gap + tol) {
                continue;
            }
            orient(&mut piece, closed, near);
            let pp: Result<Vec<PlanePoint>, Refusal> = piece.into_iter().map(|q| PlanePoint::new(q, z)).collect();
            if let Ok(c) = Curve::new(pp?, None, closed, near.k(), Family::Ring) {
                out.push(c);
            }
        }
    }
    Ok(out)
}

/// The runs of `pts` (a closed loop) where `on` holds: the whole loop when
/// it holds everywhere, else open pieces of two or more points.
fn runs(pts: &[Point2], on: &[bool]) -> Vec<(Vec<Point2>, bool)> {
    let n = pts.len();
    let Some(off) = on.iter().position(|b| !b) else {
        return vec![(pts.to_vec(), true)];
    };
    let (mut out, mut cur) = (Vec::new(), Vec::new());
    for j in 1..=n {
        let i = (off + j) % n;
        if on.get(i).copied().unwrap_or(false) {
            cur.extend(pts.get(i).copied());
        } else if !cur.is_empty() {
            if cur.len() >= 2 {
                out.push((std::mem::take(&mut cur), false));
            }
            cur.clear();
        }
    }
    out
}

/// Turns `piece` to run as `near` runs beside it; a closed piece starts at
/// its point nearest the start of `near`.
fn orient(piece: &mut [Point2], closed: bool, near: &Curve) {
    let line: Vec<Point2> = near.pts().iter().map(PlanePoint::at).collect();
    let dir = |q: Point2| -> Point2 {
        let best = line.windows(2).min_by(|a, b| crate::seg_dist(q, a[0], a[1]).total_cmp(&crate::seg_dist(q, b[0], b[1])));
        best.map_or(Point2::new(0.0, 0.0), |w| Point2::new(w[1].x - w[0].x, w[1].y - w[0].y))
    };
    let along: f64 = piece.windows(2).map(|w| {
        let u = dir(w[0]);
        (w[1].x - w[0].x) * u.x + (w[1].y - w[0].y) * u.y
    }).sum();
    if along < 0.0 {
        piece.reverse();
    }
    if closed && let Some(s) = line.first() {
        let k = piece.iter().enumerate().min_by(|a, b| (a.1.x - s.x).hypot(a.1.y - s.y).total_cmp(&(b.1.x - s.x).hypot(b.1.y - s.y))).map_or(0, |(k, _)| k);
        piece.rotate_left(k);
    }
}

/// The worst read above the cap of each curve, and the count of them.
#[derive(Clone, Copy, Debug, Default)]
struct Hot {
    count: usize,
    worst: f64,
    at: Point2,
}

fn hot(reads: &[Read], n: usize, cap: f64) -> Vec<Hot> {
    let mut out = vec![Hot::default(); n];
    for rd in reads.iter().filter(|r| r.load > cap) {
        if let Some(h) = out.get_mut(rd.curve) {
            h.count += 1;
            if rd.load > h.worst {
                (h.worst, h.at) = (rd.load, rd.at);
            }
        }
    }
    out
}

/// An insertion lowered the load of its cut: no read above the cap is
/// left, or the worst excess halved, or a quarter of the hot reads went.
fn improved(before: Hot, after: Hot, cap: f64) -> bool {
    after.count == 0 || (after.worst - cap) <= 0.5 * (before.worst - cap) || after.count * 4 <= before.count * 3
}

/// What one hot ring gets.
enum Step {
    Insert(Vec<Curve>),
    Named(&'static str),
}

/// The passes to insert before curve `i` (hot at `at`), or the reason
/// that none can be.
fn plan_for(curves: &[Curve], i: usize, at: Point2, p: &PlanarRegion, (r, tol, floor): (f64, f64, f64)) -> Result<Step, Refusal> {
    // Only a ring is a level of a distance field: a pass is never inserted
    // before a row, a medial line, a trace or a helix (#4176), even where
    // its points are at one distance to the walls (a helix on a circle
    // round the centre of a hole).
    let Some(c) = curves.get(i).filter(|c| c.family() == Family::Ring) else { return Ok(Step::Named(NOT_A_RING)) };
    let prev = curves.iter().take(i).map(|e| (poly_dist(at, e), e)).min_by(|a, b| a.0.total_cmp(&b.0));
    let Some((dist, j)) = prev.filter(|(d, _)| *d <= 2.0 * r + tol) else { return Ok(Step::Named(FRESH_CUT)) };
    for f in fields(p) {
        let (Some(di), Some(dj)) = (depth(c, p, f, tol), depth(j, p, f, tol)) else { continue };
        let gap = (di - dj).abs();
        if gap < 2.0 * floor {
            return Ok(Step::Named(AT_FLOOR));
        }
        let mid = between(p, f, (di + dj) / 2.0, c, gap.max(dist), tol)?;
        if !mid.is_empty() {
            return Ok(Step::Insert(mid));
        }
    }
    Ok(Step::Named(NOT_A_RING))
}

/// Constant engagement at `load` (ae/D) on the ordered rings `s`.
pub(crate) fn control(s: &OrderedSet, p: &PlanarRegion, stock: &Stock, load: f64, r: f64) -> Result<Verdict<Controlled>, Refusal> {
    let cap = load * (1.0 + PLAN_TOL);
    let tol = (0.01 * r).max(0.01);
    let floor = (FLOOR_SHARE * load * 2.0 * r).max(2.0 * tol);
    let most = s.curves().len().saturating_mul(GROWTH).saturating_add(64);
    // Each curve keeps an id: its index in `s`, or a new one for an inserted pass.
    let mut items: Vec<(usize, Curve)> = s.curves().iter().cloned().enumerate().collect();
    let mut next = items.len();
    let (mut why, mut stuck): (HashMap<usize, &'static str>, HashSet<usize>) = (HashMap::new(), HashSet::new());
    let mut pending: Vec<(usize, Vec<usize>, Hot)> = Vec::new();
    for _ in 0..MAX_ROUNDS {
        let curves: Vec<Curve> = items.iter().map(|(_, c)| c.clone()).collect();
        let heat = hot(&read(stock, &curves, r), curves.len(), cap);
        let mut reverted = false;
        for (target, added, before) in pending.drain(..) {
            let after = items.iter().position(|(id, _)| *id == target).and_then(|k| heat.get(k)).copied().unwrap_or_default();
            if !improved(before, after, cap) {
                items.retain(|(id, _)| !added.contains(id));
                stuck.insert(target);
                why.insert(target, NOT_FIXED);
                reverted = true;
            }
        }
        if reverted {
            continue;
        }
        let mut adds: Vec<(usize, Vec<Curve>, Hot)> = Vec::new();
        for (k, h) in heat.iter().enumerate().filter(|(_, h)| h.count > 0) {
            let id = items.get(k).map_or(usize::MAX, |(id, _)| *id);
            if stuck.contains(&id) {
                continue;
            }
            match plan_for(&curves, k, h.at, p, (r, tol, floor))? {
                Step::Insert(mid) => adds.push((k, mid, *h)),
                Step::Named(reason) => {
                    why.insert(id, reason);
                }
            }
        }
        let extra: usize = adds.iter().map(|(_, m, _)| m.len()).sum();
        if adds.is_empty() {
            break;
        }
        if items.len().saturating_add(extra) > most {
            for (k, _, _) in &adds {
                if let Some((id, _)) = items.get(*k) {
                    why.insert(*id, TOO_MANY_PASSES);
                }
            }
            break;
        }
        for (k, mid, h) in adds.into_iter().rev() {
            let target = items.get(k).map_or(usize::MAX, |(id, _)| *id);
            let ids: Vec<usize> = (next..next + mid.len()).collect();
            next += mid.len();
            for (c, id) in mid.into_iter().zip(ids.iter()).rev() {
                items.insert(k, (*id, c));
            }
            pending.push((target, ids, h));
        }
    }
    let curves: Vec<Curve> = items.iter().map(|(_, c)| c.clone()).collect();
    let reads = read(stock, &curves, r);
    let changed = items.len() != s.curves().len();
    let at: HashMap<usize, usize> = items.iter().enumerate().map(|(k, (id, _))| (*id, k)).collect();
    // A full-width stretch with no read above the cap is resolved: its mark goes.
    let mut marks = Vec::new();
    for m in s.marks() {
        let Some(&k) = at.get(&m.curve()) else { continue };
        let still = reads.iter().any(|rd| rd.curve == k && rd.load > cap && on_span(m, rd.at));
        if m.kind() != MarkKind::FullWidth || still {
            marks.push(m.clone().renumber(k));
        }
    }
    let ids: Vec<usize> = items.iter().map(|(id, _)| *id).collect();
    let reason = |rd: &Read| -> &'static str {
        if !beside(&curves, rd.curve, rd.at, 2.0 * r + tol) {
            return FRESH_CUT;
        }
        ids.get(rd.curve).and_then(|id| why.get(id)).copied().unwrap_or(NOT_FIXED)
    };
    let marks = if changed { marks } else { s.marks().to_vec() };
    let out = Outcome { curves: changed.then(|| curves.clone()), marks, reads };
    finish(s, out, load, "constant engagement", reason)
}
