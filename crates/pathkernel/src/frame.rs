//! The frame the stages plan in: the work plane, the tool axis, the tool
//! shape, a point with a depth along the plane normal, and the validated
//! scalars and spots that the enum variants of the crate carry (record
//! "What phase 1 keeps open", points 1, 2, 3 and 8).
//! Provenance: our own design record (`RECOMMENDED.md`, `## #4024`);
//! right-handed orthonormal frames as in any machine-tool handbook (ISO 841
//! axis naming).

use crate::verdict::{Refusal, Stage};
use crate::{KERNEL_TOL, Point2, finite};

/// Refusal reason: a spot that is not a finite point.
pub const BAD_SPOT: &str = "a place must be a finite point";
/// Refusal reason: a value that must be a positive finite number.
pub const NOT_POSITIVE: &str = "the value must be a positive finite number";

/// A finite point in the plane.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spot(Point2);

impl Spot {
    pub fn new(stage: Stage, p: Point2) -> Result<Self, Refusal> {
        if finite(&[p.x, p.y]) { Ok(Self(p)) } else { Err(Refusal::new(stage, BAD_SPOT)) }
    }
    #[must_use]
    pub fn get(self) -> Point2 {
        self.0
    }
}

/// A positive finite number.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Positive(f64);

impl Positive {
    pub fn new(stage: Stage, v: f64) -> Result<Self, Refusal> {
        if v.is_finite() && v > 0.0 { Ok(Self(v)) } else { Err(Refusal::new(stage, NOT_POSITIVE)) }
    }
    #[must_use]
    pub fn get(self) -> f64 {
        self.0
    }
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// True for a finite vector of length 1 (within the kernel tolerance).
pub(crate) fn unit(v: [f64; 3]) -> bool {
    finite(&v) && (dot(v, v).sqrt() - 1.0).abs() <= KERNEL_TOL
}

/// A flat frame: a finite origin, a unit normal (the tool side) and a unit
/// X direction at right angles to it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlanarFrame {
    origin: [f64; 3],
    normal: [f64; 3],
    x_dir: [f64; 3],
}

/// Refusal reason: a work plane whose axes are not finite unit vectors
/// at right angles.
pub const BAD_PLANE: &str = "the work plane needs a finite origin and two unit axes at right angles";

impl PlanarFrame {
    pub fn new(origin: [f64; 3], normal: [f64; 3], x_dir: [f64; 3]) -> Result<Self, Refusal> {
        if !finite(&origin) || !unit(normal) || !unit(x_dir) || dot(normal, x_dir).abs() > KERNEL_TOL {
            return Err(Refusal::new(Stage::Recipe, BAD_PLANE));
        }
        Ok(Self { origin, normal, x_dir })
    }
    #[must_use]
    pub fn origin(&self) -> [f64; 3] {
        self.origin
    }
    #[must_use]
    pub fn normal(&self) -> [f64; 3] {
        self.normal
    }
    #[must_use]
    pub fn x_dir(&self) -> [f64; 3] {
        self.x_dir
    }
}

/// The plane that the stages plan in. No stage reads "XY" or "+Z"
/// directly: a later phase adds the lathe's Z-X plane or a wrapped plane
/// as a new variant.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum WorkPlane {
    /// Phase 1: the setup's XY plane, or its 3+2 tilt.
    Planar(PlanarFrame),
}

impl WorkPlane {
    pub fn planar(origin: [f64; 3], normal: [f64; 3], x_dir: [f64; 3]) -> Result<Self, Refusal> {
        PlanarFrame::new(origin, normal, x_dir).map(Self::Planar)
    }

    /// The setup's own XY plane at height `z`, the tool along +Z.
    pub fn setup_xy(z: f64) -> Result<Self, Refusal> {
        Self::planar([0.0, 0.0, z], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0])
    }

    /// The unit normal of the plane (the direction away from the material).
    #[must_use]
    pub fn normal(&self) -> [f64; 3] {
        match self {
            Self::Planar(f) => f.normal,
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

/// The disc of a milling cutter: a positive radius, a corner radius from 0
/// to the radius, a positive flute length.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Disc {
    radius: f64,
    corner_radius: f64,
    can_plunge: bool,
    flute_length: f64,
}

/// Refusal reason: a disc with no positive radius, a corner radius out of
/// range, or no positive flute length.
pub const BAD_TOOL: &str = "the tool needs a positive radius, a corner radius from 0 to the radius, and a positive flute length";

impl Disc {
    pub fn new(radius: f64, corner_radius: f64, can_plunge: bool, flute_length: f64) -> Result<Self, Refusal> {
        let ok = finite(&[radius, corner_radius, flute_length])
            && radius > 0.0
            && (0.0..=radius).contains(&corner_radius)
            && flute_length > 0.0;
        if !ok {
            return Err(Refusal::new(Stage::Recipe, BAD_TOOL));
        }
        Ok(Self { radius, corner_radius, can_plunge, flute_length })
    }
    #[must_use]
    pub fn corner_radius(&self) -> f64 {
        self.corner_radius
    }
    #[must_use]
    pub fn flute_length(&self) -> f64 {
        self.flute_length
    }
}

/// The tool as the stages read it. Phase 1 has only the disc of a milling
/// cutter; turning adds an insert with a nose radius and its angles.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum ToolShape {
    Disc(Disc),
}

impl ToolShape {
    pub fn disc(radius: f64, corner_radius: f64, can_plunge: bool, flute_length: f64) -> Result<Self, Refusal> {
        Disc::new(radius, corner_radius, can_plunge, flute_length).map(Self::Disc)
    }

    /// The radius of the cutter-centre offset in the plane.
    #[must_use]
    pub fn radius(&self) -> f64 {
        match self {
            Self::Disc(d) => d.radius,
        }
    }

    /// True when the tool can go straight down into material.
    #[must_use]
    pub fn can_plunge(&self) -> bool {
        match self {
            Self::Disc(d) => d.can_plunge,
        }
    }
}

/// A point of a guide curve or a move: its finite place in the plane and
/// its finite depth along the plane normal (0 = the level of the plan;
/// V-Carve and Thread use it).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlanePoint {
    at: Point2,
    depth: f64,
}

/// Refusal reason: a plane point with a value that is not finite.
pub const BAD_POINT: &str = "a point needs a finite place and a finite depth";

impl PlanePoint {
    pub fn new(at: Point2, depth: f64) -> Result<Self, Refusal> {
        if finite(&[at.x, at.y, depth]) { Ok(Self { at, depth }) } else { Err(Refusal::new(Stage::Guide, BAD_POINT)) }
    }
    /// A point at the level of the plan (depth 0).
    pub fn flat(at: Point2) -> Result<Self, Refusal> {
        Self::new(at, 0.0)
    }
    #[must_use]
    pub fn at(&self) -> Point2 {
        self.at
    }
    #[must_use]
    pub fn depth(&self) -> f64 {
        self.depth
    }
    /// The same point within [`KERNEL_TOL`] in place and depth.
    #[must_use]
    pub fn near(&self, o: &Self) -> bool {
        crate::near(self.at, o.at) && (self.depth - o.depth).abs() <= KERNEL_TOL
    }
}
