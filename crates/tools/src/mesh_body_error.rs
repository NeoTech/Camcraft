//! #3643: why a mesh file is no tool component body. Each refusal names
//! what is wrong. Our own record; no vendor source consulted.

use crate::component_geometry::GeometryError;
use crate::frame::FrameError;
use crate::holder_profile::EnvelopeError;
use crate::lathe_holder::LatheBodyError;
use crate::mesh_file::{MeshFileError, MAX_BODY_FACETS};

/// Why a mesh body is refused.
#[derive(Clone, Debug, PartialEq)]
pub enum MeshBodyError {
    /// The STL or OBJ reader refused the file.
    File(MeshFileError),
    /// The STEP reader, kernel or mesher refused the file.
    Step(String),
    /// The STEP model does not fit the facet budget at any chord tried.
    StepOverBudget,
    /// A frame name is empty.
    EmptyFrameName,
    /// A frame is not finite or not orthonormal.
    Frame { name: String, err: FrameError },
    /// No mounting frame (MCS or MCS_INSERT): what the file names instead.
    NoMount { found: String },
    /// More than one mounting frame.
    TwoMounts(Vec<String>),
    /// A mounting frame, port or cutting point that the file states but
    /// the reader refused (degenerate, non-finite or ambiguous): never
    /// read as absent, so no default takes its place.
    RefusedFrame { name: String, why: String },
    /// More than one cutting point (CIP).
    TwoCutPoints(Vec<String>),
    /// A workpiece-side port or the cutting point is not below the
    /// mounting frame, so its +Z does not point toward the machine.
    NotBelowMount { name: String, z: f64 },
    /// The revolved envelope of a spindle body is refused.
    Envelope(EnvelopeError),
    /// The envelope cannot be drawn as a spun outline that encloses it.
    Outline,
    /// The component geometry is refused.
    Geometry(GeometryError),
    /// The lathe body is refused.
    Lathe(LatheBodyError),
}

impl core::fmt::Display for MeshBodyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::File(e) => write!(f, "the mesh file is refused: {e}"),
            Self::Step(e) => write!(f, "the STEP file is refused: {e}"),
            Self::StepOverBudget => write!(
                f,
                "the STEP model does not fit the budget of {MAX_BODY_FACETS} facets for one tool component at any \
                 chord tried (#1041: a coarser mesh is not made yet) - export it again simpler, or as an STL"
            ),
            Self::EmptyFrameName => write!(f, "a coordinate system of the body needs a name"),
            Self::Frame { name, err } => write!(f, "the coordinate system '{name}' is refused: {err}"),
            Self::NoMount { found } => write!(
                f,
                "the body has no mounting coordinate system (MCS or MCS_INSERT), so it cannot be placed: {found}"
            ),
            Self::TwoMounts(names) => write!(
                f,
                "the body has {} mounting coordinate systems ({}) - one only",
                names.len(),
                names.join(", ")
            ),
            Self::RefusedFrame { name, why } => write!(
                f,
                "the body states the coordinate system '{name}', but it cannot be read ({why}) - correct it in the \
                 file or remove it; a default does not take its place"
            ),
            Self::TwoCutPoints(names) => write!(
                f,
                "the body has {} cutting points ({}) - one only",
                names.len(),
                names.join(", ")
            ),
            Self::NotBelowMount { name, z } => write!(
                f,
                "the coordinate system '{name}' is {z} mm along the mounting frame's +Z, not below it - the \
                 mounting frame's +Z must point toward the machine"
            ),
            Self::Envelope(e) => write!(f, "the body is refused: {e}"),
            Self::Outline => write!(f, "the body's envelope cannot be drawn as an outline that encloses it"),
            Self::Geometry(e) => write!(f, "{e}"),
            Self::Lathe(e) => write!(f, "the lathe body is refused: {e}"),
        }
    }
}

impl std::error::Error for MeshBodyError {}
