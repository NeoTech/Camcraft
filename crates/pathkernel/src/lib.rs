//! The path kernel (#4023, #4055): one planner for the 2D milling kinds,
//! cut into six stages - region, guide curves, order, load control, link,
//! entry and exit. Each stage has its own type, its invariants as checked
//! constructors, and a trait so that a later family can supply its own
//! stage. This step holds the types only: no stage plans a path yet.
//!
//! Provenance: our own design record `docs/decisions/4024-path-kernel-design.md`
//! (clean-room); the stage split follows the published offset-pocketing and
//! engagement literature (Held 1991; Stori & Wright 2000; Held & Spielberger
//! 2009). No HSMWorks material consulted.
//!
//! The data flow is `region -> guide -> order -> load -> link -> entry/exit`.
//! The start side reaches the curves only through [`order`] (law 3): the
//! region stage takes no start side, the order stage takes no material side.

pub mod engine;
pub mod entry;
pub mod frame;
pub mod guide;
pub mod link;
pub mod load;
pub mod order;
pub mod plan;
pub mod recipe;
pub mod region;
pub mod verdict;

pub use engine::{PLANNER_ENV, PlannerEngine, engine_from_switch};
pub use frame::{AxisControl, PlanePoint, ToolShape, WorkPlane};
pub use recipe::Recipe;
pub use verdict::{Fallback, Gap, Note, Refusal, Stage, Verdict};

/// The 2D types of the kernel's data, re-exported so that a caller names
/// one crate.
pub use camcraft_geometry::geo2d::{Loop, Point2, Region};

/// The tolerance (mm) of the kernel's own invariant checks: two points
/// closer than this are the same point.
pub const KERNEL_TOL: f64 = 1e-6;

/// True when every value is a finite number (no NaN, no infinity).
pub(crate) fn finite(values: &[f64]) -> bool {
    values.iter().all(|v| v.is_finite())
}
