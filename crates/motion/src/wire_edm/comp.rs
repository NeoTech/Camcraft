//! The control's wire offset for a Wire Contour (#4134, the app side of
//! #4076): over a wire-centre path already generated and judged, one
//! [`CompSpan`] for each pass, whose programmed geometry is the PART
//! contour that the post writes under G41 / G42 (the control adds the
//! register), with G40 on the lead-out.
//!
//! Provenance: RS-274 / ISO 6983 G40 / G41 / G42 and the startup and
//! cancel rule `cl::comp` states (the offset is built over the straight
//! block that carries G41 / G42 and withdrawn over the one that carries
//! G40, both at least the offset long); the rules of the owner of a
//! Sodick AQ600L (LN2W control) on #4076, 2026-11-25 (one register for
//! each pass from H001, a straight lead-in and lead-out at least the
//! offset radius long, G40 on the lead-out). The part contour is the
//! wire-centre path moved back by the offset along each element's normal
//! (a line by translation, an arc by its radius) - plane geometry, our
//! own code; no vendor material consulted.
//!
//! The moves stay the wire-centre truth: the kerf, threading and slug
//! laws judge them, and they are what the control cuts when each
//! register holds the radius that its span states (the pass's kerf
//! offset, wire radius plus spark gap). The wear variant programs those
//! moves themselves; the register then holds only the wear delta.

use crate::cl::{CompSide, CompSpan, Move, MoveKind, Toolpath};
use crate::geo2d::Point2;
use crate::geometry::Point3;
use crate::links::Compensation;
use crate::machining2d::GenError;

use super::WireSide;

/// How near (mm) two moved end points must agree to be one vertex of the
/// part contour (a tangent joint): ten lattice steps, so an arc centre
/// snapped to the 1 um lattice still meets its neighbour.
const JOINT_TOL: f64 = 1e-5;
/// How near (mm) a pass must come back to its start hole to close.
const CLOSE_TOL: f64 = 1e-6;
/// The least `1 + cos` of the turn at a sharp joint: a sharper turn
/// back (a hairpin) has no stable corner point.
const HAIRPIN: f64 = 1e-3;
/// The chord tolerance (mm) the walk's turning sense is read with.
const AREA_CHORD: f64 = 1e-3;

fn xy(p: Point3) -> Point2 {
    Point2::new(p.x, p.y)
}

fn dist(a: Point2, b: Point2) -> f64 {
    (b.x - a.x).hypot(b.y - a.y)
}

fn refused(mode: Compensation, why: impl std::fmt::Display) -> GenError {
    GenError::Invalid(format!("wire contour: {} compensation {why}", mode.label()))
}

/// Record a [`CompSpan`] for each pass of the wire-centre `path`. `radii`
/// holds each pass's kerf offset (mm), the rough pass first, in cut
/// order. `Compensation::Computer` records nothing.
///
/// # Errors
/// An open run, a pass that does not start and end at the start hole, a
/// lead that is not straight or is shorter than its pass's offset, an arc
/// tighter than the offset, a hairpin turn, or a pass count that is not
/// the number of radii - each by name; none is a panic.
pub fn wire_comp_spans(path: &mut Toolpath, side: WireSide, mode: Compensation, radii: &[f64]) -> Result<(), GenError> {
    if !mode.in_control() {
        return Ok(());
    }
    let die = match side {
        WireSide::Die => true,
        WireSide::Punch => false,
        WireSide::Left | WireSide::Right | WireSide::Centre => {
            return Err(refused(
                mode,
                "is stated on an open run - the control's wire offset engages on a closed profile's lead-in from its start hole; cut the run with computer compensation",
            ))
        }
    };
    if !path.comps.is_empty() {
        return Err(refused(mode, "found offset spans on the path already - regenerate it"));
    }
    let moves = &path.moves;
    let Some(s0) = moves.first().map(|m| xy(m.from)) else {
        return Err(refused(mode, "found no wire path to program"));
    };
    let mut spans = Vec::with_capacity(radii.len());
    let mut b = 0usize;
    for (i, &k) in radii.iter().enumerate() {
        let n = i + 1;
        if !(k.is_finite() && k > 0.0) {
            return Err(refused(mode, format!("found pass {n} with no usable offset radius ({k}) - refused")));
        }
        let Some(e) = (b.saturating_add(2)..moves.len()).find(|&j| moves.get(j).is_some_and(|m| dist(xy(m.to), s0) < CLOSE_TOL)) else {
            return Err(refused(
                mode,
                format!("found pass {n} that does not return to its start hole - each pass is programmed from the start hole and back, so the wire path cannot carry the offset"),
            ));
        };
        let span = if mode == Compensation::Wear {
            wear_span(moves, b, e, die, k, n)?
        } else {
            control_span(moves, b, e, die, k, n)?
        };
        spans.push(span);
        b = e + 1;
    }
    if b != moves.len() {
        return Err(refused(
            mode,
            format!("found moves after the last of {} pass(es) - every move of the cut must be inside a pass's offset", radii.len()),
        ));
    }
    path.comps.extend(spans);
    Ok(())
}

/// The pass's moves `b..=e` checked as one programmable pass: from the
/// start hole by a straight lead-in, around at one height, and back by a
/// straight lead-out, no rapid and no pose. Returns the side of travel
/// that the wire stands on (G41 / G42).
fn pass_side(moves: &[Move], b: usize, e: usize, die: bool, mode: Compensation, n: usize) -> Result<CompSide, GenError> {
    let pass = moves.get(b..=e).ok_or_else(|| refused(mode, format!("lost pass {n}")))?;
    let (Some(lead_in), Some(lead_out)) = (pass.first(), pass.last()) else {
        return Err(refused(mode, format!("lost pass {n}")));
    };
    if lead_in.arc.is_some() || lead_out.arc.is_some() {
        return Err(refused(
            mode,
            format!("needs a straight lead-in and lead-out - pass {n} has an arc lead; the offset engages on a straight lead-in and G40 goes on a straight lead-out"),
        ));
    }
    let z = lead_in.from.z;
    if pass.iter().any(|m| m.kind == MoveKind::Rapid || m.axis.is_some() || m.from.z != z || m.to.z != z || !m.is_finite()) {
        return Err(refused(
            mode,
            format!("needs a vertical cut at one programmed plane - pass {n} has a rapid, a pose or a change of height"),
        ));
    }
    // The walk's turning sense, from the ring between the leads.
    let mut area = 0.0;
    for m in pass.get(1..pass.len() - 1).unwrap_or_default() {
        for (a, c) in m.chords(AREA_CHORD) {
            area += a.x * c.y - c.x * a.y;
        }
    }
    if !(area.is_finite() && area.abs() > 1e-12) {
        return Err(refused(mode, format!("found pass {n} that encloses no area - there is no side for the offset")));
    }
    // A die's wire runs inside the profile: on the left of a
    // counter-clockwise walk. A punch's runs outside it.
    Ok(if (area > 0.0) == die { CompSide::Left } else { CompSide::Right })
}

/// The wear span (the handbooks' wear-offset practice): the programmed
/// moves ARE the wire-centre moves of the pass, and the register holds
/// only the wear delta. The stated radius is the pass's kerf offset.
fn wear_span(moves: &[Move], b: usize, e: usize, die: bool, k: f64, n: usize) -> Result<CompSpan, GenError> {
    let mode = Compensation::Wear;
    let side = pass_side(moves, b, e, die, mode, n)?;
    let programmed = moves.get(b..=e).map(<[Move]>::to_vec).unwrap_or_default();
    let span = CompSpan { side, radius: k, wear: true, first: b, last: e, programmed };
    if !span.is_sane(moves) {
        return Err(refused(mode, format!("could not chain pass {n}'s wire-centre walk")));
    }
    Ok(span)
}

/// The unit normal on the LEFT of travel of `m` at its start (`at_end`
/// false) or its end.
fn left_normal(m: &Move, at_end: bool) -> Option<Point2> {
    let p = xy(if at_end { m.to } else { m.from });
    let v = match m.arc {
        // Left of a counter-clockwise arc is toward its centre.
        Some(a) => {
            let r = Point2::new(a.center.x - p.x, a.center.y - p.y);
            if a.ccw {
                r
            } else {
                Point2::new(-r.x, -r.y)
            }
        }
        None => {
            let d = Point2::new(m.to.x - m.from.x, m.to.y - m.from.y);
            Point2::new(-d.y, d.x)
        }
    };
    let l = v.x.hypot(v.y);
    (l.is_finite() && l > 1e-12).then(|| Point2::new(v.x / l, v.y / l))
}

/// The control span: the lead-in from the start hole to the part
/// contour, the part contour (each ring move moved back by `k` toward
/// the product, sharp joints met at their corner, arcs that the offset
/// rounded in collapsed to their corner), and the lead-out back.
fn control_span(moves: &[Move], b: usize, e: usize, die: bool, k: f64, n: usize) -> Result<CompSpan, GenError> {
    let mode = Compensation::Control;
    let side = pass_side(moves, b, e, die, mode, n)?;
    // The wire stands `k` on side `s` of the part contour: the contour is
    // the wire point moved by `k` the other way.
    let s = if side == CompSide::Left { 1.0 } else { -1.0 };
    let ring = moves.get(b + 1..e).ok_or_else(|| refused(mode, format!("lost pass {n}'s ring")))?;
    let normal = |m: &Move, at_end: bool| {
        left_normal(m, at_end).ok_or_else(|| refused(mode, format!("found a move of pass {n} with no direction")))
    };
    let back = |p: Point3, nv: Point2| Point2::new(p.x - s * k * nv.x, p.y - s * k * nv.y);
    // The part contour's vertices: the start of the first ring move, each
    // joint between ring moves, the end of the last one.
    let (Some(r0), Some(r1)) = (ring.first(), ring.last()) else {
        return Err(refused(mode, format!("found pass {n} with no cut between its leads")));
    };
    let mut pts: Vec<Point2> = vec![back(r0.from, normal(r0, false)?)];
    for w in ring.windows(2) {
        let (n1, n2) = (normal(&w[0], true)?, normal(&w[1], false)?);
        let (p1, p2) = (back(w[0].to, n1), back(w[1].from, n2));
        if dist(p1, p2) <= JOINT_TOL {
            pts.push(p1);
            continue;
        }
        // A sharp joint: two lines of the part contour meet at the
        // corner the mitred wire path was offset from.
        if w[0].arc.is_some() || w[1].arc.is_some() {
            return Err(refused(
                mode,
                format!("found an arc of pass {n} that meets the next move at a corner (X{:.3} Y{:.3}) - the part contour there is not programmed; cut it with computer compensation", w[0].to.x, w[0].to.y),
            ));
        }
        let c = 1.0 + n1.x * n2.x + n1.y * n2.y;
        if c < HAIRPIN {
            return Err(refused(mode, format!("found a turn back of pass {n} at X{:.3} Y{:.3} - no corner of the part contour to program", w[0].to.x, w[0].to.y)));
        }
        let m = Point2::new((n1.x + n2.x) / c, (n1.y + n2.y) / c);
        pts.push(back(w[0].to, m));
    }
    pts.push(back(r1.to, normal(r1, true)?));
    let (lead_in, lead_out) = match (moves.get(b), moves.get(e)) {
        (Some(a), Some(z)) => (*a, *z),
        _ => return Err(refused(mode, format!("lost pass {n}'s leads"))),
    };
    let z = lead_in.from.z;
    let at = |p: Point2| Point3::new(p.x, p.y, z);
    let (first, last) = match (pts.first(), pts.last()) {
        (Some(f), Some(l)) => (*f, *l),
        _ => return Err(refused(mode, format!("lost pass {n}'s contour"))),
    };
    for (what, len) in [("lead-in", dist(xy(lead_in.from), first)), ("lead-out", dist(last, xy(lead_out.to)))] {
        if len + 1e-9 < k {
            return Err(refused(
                mode,
                format!("needs a straight {what} at least the offset radius ({k:.4} mm) long - pass {n}'s {what} to the part contour is {len:.4} mm; move the start hole farther from the profile"),
            ));
        }
    }
    let mut programmed = vec![Move::new(MoveKind::Feed, lead_in.from, at(first), lead_in.feed)];
    // Each programmed move starts where the one before it ended, so the
    // contour chains exactly.
    let mut here = first;
    for (i, m) in ring.iter().enumerate() {
        let Some(p2) = pts.get(i + 1) else {
            return Err(refused(mode, format!("lost pass {n}'s contour")));
        };
        if dist(here, *p2) <= JOINT_TOL {
            continue; // a round the offset put in: its corner is one point
        }
        let pm = Move::new(MoveKind::Feed, at(here), at(*p2), m.feed);
        programmed.push(match m.arc {
            None => pm,
            Some(a) => {
                let r = |p: Point2| dist(p, a.center);
                let r0 = r(xy(m.from));
                let (q0, q1) = (r(here), r(*p2));
                // The offset of an arc keeps its centre and moves its
                // radius by the offset; any other is no part arc.
                if (q0 - q1).abs() > 1e-4 || ((q0 - r0).abs() - k).abs() > 1e-4 || q0 <= JOINT_TOL {
                    return Err(refused(
                        mode,
                        format!("found an arc of pass {n} (radius {r0:.4} mm) with no part arc at the offset {k:.4} mm - cut it with computer compensation"),
                    ));
                }
                pm.with_arc(a.center, a.ccw)
            }
        });
        here = *p2;
    }
    programmed.push(Move::new(MoveKind::Feed, at(here), lead_out.to, lead_out.feed));
    let span = CompSpan { side, radius: k, wear: false, first: b, last: e, programmed };
    if !span.is_sane(moves) {
        return Err(refused(mode, format!("could not chain pass {n}'s part contour")));
    }
    Ok(span)
}

#[cfg(test)]
#[path = "comp_tests.rs"]
mod tests;
