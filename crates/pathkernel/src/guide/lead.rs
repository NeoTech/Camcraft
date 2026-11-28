//! The lead of a helix curve (#4176): its pitch and its hand, and the
//! checked constructor of a helix curve, so that a helix is valid by
//! construction (its points on its circle, its depth along its pitch).
//! Provenance: the circular helix of the machine-tool handbooks (one turn
//! for each pitch along the axis); our own design record
//! (`RECOMMENDED.md`, `## #4024`) "Stage 2: Guide curves".

use std::cmp::Ordering;

use super::{Circle, Curve, Family, Hand};
use crate::frame::{PlanePoint, Positive};
use crate::verdict::{Refusal, Stage};
use crate::KERNEL_TOL;

/// Refusal reason: a curve of the helix family made without a lead.
pub const HELIX_NEEDS_LEAD: &str = "a helix curve needs its circle and its lead: make it with Curve::helix";
/// Refusal reason: a helix point off its circle or off its pitch.
pub const NOT_ON_HELIX: &str = "a helix point is not on its circle at the depth of its pitch and hand";

/// The pitch (the descent along the plane normal in one turn) and the hand
/// of a helix.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lead {
    pitch: Positive,
    hand: Hand,
}

impl Lead {
    pub fn new(pitch: f64, hand: Hand) -> Result<Self, Refusal> {
        Ok(Self { pitch: Positive::new(Stage::Guide, pitch)?, hand })
    }
    #[must_use]
    pub fn pitch(&self) -> f64 {
        self.pitch.get()
    }
    #[must_use]
    pub fn hand(&self) -> Hand {
        self.hand
    }
}

pub(super) fn cmp_lead(a: Option<Lead>, b: Option<Lead>) -> Ordering {
    let key = |l: Option<Lead>| l.map(|l| (l.hand, l.pitch()));
    match (key(a), key(b)) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
        (Some(x), Some(y)) => x.0.cmp(&y.0).then(x.1.total_cmp(&y.1)),
    }
}

pub(super) fn same_lead(a: Option<Lead>, b: Option<Lead>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(x), Some(y)) => x.hand == y.hand && (x.pitch() - y.pitch()).abs() <= KERNEL_TOL,
        _ => false,
    }
}

impl Curve {
    /// An open helix curve of family [`Family::Helix`] on `circle`. Refused
    /// unless each point is on the circle and each step turns by less than
    /// half a turn and changes the depth by `pitch * turn / (2 pi)`: up for a
    /// counter-clockwise step of a right hand, down for a clockwise one, the
    /// other way for a left hand (all within [`KERNEL_TOL`]).
    pub fn helix(circle: Circle, lead: Lead, pts: Vec<PlanePoint>, k: u32) -> Result<Self, Refusal> {
        let (c, r) = (circle.center(), circle.radius());
        let off = |p: &PlanePoint| ((p.at().x - c.x).hypot(p.at().y - c.y) - r).abs() > KERNEL_TOL;
        if pts.iter().any(off) {
            return Err(Refusal::new(Stage::Guide, NOT_ON_HELIX).at(pts.iter().find(|p| off(p)).map_or(c, PlanePoint::at)));
        }
        let sign = match lead.hand {
            Hand::Right => 1.0,
            Hand::Left => -1.0,
        };
        for w in pts.windows(2) {
            let (a, b) = ((w[0].at().x - c.x, w[0].at().y - c.y), (w[1].at().x - c.x, w[1].at().y - c.y));
            let turn = (a.0 * b.1 - a.1 * b.0).atan2(a.0 * b.0 + a.1 * b.1);
            let rise = sign * lead.pitch() * turn / std::f64::consts::TAU;
            let bad = turn.abs() >= std::f64::consts::PI - KERNEL_TOL || (w[1].depth() - w[0].depth() - rise).abs() > KERNEL_TOL;
            if bad {
                return Err(Refusal::new(Stage::Guide, NOT_ON_HELIX).at(w[1].at()));
            }
        }
        Self::checked(pts, Some(circle), false, k, Family::Helix, Some(lead))
    }
}
