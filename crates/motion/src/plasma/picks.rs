//! The plasma cut's start and exit picks (#3334, design #1448): which
//! closed contour takes a pick, where the pick lands on its profile, and
//! the run of the cut past its start to the exit. A pick is a wish: a pick
//! that no closed profile is near, or that a law cannot hold, is refused
//! by name - it is never moved.
//!
//! Provenance: first principles - the closest point of a polyline to a
//! point (orthogonal projection clamped to each segment); the chosen
//! pierce and exit position is general plasma practice (the pierce mark
//! and the lead belong in the scrap). Our own code, no vendor material.

use crate::cl::ContourRole;
use crate::geo2d::Point2;
use crate::machining2d::GenError;
use crate::path_picks::{PathPicks, PickRole};
use crate::sheet::nest::Contour;
use crate::sheet::ring::{dist, nearest};
use crate::sheet::Voice;

use super::PlasmaParams;

/// Why an exit pick is refused under control compensation: the control
/// builds the G40 cancel over the programmed contour, and that contour
/// ends at the seam. The application's door says the same words.
pub const EXIT_WITH_CONTROL_COMP: &str = "an exit point is not taken with control compensation (G41/G42) yet - the programmed contour ends at its start; clear the exit point or compensate in the computer";

/// How far (in kerf widths) a pick may sit from the profile it lands on:
/// the 2D contour's measure (#1448), one and a half tool diameters.
pub const PICK_REACH_KERFS: f64 = 1.5;

/// A matched pick: the point as the user gave it (the refusals name it)
/// and its projection on the profile of its contour.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Seat {
    pub pick: Point2,
    pub foot: Point2,
}

impl Seat {
    /// The pick's name in a refusal or a note: `start point (x, y)`.
    pub(super) fn at(&self, role: PickRole) -> String {
        name(role, self.pick)
    }
}

fn name(role: PickRole, q: Point2) -> String {
    format!("{} point ({:.3}, {:.3})", role.word(), q.x, q.y)
}

/// What the picks ask of one contour. The default asks nothing: the
/// automatic seam of every earlier file.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Ask {
    pub start: Option<Seat>,
    pub exit: Option<Seat>,
}

/// Give each pick to the closed contour whose profile is nearest: one
/// [`Ask`] for each contour of `cs`. No pick answers the default for all
/// of them. Refused by name: a pick that no planner can read, an exit
/// pick under control compensation, a pick with no closed contour to
/// land on, and a pick farther than the reach from each profile.
pub(super) fn assign(cs: &[Contour], picks: &PathPicks, p: &PlasmaParams) -> Result<Vec<Ask>, GenError> {
    let mut asks = vec![Ask::default(); cs.len()];
    if picks.is_empty() {
        return Ok(asks);
    }
    let invalid = |why: String| Voice::PLASMA.invalid(why);
    if let Some(why) = picks.refusal() {
        return Err(invalid(why));
    }
    let reach = PICK_REACH_KERFS * p.kerf;
    for role in [PickRole::Start, PickRole::Exit] {
        let Some(q) = picks.first(role).map(|k| k.xy) else {
            continue;
        };
        let (at, what) = (name(role, q), role.word());
        if role == PickRole::Exit && p.control_comp {
            return Err(invalid(format!("{at}: {EXIT_WITH_CONTROL_COMP}")));
        }
        let best = cs
            .iter()
            .enumerate()
            .filter(|(_, c)| c.role != ContourRole::Open && !c.pts.is_empty())
            .map(|(k, c)| {
                let foot = nearest(&c.pts, q).1;
                (k, foot, dist(foot, q))
            })
            .filter(|(_, _, d)| d.is_finite())
            .min_by(|a, b| a.2.total_cmp(&b.2));
        let Some((k, foot, d)) = best else {
            return Err(invalid(format!(
                "{at}: a {what} point is taken on a closed contour only - an open run is pierced at its first point; clear the {what} point"
            )));
        };
        if d > reach {
            return Err(invalid(format!(
                "{at} is {d:.3} mm from the nearest closed profile, farther than the {reach:.3} mm a {what} point may sit from it - pick on the profile to cut, or clear the {what} point"
            )));
        }
        let seat = Some(Seat { pick: q, foot });
        if let Some(ask) = asks.get_mut(k) {
            match role {
                PickRole::Start => ask.start = seat,
                PickRole::Exit => ask.exit = seat,
            }
        }
    }
    Ok(asks)
}

/// The run of the cut past its start to `e`: the points of the closed
/// `walk` (start .. back to the start) after its first point, up to the
/// point of the walk nearest `e`. Empty when `e` is the start itself.
/// The nearest stretch wins and the first one wins a tie, so an exit at
/// the start adds no second lap.
pub(super) fn run_on(walk: &[Point2], e: Point2) -> Vec<Point2> {
    // Under this distance two points of a walk are one (`ring::walk_from`).
    const SAME: f64 = 1e-9;
    let mut best: Option<(usize, Point2, f64)> = None;
    for (i, w) in walk.windows(2).enumerate() {
        let (a, b) = (w[0], w[1]);
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let dd = dx * dx + dy * dy;
        let t = if dd > 0.0 { (((e.x - a.x) * dx + (e.y - a.y) * dy) / dd).clamp(0.0, 1.0) } else { 0.0 };
        let f = Point2::new(a.x + t * dx, a.y + t * dy);
        let d = dist(f, e);
        if d.is_finite() && best.is_none_or(|(_, _, bd)| d < bd - SAME) {
            best = Some((i, f, d));
        }
    }
    let Some((i, f, _)) = best else {
        return Vec::new();
    };
    let mut out: Vec<Point2> = walk[1..=i].to_vec();
    let last = out.last().or(walk.first()).copied();
    if last.is_none_or(|q| dist(q, f) > SAME) {
        out.push(f);
    }
    out
}

/// The length of the stretch `from` .. `pts`, mm.
pub(super) fn length(from: Point2, pts: &[Point2]) -> f64 {
    let mut at = from;
    let mut sum = 0.0;
    for &q in pts {
        sum += dist(at, q);
        at = q;
    }
    sum
}
