//! The exit of a cut (K5; the maintainer's answer on the record: the exit
//! is part of the entry stage): a lead-out in the plane where there is
//! room, then the retract. The lead-out starts at the end of the last cut
//! and stays where the tool cuts nothing - in cleared material, in the band
//! that the last cut swept, or in the air round the stock - so there is no
//! cut on the way out. The retract is at or above the material top, so no
//! rapid move runs in material. Where no lead-out fits, the tool goes
//! straight up from the end of the cut.
//! Provenance: the tangential lead-out of general machining practice
//! (RS-274 / ISO 6983 G17 G2/G3); our own design record
//! `docs/decisions/4024-path-kernel-design.md` "Stage 6: Entry and exit".

use super::air::arc_onto;
use super::proof::{Area, Lead};
use super::room::{ROOM_TOL, geo, legal_room, path_of, planar, unit};
use super::stage::EntryStage;
use crate::frame::PlanePoint;
use crate::plan::Cut;
use crate::region::KernelRegion;
use crate::verdict::{Refusal, Stage};
use crate::{Point2, Region};

/// The exit of a cut: an optional lead-out, then the retract height above
/// the plane.
#[derive(Clone, Debug, PartialEq)]
pub struct Exit {
    lead_out: Option<Lead>,
    retract: f64,
}

/// Refusal reason: an exit with a negative retract, or an empty lead-out.
pub const BAD_EXIT: &str = "the exit needs a finite retract of 0 or more and a lead-out of one or more points if any";
/// Refusal reason: a lead-out with no proof of its room.
pub const UNPROVED_LEAD_OUT: &str = "a lead-out needs the proof of its room: build it with Exit::proved";
/// Refusal reason: a lead-out that does not start at the end of the last cut.
pub const EXIT_OFF_CUT: &str = "the lead-out must start at the end of the last cut, at its depth";
/// Refusal reason: a retract below the material top.
pub const RAPID_IN_MATERIAL: &str = "the retract must be at or above the material top: no rapid move in material";

/// The end point of a cut and its direction of travel there.
fn end_of(last: &Cut) -> Option<(PlanePoint, Point2)> {
    let pts = last.curve().pts();
    let path = path_of(last);
    let end = if last.curve().closed() { *pts.first()? } else { *pts.last()? };
    let e = end.at();
    let dir = path.iter().rev().skip(1).find_map(|q| unit(Point2::new(e.x - q.x, e.y - q.y)))?;
    Some((end, dir))
}

impl Exit {
    /// An exit with no lead-out: the tool goes straight up to `retract`. A
    /// lead-out needs its proof: `Some` is refused here ([`BAD_EXIT`] when
    /// empty, else [`UNPROVED_LEAD_OUT`]); use [`Exit::proved`].
    pub fn new(lead_out: Option<Vec<PlanePoint>>, retract: f64) -> Result<Self, Refusal> {
        let lead_ok = lead_out.as_ref().is_none_or(|l| !l.is_empty());
        if !retract.is_finite() || retract < 0.0 || !lead_ok {
            return Err(Refusal::new(Stage::Exit, BAD_EXIT));
        }
        if lead_out.is_some() {
            return Err(Refusal::new(Stage::Exit, UNPROVED_LEAD_OUT));
        }
        Ok(Self { lead_out: None, retract })
    }

    /// The checked exit of `last`: the lead-out (proved in its room by
    /// [`Lead::new`]) starts at the end of the cut, in place and depth, and
    /// the retract is finite and at or above the material `top`.
    pub fn proved(last: &Cut, lead_out: Option<Lead>, retract: f64, top: f64) -> Result<Self, Refusal> {
        if !(retract.is_finite() && top.is_finite() && retract >= 0.0) {
            return Err(Refusal::new(Stage::Exit, BAD_EXIT));
        }
        if retract < top {
            return Err(Refusal::new(Stage::Exit, RAPID_IN_MATERIAL));
        }
        if let Some(l) = &lead_out {
            let ok = end_of(last).zip(l.pts().first()).is_some_and(|((end, _), p)| p.near(&end));
            if !ok {
                return Err(Refusal::new(Stage::Exit, EXIT_OFF_CUT));
            }
        }
        Ok(Self { lead_out, retract })
    }
    #[must_use]
    pub fn lead_out(&self) -> Option<&[PlanePoint]> {
        self.lead_out.as_ref().map(Lead::pts)
    }
    #[must_use]
    pub fn retract(&self) -> f64 {
        self.retract
    }
}

/// The room where the tool centre cuts nothing after `last`: its disc stays
/// in the cleared material, the band that the last cut swept, or the air
/// round the stock; and the centre stays in the legal room.
fn free_room(last: &Cut, p: &crate::region::PlanarRegion, cleared: &Region) -> Result<Region, Refusal> {
    let r = p.tool().radius();
    let mut free = geo(Region::sweep(&[path_of(last)], r))?;
    if !cleared.is_empty() {
        free = geo(free.union(cleared))?;
    }
    if !p.air().is_empty() {
        free = geo(free.union(&geo(p.air().offset(r))?))?;
    }
    let free = geo(free.offset(-(r - ROOM_TOL).max(0.0)))?;
    geo(free.intersect(&legal_room(p)?))
}

/// The exit of `last`: a lead-out arc where it fits, at the asked radius or
/// a smaller one, on the side where the tool cuts nothing; else none.
pub(super) fn plan(stage: &EntryStage, last: &Cut, r: &KernelRegion, cleared: &Region) -> Result<Exit, Refusal> {
    let p = planar(r)?;
    let (retract, top) = (stage.heights().retract(), stage.heights().top());
    let Some((end, dir)) = end_of(last) else {
        return Exit::proved(last, None, retract, top);
    };
    let Ok(free) = Area::new(free_room(last, p, cleared)?) else {
        return Exit::proved(last, None, retract, top);
    };
    let arc = stage.arc();
    // The lead-out is the lead-in reversed: an arc onto the end point,
    // tangent to the reversed direction, walked backwards.
    let back = Point2::new(-dir.x, -dir.y);
    let mut radius = arc.radius();
    for _ in 0..5 {
        for ccw in [true, false] {
            let (_, _, mut xy) = arc_onto(end.at(), back, radius, arc.sweep(), ccw, stage.chord_tol())?;
            xy.reverse();
            let pts = xy.iter().map(|q| PlanePoint::new(*q, end.depth())).collect::<Result<Vec<_>, _>>()?;
            if let Ok(lead) = Lead::new(pts, &free) {
                return Exit::proved(last, Some(lead), retract, top);
            }
        }
        radius *= 0.5;
    }
    Exit::proved(last, None, retract, top)
}
