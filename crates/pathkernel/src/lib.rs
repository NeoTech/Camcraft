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
//!
//! **Valid by construction** (#4055 review): a value of a public type of
//! this crate is always valid. A field that could hold an invalid value is
//! private and is set only by a checked constructor that returns a
//! [`Refusal`]. An enum variant that carries data holds a validated
//! newtype, never a raw number or a raw point. A list in a value holds only
//! validated records, and the constructor that takes the list checks the
//! rules between its records. So no constructor relies on another one
//! having run: it cannot receive an unchecked value. The open enums of the
//! record (`WorkPlane`, `ToolShape`, `KernelRegion`, `Step`, ...) stay
//! `#[non_exhaustive]`.

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
pub use frame::{AxisControl, Disc, PlanarFrame, PlanePoint, Positive, Spot, ToolShape, WorkPlane};
pub use recipe::Recipe;
pub use verdict::{Fallback, Gap, Gaps, Note, Refusal, Stage, Verdict};

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

/// True when the two points are the same point within [`KERNEL_TOL`].
pub(crate) fn near(a: Point2, b: Point2) -> bool {
    (a.x - b.x).hypot(a.y - b.y) <= KERNEL_TOL
}

/// The distance from `p` to the segment `a`-`b`.
fn seg_dist(p: Point2, a: Point2, b: Point2) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len2 = dx * dx + dy * dy;
    let t = if len2 > 0.0 { (((p.x - a.x) * dx + (p.y - a.y) * dy) / len2).clamp(0.0, 1.0) } else { 0.0 };
    (p.x - (a.x + t * dx)).hypot(p.y - (a.y + t * dy))
}

/// True when `p` is inside `area` or on its boundary within [`KERNEL_TOL`].
pub(crate) fn in_area(area: &Region, p: Point2) -> bool {
    area.contains(p)
        || area.loops().iter().any(|l| {
            let pts = l.points();
            pts.iter().zip(pts.iter().cycle().skip(1)).any(|(a, b)| seg_dist(p, *a, *b) <= KERNEL_TOL)
        })
}

/// True when every point of every loop of `r` is finite.
pub(crate) fn region_finite(r: &Region) -> bool {
    r.loops().iter().all(|l| l.points().iter().all(|p| finite(&[p.x, p.y])))
}
