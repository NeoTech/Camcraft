//! Trochoidal load control (K3, #4026): each full-width stretch that the
//! guide names (a `FullWidth` mark: a slot, a neck) is replaced by
//! trochoidal loops that walk the stretch itself (#3932: they follow the
//! contour, they do not slot to a wall). Each loop touches the stretch at
//! its start and its end and lies on the side of the stretch with the more
//! room; its radius is the room that is free there, at most half the loop
//! width, so that the loop stays in the cutter-centre area. The loops
//! advance one step a loop, and the step is at most the load width of the
//! limit. Where no loop fits, the stretch stays and its cut is named. All
//! curves, the clean-up rings too, are read with the one law (#3849).
//! Provenance: trochoidal milling with the step per loop as the radial
//! engagement of the front arc (Otkur & Lazoglu 2007, Int. J. Mach. Tools
//! Manuf. 47; Rauch, Duc & Hascoet 2009, Int. J. Mach. Tools Manuf. 49);
//! our own design record `docs/decisions/4024-path-kernel-design.md`
//! "Stage 3".

use super::law::{Read, Stock, read, samples};
use super::stage::{Outcome, finish, on_span};
use super::{Controlled, OrderedSet, Trochoid};
use crate::frame::PlanePoint;
use crate::guide::{Curve, Mark, MarkKind, Span};
use crate::region::PlanarRegion;
use crate::verdict::{Refusal, Stage, Verdict};
use crate::{Point2, Region, in_area, to_loops};

/// Over-limit reason: no loop fits beside the full-width stretch.
pub const NO_ROOM: &str = "no room for trochoidal loops here: this full-width stretch stays";
/// Over-limit reason: the loops read above the limit.
pub const LOOPS_OVER: &str = "the trochoidal loops read above the load limit here";
/// Over-limit reason: trochoidal control changes only full-width stretches.
pub const NOT_MARKED: &str = "trochoidal control changes only the full-width stretches: this cut stays above the load limit";
/// Refusal reason: a loop point outside the cutter-centre area.
pub const LOOP_LEAVES: &str = "a trochoidal loop leaves the cutter-centre area";

/// The points of one loop.
const PER_LOOP: usize = 24;
/// The smallest loop radius: this share of the tool radius.
const MIN_LOOP_SHARE: f64 = 0.05;
/// The halvings of the room search.
const BISECT: usize = 14;
/// The most loops of one stretch.
const MAX_LOOPS: usize = 20_000;

/// The largest radius up to `most` of a disc that touches `q` from the
/// side `n` and lies in `center`.
fn room(center: &Region, q: Point2, n: Point2, most: f64) -> f64 {
    let fits = |rho: f64| {
        let c = Point2::new(q.x + rho * n.x, q.y + rho * n.y);
        center.contains(c) && to_loops(c, center.loops()) >= rho * (1.0 - 1e-3)
    };
    if fits(most) {
        return most;
    }
    let (mut lo, mut hi) = (0.0, most);
    for _ in 0..BISECT {
        let m = 0.5 * (lo + hi);
        if fits(m) { lo = m } else { hi = m }
    }
    lo
}

/// The places of a run at `n + 1` even steps along it, with directions.
fn resample(line: &[Point2], n: usize) -> Vec<(Point2, Point2)> {
    let seg: Vec<f64> = line.windows(2).map(|w| (w[1].x - w[0].x).hypot(w[1].y - w[0].y)).collect();
    let total: f64 = seg.iter().sum();
    let mut out = Vec::with_capacity(n + 1);
    let (mut j, mut acc) = (0usize, 0.0);
    for k in 0..=n {
        let s = total * k as f64 / n.max(1) as f64;
        while j + 1 < seg.len() && acc + seg[j] < s {
            acc += seg[j];
            j += 1;
        }
        let (Some(a), Some(b), Some(&l)) = (line.get(j), line.get(j + 1), seg.get(j)) else { break };
        let t = if l > 0.0 { ((s - acc) / l).clamp(0.0, 1.0) } else { 0.0 };
        let d = if l > 0.0 { Point2::new((b.x - a.x) / l, (b.y - a.y) / l) } else { Point2::new(1.0, 0.0) };
        out.push((Point2::new(a.x + t * (b.x - a.x), a.y + t * (b.y - a.y)), d));
    }
    out
}

/// The loops along one run of a stretch: `n` loops of `PER_LOOP` points,
/// and true when one of them has a radius. A loop is
/// `q + rho n (1 - cos t) + rho d sin t`: it leaves the stretch and comes
/// back to it; `rho` is the room on the side with more room (0 below `least`).
fn loops(center: &Region, run: &[Point2], pitch: f64, (most, least): (f64, f64)) -> Result<(Vec<Point2>, bool), Refusal> {
    let len: f64 = run.windows(2).map(|w| (w[1].x - w[0].x).hypot(w[1].y - w[0].y)).sum();
    let n = ((len / pitch).ceil().max(1.0) as usize).min(MAX_LOOPS);
    let spine = resample(run, n * PER_LOOP);
    let left = |d: Point2| Point2::new(-d.y, d.x);
    let (mut out, mut made) = (Vec::with_capacity(spine.len()), false);
    for part in spine.get(..n * PER_LOOP).unwrap_or(&[]).chunks(PER_LOOP) {
        let side = |sgn: f64| part.iter().map(|(q, d)| room(center, *q, Point2::new(sgn * left(*d).x, sgn * left(*d).y), most)).fold(most, f64::min);
        let (a, b) = (side(1.0), side(-1.0));
        let (sgn, rho) = if a >= b { (1.0, a) } else { (-1.0, b) };
        let rho = if rho >= least { rho } else { 0.0 };
        made |= rho > 0.0;
        for (j, (q, d)) in part.iter().enumerate() {
            let t = std::f64::consts::TAU * j as f64 / PER_LOOP as f64;
            let nn = left(*d);
            let (u, v) = (rho * (1.0 - t.cos()) * sgn, rho * t.sin());
            let pt = Point2::new(q.x + u * nn.x + v * d.x, q.y + u * nn.y + v * d.y);
            if !in_area(center, pt) {
                return Err(Refusal::new(Stage::Load, LOOP_LEAVES).at(pt));
            }
            out.push(pt);
        }
    }
    out.extend(spine.last().map(|(q, _)| *q));
    Ok((out, made))
}

/// Curve `c` with each stretch of `marks` replaced by loops, or `None`
/// when no loop fits.
fn change(c: &Curve, marks: &[&Mark], center: &Region, pitch: f64, radii: (f64, f64)) -> Result<Option<Curve>, Refusal> {
    let mut pts: Vec<Point2> = samples(c, pitch / PER_LOOP as f64).into_iter().map(|(q, _)| q).collect();
    let mut on: Vec<bool> = pts.iter().map(|q| marks.iter().any(|m| on_span(m, *q))).collect();
    if c.closed()
        && let Some(off) = on.iter().position(|b| !b)
    {
        pts.rotate_left(off);
        on.rotate_left(off);
    }
    let whole = on.iter().all(|b| *b);
    if whole && c.closed() {
        pts.extend(pts.first().copied());
    }
    let (mut out, mut run, mut any) = (Vec::new(), Vec::new(), false);
    for (q, hit) in pts.iter().zip(&on) {
        if *hit {
            run.push(*q);
            continue;
        }
        if run.len() >= 2 {
            let (lp, made) = loops(center, &run, pitch, radii)?;
            any |= made;
            out.extend(lp);
        } else {
            out.append(&mut run);
        }
        run.clear();
        out.push(*q);
    }
    if run.len() >= 2 {
        let (lp, made) = loops(center, &run, pitch, radii)?;
        any |= made;
        out.extend(lp);
    } else {
        out.extend(run);
    }
    out.dedup_by(|a, b| crate::near(*a, *b));
    if c.closed() && out.len() > 1 && out.first().zip(out.last()).is_some_and(|(a, b)| crate::near(*a, *b)) {
        out.pop();
    }
    if !any {
        return Ok(None);
    }
    // A closed curve keeps its start (the order stage chose it).
    if c.closed()
        && let Some(s) = c.pts().first().map(PlanePoint::at)
    {
        let k = out.iter().enumerate().min_by(|a, b| (a.1.x - s.x).hypot(a.1.y - s.y).total_cmp(&(b.1.x - s.x).hypot(b.1.y - s.y))).map_or(0, |(k, _)| k);
        out.rotate_left(k);
    }
    let z = c.pts().first().map_or(0.0, PlanePoint::depth);
    let pp: Result<Vec<PlanePoint>, Refusal> = out.into_iter().map(|q| PlanePoint::new(q, z)).collect();
    Curve::new(pp?, None, c.closed(), c.k(), c.family()).map(Some)
}

/// Trochoidal control of the ordered set `s` with the stated `limit`.
pub(crate) fn control(s: &OrderedSet, p: &PlanarRegion, stock: &Stock, t: &Trochoid, limit: f64, r: f64) -> Result<Verdict<Controlled>, Refusal> {
    let pitch = t.pitch().min(limit * 2.0 * r);
    let radii = (t.width() / 2.0, MIN_LOOP_SHARE * r);
    let mut curves = s.curves().to_vec();
    let mut changed = vec![false; curves.len()];
    for (i, c) in s.curves().iter().enumerate() {
        let mine: Vec<&Mark> = s.marks().iter().filter(|m| m.curve() == i && m.kind() == MarkKind::FullWidth).collect();
        if mine.is_empty() {
            continue;
        }
        if let Some(n) = change(c, &mine, p.center(), pitch, radii)? {
            curves[i] = n;
            changed[i] = true;
        }
    }
    let reads = read(stock, &curves, r);
    let marks: Vec<Mark> = s.marks().iter().filter(|m| !changed.get(m.curve()).copied().unwrap_or(false) || matches!(m.span(), Span::Whole) && m.kind() != MarkKind::FullWidth).cloned().collect();
    let full = |rd: &Read| s.marks().iter().any(|m| m.curve() == rd.curve && m.kind() == MarkKind::FullWidth && on_span(m, rd.at));
    let reason = |rd: &Read| -> &'static str {
        if changed.get(rd.curve).copied().unwrap_or(false) {
            LOOPS_OVER
        } else if full(rd) {
            NO_ROOM
        } else {
            NOT_MARKED
        }
    };
    let out = Outcome { curves: changed.iter().any(|b| *b).then(|| curves.clone()), marks, reads };
    finish(s, out, limit, "trochoidal", reason)
}
