//! Polyline clipping and keep-out validation helpers for 2D Contour "Do not touch" (#3344).
//! Provenance: clean-room 2D segment-polygon clipping and sampled arc/chord verification over `geo2d::Region`; no vendor material consulted.

use std::f64::consts::TAU;

use camcraft_sim::verify::{AVOID_VERIFY_TOL, AvoidSpec, Warning};

use crate::cl::{MoveKind, Toolpath};
use crate::geo2d::{Point2, Region};

use super::arcs::{Seg, fit_arcs_sided};
use super::contour_entry::flatten;
use super::face::GenError;
use super::fillet::Legal;
use super::ring::dist;
use super::ContourParams;

/// Epsilon for deduplicating clip parameters along a segment.
const CLIP_PARAM_EPS: f64 = 1e-8;
/// Minimum kept polyline arc length (mm) to emit after clipping.
const MIN_PIECE_LEN: f64 = 1e-4;
/// Sampling interval (mm) when validating arcs against a keep-out region.
const CHECK_STEP: f64 = 0.02;
/// How far (mm) a cut end steps back from the keep-out boundary. A clip
/// end lies on that boundary, and `Region::contains` counts the boundary
/// as inside, so an end left there reads as blocked.
const END_TRIM: f64 = 1e-3;
const MAX_SAMPLES: usize = 4096;

/// Result of clipping a closed loop or open polyline against the outside of a keep-out region.
pub(super) enum Clip {
    Unclipped,
    Pieces(Vec<Vec<Point2>>),
    AllBlocked,
}

fn lerp(a: Point2, b: Point2, t: f64) -> Point2 {
    Point2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
}

fn poly_len(pts: &[Point2]) -> f64 {
    pts.windows(2).map(|w| dist(w[0], w[1])).sum()
}

fn push_intersections(a: Point2, b: Point2, c: Point2, d: Point2, ts: &mut Vec<f64>) {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let (ex, ey) = (d.x - c.x, d.y - c.y);
    let denom = dx * ey - dy * ex;
    let ab_len2 = dx * dx + dy * dy;
    if ab_len2 <= 1e-18 {
        return;
    }
    let (cx, cy) = (c.x - a.x, c.y - a.y);
    if denom.abs() > 1e-12 * ab_len2.sqrt() * ex.hypot(ey).max(1e-9) {
        let t = (cx * ey - cy * ex) / denom;
        let u = (cx * dy - cy * dx) / denom;
        if t > CLIP_PARAM_EPS && t < 1.0 - CLIP_PARAM_EPS && (-1e-7..=1.0 + 1e-7).contains(&u) {
            ts.push(t);
        }
        return;
    }
    // Nearly collinear: split at projections of c and d if on the same line.
    let perp = (cx * dy - cy * dx).abs() / ab_len2.sqrt();
    if perp <= 1e-7 {
        for q in [c, d] {
            let t = ((q.x - a.x) * dx + (q.y - a.y) * dy) / ab_len2;
            if t > CLIP_PARAM_EPS && t < 1.0 - CLIP_PARAM_EPS {
                ts.push(t);
            }
        }
    }
}

/// Clip `pts` to maximal contiguous pieces whose interior lies outside `ko`.
pub(super) fn clip_polyline(pts: &[Point2], ko: &Region, closed: bool) -> Clip {
    if pts.len() < 2 || ko.is_empty() {
        return Clip::Unclipped;
    }
    let mut segs: Vec<(Point2, Point2)> = pts.windows(2).map(|w| (w[0], w[1])).collect();
    if closed && let (Some(&first), Some(&last)) = (pts.first(), pts.last()) && dist(first, last) > 1e-9 {
        segs.push((last, first));
    }

    let mut pieces: Vec<Vec<Point2>> = Vec::new();
    let mut cur: Vec<Point2> = Vec::new();
    let mut all_kept = true;

    for (a, b) in segs {
        if dist(a, b) <= 1e-11 {
            continue;
        }
        let mut ts = vec![0.0, 1.0];
        for loop_ in ko.loops() {
            let lp = loop_.points();
            let n = lp.len();
            for i in 0..n {
                push_intersections(a, b, lp[i], lp[(i + 1) % n], &mut ts);
            }
        }
        ts.sort_by(f64::total_cmp);
        ts.dedup_by(|x, y| (*x - *y).abs() <= CLIP_PARAM_EPS);

        for w in ts.windows(2) {
            let (p0, p1) = (lerp(a, b, w[0]), lerp(a, b, w[1]));
            if dist(p0, p1) <= 1e-9 {
                continue;
            }
            let mid = Point2::new((p0.x + p1.x) * 0.5, (p0.y + p1.y) * 0.5);
            if ko.contains(mid) {
                all_kept = false;
                if cur.len() >= 2 {
                    pieces.push(std::mem::take(&mut cur));
                } else {
                    cur.clear();
                }
            } else if cur.is_empty() {
                cur.push(p0);
                cur.push(p1);
            } else if let Some(&last) = cur.last()
                && dist(last, p0) <= 1e-6
            {
                cur.push(p1);
            } else {
                if cur.len() >= 2 {
                    pieces.push(std::mem::take(&mut cur));
                }
                cur.clear();
                cur.push(p0);
                cur.push(p1);
            }
        }
    }
    if cur.len() >= 2 {
        pieces.push(cur);
    }

    if all_kept {
        return Clip::Unclipped;
    }
    // Stitch a closed loop whose safe run wraps past vertex 0.
    if closed
        && pieces.len() >= 2
        && let (Some(first_pt), Some(last_pt)) = (
            pieces.first().and_then(|v| v.first()).copied(),
            pieces.last().and_then(|v| v.last()).copied(),
        )
        && dist(first_pt, last_pt) <= 1e-6
    {
        let first = pieces.remove(0);
        if let Some(last_piece) = pieces.last_mut() {
            last_piece.extend_from_slice(&first[1..]);
        }
    }
    for v in &mut pieces {
        trim_ends(v, ko);
    }
    pieces.retain(|v| v.len() >= 2 && poly_len(v) > MIN_PIECE_LEN);
    if pieces.is_empty() {
        Clip::AllBlocked
    } else {
        Clip::Pieces(pieces)
    }
}

/// Step each end of `v` that touches `ko` back along its own segment by
/// [`END_TRIM`] (never past the segment's other point).
fn trim_ends(v: &mut [Point2], ko: &Region) {
    let n = v.len();
    if n < 2 || poly_len(v) <= 2.0 * END_TRIM {
        return;
    }
    let step = |end: Point2, next: Point2| {
        let d = dist(end, next);
        if d <= END_TRIM { end } else { lerp(end, next, END_TRIM / d) }
    };
    if ko.contains(v[0]) {
        v[0] = step(v[0], v[1]);
    }
    if ko.contains(v[n - 1]) {
        v[n - 1] = step(v[n - 1], v[n - 2]);
    }
}

/// Combined legal regions `(for_cut, for_entry)` keeping outside `ko_cut` and `ko_entry`.
pub(super) fn legal_regions(
    base: &Region,
    inside: bool,
    ko_cut: &Region,
    ko_entry: &Region,
) -> Result<(Region, Region), GenError> {
    if inside {
        let cut = if ko_cut.is_empty() { base.clone() } else { base.difference(ko_cut)? };
        let entry = if ko_entry.is_empty() { cut.clone() } else { cut.difference(ko_entry)? };
        Ok((cut, entry))
    } else {
        let cut = if ko_cut.is_empty() { base.clone() } else { base.union(ko_cut)? };
        let entry = if ko_entry.is_empty() { cut.clone() } else { cut.union(ko_entry)? };
        Ok((cut, entry))
    }
}

/// True when every sampled point on `(from, segs)` stays outside `ko`.
pub(super) fn segs_clear_of(from: Point2, segs: &[Seg], ko: &Region) -> bool {
    if ko.is_empty() {
        return true;
    }
    let pt_clear = |q: Point2| q.x.is_finite() && q.y.is_finite() && !ko.contains(q);
    if !pt_clear(from) {
        return false;
    }
    let mut at = from;
    for seg in segs {
        match seg {
            Seg::Line(qs) => {
                for &q in qs {
                    let n = ((dist(at, q) / CHECK_STEP).ceil() as usize).clamp(1, MAX_SAMPLES);
                    for k in 1..=n {
                        if !pt_clear(lerp(at, q, k as f64 / n as f64)) {
                            return false;
                        }
                    }
                    at = q;
                }
            }
            Seg::Arc { center, ccw, to } => {
                let rad = dist(at, *center);
                let a0 = (at.y - center.y).atan2(at.x - center.x);
                let a1 = (to.y - center.y).atan2(to.x - center.x);
                let sweep = if *ccw { a1 - a0 } else { a0 - a1 }.rem_euclid(TAU);
                if !(rad.is_finite() && sweep.is_finite()) {
                    return false;
                }
                let n = ((sweep * rad / CHECK_STEP).ceil() as usize).clamp(8, MAX_SAMPLES);
                let dir = if *ccw { 1.0 } else { -1.0 };
                for k in 1..=n {
                    let a = a0 + dir * sweep * k as f64 / n as f64;
                    if !pt_clear(Point2::new(center.x + rad * a.cos(), center.y + rad * a.sin())) {
                        return false;
                    }
                }
                at = *to;
            }
        }
    }
    true
}

/// Fit arcs on `poly` and keep only those that stay outside `ko_cut` (else linear).
pub(super) fn safe_walk(
    poly: &[Point2],
    p: &ContourParams,
    legal_cut: &Legal,
    ko_cut: &Region,
) -> Vec<Seg> {
    let walk = fit_arcs_sided(poly, p.tolerance, legal_cut.air_left);
    if segs_clear_of(poly[0], &walk, ko_cut) {
        walk
    } else if poly.len() > 1 {
        vec![Seg::Line(poly[1..].to_vec())]
    } else {
        Vec::new()
    }
}

/// True when `(from, segs)` properly crosses any chord in `prior` at a non-shared point.
pub(super) fn segs_cross_prior(from: Point2, segs: &[Seg], prior: &[Vec<Point2>]) -> bool {
    if prior.is_empty() || segs.is_empty() {
        return false;
    }
    let pa = flatten(from, segs);
    let orient = |a: Point2, b: Point2, c: Point2| (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    let near = |u: Point2, v: Point2| dist(u, v) <= 1e-7;
    pa.windows(2).any(|s| {
        prior.iter().any(|pl| {
            pl.windows(2).any(|t| {
                if near(s[0], t[0]) || near(s[0], t[1]) || near(s[1], t[0]) || near(s[1], t[1]) {
                    return false;
                }
                let (d1, d2) = (orient(s[0], s[1], t[0]), orient(s[0], s[1], t[1]));
                let (d3, d4) = (orient(t[0], t[1], s[0]), orient(t[0], t[1], s[1]));
                d1 * d2 < 0.0 && d3 * d4 < 0.0
            })
        })
    })
}

/// True when `path` contains at least one Feed move of positive XY/Z travel.
pub(super) fn has_cut_moves(path: &Toolpath) -> bool {
    path.moves
        .iter()
        .any(|m| m.kind == MoveKind::Feed && (m.to.x != m.from.x || m.to.y != m.from.y || m.to.z != m.from.z))
}

/// Elevate rapids and run independent full-tool verification, refusing by item name if any hit persists.
pub(super) fn finalize_avoid(mut path: Toolpath, avoid: &AvoidSpec) -> Result<Toolpath, GenError> {
    if !has_cut_moves(&path) {
        return Err(GenError::Invalid(format!(
            "cannot cut clear of protected 'Do not touch' {}: no safe cutting room remains",
            avoid.names_list()
        )));
    }
    let _ = camcraft_sim::verify::elevate_rapid_links(&mut path, avoid, None, camcraft_sim::verify::AVOID_PAD);
    match avoid.hits_named(&path, AVOID_VERIFY_TOL) {
        Ok(hits) if hits.is_empty() => {}
        Ok(hits) => {
            if let Some((Warning::AvoidHit { part, short_mm, .. }, label)) = hits.first() {
                return Err(GenError::Invalid(format!(
                    "cannot cut clear of protected 'Do not touch' {label} ({part:?}, {short_mm:.3} mm short of {:.2} mm clearance)",
                    avoid.clearance
                ))
                .with_ghost(&path));
            }
        }
        Err(e) => return Err(GenError::Invalid(e).with_ghost(&path)),
    }
    if let Some(why) = crate::approach::refusal(&path) {
        return Err(GenError::Invalid(why).with_ghost(&path));
    }
    if !path.all_finite() {
        return Err(GenError::Invalid("path left the finite horizon".into()));
    }
    Ok(path)
}
