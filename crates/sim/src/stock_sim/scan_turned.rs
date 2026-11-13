//! The turned reading of the hidden-rest scan (#1408): a turning nose
//! path's hidden tops judged as the turning work sees them.
//! Provenance: rotation of a point about a line (elementary solid
//! geometry) and the lathe's own working - the work turns under an
//! insert whose nose travels only the axial band its path walks
//! (machine-tool handbooks, turning chapters); nothing from vendor
//! binaries.
//!
//! ## The law here
//!
//! - **The work turns.** A lathe path is written in ONE meridian
//!   plane of the spindle (`turning::Frame`), but the material it cuts
//!   is every point whose orbit about the spindle meets the nose. A
//!   hidden top across the axis from that plane was cut by the same
//!   nose half a turn later, so its distance to the path is read from
//!   its orbit, not from where the facets happen to put it. The orbit
//!   is sampled at the top itself and at its two turns into the path's
//!   plane (both sides of the axis): each sample is a point of the
//!   orbit, so the least distance read can only be at or ABOVE the
//!   orbit's true least - an over-read rest (a refusal), never a
//!   hidden one, even on a path that left its plane.
//! - **A band the nose never travels is not its rest.** A hidden top
//!   standing farther along the spindle than every nose centre of the
//!   path, by more than the nose and the scan's floor (and the arcs'
//!   bulge), is out of the reach of every ball of this pass - it is
//!   another operation's (a groove's floor below a facing; a groove
//!   above a profile window). Excused here, by name in the law; a top
//!   inside the band is judged by the same distance law as ever.
//! - **A crease narrower than the nose is not its rest.** A hidden top
//!   the nose cannot touch at all - the ball tangent there along the
//!   top's normal would cut a neighbouring face, a groove floor's inner
//!   corner under a 0.8 nose - is excused by the ridge law's own
//!   accessibility test (`ridge::Access`), the same excuse the visible
//!   half gives a flanked crease. A top the nose CAN touch is judged.
//! - **The leave is the promise.** A rough told to stand `leave` off
//!   the part rolls a ball `r + leave` wide as far as the hidden half
//!   is concerned: a top standing up to `leave` outside the nose is the
//!   allowance the finish owns (what the visible half's band already
//!   lets stand), and the accessibility test above reads the grown
//!   ball - a groove the rough cannot enter WITH its leave is the
//!   finish's. A top past the leave convicts as ever.
//! - **Mills read as before.** Without a spindle (every mill path,
//!   and a turning path no axis fits) neither clause runs, bit for bit.

use super::field_lathe::LatheAxis;
use super::ridge::Access;

use crate::geometry::{Point3, Triangle};

/// A turning path's spindle frame as the hidden-rest scan reads it:
/// the axis, the stations its nose centres span, the radial
/// direction of the plane they stand in, and the nose-accessibility
/// reader of the part.
pub(super) struct TurnedBand<'a> {
    axis: LatheAxis,
    access: Access<'a>,
    /// The nose radius grown by the leave the op is told to stand,
    /// the leave itself, and the accessibility slack (the op's
    /// tolerance).
    r: f64,
    leave: f64,
    slack: f64,
    /// The least and greatest station of any centre-path end.
    s0: f64,
    s1: f64,
    /// The unit radial of the path's own meridian plane; `None` when
    /// every centre stands on the axis (only the top itself is read).
    radial: Option<[f64; 3]>,
}

impl<'a> TurnedBand<'a> {
    /// The band of `segs` about `axis`, a nose of radius `r` that
    /// stands `leave` off the part, judged at `tol`; `None` when no
    /// segment end is finite (the scan then reads as a mill's).
    pub(super) fn of(
        axis: LatheAxis,
        segs: &[(Point3, Point3)],
        facets: &'a [Triangle],
        (r, leave, tol): (f64, f64, f64),
    ) -> Option<Self> {
        let (mut s0, mut s1) = (f64::INFINITY, f64::NEG_INFINITY);
        let mut radial = None;
        for q in segs.iter().flat_map(|(a, b)| [*a, *b]) {
            if !q.is_finite() {
                continue;
            }
            let (w, s) = axis.split(q);
            s0 = s0.min(s);
            s1 = s1.max(s);
            let n = (w[0] * w[0] + w[1] * w[1] + w[2] * w[2]).sqrt();
            if radial.is_none() && n.is_finite() && n > 1e-9 {
                radial = Some([w[0] / n, w[1] / n, w[2] / n]);
            }
        }
        if !(s0.is_finite() && s1.is_finite()) {
            return None;
        }
        let leave = if leave.is_finite() { leave.max(0.0) } else { 0.0 };
        Some(TurnedBand { axis, access: Access::new(facets, r + leave), r: r + leave, leave, slack: tol, s0, s1, radial })
    }

    /// Whether the nose can touch the hidden top `p` at all: the ball
    /// tangent there along the top's upward normal stands clear of the
    /// part (`ridge::Access`, the ridge law's own accessibility test).
    /// `false` is a crease narrower than the nose - a groove's inner
    /// corner - that no turning pass of this insert reaches (module
    /// law).
    pub(super) fn touchable(&self, p: Point3) -> bool {
        self.access.reachable(p, [0.0, 0.0, 1.0], self.r, self.slack)
    }

    /// The leave the op is told to stand (a rough's finish allowance):
    /// a hidden top that far outside the nose is the promise kept, the
    /// finish pass's to take (module law).
    pub(super) fn leave(&self) -> f64 {
        self.leave
    }

    /// Whether a ball of radius `reach` on some centre of the path
    /// could stand level with `p` along the spindle; `false` is the
    /// module law's excuse.
    pub(super) fn reaches(&self, p: Point3, reach: f64) -> bool {
        let (_, s) = self.axis.split(p);
        // A non-finite station is no proof of being out of reach.
        !s.is_finite() || (s >= self.s0 - reach && s <= self.s1 + reach)
    }

    /// `p`'s orbit samples in the path's meridian plane, both sides
    /// of the axis (the module law); empty without a plane.
    pub(super) fn turned(&self, p: Point3) -> Vec<Point3> {
        let Some(u) = self.radial else { return Vec::new() };
        let (w, s) = self.axis.split(p);
        let rho = (w[0] * w[0] + w[1] * w[1] + w[2] * w[2]).sqrt();
        if !(rho.is_finite() && s.is_finite()) {
            return Vec::new();
        }
        let (o, d) = (self.axis.origin(), self.axis.dir());
        let at = |k: f64| {
            Point3::new(
                o.x + d[0] * s + u[0] * k,
                o.y + d[1] * s + u[1] * k,
                o.z + d[2] * s + u[2] * k,
            )
        };
        vec![at(rho), at(-rho)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_band_spans_the_centres_and_the_orbit_lands_in_their_plane() {
        let segs = [(Point3::new(10.0, 0.0, 48.8), Point3::new(0.0, 0.0, 48.8)), (Point3::new(10.0, 0.0, 50.3), Point3::new(0.0, 0.0, 50.3))];
        let b = TurnedBand::of(LatheAxis::Z, &segs, &[], (0.8, 0.0, 0.05)).expect("a band");
        assert!(b.reaches(Point3::new(-7.0, 0.0, 48.0), 0.81), "level with the band, either side of the axis");
        assert!(!b.reaches(Point3::new(-7.0, 0.0, 34.0), 0.81), "a groove floor 14 mm under the band is out of reach");
        assert!(!b.reaches(Point3::new(0.0, 7.0, 52.0), 0.81), "and so is a top above it");
        let t = b.turned(Point3::new(0.0, -7.0, 34.0));
        assert_eq!(t.len(), 2);
        assert!((t[0].x - 7.0).abs() < 1e-12 && t[0].y.abs() < 1e-12 && (t[0].z - 34.0).abs() < 1e-12, "{t:?}");
        assert!((t[1].x + 7.0).abs() < 1e-12, "{t:?}");
    }

    #[test]
    fn a_path_on_the_axis_has_no_plane_and_no_finite_end_no_band() {
        let on = [(Point3::new(0.0, 0.0, 1.0), Point3::new(0.0, 0.0, 2.0))];
        let b = TurnedBand::of(LatheAxis::Z, &on, &[], (0.8, 0.0, 0.05)).expect("a band");
        assert!(b.turned(Point3::new(3.0, 0.0, 1.5)).is_empty());
        let nan = [(Point3::new(f64::NAN, 0.0, 1.0), Point3::new(0.0, f64::NAN, 2.0))];
        assert!(TurnedBand::of(LatheAxis::Z, &nan, &[], (0.8, 0.0, 0.05)).is_none());
    }
}
