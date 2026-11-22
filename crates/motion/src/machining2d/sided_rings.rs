//! The rings of a level that starts on one side (#3316, Inside /
//! Outside): level curves of the distance from the start side, clipped
//! to the cutter-center area, then the far wall last. Provenance: the
//! contour-parallel offset curves of published pocketing literature
//! (offsets of one boundary family, clipped to the domain); no vendor
//! material consulted.
//!
//! A closed-ring order cannot be right once a band splits: one ring of
//! the area's own offset family has arcs at the stock side AND arcs at
//! the part side, and the part-side arcs then cut uncut stock on both
//! sides - a full slot. Here ring `j` is the boundary of the start side
//! alone, moved `j` stepovers into the area (Outside: the outer loops
//! filled and eroded; Inside: the holes filled and grown). Its parts
//! that fall inside the area are OPEN pieces ending on the far wall.
//! Ring `j` is cut before ring `j + 1`, so each piece has cut material
//! one stepover behind it and engages one stepover. The far wall ring
//! (the area's own far loops) comes last, over material that the pieces
//! already cut to within a stepover.

use crate::geo2d::{Loop, Point2, Region};

use super::face::GenError;
use super::pass_start::PassStart;
use super::ring::dist;
use super::sided_chain::chain;

/// One ring piece: its ring (numbered in cutting order), its points in
/// cutting order, and whether it is a whole loop (travelled back to its
/// first point).
pub(super) struct Piece {
    pub(super) ring: usize,
    pub(super) pts: Vec<Point2>,
    pub(super) closed: bool,
}

/// The number of the next ring: one past the last piece's.
fn next_ring(out: &[Piece]) -> usize {
    out.last().map_or(0, |p| p.ring + 1)
}

/// The passes of one level for [`PassStart::Inside`] / `Outside` (Auto
/// never comes here). Each component of `center` (an outer loop with its
/// own holes) is planned alone, in `center`'s loop order.
pub(super) fn sided_passes(
    center: &Region,
    stepover: f64,
    climb: bool,
    start: PassStart,
) -> Result<Vec<Vec<Point2>>, GenError> {
    if !(stepover.is_finite() && stepover > 0.0) {
        return Err(GenError::Invalid("stepover must be positive".into()));
    }
    let Some((lo, hi)) = center.bounds() else {
        return Err(GenError::Empty("empty pocket region".into()));
    };
    let cap = (dist(lo, hi) / stepover + 10.0).max(16.0) as u32;
    let mut pieces: Vec<Piece> = Vec::new();
    for (outer, holes) in components(center)? {
        component_pieces(&outer, &holes, stepover, climb, start, cap, &mut pieces)?;
    }
    Ok(chain(center, &pieces, stepover))
}

/// Split `center` into its components: each counter-clockwise loop with
/// the clockwise loops whose nearest enclosing outer loop it is. A hole
/// with no enclosing loop is refused: the rings would run through it.
fn components(center: &Region) -> Result<Vec<(Loop, Vec<Loop>)>, GenError> {
    let fill = |l: &Loop| Region::new(vec![l.clone()]).ok();
    let outers: Vec<(&Loop, Option<Region>)> =
        center.loops().iter().filter(|l| l.is_ccw()).map(|l| (l, fill(l))).collect();
    let mut out: Vec<(Loop, Vec<Loop>)> = outers.iter().map(|(l, _)| ((*l).clone(), Vec::new())).collect();
    for h in center.loops().iter().filter(|l| !l.is_ccw()) {
        let Some(&p) = h.points().first() else { continue };
        let parent = outers
            .iter()
            .enumerate()
            .filter(|(_, (_, f))| f.as_ref().is_some_and(|f| f.contains(p)))
            .min_by(|a, b| a.1.0.area().abs().total_cmp(&b.1.0.area().abs()))
            .map(|(i, _)| i);
        match parent {
            Some(i) => out[i].1.push(h.clone()),
            None => return Err(GenError::Invalid("an island of the area has no enclosing boundary".into())),
        }
    }
    Ok(out)
}

/// A loop's points in cutting order. `uncut_left` says on which side of
/// the loop's own orientation the material that is still to cut lies.
/// Climb keeps that material on the right of the travel (#1555).
fn walk(l: &Loop, uncut_left: bool, climb: bool) -> Vec<Point2> {
    if uncut_left == climb {
        l.points().iter().rev().copied().collect()
    } else {
        l.points().to_vec()
    }
}

#[allow(clippy::too_many_arguments)] // one component: its loops, the step, the walk law and the output
fn component_pieces(
    outer: &Loop,
    holes: &[Loop],
    stepover: f64,
    climb: bool,
    start: PassStart,
    cap: u32,
    out: &mut Vec<Piece>,
) -> Result<(), GenError> {
    let mut loops = vec![outer.clone()];
    loops.extend(holes.iter().cloned());
    let area = Region::new(loops)?;
    // Inside on a component without holes has no inside to start from:
    // its rings are the area's own offsets, core first (Auto's order).
    let (seed, grow, far) = match start {
        PassStart::Outside => (Region::new(vec![outer.clone()])?, -1.0, holes.to_vec()),
        _ if holes.is_empty() => return core_first(&area, stepover, climb, cap, out),
        _ => (Region::new(holes.to_vec())?, 1.0, vec![outer.clone()]),
    };
    // The seed's loops are region loops (material on the left). Outside
    // cuts toward the seed's inside (left); Inside away from it (right).
    let uncut_left = start == PassStart::Outside;
    let rings = level_rings(&seed, grow, stepover, cap, &area, |l| walk(l, uncut_left, climb))?;
    for ring in rings {
        let id = next_ring(out);
        out.extend(ring.into_iter().map(|(pts, closed)| Piece { ring: id, pts, closed }));
    }
    // The far wall last: the area's own loops on that side, walked with
    // the finish wall on the right for climb (the Auto ring law).
    let id = next_ring(out);
    for l in &far {
        let pts = if climb { l.points().to_vec() } else { l.points().iter().rev().copied().collect() };
        out.push(Piece { ring: id, pts, closed: true });
    }
    Ok(())
}

/// One clipped part of a ring: its points, and whether it is the whole
/// loop.
type Cut = (Vec<Point2>, bool);

/// The rings of one family: ring 0 is `seed`'s own loops, whole; ring
/// `j` is `seed` offset by `grow * j * stepover`, its parts inside
/// `keep`. Stops at the first ring with no part in `keep`.
fn level_rings(
    seed: &Region,
    grow: f64,
    stepover: f64,
    cap: u32,
    keep: &Region,
    walk: impl Fn(&Loop) -> Vec<Point2>,
) -> Result<Vec<Vec<Cut>>, GenError> {
    let mut rings = vec![seed.loops().iter().map(|l| (walk(l), true)).collect::<Vec<_>>()];
    for j in 1..=cap {
        let ring = seed.offset(grow * f64::from(j) * stepover)?;
        let mut pieces: Vec<(Vec<Point2>, bool)> = Vec::new();
        for l in ring.loops() {
            pieces.extend(clip_closed(&walk(l), keep)?);
        }
        if pieces.is_empty() {
            return Ok(rings);
        }
        rings.push(pieces);
    }
    Err(GenError::Invalid("start-side rings did not converge; check the stepover".into()))
}

/// Auto's ring family for one component: offsets of its area, core
/// first, each loop whole.
fn core_first(area: &Region, stepover: f64, climb: bool, cap: u32, out: &mut Vec<Piece>) -> Result<(), GenError> {
    let mut rings: Vec<Vec<Vec<Point2>>> = Vec::new();
    for k in 0..=cap {
        let ring = if k == 0 { area.clone() } else { area.offset(-f64::from(k) * stepover)? };
        if ring.is_empty() {
            for loops in rings.into_iter().rev() {
                let id = next_ring(out);
                out.extend(loops.into_iter().map(|pts| Piece { ring: id, pts, closed: true }));
            }
            return Ok(());
        }
        rings.push(
            ring.loops()
                .iter()
                .map(|l| if climb { l.points().to_vec() } else { l.points().iter().rev().copied().collect() })
                .collect(),
        );
    }
    Err(GenError::Invalid("waterline rings did not converge; check the stepover".into()))
}

/// The parts of the closed polyline `pts` that lie inside `area`, as
/// pieces in `pts`' order: split at every crossing with `area`'s loops.
fn clip_closed(pts: &[Point2], area: &Region) -> Result<Vec<(Vec<Point2>, bool)>, GenError> {
    let n = pts.len();
    if n < 2 {
        return Ok(Vec::new());
    }
    let cutters: Vec<&Loop> = area.loops().iter().collect();
    // Every point of the ring with the split points inserted, and the
    // indices (in that list) where the ring crosses a far loop.
    let mut walkp: Vec<Point2> = Vec::with_capacity(n + 8);
    let mut cuts: Vec<usize> = Vec::new();
    for i in 0..n {
        let (a, b) = (pts[i], pts[(i + 1) % n]);
        walkp.push(a);
        let mut ts: Vec<f64> = Vec::new();
        for c in &cutters {
            crossings(a, b, c.points(), &mut ts);
        }
        ts.sort_by(f64::total_cmp);
        for t in ts {
            cuts.push(walkp.len());
            walkp.push(Point2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t));
        }
    }
    let mid = |p: Point2, q: Point2| Point2::new(0.5 * (p.x + q.x), 0.5 * (p.y + q.y));
    if cuts.is_empty() {
        // Whole ring in or out: judge it by its first edge's midpoint.
        return Ok(if area.contains(mid(pts[0], pts[1 % n])) { vec![(pts.to_vec(), true)] } else { Vec::new() });
    }
    let m = walkp.len();
    let mut out = Vec::new();
    for (ci, &s) in cuts.iter().enumerate() {
        let e = cuts[(ci + 1) % cuts.len()];
        let span = if e > s { e - s } else { e + m - s };
        let piece: Vec<Point2> = (0..=span).map(|k| walkp[(s + k) % m]).collect();
        let len: f64 = piece.windows(2).map(|w| dist(w[0], w[1])).sum();
        if len < 1e-6 || piece.len() < 2 {
            continue;
        }
        // The piece's inside test: the midpoint of its longest edge, away
        // from the split points where containment is ambiguous.
        let Some(w) = piece.windows(2).max_by(|x, y| dist(x[0], x[1]).total_cmp(&dist(y[0], y[1]))) else {
            continue;
        };
        if area.contains(mid(w[0], w[1])) {
            // The ends stand on the area's boundary; every vertex between
            // must be inside it, or a crossing was missed and the piece
            // would leave the area - a gouge. Refuse, never emit it.
            if piece[1..piece.len() - 1].iter().any(|&q| !area.contains(q)) {
                return Err(GenError::Invalid("a start-side ring leaves the cutter-center area".into()));
            }
            if let Some(p) = trimmed(&piece, END_TRIM) {
                out.push((p, false));
            }
        }
    }
    Ok(out)
}

/// How far (mm) an open piece stops short of the far wall at each end.
/// A split point stands on the wall loop; after the micron lattice it
/// could stand a hair across the wall ring, and the verifier would read
/// a crossing. The wall ring cuts the last 50 um.
const END_TRIM: f64 = 0.05;

/// `pts` with `d` mm cut off each end along the polyline; `None` when
/// the polyline is not longer than `4 d`.
fn trimmed(pts: &[Point2], d: f64) -> Option<Vec<Point2>> {
    let len: f64 = pts.windows(2).map(|w| dist(w[0], w[1])).sum();
    if len <= 4.0 * d {
        return None;
    }
    let cut_front = |pts: &[Point2]| -> Vec<Point2> {
        let mut left = d;
        for (i, w) in pts.windows(2).enumerate() {
            let l = dist(w[0], w[1]);
            if l > left {
                let t = left / l;
                let mut out = vec![Point2::new(w[0].x + (w[1].x - w[0].x) * t, w[0].y + (w[1].y - w[0].y) * t)];
                out.extend_from_slice(&pts[i + 1..]);
                return out;
            }
            left -= l;
        }
        pts.to_vec()
    };
    let mut v = cut_front(pts);
    v.reverse();
    let mut v = cut_front(&v);
    v.reverse();
    Some(v)
}

/// The crossings of segment `a -> b` with the closed polyline `l`, as
/// parameters along `a -> b` in [0, 1).
fn crossings(a: Point2, b: Point2, l: &[Point2], ts: &mut Vec<f64>) {
    let (minx, maxx) = (a.x.min(b.x), a.x.max(b.x));
    let (miny, maxy) = (a.y.min(b.y), a.y.max(b.y));
    let n = l.len();
    for i in 0..n {
        let (c, d) = (l[i], l[(i + 1) % n]);
        if c.x.max(d.x) < minx || c.x.min(d.x) > maxx || c.y.max(d.y) < miny || c.y.min(d.y) > maxy {
            continue;
        }
        let (rx, ry) = (b.x - a.x, b.y - a.y);
        let (sx, sy) = (d.x - c.x, d.y - c.y);
        let den = rx * sy - ry * sx;
        if den.abs() < 1e-15 {
            continue;
        }
        let t = ((c.x - a.x) * sy - (c.y - a.y) * sx) / den;
        let u = ((c.x - a.x) * ry - (c.y - a.y) * rx) / den;
        // Half-open on both segments, so a crossing at a shared vertex
        // counts once (the next ring edge or loop edge owns its end).
        if (0.0..1.0).contains(&t) && (0.0..1.0).contains(&u) {
            ts.push(t);
        }
    }
}

