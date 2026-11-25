//! The frame the stages plan in: the work plane, the tool axis, the tool
//! shape and a point with a depth along the plane normal (record "What
//! phase 1 keeps open", points 1, 2, 3 and 8).
//! Provenance: our own design record `docs/decisions/4024-path-kernel-design.md`;
//! right-handed orthonormal frames as in any machine-tool handbook (ISO 841
//! axis naming).

use crate::verdict::{Refusal, Stage};
use crate::{KERNEL_TOL, Point2, finite};

/// The plane that the stages plan in. No stage reads "XY" or "+Z"
/// directly: a later phase adds the lathe's Z-X plane or a wrapped plane
/// as a new variant.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum WorkPlane {
    /// A flat plane: an origin, a unit normal (the tool side) and a unit
    /// X direction normal to it. Phase 1: the setup's XY plane, or its 3+2
    /// tilt.
    Planar { origin: [f64; 3], normal: [f64; 3], x_dir: [f64; 3] },
}

/// Refusal reason: a work plane whose axes are not finite unit vectors
/// at right angles.
pub const BAD_PLANE: &str = "the work plane needs a finite origin and two unit axes at right angles";

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// True for a finite vector of length 1 (within the kernel tolerance).
pub(crate) fn unit(v: [f64; 3]) -> bool {
    finite(&v) && (dot(v, v).sqrt() - 1.0).abs() <= KERNEL_TOL
}

impl WorkPlane {
    /// A planar frame. Refused unless `normal` and `x_dir` are unit vectors
    /// at right angles and every value is finite.
    pub fn planar(origin: [f64; 3], normal: [f64; 3], x_dir: [f64; 3]) -> Result<Self, Refusal> {
        if !finite(&origin) || !unit(normal) || !unit(x_dir) || dot(normal, x_dir).abs() > KERNEL_TOL {
            return Err(Refusal::new(Stage::Recipe, BAD_PLANE));
        }
        Ok(Self::Planar { origin, normal, x_dir })
    }

    /// The setup's own XY plane at height `z`, the tool along +Z.
    pub fn setup_xy(z: f64) -> Result<Self, Refusal> {
        Self::planar([0.0, 0.0, z], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0])
    }

    /// The unit normal of the plane (the direction away from the material).
    #[must_use]
    pub fn normal(&self) -> [f64; 3] {
        match self {
            Self::Planar { normal, .. } => *normal,
        }
    }
}

/// How the tool axis is held along a cut. Phase 1 has only the plane
/// normal; a cut may still carry its own axis vector ([`crate::plan::Cut`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum AxisControl {
    #[default]
    Normal,
}

/// The tool as the stages read it. Phase 1 has only the disc of a milling
/// cutter; turning adds an insert with a nose radius and its angles.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum ToolShape {
    Disc { radius: f64, corner_radius: f64, can_plunge: bool, flute_length: f64 },
}

/// Refusal reason: a disc with no positive radius, a corner radius out of
/// range, or no positive flute length.
pub const BAD_TOOL: &str = "the tool needs a positive radius, a corner radius from 0 to the radius, and a positive flute length";

impl ToolShape {
    pub fn disc(radius: f64, corner_radius: f64, can_plunge: bool, flute_length: f64) -> Result<Self, Refusal> {
        let ok = finite(&[radius, corner_radius, flute_length])
            && radius > 0.0
            && (0.0..=radius).contains(&corner_radius)
            && flute_length > 0.0;
        if !ok {
            return Err(Refusal::new(Stage::Recipe, BAD_TOOL));
        }
        Ok(Self::Disc { radius, corner_radius, can_plunge, flute_length })
    }

    /// The radius of the cutter-centre offset in the plane.
    #[must_use]
    pub fn radius(&self) -> f64 {
        match self {
            Self::Disc { radius, .. } => *radius,
        }
    }

    /// True when the tool can go straight down into material.
    #[must_use]
    pub fn can_plunge(&self) -> bool {
        match self {
            Self::Disc { can_plunge, .. } => *can_plunge,
        }
    }
}

/// A point of a guide curve: its place in the plane and its depth along
/// the plane normal (0 = the level of the plan; V-Carve and Thread use it).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PlanePoint {
    pub at: Point2,
    pub depth: f64,
}

impl PlanePoint {
    #[must_use]
    pub fn flat(at: Point2) -> Self {
        Self { at, depth: 0.0 }
    }
    #[must_use]
    pub fn is_finite(&self) -> bool {
        finite(&[self.at.x, self.at.y, self.depth])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_work_plane_needs_unit_axes_at_right_angles() {
        assert!(WorkPlane::setup_xy(5.0).is_ok());
        assert_eq!(WorkPlane::setup_xy(f64::NAN).unwrap_err().reason, BAD_PLANE);
        assert!(WorkPlane::planar([0.0; 3], [0.0, 0.0, 2.0], [1.0, 0.0, 0.0]).is_err());
        assert!(WorkPlane::planar([0.0; 3], [0.0, 0.0, 1.0], [0.0, 0.6, 0.8]).is_err());
        let s = std::f64::consts::FRAC_1_SQRT_2;
        let tilted = WorkPlane::planar([0.0; 3], [0.0, s, s], [1.0, 0.0, 0.0]).unwrap();
        assert_eq!(tilted.normal(), [0.0, s, s]);
    }

    #[test]
    fn a_disc_needs_a_radius_and_a_corner_inside_it() {
        assert!(ToolShape::disc(4.0, 0.0, true, 20.0).is_ok());
        assert!(ToolShape::disc(4.0, 4.0, false, 20.0).is_ok());
        assert_eq!(ToolShape::disc(0.0, 0.0, true, 20.0).unwrap_err().reason, BAD_TOOL);
        assert!(ToolShape::disc(4.0, 4.5, true, 20.0).is_err());
        assert!(ToolShape::disc(4.0, -0.1, true, 20.0).is_err());
        assert!(ToolShape::disc(4.0, 0.0, true, 0.0).is_err());
        assert!(ToolShape::disc(f64::INFINITY, 0.0, true, 20.0).is_err());
        let t = ToolShape::disc(3.0, 0.5, false, 12.0).unwrap();
        assert_eq!((t.radius(), t.can_plunge()), (3.0, false));
    }

    #[test]
    fn the_default_axis_is_the_plane_normal() {
        assert_eq!(AxisControl::default(), AxisControl::Normal);
        assert!(!PlanePoint { at: Point2::new(0.0, f64::NAN), depth: 0.0 }.is_finite());
        assert!(PlanePoint::flat(Point2::new(1.0, 2.0)).is_finite());
    }
}
