//! #3643: why a lathe tool assembly is refused. Each refusal names the
//! component, or the value, that it is about. Our own wording.

use crate::lathe_holder::LatheBodyError;
use crate::tool_assembly::AssemblyError;

/// Why a lathe assembly is refused.
#[derive(Clone, Debug, PartialEq)]
pub enum LatheAssemblyError {
    /// A refusal of the component chain (fits, frames, ports, cycles).
    Assembly(AssemblyError),
    NotInsert {
        name: String,
    },
    NoSeat {
        name: String,
    },
    NoNose {
        name: String,
        code: String,
    },
    Orientation {
        q: u8,
    },
    NeedOrientation {
        name: String,
        coupling: String,
    },
    Station(LatheBodyError),
    SpunBody {
        name: String,
    },
    /// A body point stands `reach` mm along the cutting direction on
    /// `axis`, past the insert's cutting line at `nose` mm.
    BodyPastEdge {
        name: String,
        axis: char,
        reach: f64,
        nose: f64,
    },
    Clearance(LatheBodyError),
}

impl core::fmt::Display for LatheAssemblyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Assembly(e) => write!(f, "{e}"),
            Self::NotInsert { name } => {
                write!(f, "the cutting item '{name}' is not an insert - a lathe assembly cuts with an insert in a seat")
            }
            Self::NoSeat { name } => write!(f, "the insert '{name}' does not sit in an insert seat"),
            Self::NoNose { name, code } => write!(
                f,
                "the insert '{name}' ({code}) states no nose radius - a turning insert needs a corner radius more than 0 mm"
            ),
            Self::Orientation { q } => write!(f, "the lathe orientation {q} is not valid - it takes 1 to 8"),
            Self::NeedOrientation { name, coupling } => write!(
                f,
                "the orientation of '{name}' cannot be derived from its coupling {coupling} - state the orientation (1 to 8)"
            ),
            Self::Station(e) => write!(f, "the lathe station is refused: {e}"),
            Self::SpunBody { name } => write!(
                f,
                "'{name}' has a revolved body - a lathe component's body is a box or a mesh, because it does not turn"
            ),
            Self::BodyPastEdge { name, axis, reach, nose } => write!(
                f,
                "the body of '{name}' reaches {reach:.3} mm from the nose centre along the cutting direction in {axis}, \
                 past the insert's cutting line at {nose:.3} mm - check the hand, the orientation or the seat frame"
            ),
            Self::Clearance(e) => write!(f, "the lathe assembly's body is refused: {e}"),
        }
    }
}

impl std::error::Error for LatheAssemblyError {}

impl From<AssemblyError> for LatheAssemblyError {
    fn from(e: AssemblyError) -> Self {
        Self::Assembly(e)
    }
}
