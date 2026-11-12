//! The thread mill's grant in the solid law (#1071): a threaded hole's
//! model shows its MINOR cylinder (what CAD draws of a tapped bore), and
//! the helix cuts the thread depth past it, out to the major circle, by
//! design. Material between the two circles is the thread being cut, not
//! part the tool must keep. Specced from the ISO 724 minor/major relation
//! the generator cites (`machining2d::thread_mill`) read against our own
//! solid law ([`super::solid`]).
//!
//! The grant is narrow on purpose. A hit is excused only when BOTH hold:
//! - the facet is the thread's own material: the hole's minor wall
//!   (near-vertical, every vertex on the declared minor circle within
//!   [`WALL_BAND`]), or the face the hole opens through (a facet with a
//!   vertex on the minor circle, wholly at or above the hole's entry
//!   height, where the thread breaks out at its mouth). A floor, a wall of
//!   another radius, another hole, or anything below the entry that is
//!   not the wall is never excused;
//! - the swept cutter piece stays inside the major cylinder: the farther
//!   tip end's distance from the axis plus the atom's radius is at most
//!   the major radius plus the law's allowance.
//!
//! So a thread cut past its major circle, a thread declared wider than
//! the model's hole, and a plunge through the hole's floor all still
//! convict.

use crate::geo2d::Point2;
use crate::geometry::{Point3, Triangle};

/// How far a wall vertex may stand off the declared minor circle and
/// still be read as on it, mm, as a share of the radius with a floor:
/// the drill rim law's own band (`rim_is` in the operation layer).
const WALL_BAND_SHARE: f64 = 0.02;
const WALL_BAND: f64 = 0.1;

/// The most a wall facet's unit normal may lean out of the XY plane.
const WALL_NZ: f64 = 0.1;

/// One threaded hole: its axis in XY, the declared minor radius (the
/// model's wall), the major radius the helix is cut to, and the height
/// the hole opens at (its entry).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThreadBore {
    pub xy: Point2,
    pub minor: f64,
    pub major: f64,
    pub top: f64,
}

impl ThreadBore {
    fn rho(&self, p: Point3) -> f64 {
        (p.x - self.xy.x).hypot(p.y - self.xy.y)
    }

    /// Whether `t` is the thread's own material (module doc): the minor
    /// wall, or the face the hole opens through.
    fn threaded(&self, t: &Triangle) -> bool {
        let band = (WALL_BAND_SHARE * self.minor).max(WALL_BAND);
        let on = |p: Point3| (self.rho(p) - self.minor).abs() <= band;
        let ons = [t.a, t.b, t.c].into_iter().filter(|&p| on(p)).count();
        if ons == 3 {
            let (u, v) = (t.b - t.a, t.c - t.a);
            let n = u.cross(v);
            let len = n.dot(n).sqrt();
            if len > f64::EPSILON && (n.z / len).abs() <= WALL_NZ {
                return true;
            }
        }
        ons > 0 && t.a.z.min(t.b.z).min(t.c.z) >= self.top - WALL_BAND
    }

    /// Whether a disc of radius `rho` swept between tips `p` and `q`
    /// stays inside the major cylinder, `allow` granted. The farthest
    /// point of a segment from the axis is one of its ends.
    fn holds(&self, p: Point3, q: Point3, rho: f64, allow: f64) -> bool {
        self.rho(p).max(self.rho(q)) + rho <= self.major + allow
    }
}

/// Whether the hit of an upright cutter disc of radius `rho`, swept
/// between tips `p` and `q`, on facet `t` is a thread being cut.
pub(super) fn excused(bores: &[ThreadBore], t: &Triangle, p: Point3, q: Point3, rho: f64, allow: f64) -> bool {
    bores.iter().any(|b| b.minor.is_finite() && b.major >= b.minor && b.top.is_finite() && b.threaded(t) && b.holds(p, q, rho, allow))
}
