//! #3643: why a tool assembly is refused. Each refusal names the
//! component (and the port) it is about. Our own wording.

use crate::component_fit::FitError;
use crate::frame::FrameError;
use crate::spun::SpunError;

/// Why an assembly is refused.
#[derive(Clone, Debug, PartialEq)]
pub enum AssemblyError {
    Empty,
    TooMany { count: usize },
    NoRoot,
    TwoRoots { a: String, b: String },
    InsertAtRoot { name: String },
    BadParent { name: String, parent: usize },
    Cycle { name: String },
    NoPort { parent: String },
    AmbiguousPort { parent: String, ports: String },
    PortTaken { parent: String, port: String },
    Fit(FitError),
    NoCutter,
    TwoCutters { a: String, b: String },
    MissingFrame { name: String, port: String },
    NonFinite { name: String, what: &'static str },
    BadFrame { name: String, err: FrameError },
    Stickout { name: String, why: &'static str },
    OffAxis { name: String },
    BadGauge { gauge_len: f64 },
    BadCutter { name: String, why: String },
    Body { name: String, err: SpunError },
    BodyInFlutes { bottom: f64, flute_len: f64 },
}

impl core::fmt::Display for AssemblyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Empty => write!(f, "the tool assembly has no components"),
            Self::TooMany { count } => {
                write!(f, "the tool assembly has {count} components - {} at most", super::MAX_NODES)
            }
            Self::NoRoot => write!(f, "no component of the tool assembly sits on the spindle (each one has a parent)"),
            Self::TwoRoots { a, b } => write!(f, "'{a}' and '{b}' both sit on the spindle - an assembly has one root"),
            Self::InsertAtRoot { name } => write!(f, "the insert '{name}' cannot sit on the spindle - it needs a seat"),
            Self::BadParent { name, parent } => write!(f, "'{name}' names component {parent} as its parent, which is not in the assembly"),
            Self::Cycle { name } => write!(f, "'{name}' is in a cycle of mounts - it never reaches the spindle"),
            Self::NoPort { parent } => write!(f, "'{parent}' has no port for a component to sit on"),
            Self::AmbiguousPort { parent, ports } => {
                write!(f, "'{parent}' has the ports {ports} - name the port the component sits on")
            }
            Self::PortTaken { parent, port } => write!(f, "two components sit on the port {port} of '{parent}'"),
            Self::Fit(e) => write!(f, "{e}"),
            Self::NoCutter => write!(f, "the tool assembly has no cutting item (an insert or a solid tool)"),
            Self::TwoCutters { a, b } => {
                write!(f, "'{a}' and '{b}' are both cutting items - a spindle assembly has one active cutting tip")
            }
            Self::MissingFrame { name, port } => write!(f, "'{name}' has no frame for its port {port}"),
            Self::NonFinite { name, what } => write!(f, "'{name}' has a non-finite {what}"),
            Self::BadFrame { name, err } => write!(f, "'{name}': {err}"),
            Self::Stickout { name, why } => write!(f, "the stickout of '{name}' is refused: {why}"),
            Self::OffAxis { name } => write!(f, "'{name}' does not stand on the spindle axis - a spindle body must be coaxial"),
            Self::BadGauge { gauge_len } => write!(
                f,
                "the gauge length works out to {gauge_len} mm - the cutting tip must sit below the spindle face, within {} mm",
                crate::holder_profile::MAX_BODY_HEIGHT
            ),
            Self::BadCutter { name, why } => write!(f, "the cutting item '{name}' is refused: {why}"),
            Self::Body { name, err } => write!(f, "the body of '{name}' is refused: {err}"),
            Self::BodyInFlutes { bottom, flute_len } => write!(
                f,
                "the assembled body reaches into the flutes (its foot is {bottom:.3} mm above the tip, the flutes run {flute_len:.3} mm)"
            ),
        }
    }
}

impl std::error::Error for AssemblyError {}
