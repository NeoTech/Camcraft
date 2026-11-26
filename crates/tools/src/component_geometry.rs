//! #3643: where a tool component's ports, cutting point and body sit,
//! in the component's own frame. The frame's origin is the machine-side
//! connection (MCS) and its +Z points toward the machine, as the tool
//! frame of the collision laws does; a workpiece-side port (CSW) or the
//! cutting point (CIP) is a frame in it. Our own record over the public
//! ISO 13399 coordinate-system roles (MCS, CSW, CIP); pure data.
//!
//! The law: geometry is checked when it is made, and each refusal names
//! what is wrong.

use crate::frame::Frame;
use crate::holder_library::{HolderEntry, LibraryError};
use crate::spun::{SpunError, SpunPoint, SpunProfile};

/// Why component geometry is refused.
#[derive(Clone, Debug, PartialEq)]
pub enum GeometryError {
    EmptyPort,
    DuplicatePort(String),
    NonFiniteBase,
    Body(SpunError),
    Holder(LibraryError),
    UnknownPort(String),
    BadStop { port: String, depth: f64 },
}

impl core::fmt::Display for GeometryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyPort => write!(f, "a port frame needs a port name"),
            Self::DuplicatePort(p) => write!(f, "the port '{p}' has two frames"),
            Self::NonFiniteBase => write!(f, "the body's base height must be a finite number"),
            Self::Body(e) => write!(f, "the body outline is refused: {e}"),
            Self::Holder(e) => write!(f, "the holder is refused: {e}"),
            Self::UnknownPort(p) => write!(f, "the port '{p}' has no frame, so it cannot have a bore stop"),
            Self::BadStop { port, depth } => {
                write!(f, "the bore stop of the port '{port}' must be a finite depth more than 0 mm, not {depth}")
            }
        }
    }
}

impl std::error::Error for GeometryError {}

/// A revolved body about the component's Z axis: a checked spun
/// profile whose own height 0 sits at `base` along the component's Z
/// (a holder hangs below its MCS, so its base is negative).
#[derive(Clone, Debug, PartialEq)]
pub struct ComponentBody {
    profile: SpunProfile,
    base: f64,
}

impl ComponentBody {
    /// The outline `points` (tip-up, as [`SpunProfile::new`] checks it)
    /// with its height 0 at `base`.
    pub fn new(points: Vec<SpunPoint>, base: f64) -> Result<Self, GeometryError> {
        if !base.is_finite() {
            return Err(GeometryError::NonFiniteBase);
        }
        let profile = SpunProfile::new(points).map_err(GeometryError::Body)?;
        Ok(Self { profile, base })
    }

    #[must_use]
    pub fn profile(&self) -> &SpunProfile {
        &self.profile
    }

    #[must_use]
    pub fn base(&self) -> f64 {
        self.base
    }
}

/// One component's frames and body.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ComponentGeometry {
    ports: Vec<(String, Frame)>,
    body: Option<ComponentBody>,
    cip: Option<Frame>,
    stops: Vec<(String, f64)>,
}

impl ComponentGeometry {
    /// Checked geometry: each port frame has a name, and no name twice.
    /// `cip` is the cutting point of a cutting item (`None`: the
    /// default, see [`Self::cip`]).
    pub fn new(
        ports: Vec<(String, Frame)>,
        body: Option<ComponentBody>,
        cip: Option<Frame>,
    ) -> Result<Self, GeometryError> {
        for (i, (name, _)) in ports.iter().enumerate() {
            if name.trim().is_empty() {
                return Err(GeometryError::EmptyPort);
            }
            if ports[..i].iter().any(|(n, _)| n == name) {
                return Err(GeometryError::DuplicatePort(name.clone()));
            }
        }
        Ok(Self { ports, body, cip, stops: Vec::new() })
    }

    /// The same geometry with a bore stop on `port`: the shank of a
    /// solid tool sits fully home when its back end is `depth` mm into
    /// the bore, measured from the port frame along its +Z. Only a
    /// stated stop gives a shank its home position (see
    /// [`crate::tool_assembly::Mount::projection_mm`]).
    pub fn with_stop(mut self, port: &str, depth: f64) -> Result<Self, GeometryError> {
        if self.port(port).is_none() {
            return Err(GeometryError::UnknownPort(port.to_owned()));
        }
        if !depth.is_finite() || depth <= 0.0 {
            return Err(GeometryError::BadStop { port: port.to_owned(), depth });
        }
        self.stops.retain(|(n, _)| n != port);
        self.stops.push((port.to_owned(), depth));
        Ok(self)
    }

    /// One port on the axis, `depth` mm below the MCS, and an optional
    /// body: a collet, an extension, a plain adapter.
    pub fn axial(port: &str, depth: f64, body: Option<ComponentBody>) -> Result<Self, GeometryError> {
        let frame = Frame::at_z(-depth).map_err(|_| GeometryError::NonFiniteBase)?;
        Self::new(vec![(port.to_owned(), frame)], body, None)
    }

    /// A built-in library holder as a component: its MCS on the gauge
    /// line, its one port `port` on the nose face `gauge_len` below,
    /// and its body the entry's own outline at that gauge length
    /// ([`HolderEntry::points`] with the nose at the body's base) - the
    /// outline the holder library draws for a tool today. The library
    /// states no bore depth, so the port has no bore stop: a solid tool
    /// in it needs a stated stickout, or a stop from [`Self::with_stop`].
    pub fn holder(entry: &HolderEntry, port: &str, gauge_len: f64) -> Result<Self, GeometryError> {
        let points = entry.points(0.0, gauge_len).map_err(GeometryError::Holder)?;
        let body = ComponentBody::new(points, -gauge_len)?;
        Self::axial(port, gauge_len, Some(body))
    }

    /// The frame of `port`, if this geometry states it.
    #[must_use]
    pub fn port(&self, port: &str) -> Option<&Frame> {
        self.ports.iter().find(|(n, _)| n == port).map(|(_, f)| f)
    }

    /// The bore stop of `port` (mm into the bore), if this geometry
    /// states one.
    #[must_use]
    pub fn stop(&self, port: &str) -> Option<f64> {
        self.stops.iter().find(|(n, _)| n == port).map(|(_, d)| *d)
    }

    #[must_use]
    pub fn ports(&self) -> &[(String, Frame)] {
        &self.ports
    }

    #[must_use]
    pub fn body(&self) -> Option<&ComponentBody> {
        self.body.as_ref()
    }

    /// The stated cutting point, if any. Without one, a solid tool's
    /// cutting point is on its axis at its full length below the MCS
    /// (the MCS at the shank's back end), and an insert's is the seat
    /// frame itself.
    #[must_use]
    pub fn cip(&self) -> Option<&Frame> {
        self.cip.as_ref()
    }
}
