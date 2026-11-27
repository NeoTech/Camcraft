//! #3643: a rigid right-handed frame (an origin and two unit axes) and
//! the forward-kinematics steps a tool assembly chains: compose a local
//! frame onto its parent, turn about the frame's own Z, move along it.
//! Textbook rigid-body linear algebra (rotation matrices from three
//! orthonormal columns); our own record, no vendor source consulted.
//!
//! The law: a frame that exists is finite and orthonormal, because
//! [`Frame::new`] checks it and refuses by name.

/// Why a frame is refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameError {
    /// A coordinate is NaN or infinite.
    NonFinite,
    /// The Z axis has no length.
    ZeroAxis,
    /// The X axis has no length, or it lies along Z.
    CollinearX,
}

impl core::fmt::Display for FrameError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::NonFinite => "the frame has a non-finite number",
            Self::ZeroAxis => "the frame's Z axis has no length",
            Self::CollinearX => "the frame's X axis has no length or lies along its Z axis",
        })
    }
}

impl std::error::Error for FrameError {}

/// The shortest axis length (before it is made unit) that is read as
/// a direction.
const MIN_LEN: f64 = 1e-12;

/// A rigid frame: an origin, a unit Z axis and a unit X axis normal to
/// it (Y is Z x X).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    origin: [f64; 3],
    z: [f64; 3],
    x: [f64; 3],
}

impl Frame {
    /// The frame of the axes themselves.
    pub const IDENTITY: Self = Self { origin: [0.0; 3], z: [0.0, 0.0, 1.0], x: [1.0, 0.0, 0.0] };

    /// A checked frame. Z is made unit; X is made normal to Z and unit,
    /// so a small rounding error in a stored frame is taken back.
    pub fn new(origin: [f64; 3], z: [f64; 3], x: [f64; 3]) -> Result<Self, FrameError> {
        if !origin.iter().chain(&z).chain(&x).all(|v| v.is_finite()) {
            return Err(FrameError::NonFinite);
        }
        let uz = unit(z).ok_or(FrameError::ZeroAxis)?;
        let zx = dot(uz, x);
        let ux = unit([x[0] - zx * uz[0], x[1] - zx * uz[1], x[2] - zx * uz[2]]).ok_or(FrameError::CollinearX)?;
        Ok(Self { origin, z: uz, x: ux })
    }

    /// The identity frame moved `dz` along Z: a port on the axis.
    pub fn at_z(dz: f64) -> Result<Self, FrameError> {
        Self::new([0.0, 0.0, dz], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0])
    }

    #[must_use]
    pub fn origin(&self) -> [f64; 3] {
        self.origin
    }

    #[must_use]
    pub fn z(&self) -> [f64; 3] {
        self.z
    }

    #[must_use]
    pub fn x(&self) -> [f64; 3] {
        self.x
    }

    #[must_use]
    pub fn y(&self) -> [f64; 3] {
        cross(self.z, self.x)
    }

    /// A point given in this frame, in the parent's coordinates.
    #[must_use]
    pub fn map_point(&self, p: [f64; 3]) -> [f64; 3] {
        let d = self.map_dir(p);
        [self.origin[0] + d[0], self.origin[1] + d[1], self.origin[2] + d[2]]
    }

    /// A direction given in this frame, in the parent's coordinates.
    #[must_use]
    pub fn map_dir(&self, v: [f64; 3]) -> [f64; 3] {
        let y = self.y();
        let (x, z) = (self.x, self.z);
        [
            v[0] * x[0] + v[1] * y[0] + v[2] * z[0],
            v[0] * x[1] + v[1] * y[1] + v[2] * z[1],
            v[0] * x[2] + v[1] * y[2] + v[2] * z[2],
        ]
    }

    /// `local`, given in this frame, in the parent's coordinates: the
    /// one step of a kinematic chain.
    #[must_use]
    pub fn then(&self, local: &Self) -> Self {
        Self { origin: self.map_point(local.origin), z: self.map_dir(local.z), x: self.map_dir(local.x) }
    }

    /// This frame turned `deg` degrees about its own Z (right hand).
    /// A non-finite angle is refused.
    pub fn turned(&self, deg: f64) -> Result<Self, FrameError> {
        if !deg.is_finite() {
            return Err(FrameError::NonFinite);
        }
        let (s, c) = deg.to_radians().sin_cos();
        let y = self.y();
        let x = [c * self.x[0] + s * y[0], c * self.x[1] + s * y[1], c * self.x[2] + s * y[2]];
        Self::new(self.origin, self.z, x)
    }

    /// This frame moved `d` along its own Z. A non-finite distance is
    /// refused.
    pub fn moved(&self, d: f64) -> Result<Self, FrameError> {
        if !d.is_finite() {
            return Err(FrameError::NonFinite);
        }
        let o = self.origin;
        Self::new([o[0] + d * self.z[0], o[1] + d * self.z[1], o[2] + d * self.z[2]], self.z, self.x)
    }

    /// Is this frame's Z the +Z of its parent, and its origin on that
    /// axis, within `tol` (mm and unit length)?
    #[must_use]
    pub fn on_axis(&self, tol: f64) -> bool {
        self.origin[0].hypot(self.origin[1]) <= tol && self.z[0].hypot(self.z[1]) <= tol && self.z[2] > 0.0
    }
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn unit(v: [f64; 3]) -> Option<[f64; 3]> {
    let len = dot(v, v).sqrt();
    (len.is_finite() && len > MIN_LEN).then(|| [v[0] / len, v[1] / len, v[2] / len])
}
