//! Offset rings as one outward spiral (#3933): the rings of the 2D
//! Pocket's Offset rings strategy, spaced by the engagement law
//! (`offset_law`), are no longer cut one by one and joined by a step
//! across - each ring morphs into the next one out in one revolution,
//! so the step out is the law's from the core to the wall, and the wall
//! ring is cut last as one whole lap. Provenance: contour-parallel offsets of the
//! cutter-centre area (Held, "On the Computational Geometry of Pocket
//! Machining", LNCS 500, 1991); consecutive offsets interpolated into
//! one spiral, each revolution blending one ring into the next (Held &
//! Spielberger 2009, "A smooth spiral tool path for high speed machining
//! of 2D pocket-shaped objects", Computer-Aided Design 41(7)) - the HSM
//! planner's own morph (`adaptive::spiral`); the contact read on the
//! planner's stock (Stori & Wright 2000). The join rules, the guards and
//! the notes are ours. No vendor material consulted.

use crate::geo2d::{Point2, Region};

use super::adaptive::{read_rim, ring_morph, worst_load};
use super::face::GenError;
use super::level_rings::unswept;
use super::ring::dist;
use super::offset_law::law_depths;
use super::waterscan::{chains_at, chord_crosses_path, waterline_passes};

/// The margin (fraction of the optimal load) the read may pass before
/// the plan names its worst contact: the HSM plan's own (`PLAN_TOL`).
const READ_TOL: f64 = super::pocket_hsm::PLAN_TOL;

/// The most area (mm2) of the cutter-centre area the spiral may leave
/// unswept; past it the area takes the rings one by one (`level_rings`'s
/// completeness law).
const LEFT_TOL: f64 = 0.1;

/// The Offset rings plan of one planar area: its passes and the notes it
/// owes the op page. `step` is the optimal load's width (ae), `tool_r`
/// the tool radius. Where two rings cannot be morphed honestly (not nested, a
/// morph that would cross a ring or itself), the step across between
/// them stays as the ring plan had it; a contact past the load anywhere
/// is read on the planner's stock and named, with where.
pub(super) fn offset_spiral(
    center: &Region,
    step: f64,
    climb: bool,
    tool_r: f64,
) -> Result<(Vec<Vec<Point2>>, Vec<String>), GenError> {
    // The rings: offsets of the area, spaced by the engagement law from
    // the core out (`offset_law`), never wider than the load's width.
    let depths = law_depths(center, step, (tool_r, read_rim(tool_r)))?;
    let cap = u32::try_from(depths.len()).unwrap_or(u32::MAX);
    let (loops, chains) = chains_at(center, &|k| depths.get(k as usize).copied(), cap, climb)?;
    // The morph blends vertex by vertex: a ring's long edges are split
    // so a blended revolution follows its rings, not chords across them.
    let fine = DENSE_PER_STEP * step;
    let loops: Vec<(u32, Vec<Point2>)> = loops.into_iter().map(|(k, l)| (k, densify(&l, fine))).collect();
    let mut passes: Vec<Vec<Point2>> = Vec::new();
    let mut read_from: Vec<usize> = Vec::new();
    for chain in &chains {
        let (pass, from) = spiral_chain(&loops, chain);
        if pass.len() >= 2 {
            passes.push(pass);
            read_from.push(from);
        }
    }
    // The completeness guard: a spiral that leaves part of the area
    // standing is never posted; the rings one by one are.
    // Read on the walk thinned within THIN_TOL, swept that much narrower:
    // what it proves swept, the walk sweeps too.
    let thin: Vec<Vec<Point2>> = passes.iter().map(|p| thinned(p, THIN_TOL)).collect();
    let left = unswept(center, &thin, tool_r - THIN_TOL)?;
    if left > LEFT_TOL {
        return Ok((
            waterline_passes(center, step, climb)?,
            vec![format!(
                "offset rings: the spiral left {left:.2} mm2 of this area uncut - the rings are cut one by one instead, each step out at up to full width"
            )],
        ));
    }
    let mut notes = Vec::new();
    // The core ring slots at full width, as the entry's own cleanup; a
    // core longer than one orbit of the tool is named.
    let core = passes
        .iter()
        .zip(&read_from)
        .filter(|(_, from)| **from > 0)
        .map(|(p, from)| (p[..=*from].windows(2).map(|w| dist(w[0], w[1])).sum::<f64>(), p[0]))
        .fold((0.0f64, Point2::default()), |a, b| if b.0 > a.0 { b } else { a });
    if core.0 > std::f64::consts::TAU * tool_r {
        notes.push(format!(
            "offset rings: the core ring cuts a {:.1} mm slot at full width from ({:.2}, {:.2}) before the spiral - Adaptive opens the core with a spiral",
            core.0, core.1.x, core.1.y
        ));
    }
    let (worst, at) = worst_load(&center.offset(tool_r)?, &passes, &read_from, tool_r)?;
    let load = step / (2.0 * tool_r);
    if worst > load * (1.0 + READ_TOL) {
        notes.push(format!(
            "offset rings: worst load {:.1}% of D at ({:.2}, {:.2}) exceeds the {:.1}% optimal load by {:.0}% - Adaptive holds the load there",
            worst * 100.0,
            at.x,
            at.y,
            load * 100.0,
            (worst / load - 1.0) * 100.0
        ));
    }
    Ok((passes, notes))
}

/// A ring's edges are split to at most this share of the step before
/// the morph (the seams stay where the chain's chords were proved).
const DENSE_PER_STEP: f64 = 0.25;

/// The ring `pts` (open, closing edge implied) with every edge longer
/// than `most` split evenly; its points and their order kept.
fn densify(pts: &[Point2], most: f64) -> Vec<Point2> {
    let n = pts.len();
    if n < 2 || !(most.is_finite() && most > 0.0) {
        return pts.to_vec();
    }
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let (a, b) = (pts[i], pts[(i + 1) % n]);
        out.push(a);
        let parts = (dist(a, b) / most).ceil().clamp(1.0, 1e5) as usize;
        for j in 1..parts {
            let t = j as f64 / parts as f64;
            out.push(Point2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t));
        }
    }
    out
}

/// The completeness guard reads the walk thinned within this (mm).
const THIN_TOL: f64 = 2e-3;

/// `pts` with the points dropped whose chord neighbours stay within
/// `tol` of them (first and last kept; a run is at most 64 points).
fn thinned(pts: &[Point2], tol: f64) -> Vec<Point2> {
    let n = pts.len();
    if n <= 2 {
        return pts.to_vec();
    }
    let mut out = vec![pts[0]];
    let mut anchor = 0usize;
    let mut i = 2usize;
    while i < n {
        let (a, b) = (pts[anchor], pts[i]);
        let fits = i - anchor <= 64 && (anchor + 1..i).all(|m| seg_dist(pts[m], a, b) <= tol);
        if !fits {
            anchor = i - 1;
            out.push(pts[anchor]);
        }
        i += 1;
    }
    out.push(pts[n - 1]);
    out
}

/// Distance from `p` to the segment `a -> b`.
fn seg_dist(p: Point2, a: Point2, b: Point2) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let dd = dx * dx + dy * dy;
    let t = if dd > 0.0 { (((p.x - a.x) * dx + (p.y - a.y) * dy) / dd).clamp(0.0, 1.0) } else { 0.0 };
    (p.x - a.x - dx * t).hypot(p.y - a.y - dy * t)
}

/// `pts` closed: the first point repeated last.
fn closed(pts: &[Point2]) -> Vec<Point2> {
    let mut out = pts.to_vec();
    if let Some(&f) = pts.first() {
        out.push(f);
    }
    out
}

/// `pts` (open) walked closed from index `k`: `pts[k..]`, `pts[..=k]`.
fn lap_from(pts: &[Point2], k: usize) -> Vec<Point2> {
    let n = pts.len();
    (0..=n).map(|j| pts[(k + j) % n]).collect()
}

/// The vertex of `pts` (open) nearest `p`, the first one on a tie: the
/// seam the morph turns its target ring to (`adaptive::front::rotate_to`).
fn nearest(pts: &[Point2], p: Point2) -> usize {
    (0..pts.len()).min_by(|&i, &j| dist(pts[i], p).total_cmp(&dist(pts[j], p))).unwrap_or(0)
}

/// Even-odd point-in-polygon test of `p` against the closed ring `pts`.
fn inside(p: Point2, pts: &[Point2]) -> bool {
    let n = pts.len();
    let mut c = false;
    for i in 0..n {
        let (a, b) = (pts[i], pts[(i + 1) % n]);
        if (a.y > p.y) != (b.y > p.y) && p.x < a.x + (p.y - a.y) * (b.x - a.x) / (b.y - a.y) {
            c = !c;
        }
    }
    c
}

/// Twice the signed area of the ring `pts` (positive: counter-clockwise).
fn signed(pts: &[Point2]) -> f64 {
    let n = pts.len();
    (0..n).map(|i| pts[i].x * pts[(i + 1) % n].y - pts[(i + 1) % n].x * pts[i].y).sum()
}

/// Do loops `i` and `j` bound one band: adjacent offsets, one inside the
/// other, travelled the same way?
fn one_band(loops: &[(u32, Vec<Point2>)], i: usize, j: usize) -> bool {
    let ((ki, a), (kj, b)) = (&loops[i], &loops[j]);
    if ki.abs_diff(*kj) != 1 || a.len() < 3 || b.len() < 3 {
        return false;
    }
    if (signed(a) > 0.0) != (signed(b) > 0.0) {
        return false;
    }
    // One lies inside the other: the deeper offset inside the shallower
    // round a pocket's core, the shallower inside the deeper round an
    // island.
    inside(a[0], b) || inside(b[0], a)
}

/// The morph `rev` from loop `i` to loop `j` stays in their band: no
/// segment of it properly crosses a ring of either offset (so it never
/// leaves the band between them, which lies in the cutter-centre area)
/// or another segment of itself (the verifier's crossing law).
fn morph_stays(loops: &[(u32, Vec<Point2>)], (ki, kj): (u32, u32), rev: &[Point2]) -> bool {
    if rev.len() < 2 {
        return false;
    }
    let mut segs: Vec<(Point2, Point2)> = Vec::new();
    for (_, l) in loops.iter().filter(|(k, _)| *k == ki || *k == kj) {
        segs.extend((0..l.len()).map(|i| (l[i], l[(i + 1) % l.len()])));
    }
    let own: Vec<(Point2, Point2)> = rev.windows(2).map(|s| (s[0], s[1])).collect();
    let (walls, mine) = (SegGrid::new(&segs), SegGrid::new(&own));
    own.iter().all(|&(a, b)| !walls.crossed(a, b) && !mine.crossed(a, b))
}

/// Segments bucketed on a square grid by their bounding boxes, so a
/// crossing test reads only the segments near the one asked about (the
/// same strict test as `chord_crosses_path`, the verifier's law).
struct SegGrid<'a> {
    segs: &'a [(Point2, Point2)],
    cell: f64,
    buckets: std::collections::HashMap<(i64, i64), Vec<usize>>,
}

impl<'a> SegGrid<'a> {
    fn new(segs: &'a [(Point2, Point2)]) -> Self {
        let mean = segs.iter().map(|&(a, b)| dist(a, b)).sum::<f64>() / segs.len().max(1) as f64;
        let cell = (4.0 * mean).max(1e-3);
        let mut g = SegGrid { segs, cell, buckets: std::collections::HashMap::new() };
        for (i, &(a, b)) in segs.iter().enumerate() {
            for key in g.cells(a, b) {
                g.buckets.entry(key).or_default().push(i);
            }
        }
        g
    }

    /// The cells the box of `a -> b` touches (capped: a segment longer
    /// than the cap reads every bucket instead).
    fn cells(&self, a: Point2, b: Point2) -> Vec<(i64, i64)> {
        let k = |v: f64| (v / self.cell).floor().clamp(-1e12, 1e12) as i64;
        let (x0, x1) = (k(a.x.min(b.x)), k(a.x.max(b.x)));
        let (y0, y1) = (k(a.y.min(b.y)), k(a.y.max(b.y)));
        if (x1 - x0 + 1).saturating_mul(y1 - y0 + 1) > 4096 {
            return self.buckets.keys().copied().collect();
        }
        (x0..=x1).flat_map(|x| (y0..=y1).map(move |y| (x, y))).collect()
    }

    /// Does `a -> b` properly cross a segment of the grid?
    fn crossed(&self, a: Point2, b: Point2) -> bool {
        self.cells(a, b).iter().filter_map(|c| self.buckets.get(c)).flatten().any(|&i| {
            let (c, d) = self.segs[i];
            chord_crosses_path(a, b, &[c, d], false)
        })
    }
}

/// One chain of rings walked as a spiral: the first ring closed, every
/// later one reached by a morph out of the last (or, where no morph
/// stays in its band, by the chain's own proved chord from seam to
/// seam), the last ring closed as a whole lap. Returns the walk and the
/// index from which it is read for load: past the first ring when that
/// ring is a core (no deeper ring inside it - the entry's own bore
/// cleanup), else from the start.
fn spiral_chain(loops: &[(u32, Vec<Point2>)], chain: &[usize]) -> (Vec<Point2>, usize) {
    let mut out: Vec<Point2> = Vec::new();
    let Some(&first) = chain.first() else { return (out, 0) };
    let ring0 = &loops[first].1;
    if ring0.is_empty() {
        return (out, 0);
    }
    out.extend(lap_from(ring0, 0));
    let core = !loops.iter().any(|(k, l)| *k > loops[first].0 && l.first().is_some_and(|&p| inside(p, ring0)));
    let read_from = if core { out.len() - 1 } else { 0 };
    // The ring the tool is on, the vertex it stands at, and whether that
    // ring has been walked whole yet.
    let (mut cur, mut k, mut walked) = (first, 0usize, true);
    for &next in chain.iter().skip(1) {
        let (a, b) = (&loops[cur].1, &loops[next].1);
        if b.is_empty() {
            continue;
        }
        if one_band(loops, cur, next) {
            let from = lap_from(a, k);
            let kb = nearest(b, a[k]);
            let to = lap_from(b, kb);
            let rev = ring_morph(&from, &to);
            if rev.first().is_some_and(|&p| dist(p, a[k]) <= 1e-9) && morph_stays(loops, (loops[cur].0, loops[next].0), &rev) {
                out.extend(rev.iter().skip(1));
                (cur, k, walked) = (next, kb, false);
                continue;
            }
        }
        // No morph: the ring is finished whole and left at its own seam,
        // where the chain's chord to the next ring was proved.
        if !walked {
            out.extend(lap_from(a, k).into_iter().skip(1));
            if k != 0 {
                out.extend(a[k + 1..].iter().copied());
                out.push(a[0]);
            }
        }
        out.extend(closed(b));
        (cur, k, walked) = (next, 0, true);
    }
    // The wall-clearing ring: the last ring of the chain, one whole lap.
    if !walked {
        out.extend(lap_from(&loops[cur].1, k).into_iter().skip(1));
    }
    (out, read_from)
}

#[cfg(test)]
#[path = "offset_spiral_tests.rs"]
mod tests;
