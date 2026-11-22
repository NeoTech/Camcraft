//! The passes of a start-side ring family (#3316, split from
//! `sided_rings.rs` for the 400-line law): the ring pieces in cutting
//! order, joined by short proved chords, and a lead-in from cut stock
//! where a new pass starts. Provenance: our own linking law on the
//! chord proofs of `waterscan.rs`; no vendor material consulted.

use crate::geo2d::{Point2, Region};

use super::ring::dist;
use super::sided_rings::Piece;
use super::slots::chord_clears;
use super::waterscan::chord_crosses_path;

/// Where a piece's walk ends. A whole loop ends where it is entered, and
/// [`chain`] enters it at the vertex nearest the previous end - so its
/// end is only known then; its first vertex stands in.
fn piece_end(p: &Piece) -> Option<Point2> {
    if p.closed { p.pts.first().copied() } else { p.pts.last().copied() }
}

/// How far `p`'s entry stands from `end`: a whole loop's nearest vertex,
/// an open piece's first point.
fn near(p: &Piece, end: Point2) -> f64 {
    if p.closed {
        p.pts.iter().map(|&q| dist(q, end)).fold(f64::INFINITY, f64::min)
    } else {
        p.pts.first().map_or(f64::INFINITY, |&q| dist(q, end))
    }
}

/// The cutting order: rings stay in order; inside one ring the next
/// piece is the one whose start lies nearest the current end.
fn cutting_order(pieces: &[Piece]) -> Vec<usize> {
    let mut order: Vec<usize> = Vec::with_capacity(pieces.len());
    let mut i = 0;
    while i < pieces.len() {
        let mut e = i;
        while e < pieces.len() && pieces[e].ring == pieces[i].ring {
            e += 1;
        }
        let mut left: Vec<usize> = (i..e).collect();
        let mut at = order.last().and_then(|&k| piece_end(&pieces[k]));
        while !left.is_empty() {
            let pick = match at {
                Some(end) => (0..left.len())
                    .min_by(|&a, &b| near(&pieces[left[a]], end).total_cmp(&near(&pieces[left[b]], end)))
                    .unwrap_or(0),
                None => 0,
            };
            let k = left.remove(pick);
            at = piece_end(&pieces[k]);
            order.push(k);
        }
        i = e;
    }
    order
}

/// Order the pieces into passes ([`cutting_order`]); a whole loop is
/// entered at its vertex nearest the current end. Two consecutive pieces
/// share a pass only over a short chord (at most two stepovers) that
/// stays inside the area and crosses no piece and no earlier chord; else
/// the caller links them with a retract and a new entry, and the new
/// pass starts with a short lead-in from a piece this level already cut
/// where one is near.
pub(super) fn chain(center: &Region, pieces: &[Piece], stepover: f64) -> Vec<Vec<Point2>> {
    let paths: Vec<(&[Point2], bool)> = pieces.iter().map(|p| (p.pts.as_slice(), p.closed)).collect();
    let mut passes: Vec<Vec<Point2>> = Vec::new();
    let mut chords: Vec<(Point2, Point2)> = Vec::new();
    let mut cur: Vec<Point2> = Vec::new();
    let mut done: Vec<Vec<Point2>> = Vec::new();
    for p in cutting_order(pieces).iter().map(|&k| &pieces[k]) {
        let mut pts = p.pts.clone();
        if p.closed
            && let Some(&end) = cur.last()
            && let Some(i) = (0..pts.len()).min_by(|&a, &b| dist(pts[a], end).total_cmp(&dist(pts[b], end)))
        {
            pts.rotate_left(i);
        }
        if p.closed
            && let Some(&f) = pts.first()
        {
            pts.push(f);
        }
        let Some(&first) = pts.first() else { continue };
        let ok = |a: Point2, chords: &[(Point2, Point2)]| {
            // The ends touch the pieces they join (a split point stands on
            // the far wall to the last bit): the crossing tests read the
            // chord pulled in by a micron at each end, so a touch is never
            // a crossing; the area test reads the true ends.
            let (a2, f2) = pulled_in(a, first, 1e-3);
            chord_clears(center, a, first)
                && !paths.iter().any(|(l, closed)| chord_crosses_path(a2, f2, l, *closed))
                && !chords.iter().any(|&(c, d)| chord_crosses_path(a2, f2, &[c, d], false))
        };
        let joined = cur.last().is_some_and(|&end| dist(end, first) <= 2.0 * stepover + 1e-9 && ok(end, &chords));
        if joined {
            if let Some(&end) = cur.last() {
                chords.push((end, first));
            }
        } else {
            if cur.len() >= 2 {
                passes.push(std::mem::take(&mut cur));
            }
            cur.clear();
            // A new pass would enter at `first` - for an open piece, on
            // the far wall in uncut stock. Lead in from the nearest point
            // of a piece this level already cut instead: the entry then
            // lands in a cleared column and the lead steps over like a
            // chord. The lead starts a few microns off the cut piece,
            // toward the new one: never ON a path it could cross once the
            // moves are on the micron lattice.
            if let Some(q) = nearest_on(&done, first)
                .filter(|&q| dist(q, first) > 0.01 && dist(q, first) <= LEAD_STEPS * stepover + 1e-9)
                .map(|q| pulled_in(q, first, 0.005).0)
                .filter(|&q| ok(q, &chords))
            {
                chords.push((q, first));
                cur.push(q);
            }
        }
        done.push(pts.clone());
        cur.extend(pts);
    }
    if cur.len() >= 2 {
        passes.push(cur);
    }
    passes
}

/// The chord `a -> b` shortened by `d` mm at each end (unchanged when it
/// is shorter than `4 d`).
fn pulled_in(a: Point2, b: Point2, d: f64) -> (Point2, Point2) {
    let l = dist(a, b);
    if l <= 4.0 * d {
        return (a, b);
    }
    let (ux, uy) = ((b.x - a.x) / l * d, (b.y - a.y) / l * d);
    (Point2::new(a.x + ux, a.y + uy), Point2::new(b.x - ux, b.y - uy))
}

/// The longest lead-in, in stepovers: a step from the cut side into a
/// new piece, as long as a chord between neighbouring rings may be.
const LEAD_STEPS: f64 = 1.5;

/// The point of the polylines `done` nearest to `p`.
fn nearest_on(done: &[Vec<Point2>], p: Point2) -> Option<Point2> {
    let mut best: Option<(f64, Point2)> = None;
    for l in done {
        for w in l.windows(2) {
            let (a, b) = (w[0], w[1]);
            let (dx, dy) = (b.x - a.x, b.y - a.y);
            let dd = dx * dx + dy * dy;
            let t = if dd > 0.0 { (((p.x - a.x) * dx + (p.y - a.y) * dy) / dd).clamp(0.0, 1.0) } else { 0.0 };
            let q = Point2::new(a.x + dx * t, a.y + dy * t);
            let d = dist(q, p);
            if best.is_none_or(|(bd, _)| d < bd) {
                best = Some((d, q));
            }
        }
    }
    best.map(|(_, q)| q)
}
