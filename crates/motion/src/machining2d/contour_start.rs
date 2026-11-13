//! A user-picked start point on a closed 2D contour (#1448): the ring
//! nearest the pick is re-sequenced to begin at the pick's projection onto
//! it, the lead-in is planned there by the lead law, and anything that law
//! cannot hold at the pick is refused by name - a pick is a wish, it never
//! overrides a check, and the entry never moves away from it silently.
//! Also the automatic seam (axis start, control seam, lead-room reseat)
//! that every ring without a pick keeps. Provenance: first principles -
//! the closest point on a segment (orthogonal projection clamped to the
//! segment) and the rotation of a closed polyline's vertex list; the
//! start-point parameter is the common contour-entry vocabulary of
//! published CAM practice. No vendor material consulted.

use crate::geo2d::{Loop, Point2};

use super::arcs::Seg;
use super::contour_comp;
use super::contour_params::{ContourParams, Lead};
use super::face::GenError;
use super::fillet::Legal;
use super::lead_ext::axis_reseat;
use super::lead_fit::reseat;

/// The closest point of segment `a`-`b` to `q`, and its squared distance.
fn foot(a: Point2, b: Point2, q: Point2) -> (Point2, f64) {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let dd = dx * dx + dy * dy;
    let t = if dd > 0.0 { (((q.x - a.x) * dx + (q.y - a.y) * dy) / dd).clamp(0.0, 1.0) } else { 0.0 };
    let f = Point2::new(a.x + dx * t, a.y + dy * t);
    (f, (q.x - f.x) * (q.x - f.x) + (q.y - f.y) * (q.y - f.y))
}

/// The closest point to `q` on the closed ring `pts` (its last vertex
/// wraps to the first): `(segment index, point, squared distance)`, the
/// first segment winning a tie. `None` for an empty ring or a non-finite
/// answer.
fn nearest_on_ring(pts: &[Point2], q: Point2) -> Option<(usize, Point2, f64)> {
    let n = pts.len();
    let mut best: Option<(usize, Point2, f64)> = None;
    for i in 0..n {
        let (f, d2) = foot(pts[i], pts[(i + 1) % n], q);
        if d2.is_finite() && best.is_none_or(|b| d2 < b.2) {
            best = Some((i, f, d2));
        }
    }
    best
}

/// The farthest a pick may sit from the cutter-centre ring it starts:
/// the wall's own offset (radius plus stock to leave) and one tool
/// diameter of slack for a click beside the wall.
fn reach(p: &ContourParams) -> f64 {
    p.tool_diameter * 1.5 + p.stock_to_leave
}

/// Which ring (index into `loops`) takes the user's start point, with the
/// pick; `Ok(None)` when no start point is set. Refused by name: a pick
/// farther than [`reach`] from every ring, and the cases this first slice
/// does not certify - tabs (their windows are measured from the automatic
/// start) and compensation in the control (its seam is the comp law's).
pub(super) fn pick_ring(loops: &[&Loop], p: &ContourParams) -> Result<Option<(usize, Point2)>, GenError> {
    let ap = &p.links.approach;
    let Some(q) = ap.start_point else {
        return Ok(None);
    };
    let at = format!("start point ({:.3}, {:.3})", q.x, q.y);
    if p.tab_count > 0 {
        return Err(GenError::Invalid(format!(
            "{at}: a picked start is not taken with tabs yet - clear the start point or the tabs"
        )));
    }
    if ap.compensation.in_control() {
        return Err(GenError::Invalid(format!(
            "{at}: a picked start is not taken with {} compensation yet - its seam is the \
             compensation law's; clear the start point or compensate in the computer",
            ap.compensation.label()
        )));
    }
    let best = loops
        .iter()
        .enumerate()
        .filter_map(|(i, l)| nearest_on_ring(l.points(), q).map(|(_, _, d2)| (i, d2)))
        .min_by(|a, b| a.1.total_cmp(&b.1));
    let Some((i, d2)) = best else {
        return Err(GenError::Invalid(format!("{at}: no contour ring to start on")));
    };
    let d = d2.sqrt();
    let lim = reach(p);
    if d.is_nan() || d > lim {
        return Err(GenError::Invalid(format!(
            "{at} is {d:.3} mm from the nearest cutter path, farther than the {lim:.3} mm a \
             start point may sit from it - pick on or beside the wall to cut"
        )));
    }
    Ok(Some((i, q)))
}

/// `pts` (a closed ring, last vertex wrapping to the first) re-sequenced
/// to begin at the projection of `q` onto it: the same polygon, one
/// vertex added where the pick lands mid-edge. The ring constructor drops
/// the duplicate when the pick lands on a vertex.
pub(super) fn start_at(pts: &[Point2], q: Point2) -> Vec<Point2> {
    let Some((i, f, _)) = nearest_on_ring(pts, q) else {
        return pts.to_vec();
    };
    let n = pts.len();
    let mut out = Vec::with_capacity(n + 1);
    out.push(f);
    for k in 1..=n {
        out.push(pts[(i + k) % n]);
    }
    out
}

/// A ring's start: the pick's projection for the picked ring; otherwise
/// the automatic seam every earlier file cuts - an axis-aligned start
/// (M11.3 C), the control-compensated mid-run seam (M13.11), then the
/// lead room's own reseat; tabbed levels keep the start their windows are
/// measured from.
pub(super) fn seat(
    pts: Vec<Point2>,
    picked: Option<Point2>,
    tabbed: bool,
    comp: bool,
    p: &ContourParams,
    legal: &Legal,
    notes: &mut Vec<String>,
) -> Vec<Point2> {
    if let Some(q) = picked {
        return start_at(&pts, q);
    }
    let ap = &p.links.approach;
    let pts = if !tabbed && ap.axis_start { axis_reseat(&pts, ap.axis_tol).unwrap_or(pts) } else { pts };
    let pts = if comp && !tabbed { contour_comp::seam_mid_run(&pts) } else { pts };
    if tabbed { pts } else { reseat(&pts, p, legal, notes).unwrap_or(pts) }
}

/// The picked start's lead law: a lead-in was asked for and the lead law
/// (`lead_fit`, shrinks included) found none that fits at the pick - the
/// automatic seam would move elsewhere, a pick may not, so it is refused
/// by name. `at` is the seam the pick projected to.
pub(super) fn lead_held(p: &ContourParams, at: Point2, lead: &[Seg]) -> Result<(), GenError> {
    let wanted = match p.lead {
        Lead::None => false,
        Lead::Linear { length } | Lead::Perpendicular { length } => length > 0.0,
        Lead::Arc { .. } | Lead::Tangential { .. } => true,
    };
    if wanted && lead.is_empty() {
        return Err(GenError::Invalid(format!(
            "start point ({:.3}, {:.3}): no lead-in fits the legal cutter region there - pick \
             another point on the wall, shorten the lead-in, or clear the start point",
            at.x, at.y
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sq() -> Vec<Point2> {
        vec![Point2::new(0.0, 0.0), Point2::new(10.0, 0.0), Point2::new(10.0, 10.0), Point2::new(0.0, 10.0)]
    }

    #[test]
    fn a_mid_edge_pick_inserts_its_foot_and_keeps_the_polygon() {
        let v = start_at(&sq(), Point2::new(12.0, 4.0));
        assert_eq!(v[0], Point2::new(10.0, 4.0));
        assert_eq!(&v[1..], &[Point2::new(10.0, 10.0), Point2::new(0.0, 10.0), Point2::new(0.0, 0.0), Point2::new(10.0, 0.0)]);
    }

    #[test]
    fn a_vertex_pick_starts_on_that_vertex() {
        let v = start_at(&sq(), Point2::new(11.0, 11.0));
        assert_eq!(v[0], Point2::new(10.0, 10.0));
        assert_eq!(v[1], Point2::new(10.0, 10.0), "the duplicate the ring constructor drops");
        assert_eq!(v.len(), 5);
    }
}
