//! Stage 2, Guide curves (K2): the guide choice, the curve type and the
//! canonical guide set. This step holds the types and their checked
//! constructors; K2 adds the ring, row, medial, trace and helix planners.
//! Provenance: our own design record `docs/decisions/4024-path-kernel-design.md`
//! "Stage 2: Guide curves"; contour-parallel and direction-parallel paths as
//! in Held (1991).

use crate::frame::PlanePoint;
use crate::region::KernelRegion;
use crate::verdict::{Refusal, Stage, Verdict};
use crate::{KERNEL_TOL, Point2, finite};

/// What the rings are offset from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RingRef {
    /// The selected contour.
    Part,
    /// All the walls of the area.
    Boundary,
    /// The band between the part and the stock edge.
    Band,
}

/// An exact circle, for G2/G3 rings (Circular, Bore).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Circle {
    center: Point2,
    radius: f64,
}

/// Refusal reason: a circle with no positive finite radius.
pub const BAD_CIRCLE: &str = "a circle needs a finite centre and a positive finite radius";

impl Circle {
    pub fn new(center: Point2, radius: f64) -> Result<Self, Refusal> {
        if !finite(&[center.x, center.y, radius]) || radius <= 0.0 {
            return Err(Refusal::new(Stage::Guide, BAD_CIRCLE));
        }
        Ok(Self { center, radius })
    }
    #[must_use]
    pub fn center(&self) -> Point2 {
        self.center
    }
    #[must_use]
    pub fn radius(&self) -> f64 {
        self.radius
    }
}

/// The guide of a strategy.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum GuideChoice {
    Rings { reference: RingRef, first_offset: f64, stepover: f64, circle: Option<Circle>, count: Option<u32> },
    Rows { angle: f64, stepover: f64, closing_ring: bool },
    /// The centre line of a slot or a channel.
    Medial,
    /// The selection itself (engrave, trace, chamfer), at an offset.
    Trace { offset: f64 },
}

/// Refusal reason: a stepover that is not positive and finite.
pub const BAD_STEPOVER: &str = "the stepover must be a positive finite distance";
/// Refusal reason: an offset or an angle that is not finite.
pub const BAD_OFFSET: &str = "an offset or an angle of the guide is not a finite number";
/// Refusal reason: a ring count of zero.
pub const ZERO_COUNT: &str = "a ring count must be one or more";

impl GuideChoice {
    /// Checks the choice when the recipe is made (validate early).
    pub fn validate(&self) -> Result<(), Refusal> {
        let bad = |r| Err(Refusal::new(Stage::Recipe, r));
        match self {
            Self::Rings { first_offset, stepover, count, .. } => {
                if !first_offset.is_finite() {
                    return bad(BAD_OFFSET);
                }
                if !stepover.is_finite() || *stepover <= 0.0 {
                    return bad(BAD_STEPOVER);
                }
                if *count == Some(0) {
                    return bad(ZERO_COUNT);
                }
            }
            Self::Rows { angle, stepover, .. } => {
                if !angle.is_finite() {
                    return bad(BAD_OFFSET);
                }
                if !stepover.is_finite() || *stepover <= 0.0 {
                    return bad(BAD_STEPOVER);
                }
            }
            Self::Medial => {}
            Self::Trace { offset } if !offset.is_finite() => return bad(BAD_OFFSET),
            Self::Trace { .. } => {}
        }
        Ok(())
    }
}

/// The family of a curve, for the canonical sort.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Family {
    Ring,
    Row,
    Medial,
    Trace,
    Helix,
}

/// One guide curve: points with a depth along the plane normal, an exact
/// circle where it has one, closed or open, its index `k` and its family.
#[derive(Clone, Debug, PartialEq)]
pub struct Curve {
    pts: Vec<PlanePoint>,
    arc: Option<Circle>,
    closed: bool,
    k: u32,
    family: Family,
}

/// Refusal reason: a curve with too few points or a point that is not finite.
pub const BAD_CURVE: &str = "a curve needs finite points: two or more when open, three or more when closed";

impl Curve {
    pub fn new(pts: Vec<PlanePoint>, arc: Option<Circle>, closed: bool, k: u32, family: Family) -> Result<Self, Refusal> {
        let need = if closed { 3 } else { 2 };
        if pts.len() < need || !pts.iter().all(PlanePoint::is_finite) {
            return Err(Refusal::new(Stage::Guide, BAD_CURVE));
        }
        Ok(Self { pts, arc, closed, k, family })
    }
    #[must_use]
    pub fn pts(&self) -> &[PlanePoint] {
        &self.pts
    }
    #[must_use]
    pub fn arc(&self) -> Option<Circle> {
        self.arc
    }
    #[must_use]
    pub fn closed(&self) -> bool {
        self.closed
    }
    #[must_use]
    pub fn k(&self) -> u32 {
        self.k
    }
    #[must_use]
    pub fn family(&self) -> Family {
        self.family
    }

    /// A closed curve starts at its lowest point (least y, then least x).
    fn canonical(mut self) -> Self {
        if self.closed {
            let low = (0..self.pts.len()).min_by(|&a, &b| {
                let (p, q) = (self.pts[a].at, self.pts[b].at);
                p.y.total_cmp(&q.y).then(p.x.total_cmp(&q.x))
            });
            self.pts.rotate_left(low.unwrap_or(0));
        }
        self
    }

    /// The same curve as `other` as a point set within the kernel
    /// tolerance: same points, in any start and either direction.
    #[must_use]
    pub fn same_points(&self, other: &Self) -> bool {
        let key = |c: &Self| {
            let mut v: Vec<(f64, f64, f64)> = c.pts.iter().map(|p| (p.at.x, p.at.y, p.depth)).collect();
            v.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)).then(a.2.total_cmp(&b.2)));
            v
        };
        let (a, b) = (key(self), key(other));
        a.len() == b.len()
            && a.iter().zip(&b).all(|(p, q)| (p.0 - q.0).abs() <= KERNEL_TOL && (p.1 - q.1).abs() <= KERNEL_TOL && (p.2 - q.2).abs() <= KERNEL_TOL)
    }
}

/// A set of guide curves: no order and no start point (they are stage 4).
/// The constructor makes the set canonical: sorted by `k`, then by family,
/// each closed curve starting at its lowest point.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GuideSet {
    curves: Vec<Curve>,
}

impl GuideSet {
    #[must_use]
    pub fn new(curves: Vec<Curve>) -> Self {
        let mut curves: Vec<Curve> = curves.into_iter().map(Curve::canonical).collect();
        curves.sort_by_key(|c| (c.k, c.family));
        Self { curves }
    }
    #[must_use]
    pub fn curves(&self) -> &[Curve] {
        &self.curves
    }
}

/// A guide planner: a pure function of the region and the choice (law 1).
pub trait GuidePlanner {
    fn guide(&self, r: &KernelRegion, g: &GuideChoice, tol: f64) -> Result<Verdict<GuideSet>, Refusal>;
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn pts(xy: &[(f64, f64)]) -> Vec<PlanePoint> {
        xy.iter().map(|&(x, y)| PlanePoint::flat(Point2::new(x, y))).collect()
    }

    #[test]
    fn a_guide_choice_is_checked_when_the_recipe_is_made() {
        let rings = |stepover, count| GuideChoice::Rings { reference: RingRef::Part, first_offset: 0.0, stepover, circle: None, count };
        assert!(rings(1.0, None).validate().is_ok());
        assert_eq!(rings(0.0, None).validate().unwrap_err().reason, BAD_STEPOVER);
        assert_eq!(rings(1.0, Some(0)).validate().unwrap_err().reason, ZERO_COUNT);
        assert_eq!(GuideChoice::Rows { angle: f64::NAN, stepover: 1.0, closing_ring: false }.validate().unwrap_err().reason, BAD_OFFSET);
        assert!(GuideChoice::Trace { offset: f64::INFINITY }.validate().is_err());
        assert!(GuideChoice::Medial.validate().is_ok());
        assert!(Circle::new(Point2::new(0.0, 0.0), 0.0).is_err());
    }

    #[test]
    fn a_curve_needs_enough_finite_points() {
        assert_eq!(Curve::new(pts(&[(0.0, 0.0), (1.0, 0.0)]), None, true, 0, Family::Ring).unwrap_err().reason, BAD_CURVE);
        assert!(Curve::new(pts(&[(0.0, 0.0), (1.0, 0.0)]), None, false, 0, Family::Trace).is_ok());
        assert!(Curve::new(pts(&[(0.0, 0.0), (f64::NAN, 0.0)]), None, false, 0, Family::Trace).is_err());
    }

    #[test]
    fn a_guide_set_is_canonical() {
        let a = Curve::new(pts(&[(1.0, 1.0), (0.0, 0.0), (1.0, 0.0)]), None, true, 2, Family::Ring).unwrap();
        let b = Curve::new(pts(&[(0.0, 5.0), (2.0, 5.0)]), None, false, 0, Family::Row).unwrap();
        let c = Curve::new(pts(&[(0.0, 9.0), (2.0, 9.0)]), None, false, 0, Family::Ring).unwrap();
        let s = GuideSet::new(vec![a.clone(), b, c]);
        let order: Vec<(u32, Family)> = s.curves().iter().map(|c| (c.k(), c.family())).collect();
        assert_eq!(order, [(0, Family::Ring), (0, Family::Row), (2, Family::Ring)]);
        assert_eq!(s.curves()[2].pts()[0].at, Point2::new(0.0, 0.0));
        assert!(s.curves()[2].same_points(&a));
        // The same input in another order and from another start gives the same set (law 1).
        let a2 = Curve::new(pts(&[(1.0, 0.0), (1.0, 1.0), (0.0, 0.0)]), None, true, 2, Family::Ring).unwrap();
        assert_eq!(GuideSet::new(vec![a2]).curves()[0], GuideSet::new(vec![a]).curves()[0]);
    }
}
