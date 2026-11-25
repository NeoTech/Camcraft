//! Pass starts at Outside round a part (#3934): the stock-parallel rings
//! out, the part-parallel rings in, and one order that blends them where
//! the two fronts meet. The cutter-centre area is eroded one step at a
//! time; at every step its boundary is the set of points one step from
//! the nearer edge. Each loop of that erosion is split at the medial line
//! into the stretch that follows the stock edge and the stretch that
//! follows the part (a ring is clipped by the part's keep-out grown by
//! the same offset). Outside cuts the stock stretches from the stock edge
//! inward, then the part stretches from the medial line to the part, and
//! the part wall last: the first passes follow the stock edge, the later
//! ones turn to follow the selected edge, and the blend is where the two
//! fronts meet. Provenance: contour-parallel pocketing as level sets of
//! the distance to the boundary (Held, "On the Computational Geometry of
//! Pocket Machining", Springer LNCS 500, 1991), measured from both edges
//! of a band; the medial split, the order and the level mapping are ours.
//! No vendor material consulted.

use crate::geo2d::{Loop, Point2, Region, RingClip};

use super::face::GenError;
use super::level_order::{Piece, WALL};
use super::ring_fill::{Band, RING, fills};
use super::ring::dist;

/// How far (mm) a point of an erosion ring may stand past the part's
/// grown keep-out and still read as on the part side: 10 um, the side
/// tolerance of `level_rings` and twice Clipper's arc flats. A point
/// nearer the stock than `step + this` from the part is a stock stretch.
const MEDIAL_EPS: f64 = 1e-2;

/// Stretches shorter than this (mm) are dropped.
const MIN_PIECE: f64 = 1e-2;

/// The blend family of one part's cutter-centre area `c` (#3934), with
/// `hc` its hole (the part's centre keep-out). `step` is the ring pitch,
/// `tool_r` the cutter radius (the fills read it), `climb` the cutter
/// sense and `cap` the offset limit. Whole loops follow either edge where
/// the other edge is farther; a stretch that ends at the medial line is
/// an open piece, and those ends lie in the core, not on a wall. Returns
/// `true` once the family is built; an area whose erosion does not empty
/// within `cap` steps is refused by name (the Prime Directive's "never
/// hang").
///
/// - Stock stretches: level `RING * j`, ring `j` of the stock side.
/// - The seam ring at half a step inside the deepest erosion (level
///   `RING * j + 2`), where the two fronts meet and the last core still
///   holds more than half a step of material.
/// - Part stretches: level `RING * (2 j_max + 1 - j)`, so the part wall
///   (`j` 0) is last and the ring next to it is cut late; their fills are
///   the levels between.
pub(super) fn outside_blend(
    c: &Region,
    hc: &Region,
    step: f64,
    tool_r: f64,
    climb: bool,
    cap: u32,
    out: &mut Vec<Piece>,
) -> Result<bool, GenError> {
    if !(step.is_finite() && step > 0.0 && tool_r.is_finite() && tool_r > 0.0) {
        return Err(GenError::Invalid("the ring step and the tool radius must be positive and finite".into()));
    }
    // Ring `j` of each side, as walks (whole loops and open medial pieces).
    let mut stock: Vec<(u32, bool, Vec<Point2>)> = Vec::new();
    let mut part: Vec<(u32, bool, Vec<Point2>)> = Vec::new();
    let mut deepest: Option<u32> = None;
    let mut converged = false;
    for j in 0..=cap {
        let ring = if j == 0 { c.clone() } else { c.offset(-f64::from(j) * step)? };
        if ring.is_empty() {
            converged = true;
            break;
        }
        deepest = Some(j);
        // The part side of this erosion: the part keep-out grown by the
        // same offset plus the medial tolerance. A ring stretch inside it
        // follows the part; the rest follows the stock.
        let part_side = hc.offset(f64::from(j) * step + MEDIAL_EPS)?;
        for l in ring.loops() {
            let walk = walk(l, climb);
            if walk.len() < 2 {
                continue;
            }
            let inside = part_side.clip_ring(&walk, true)?;
            let outside = part_side.clip_ring(&walk, false)?;
            add(&mut stock, j, &walk, outside);
            add(&mut part, j, &walk, inside);
        }
    }
    let Some(deep) = deepest else { return Ok(false) };
    if !converged {
        return Err(GenError::Invalid("pass start rings did not converge; check the stepover".into()));
    }
    // The stock side, outward in: ring 0 on the stock edge, ring j one
    // step further in.
    for (j, closed, pts) in &stock {
        push(out, RING * j, *closed, pts.clone());
    }
    // The seam: where the two fronts meet, the deepest erosion still holds
    // a core. Its half-step level curve lies between the last stock ring
    // and the first part ring, so no core is left wider than half a step.
    if let Ok(seam) = c.offset(-(f64::from(deep) + 0.5) * step) {
        for l in seam.loops() {
            let pts = walk(l, climb);
            if pts.len() >= 2 {
                push(out, RING * deep + 2, true, pts);
            }
        }
    }
    // The part side, nearest the medial first: hole j_max is next to the
    // medial line, hole 1 is one step off the part wall, hole 0 is the
    // wall itself.
    let level = |j: u32| if j == 0 { WALL } else { RING * (2 * deep + 1 - j) };
    let inner_of = |j: u32| -> Vec<Vec<Point2>> {
        part.iter().filter(|(k, _, _)| *k == j).map(|(_, closed, pts)| close(*closed, pts)).collect()
    };
    for j in 1..=deep {
        let band = Band { c, hc, j, step, tool_r, climb };
        out.extend(fills(&band, &inner_of(j - 1), &inner_of(j), (level(j - 1), level(j)))?);
    }
    // The part pieces, deepest j first so their levels ascend toward the
    // wall (j is collected ascending, so walk it back).
    for (j, closed, pts) in part.iter().rev() {
        push(out, level(*j), *closed, pts.clone());
    }
    Ok(true)
}

/// Add the stretches of one loop to `side`: the `Whole` clip keeps the
/// whole closed loop, each `Pieces` stretch is an open walk.
fn add(out: &mut Vec<(u32, bool, Vec<Point2>)>, j: u32, walk: &[Point2], clip: RingClip) {
    match clip {
        RingClip::Whole if walk.len() >= 2 => out.push((j, true, walk.to_vec())),
        RingClip::Whole => {}
        RingClip::Pieces(ps) => {
            for pts in ps {
                let len: f64 = pts.windows(2).map(|w| dist(w[0], w[1])).sum();
                if pts.len() >= 2 && len >= MIN_PIECE {
                    out.push((j, false, pts));
                }
            }
        }
    }
}

/// The loop `l` as it is walked for `climb` (the uncut side on the right),
/// as `waterscan` walks the loops of one erosion step.
fn walk(l: &Loop, climb: bool) -> Vec<Point2> {
    if climb { l.points().to_vec() } else { l.points().iter().rev().copied().collect() }
}

/// A walk as the fill law reads it: a whole loop repeats its first point.
fn close(closed: bool, pts: &[Point2]) -> Vec<Point2> {
    let mut w = pts.to_vec();
    if closed && let Some(&f) = w.first() {
        w.push(f);
    }
    w
}

fn push(out: &mut Vec<Piece>, level: u32, closed: bool, pts: Vec<Point2>) {
    out.push(Piece { level, pts, closed, core: false });
}

#[cfg(test)]
#[path = "level_blend_tests.rs"]
mod tests;
