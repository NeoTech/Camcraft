//! #3646: why a multi-seat milling cutter is refused. Each refusal
//! names the cutter body (and the seat) it is about. Our own wording.

use crate::component_fit::FitError;
use crate::tool_assembly::AssemblyError;

/// Why a multi-seat milling cutter is refused.
#[derive(Clone, Debug, PartialEq)]
pub enum MillingError {
    /// A stated number is NaN or infinite.
    NonFinite { name: String, what: &'static str, value: f64 },
    /// The cutting diameter `DC` is not over 0, or past the tool cap.
    BadDiameter { name: String, dc: f64 },
    /// The cutting edge angle is not over 0 and at most 90 degrees.
    BadLeadAngle { name: String, deg: f64 },
    /// The largest axial depth `APMX` is not over 0.
    BadDepth { name: String, apmx: f64 },
    /// The component is not a tool item (a cutter body with seats).
    NotABody { name: String },
    /// A stated seat count of 0.
    NoSeats { name: String },
    /// More seats than a tool record counts flutes.
    TooManySeats { name: String, count: usize },
    /// A stated seat count that the body's seat ports contradict.
    SeatCount { name: String, stated: u32, ports: usize },
    /// A seat that does not take the shared insert (#3642's check).
    Fit(FitError),
    /// The insert's corner does not fit the cutter.
    Corner { name: String, why: String },
    /// The tool record that the cutter gives fails the tool law.
    Tool { name: String, why: String },
    /// The seat geometry of the assembly puts the cutting point on a
    /// diameter other than the stated `DC`.
    SeatDiameter { name: String, seat_d: f64, dc: f64 },
    /// A judged body stands outside the cutter inside the cutting depth:
    /// `cut_r` is how far the tool reaches at the band's lowest point.
    BodyOutside { name: String, above: f64, r: f64, cut_r: f64 },
    /// The spindle assembly is refused by the resolver (#3643); boxed,
    /// so that every result of this module stays small.
    Assembly { name: String, err: Box<AssemblyError> },
    /// The assembly mounts a cutter body or an insert other than the
    /// ones the cutter was derived from.
    Mismatch { name: String, why: String },
    /// A seat of the assembly is not the mounted seat turned about the
    /// spindle axis: `what` is "frame", "radius", "height" or
    /// "orientation".
    SeatOff { name: String, port: String, what: &'static str, why: String },
}

impl core::fmt::Display for MillingError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NonFinite { name, what, value } => write!(f, "the cutter '{name}' has a non-finite {what} ({value})"),
            Self::BadDiameter { name, dc } => {
                write!(f, "the cutter '{name}' has a cutting diameter of {dc} mm - it must be over 0 and at most 500 mm")
            }
            Self::BadLeadAngle { name, deg } => write!(
                f,
                "the cutter '{name}' has a cutting edge angle of {deg} degrees - it must be over 0 and at most 90"
            ),
            Self::BadDepth { name, apmx } => {
                write!(f, "the cutter '{name}' has a largest cutting depth of {apmx} mm - it must be over 0")
            }
            Self::NotABody { name } => {
                write!(f, "'{name}' is not a cutter body - a multi-seat cutter needs a tool item with insert seats")
            }
            Self::NoSeats { name } => write!(f, "the cutter '{name}' states 0 seats - it needs one at least"),
            Self::TooManySeats { name, count } => {
                write!(f, "the cutter '{name}' has {count} seats - a tool counts 32 cutting edges at most")
            }
            Self::SeatCount { name, stated, ports } => write!(
                f,
                "the cutter '{name}' states {stated} seats, but its body has {ports} seat ports - state one seat port for a seat count, or a port for each seat"
            ),
            Self::Fit(e) => write!(f, "{e}"),
            Self::Corner { name, why } => write!(f, "the insert corner of the cutter '{name}' is refused: {why}"),
            Self::Tool { name, why } => write!(f, "the cutter '{name}' gives a tool that is refused: {why}"),
            Self::SeatDiameter { name, seat_d, dc } => write!(
                f,
                "the seats of the cutter '{name}' put the cutting point on Ø{seat_d:.3} mm, but the cutter states DC {dc} mm"
            ),
            Self::BodyOutside { name, above, r, cut_r } => write!(
                f,
                "a body of the cutter '{name}' reaches {r:.3} mm from the axis at {above:.3} mm above the tip - outside the {cut_r:.3} mm that the tool reaches there, inside the cutting depth"
            ),
            Self::Assembly { name, err } => write!(f, "the assembly of the cutter '{name}' is refused: {err}"),
            Self::Mismatch { name, why } => write!(f, "the assembly does not match the cutter '{name}': {why}"),
            Self::SeatOff { name, port, why, .. } => {
                write!(f, "the seat {port} of the cutter '{name}' is refused: {why}")
            }
        }
    }
}

impl std::error::Error for MillingError {}

impl From<FitError> for MillingError {
    fn from(e: FitError) -> Self {
        Self::Fit(e)
    }
}
