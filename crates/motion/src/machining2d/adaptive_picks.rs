//! 2D Adaptive start and exit points (#3327, design #1448): the start
//! point is the seed of the HSM plan - the entry (helix or plunge) goes
//! down there and the engagement law grows the cleared area from it - and
//! the exit point is where the last pass of each level ends: it runs back
//! along its own closing ride (the same moves, reversed) to the point
//! nearest the exit point. A point that the plan cannot hold is refused by
//! name, never moved.
//!
//! Provenance: our own design over the general machining practice of a
//! chosen entry and exit position (machine-tool handbooks) and elementary
//! plane geometry (point to segment distance); no vendor material.

use crate::approach::Approach;
use crate::cl::{ArcMove, Move, MoveKind, Toolpath};
use crate::entry::{Entry3D, dive_room};
use crate::geo2d::{Point2, Region};
use crate::geometry::Point3;
use crate::path_picks::{PathPicks, PickRole};

use super::adaptive::boundary_dist;
use super::adaptive_method::AdaptiveMethod;
use super::face::GenError;
use super::pass_start::PassStart;
use super::pocket::PocketParams;
use super::pocket_hsm::{EntryRoom, HELIX_SHARE, triangle_about};

/// The settings of a 2D Adaptive that decide if it takes a pick.
#[derive(Clone, Copy, Debug)]
pub struct AdaptivePickDoor {
    pub method: AdaptiveMethod,
    pub pass_start: PassStart,
    pub entry: Entry3D,
    /// Rest machining is on.
    pub rest: bool,
    /// The op is tilted (3+2).
    pub tilted: bool,
}

/// The by-name refusal of a start or exit point that a 2D Adaptive with
/// these settings cannot hold, or `None`. The door and the generator say
/// the same words.
#[must_use]
pub fn adaptive_pick_refusal(picks: &PathPicks, d: &AdaptivePickDoor) -> Option<String> {
    let role = [PickRole::Start, PickRole::Exit].into_iter().find(|r| !picks.of(*r).is_empty())?;
    let what = role.word();
    if d.tilted {
        return Some(format!("a {what} point is not taken on a tilted (3+2) 2D Adaptive yet - clear it or the tilt"));
    }
    if d.method == AdaptiveMethod::Trochoidal {
        return Some(format!(
            "2D Adaptive: a {what} point is not taken with the Trochoidal method - its loops follow the medial line from their own seed; clear the point or use Spiral"
        ));
    }
    if d.pass_start != PassStart::Auto {
        return Some(format!(
            "2D Adaptive: a {what} point is not taken with Pass starts at {} - both choose where the cut starts; clear the point or set Pass starts at Auto",
            d.pass_start.label()
        ));
    }
    if d.rest {
        return Some(format!(
            "2D Adaptive: a {what} point is not taken on a rest pass - the rest areas are cut in their own order; clear the point or turn rest machining off"
        ));
    }
    if !picks.start.is_empty() && !matches!(d.entry, Entry3D::Helix | Entry3D::Plunge) {
        return Some(format!(
            "2D Adaptive: a start point needs a Helix or Plunge entry - the {:?} entry descends along its own first cut; clear the start point or change the entry",
            d.entry
        ));
    }
    None
}

/// The entry at the user's start point `q`: the plan's seed is `q`
/// itself (the ladder's `dive_room` finds the centroid of the triangle
/// about it). Refused by name: a point outside the area the tool centre
/// can reach, a helix that has no room there, an entry other than a helix
/// or a plunge.
pub(super) fn picked_entry(center: &Region, p: &PocketParams, q: Point2) -> Result<EntryRoom, GenError> {
    let helix = match p.entry {
        Entry3D::Helix => true,
        Entry3D::Plunge => false,
        e => {
            return Err(GenError::Invalid(format!(
                "2D Adaptive: a start point needs a Helix or Plunge entry - the {e:?} entry descends along its own first cut; clear the start point or change the entry"
            )));
        }
    };
    if !center.contains(q) {
        return Err(GenError::Invalid(format!(
            "2D Adaptive: the start point ({:.3}, {:.3}) is outside the area the tool centre can reach - it must be the tool radius plus the stock to leave inside the walls and clear of each island and keep-out",
            q.x, q.y
        )));
    }
    let clear = boundary_dist(center, q);
    let hr = if helix { p.helix_radius } else { 0.0 };
    if helix && !(hr > 1e-3 && hr <= HELIX_SHARE * clear) {
        return Err(GenError::Invalid(format!(
            "2D Adaptive: the helix of radius {hr:.3} has no room at the start point ({:.3}, {:.3}) - the tool centre has {clear:.3} mm of room there and the helix needs {:.3}; move the start point or make the helix smaller",
            q.x,
            q.y,
            hr / HELIX_SHARE
        )));
    }
    let tess = triangle_about(q, hr + 0.01)?;
    let Some(seed) = dive_room(&tess, if helix { hr + 1e-3 } else { 0.0 }) else {
        return Err(GenError::Invalid(format!("2D Adaptive: the entry has no room at the start point ({:.3}, {:.3})", q.x, q.y)));
    };
    Ok(EntryRoom { seed, clear: boundary_dist(center, seed) - 1e-6, helix_r: hr, tess })
}

/// Does the op state a start or an exit point?
#[must_use]
pub(super) fn picked(a: &Approach) -> bool {
    a.start_point.is_some() || a.exit_point.is_some()
}

/// The refusal of a pick when the HSM plan is not posted: the Offset
/// rings and the trochoid that cut the area instead choose their own
/// entry and end, so the pick could not hold. `None` without a pick.
pub(super) fn fallback_refusal(a: &Approach, why: &str) -> Option<GenError> {
    picked(a).then(|| {
        GenError::Invalid(format!(
            "2D Adaptive: the HSM plan {why} - the Offset rings or the trochoid that would cut the area instead do not take a start or exit point; move or clear the point"
        ))
    })
}

/// The words of the planner's notes that say a cut over the optimal
/// load was kept (`adaptive::grow`, `adaptive::morph`, `adaptive::plan`).
const OVERLOAD: [&str; 2] = ["the least overload was cut", "optimal load by"];

/// The refusal of a plan from the user's start point that cut over the
/// optimal load somewhere: the engagement law holds from the first pass
/// on, so the planner's named overload is not posted for a picked start.
/// `None` without a start point, or without such a note.
pub(super) fn overload_refusal(a: &Approach, notes: &[String]) -> Option<GenError> {
    a.start_point?;
    let n = notes.iter().find(|n| OVERLOAD.iter().any(|w| n.contains(w)))?;
    Some(GenError::Invalid(format!(
        "2D Adaptive: the plan from the start point does not hold the optimal load ({n}) - move the start point"
    )))
}

/// The refusal of a start or exit point inside a "Do not touch" keep-out
/// `ko` (the items `names`), or `None`.
pub(super) fn keepout_refusal(a: &Approach, ko: &Region, names: &str) -> Option<GenError> {
    [("start", a.start_point), ("exit", a.exit_point)].into_iter().find_map(|(what, q)| {
        let q = q?;
        ko.contains(q).then(|| {
            GenError::Invalid(format!(
                "2D Adaptive: the {what} point ({:.3}, {:.3}) is inside the 'Do not touch' keep-out of {names} - move the point",
                q.x, q.y
            ))
        })
    })
}

/// The order of the pieces of the area: the start's piece first (the
/// `seed`), the piece nearest the exit point last. Refused by name: an
/// exit point at the start's piece while other pieces are cut after it.
pub(super) fn order_pieces(comps: &mut [Region], seed: Point2, exit: Option<Point2>) -> Result<(), GenError> {
    comps.sort_by_key(|c| !c.contains(seed));
    let Some(q) = exit.filter(|_| !comps.is_empty()) else { return Ok(()) };
    // The piece nearest the exit point (the closing ride of its last pass
    // is judged against the reach in `exit_on_path`).
    let near = |c: &Region| if c.contains(q) { 0.0 } else { boundary_dist(c, q) };
    let k = (0..comps.len()).min_by(|&a, &b| near(&comps[a]).total_cmp(&near(&comps[b]))).unwrap_or(0);
    if k == 0 && comps.len() > 1 && comps[0].contains(seed) {
        return Err(GenError::Invalid(format!(
            "2D Adaptive: the exit point ({:.3}, {:.3}) is at the piece of the area where the cut starts, and {} other piece(s) are cut after it - move the exit point into another piece",
            q.x,
            q.y,
            comps.len() - 1
        )));
    }
    let last = comps.len() - 1;
    if k != last {
        comps[k..].rotate_left(1);
    }
    Ok(())
}

/// Two points this near are one point (mm).
const SAME: f64 = 1e-6;

/// The exit point of `p` ends the last pass of each level of `path` (an
/// HSM plan over `center`): from the level's last feed the tool runs back
/// along the same moves in reverse - over one round of the area's outer
/// wall at most, its closing ride - to the point of that tail nearest the
/// exit point, and the climb out of the level moves there. A reversed
/// move is the same line or the same arc, so the run back takes no new
/// material and makes no feed-feed crossing. Refused by name: an exit
/// point farther than [`reach`] from that tail, and a level whose last
/// feed is not followed by a straight climb and a rapid. Without an exit
/// point `path` comes back as it is.
pub(super) fn exit_on_path(mut path: Toolpath, center: &Region, p: &PocketParams) -> Result<Toolpath, GenError> {
    let Some(q) = p.links.approach.exit_point else { return Ok(path) };
    // The moves are spliced: a path whose records point at move indices
    // is not one the HSM plan makes, and is refused rather than shifted.
    if !(path.cycles.is_empty() && path.comps.is_empty() && path.threads.is_empty() && path.probes.is_empty() && path.breaks.is_empty()) {
        return Err(GenError::Invalid("2D Adaptive: an exit point cannot be placed on this path - clear the exit point".into()));
    }
    let window = outer_round(center);
    let zs = super::equal_steps::cut_levels_for(p.top_z, &p.links.approach, p.bottom_z, p.stepdown)?;
    let planar = |m: &Move, z: f64| m.kind == MoveKind::Feed && (m.from.z - z).abs() < 1e-9 && (m.to.z - z).abs() < 1e-9;
    // The last feed of each level, last level first, so the indices hold.
    let mut ends: Vec<(usize, f64)> =
        zs.iter().filter_map(|&z| path.moves.iter().rposition(|m| planar(m, z)).map(|i| (i, z))).collect();
    ends.sort_by_key(|a| std::cmp::Reverse(a.0));
    for (i, z) in ends {
        let back = run_back(&path.moves, (i, z), q, (window, reach(p)), &planar)?;
        let Some(end) = back.last().map(|m| m.to) else { continue };
        reseat_climb(&mut path.moves, i + 1, end)?;
        path.moves.splice(i + 1..i + 1, back);
    }
    Ok(path)
}

/// The reversed moves from the end of move `i` back to the point nearest
/// `q` over at most `window` mm of the level's last planar feeds (the
/// level `z` ends at move `i`); refused past `reach`.
fn run_back(moves: &[Move], (i, z): (usize, f64), q: Point2, (window, reach): (f64, f64), planar: &dyn Fn(&Move, f64) -> bool) -> Result<Vec<Move>, GenError> {
    let xy = |p: Point3| Point2::new(p.x, p.y);
    // (move index, the point on it, distance to `q`): the end itself first.
    let end = xy(moves[i].to);
    let mut best = (i + 1, end, (end.x - q.x).hypot(end.y - q.y));
    let mut run = 0.0;
    for j in (0..=i).rev() {
        let m = &moves[j];
        if !planar(m, z) || run >= window {
            break;
        }
        let (a, b) = (xy(m.from), xy(m.to));
        let f = if m.arc.is_some() { a } else { foot(a, b, q) };
        let d = (f.x - q.x).hypot(f.y - q.y);
        if d < best.2 - SAME {
            best = (j, f, d);
        }
        run += (b.x - a.x).hypot(b.y - a.y);
    }
    let (k, f, d) = best;
    if d > reach {
        return Err(GenError::Invalid(format!(
            "2D Adaptive: the exit point ({:.3}, {:.3}) is {d:.3} mm from the closing ride of the last pass along the walls - the most is {reach:.3}; move the exit point to the walls of the area",
            q.x, q.y
        )));
    }
    let reversed = |m: &Move| Move { from: m.to, to: m.from, arc: m.arc.map(|a| ArcMove { ccw: !a.ccw, ..a }), ..*m };
    let mut out: Vec<Move> = moves.get(k + 1..=i).unwrap_or(&[]).iter().rev().map(reversed).collect();
    if let Some(m) = moves.get(k).filter(|_| k <= i) {
        if m.arc.is_some() || (f.x - m.from.x).hypot(f.y - m.from.y) <= SAME {
            out.push(reversed(m));
        } else if (f.x - m.to.x).hypot(f.y - m.to.y) > SAME {
            out.push(Move { from: m.to, to: Point3::new(f.x, f.y, z), arc: None, ..*m });
        }
    }
    Ok(out)
}

/// The climb out of a level at move `at` (and each straight climb after
/// it) moves over to `end`; the rapid after them starts there. Refused by
/// name when the level ends another way (a link that stays down).
fn reseat_climb(moves: &mut [Move], at: usize, end: Point3) -> Result<(), GenError> {
    let vertical = |m: &Move| (m.from.x - m.to.x).abs() < 1e-12 && (m.from.y - m.to.y).abs() < 1e-12 && m.to.z > m.from.z && m.arc.is_none();
    if !moves.get(at).is_some_and(vertical) {
        return Err(GenError::Invalid(
            "2D Adaptive: an exit point needs the tool to climb out after the last pass of each level - set the level link to a retract".into(),
        ));
    }
    let mut j = at;
    while let Some(m) = moves.get_mut(j) {
        let climb = vertical(m);
        m.from.x = end.x;
        m.from.y = end.y;
        if !climb {
            if m.kind != MoveKind::Rapid {
                return Err(GenError::Invalid(
                    "2D Adaptive: an exit point needs a rapid after the climb out of each level - set the level link to a retract".into(),
                ));
            }
            break;
        }
        m.to.x = end.x;
        m.to.y = end.y;
        j += 1;
    }
    Ok(())
}

/// The length of the largest loop of `center` (one round of the outer wall).
fn outer_round(center: &Region) -> f64 {
    center.loops().iter().max_by(|a, b| a.area().abs().total_cmp(&b.area().abs())).map_or(0.0, |l| {
        let pts = l.points();
        (0..pts.len())
            .map(|i| {
                let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
                (b.x - a.x).hypot(b.y - a.y)
            })
            .sum()
    })
}

/// The point of segment `ab` nearest `p`.
fn foot(a: Point2, b: Point2, p: Point2) -> Point2 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let l2 = dx * dx + dy * dy;
    if l2.is_nan() || l2 <= 0.0 {
        return a;
    }
    let t = (((p.x - a.x) * dx + (p.y - a.y) * dy) / l2).clamp(0.0, 1.0);
    Point2::new(a.x + dx * t, a.y + dy * t)
}

/// The farthest an exit point may sit from the closing ride: the wall's
/// own offset (radius plus stock to leave) and one tool diameter of
/// slack for a click beside the wall (the contour's reach law, #1448).
pub(super) fn reach(p: &PocketParams) -> f64 {
    p.tool_diameter * 1.5 + p.stock_to_leave
}
