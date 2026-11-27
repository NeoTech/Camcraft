//! The approach from the air (#4254): the last stretch of a stay-down link
//! into the start of an open piece whose start is on the stock edge (a
//! ring cut open there). The tool comes from the air in the cut plane,
//! straight to the start, and the stretch cuts no stock off the planned
//! curve: each piece of stock that the disc sweeps on the stretch lies in
//! the swath of the piece that it reaches, so the tool begins to cut along
//! the planned curve. Elsewhere the disc stands outside the stock: within
//! a tool radius of the air (the air starts one tool radius outside the
//! stock, so no point inside the stock is that near to it). The proof is
//! exact area geometry of the swept disc, as the lead-in of an air entry is
//! proved in its room (`entry/air.rs`), not a strip of stock that is open
//! by a constant. The way to the air is proved as any link (`proof.rs`).
//! Provenance: the tangential approach from outside the stock of public
//! machining handbooks; the swept disc as the Minkowski sum of the path
//! and the tool disc (Held 1991); our own design record
//! `docs/decisions/4024-path-kernel-design.md` (#3934: the split pieces of
//! a band are joined from the air) and `docs/decisions/4254-kernel-air-approach.md`.

use super::proof::{Proof, tries};
use super::route::project;
use crate::entry::ROOM_TOL;
use crate::plan::Cut;
use crate::{Point2, Region, in_area, near, to_loops};

/// The tolerance (mm) of the area test: the swept discs and the grown
/// areas are polygons flattened within [`ROOM_TOL`] each, so the cover is
/// grown by twice that. It is the flattening of the polygon offset, not
/// room for the tool.
const AREA_TOL: f64 = 2.0 * ROOM_TOL;

/// The proved link from `a` (the end of `from`) to `b` (the start of `to`)
/// whose last stretch is an approach from the air; `None` when `to` is not
/// an open piece that starts on the stock edge, or when no approach and no
/// way out to it is proved.
pub(super) fn from_air(proof: &mut Proof<'_>, from: &Cut, to: &Cut, a: Point2, b: Point2) -> Option<Vec<Point2>> {
    let (p, r) = (proof.region(), proof.radius());
    if to.curve().closed() || p.air().is_empty() || to_loops(b, p.stock_edge()) > ROOM_TOL {
        return None;
    }
    // The doors: the places of the air nearest to the start, one for each
    // air loop within the edge band of it (a tool radius and the room
    // tolerance), the nearest first.
    let mut doors: Vec<(f64, Point2)> = p
        .air()
        .loops()
        .iter()
        .filter_map(|l| project(l.points(), b))
        .map(|(_, _, q, d)| (d, q))
        .filter(|&(d, _)| d <= r + ROOM_TOL)
        .collect();
    doors.sort_by(|x, y| x.0.total_cmp(&y.0));
    for (_, door) in doors {
        if !approach(proof, to, door, b) {
            continue;
        }
        if let Ok(mut path) = tries(proof, from, a, door) {
            path.push(b);
            path.dedup_by(|x, y| near(*x, *y));
            return Some(path);
        }
    }
    None
}

/// The points that the tool centre walks along `cut`.
fn path_of(cut: &Cut) -> Vec<Point2> {
    cut.curve().pts().iter().map(|q| q.at()).collect()
}

/// True when the straight stretch from `door` (in the air) to `b` (the
/// start of `to`) is a proved approach:
/// - each sample is in the air or the edge band (clear of the part zone),
///   more than a tool radius from each keep-out area, and not inside the
///   cutter-centre area (the tool comes from outside the stock);
/// - the stretch holds in the legal room of the entry stage;
/// - it crosses no cut made and no link, and touches no planned cut away
///   from the ends of the link;
/// - the disc swept along it, less the swath of `to`, lies outside the
///   stock (within a tool radius of the air).
pub(super) fn approach(proof: &Proof<'_>, to: &Cut, door: Point2, b: Point2) -> bool {
    let (p, r) = (proof.region(), proof.radius());
    let keep = p.keep_out();
    let clear = |q: Point2| keep.is_empty() || (!in_area(keep, q) && to_loops(q, keep.loops()) > r);
    let center = p.center();
    let outside_center = |q: Point2| near(q, b) || !center.contains(q) || to_loops(q, center.loops()) <= ROOM_TOL;
    let pts = proof.samples(door, b);
    if !in_area(p.air(), door) || !pts.iter().all(|&q| (in_area(p.air(), q) || proof.in_band(q)) && clear(q) && outside_center(q)) {
        return false;
    }
    if !proof.legal().is_some_and(|l| matches!(l.holds_path(&[door, b]), Ok(true))) {
        return false;
    }
    if proof.crossing(door, b, None).is_err() {
        return false;
    }
    let Some(outside) = proof.outside.get_or_init(|| p.air().offset(r + AREA_TOL).ok()) else {
        return false;
    };
    let off_curve = Region::sweep(&[vec![door, b]], r)
        .and_then(|swept| Region::sweep(&[path_of(to)], r + AREA_TOL).and_then(|on| swept.difference(&on)))
        .and_then(|rest| rest.difference(outside));
    matches!(off_curve, Ok(rest) if rest.is_empty())
}
