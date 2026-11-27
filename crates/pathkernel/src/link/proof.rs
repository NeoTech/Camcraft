//! The link proof of the kernel (#4177, K4): a hop stays at depth when an
//! in-plane path is proved clear, else it lifts with the named reason of
//! the straight link. A path is clear when each of its chords stays in the
//! cutter-centre area or the air (so it cuts no part wall and no keep-out
//! area), crosses no cut of the level, and runs only over cleared stock
//! (within the swath of a cut already made) or the air of the region (the
//! band outside the stock where the tool does not touch it). The strip just
//! inside the stock edge is stock as any other: it is open only where the
//! swath of a cut already made covers it. The tries, in order:
//! the straight chord; back along the cut just made, then a chord (the
//! chain link of the air plan); a walk along an edge of the centre area.
//! Provenance: ported from `camcraft-motion` with its named reasons - the
//! chord proof of the ring links (`machining2d/ring_links.rs`, #3747,
//! #3822) and the chain link of the Adaptive air plan
//! (`machining2d/adaptive/air_chain.rs`, `link`; the #3934 decision of our
//! design record `docs/decisions/4024-path-kernel-design.md`); stay-down
//! linking as in Held & Spielberger (2009).

use camcraft_geometry::geo2d::{point_segment_distance, segment_gap, segments_cross};

use super::{BAD_PATH, CROSSES_CUT, CUTS_PART, HITS_FIXTURE, Hop, LiftReason, LinkPolicy, LinkProof, LinkTally, Level, NOT_PLANAR_LINK, UNCUT_STOCK, USER_RETRACT};
use crate::plan::Cut;
use crate::region::{KernelRegion, PlanarRegion};
use crate::{Point2, in_area, near, to_loops};

/// The part of the tool radius that a link may stand from a cut already
/// made and still run over cleared stock (the ring links' swath).
pub const SWATH: f64 = 0.9;
/// The sample step of a chord, in tool radii (the ring links' step).
const STEP: f64 = 0.25;
/// Two segments closer than this (mm) meet.
const APART: f64 = 1e-6;

/// The kernel's link stage.
#[derive(Clone, Copy, Debug, Default)]
pub struct LinkStage;

/// A cut as a polyline (a closed cut repeats its start) and its bounds.
pub(super) struct Line {
    pts: Vec<Point2>,
    lo: Point2,
    hi: Point2,
}

impl Line {
    fn of(c: &Cut) -> Self {
        let mut pts: Vec<Point2> = c.curve().pts().iter().map(|p| p.at()).collect();
        if c.curve().closed()
            && let Some(&f) = pts.first()
        {
            pts.push(f);
        }
        Self::path(pts)
    }
    fn path(pts: Vec<Point2>) -> Self {
        let lo = pts.iter().fold(Point2::new(f64::INFINITY, f64::INFINITY), |m, p| Point2::new(m.x.min(p.x), m.y.min(p.y)));
        let hi = pts.iter().fold(Point2::new(f64::NEG_INFINITY, f64::NEG_INFINITY), |m, p| Point2::new(m.x.max(p.x), m.y.max(p.y)));
        Self { pts, lo, hi }
    }
    /// The distance from `q` to the line, or a bound above `cap` when the
    /// bounds are already farther than `cap`.
    fn dist(&self, q: Point2, cap: f64) -> f64 {
        let dx = (self.lo.x - q.x).max(q.x - self.hi.x).max(0.0);
        let dy = (self.lo.y - q.y).max(q.y - self.hi.y).max(0.0);
        if dx.hypot(dy) > cap {
            return f64::INFINITY;
        }
        self.pts.windows(2).map(|w| point_segment_distance(q, w[0], w[1])).fold(f64::INFINITY, f64::min)
    }
    /// True when the segment `a`-`b` meets the line. A cut already made is
    /// met only by a proper crossing (the verifier's crossing law): a link
    /// may run on cut ground and along a made cut. A planned cut is met by
    /// any touch away from the ends `ends` of the link (where the link
    /// joins its two cuts): a link that touches it cuts its material.
    fn meets(&self, a: Point2, b: Point2, ends: (Point2, Point2), made: bool) -> bool {
        let pad = APART;
        let off = a.x.min(b.x) > self.hi.x + pad || a.x.max(b.x) < self.lo.x - pad;
        if off || a.y.min(b.y) > self.hi.y + pad || a.y.max(b.y) < self.lo.y - pad {
            return false;
        }
        self.pts.windows(2).any(|w| {
            if made {
                return segments_cross(a, b, w[0], w[1]);
            }
            let joint = point_segment_distance(ends.0, w[0], w[1]) <= APART || point_segment_distance(ends.1, w[0], w[1]) <= APART;
            !joint && segment_gap(a, b, w[0], w[1]) <= APART
        })
    }
}

/// What a proof reads: the region, the tool radius, the cuts made and all
/// the cuts of the level (made first), and the two ends of the link.
pub(super) struct Proof<'a> {
    p: &'a PlanarRegion,
    r: f64,
    made: Vec<Line>,
    planned: Vec<Line>,
    links: Vec<Line>,
    ends: (Point2, Point2),
}

impl Proof<'_> {
    /// The samples of the chord `a`-`b`, both ends included.
    fn samples(&self, a: Point2, b: Point2) -> Vec<Point2> {
        let len = (b.x - a.x).hypot(b.y - a.y);
        let step = STEP * self.r;
        let n = if len.is_finite() && step > 0.0 { (len / step).ceil().clamp(1.0, 1e6) } else { 1.0 };
        let n = n as usize;
        (0..=n).map(|i| {
            let t = i as f64 / n as f64;
            Point2::new(a.x + t * (b.x - a.x), a.y + t * (b.y - a.y))
        })
        .collect()
    }

    /// The proof of one chord, `skip` the index of a made cut that the
    /// chord may run along (the cut it goes back on). The reason names the
    /// first test that fails.
    ///
    /// Open ground is only what is proved open: the air of the region, or a
    /// sample within the swath of a cut already made. A centre inside the
    /// stock near its edge is not open by itself: with no cut made beside
    /// it, its tool cuts more than the swath into uncut stock (`UNCUT_STOCK`).
    pub(super) fn chord(&self, a: Point2, b: Point2, skip: Option<usize>) -> Result<(), LiftReason> {
        if near(a, b) {
            return Ok(());
        }
        let pts = self.samples(a, b);
        let air = self.p.air();
        if !pts.iter().all(|&q| in_area(self.p.center(), q) || in_area(air, q)) {
            let keep = self.p.keep_out();
            let fixture = !keep.is_empty() && pts.iter().any(|&q| in_area(keep, q) || to_loops(q, keep.loops()) <= self.r);
            return Err(LiftReason(if fixture { HITS_FIXTURE } else { CUTS_PART }));
        }
        let made = self.made.iter().enumerate().filter(|(i, _)| Some(*i) != skip).chain(self.links.iter().enumerate()).any(|(_, l)| l.meets(a, b, self.ends, true));
        if made || self.planned.iter().any(|l| l.meets(a, b, self.ends, false)) {
            return Err(LiftReason(CROSSES_CUT));
        }
        let reach = SWATH * self.r;
        if !pts.iter().all(|&q| in_area(air, q) || self.made.iter().any(|l| l.dist(q, reach) <= reach)) {
            return Err(LiftReason(UNCUT_STOCK));
        }
        Ok(())
    }

    /// The proof of a path: each chord.
    pub(super) fn path(&self, pts: &[Point2]) -> Result<(), LiftReason> {
        pts.windows(2).try_for_each(|w| self.chord(w[0], w[1], None))
    }

    pub(super) fn region(&self) -> &PlanarRegion {
        self.p
    }
    pub(super) fn radius(&self) -> f64 {
        self.r
    }
}

fn start_of(c: &Cut) -> Option<Point2> {
    c.curve().pts().first().map(|p| p.at())
}

fn end_of(c: &Cut) -> Option<Point2> {
    if c.curve().closed() { c.curve().pts().first() } else { c.curve().pts().last() }.map(|p| p.at())
}

/// The in-plane path from the end of `from` to the start of `to`, or the
/// reason of the straight chord when no path is proved.
fn stay_path(from: &Cut, to: &Cut, p: &PlanarRegion, level: Level<'_>) -> Result<Vec<Point2>, LiftReason> {
    let (Some(a), Some(b)) = (end_of(from), start_of(to)) else {
        return Err(LiftReason(BAD_PATH));
    };
    let mut proof = Proof {
        p,
        r: p.tool().radius(),
        made: level.made().iter().map(Line::of).collect(),
        planned: level.planned().iter().map(Line::of).collect(),
        links: level.links().iter().map(|l| Line::path(l.clone())).collect(),
        ends: (a, b),
    };
    let why = match proof.chord(a, b, None) {
        Ok(()) => return Ok(vec![a, b]),
        Err(why) => why,
    };
    // Back along the open cut just made, then a chord to the next start
    // (the chain link of the air plan). The cut just made is the last one
    // made; the chord may start on it.
    let back: Vec<Point2> = from.curve().pts().iter().rev().map(|q| q.at()).collect();
    if let (false, Some(&s), Some(last)) = (from.curve().closed(), back.last(), proof.made.len().checked_sub(1)) {
        proof.ends = (s, b);
        let ok = proof.chord(s, b, Some(last)).is_ok();
        proof.ends = (a, b);
        if ok {
            let mut out = back;
            out.push(b);
            return Ok(out);
        }
    }
    super::route::edge_route(&proof, a, b).ok_or(why)
}

/// The in-plane path that the link stage proves from `from` to `to`, if
/// any (the order stage reads it to pick a start that the link can reach).
pub(crate) fn proved_path(from: &Cut, to: &Cut, p: &PlanarRegion, level: Level<'_>) -> Option<Vec<Point2>> {
    stay_path(from, to, p, level).ok()
}

impl LinkProof for LinkStage {
    fn link(&self, from: &Cut, to: &Cut, r: &KernelRegion, level: Level<'_>, policy: LinkPolicy, tally: &mut LinkTally) -> Hop {
        // Each lift reason is a named constant of `link.rs`, so a lift can
        // not be blank; each case has its own arm and no fallback.
        let path = match (policy, r.planar()) {
            (LinkPolicy::StayWhereProved, Some(p)) => stay_path(from, to, p, level),
            (LinkPolicy::StayWhereProved, None) => Err(LiftReason(NOT_PLANAR_LINK)),
            (LinkPolicy::Retract, _) => Err(LiftReason(USER_RETRACT)),
        };
        let hop = match path.map(Hop::stay) {
            Ok(Ok(hop)) => hop,
            Ok(Err(_)) => Hop::Lift(LiftReason(BAD_PATH)),
            Err(why) => Hop::Lift(why),
        };
        tally.record(hop)
    }
}
