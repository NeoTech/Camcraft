//! The guide choices of stage 2 (#4176): what a strategy asks of the guide
//! stage, each a checked value. Moved out of `guide.rs` (first part) with
//! no change; the helix choice is new (second part).
//! Provenance: our own design record (`RECOMMENDED.md`, `## #4024`)
//! "Stage 2: Guide curves"; the helix of a thread or a helical bore as in
//! the machine-tool handbooks (one turn for each pitch of descent).

use crate::frame::{Positive, Spot};
use crate::verdict::{Refusal, Stage};
use crate::KERNEL_TOL;

/// What the rings are offset from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RingRef {
    /// The selected contour.
    Part,
    /// All the walls of the area.
    Boundary,
    /// The band between the part and the stock edge.
    Band,
}

/// An exact circle, for G2/G3 rings (Circular, Bore): a finite centre and
/// a positive finite radius.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Circle {
    center: Spot,
    radius: Positive,
}

impl Circle {
    pub fn new(center: crate::Point2, radius: f64) -> Result<Self, Refusal> {
        Ok(Self { center: Spot::new(Stage::Guide, center)?, radius: Positive::new(Stage::Guide, radius)? })
    }
    #[must_use]
    pub fn center(&self) -> crate::Point2 {
        self.center.get()
    }
    #[must_use]
    pub fn radius(&self) -> f64 {
        self.radius.get()
    }
    /// The same circle within [`KERNEL_TOL`].
    #[must_use]
    pub fn near(&self, o: &Self) -> bool {
        crate::near(self.center(), o.center()) && (self.radius() - o.radius()).abs() <= KERNEL_TOL
    }
}

/// Refusal reason: an offset or an angle that is not finite.
pub const BAD_OFFSET: &str = "an offset or an angle of the guide is not a finite number";
/// Refusal reason: a ring count of zero.
pub const ZERO_COUNT: &str = "a ring count must be one or more";
/// Refusal reason: a helix of more turns than the planner makes.
pub const TOO_MANY_TURNS: &str = "the helix needs more turns than the planner makes (10000)";

/// The most turns of one helix.
pub const MAX_TURNS: f64 = 10_000.0;

/// The rings of a guide: a finite first offset, a positive stepover, an
/// optional exact circle and an optional count of one or more.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RingsChoice {
    reference: RingRef,
    first_offset: f64,
    stepover: Positive,
    circle: Option<Circle>,
    count: Option<u32>,
}

impl RingsChoice {
    /// With `circle`, the rings are exact circles about its centre and
    /// `circle` is the boundary of the cutter-centre area of the hole: ring
    /// `k` has the radius `circle.radius() - first_offset - k * stepover`.
    pub fn new(reference: RingRef, first_offset: f64, stepover: f64, circle: Option<Circle>, count: Option<u32>) -> Result<Self, Refusal> {
        if !first_offset.is_finite() {
            return Err(Refusal::new(Stage::Recipe, BAD_OFFSET));
        }
        if count == Some(0) {
            return Err(Refusal::new(Stage::Recipe, ZERO_COUNT));
        }
        Ok(Self { reference, first_offset, stepover: Positive::new(Stage::Recipe, stepover)?, circle, count })
    }
    #[must_use]
    pub fn reference(&self) -> RingRef {
        self.reference
    }
    #[must_use]
    pub fn first_offset(&self) -> f64 {
        self.first_offset
    }
    #[must_use]
    pub fn stepover(&self) -> f64 {
        self.stepover.get()
    }
    #[must_use]
    pub fn circle(&self) -> Option<Circle> {
        self.circle
    }
    #[must_use]
    pub fn count(&self) -> Option<u32> {
        self.count
    }
}

/// Parallel rows: a finite angle (degrees), a positive stepover, and a
/// closing ring or none.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RowsChoice {
    angle: f64,
    stepover: Positive,
    closing_ring: bool,
}

impl RowsChoice {
    pub fn new(angle: f64, stepover: f64, closing_ring: bool) -> Result<Self, Refusal> {
        if !angle.is_finite() {
            return Err(Refusal::new(Stage::Recipe, BAD_OFFSET));
        }
        Ok(Self { angle, stepover: Positive::new(Stage::Recipe, stepover)?, closing_ring })
    }
    #[must_use]
    pub fn angle(&self) -> f64 {
        self.angle
    }
    #[must_use]
    pub fn stepover(&self) -> f64 {
        self.stepover.get()
    }
    #[must_use]
    pub fn closing_ring(&self) -> bool {
        self.closing_ring
    }
}

/// The finite offset of a trace guide: above zero to the left of the
/// chain (in its point order; a closed chain is read counter-clockwise, so
/// its left is its inside), below zero to the right, zero on the chain.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TraceOffset(f64);

impl TraceOffset {
    pub fn new(offset: f64) -> Result<Self, Refusal> {
        if offset.is_finite() { Ok(Self(offset)) } else { Err(Refusal::new(Stage::Recipe, BAD_OFFSET)) }
    }
    #[must_use]
    pub fn get(self) -> f64 {
        self.0
    }
}

/// The hand of a helix: `Right` rises when it turns counter-clockwise (as
/// seen from above the plane), `Left` rises when it turns clockwise. The
/// hand is the shape of the curve; the direction of travel is stage 4.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Hand {
    Right,
    Left,
}

/// One helix about the centre of `circle` at its radius (the cutter-centre
/// path), from the level of the plan down by `drop`, one turn for each
/// `pitch` of descent (Thread, a helical bore).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HelixChoice {
    circle: Circle,
    pitch: Positive,
    drop: Positive,
    hand: Hand,
}

impl HelixChoice {
    /// Refused for a pitch or a drop that is not a positive finite number,
    /// or more than [`MAX_TURNS`] turns.
    pub fn new(circle: Circle, pitch: f64, drop: f64, hand: Hand) -> Result<Self, Refusal> {
        let (pitch, drop) = (Positive::new(Stage::Recipe, pitch)?, Positive::new(Stage::Recipe, drop)?);
        let turns = drop.get() / pitch.get();
        if !turns.is_finite() || turns > MAX_TURNS {
            return Err(Refusal::new(Stage::Recipe, TOO_MANY_TURNS).fix("use a larger pitch or a smaller depth"));
        }
        Ok(Self { circle, pitch, drop, hand })
    }
    #[must_use]
    pub fn circle(&self) -> Circle {
        self.circle
    }
    #[must_use]
    pub fn pitch(&self) -> f64 {
        self.pitch.get()
    }
    #[must_use]
    pub fn drop(&self) -> f64 {
        self.drop.get()
    }
    #[must_use]
    pub fn hand(&self) -> Hand {
        self.hand
    }
}

/// The guide of a strategy. Each variant with data holds a checked choice.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum GuideChoice {
    Rings(RingsChoice),
    Rows(RowsChoice),
    /// The centre line of a slot or a channel.
    Medial,
    /// The selection itself (engrave, trace, chamfer), at a finite offset.
    Trace(TraceOffset),
    /// One helix (Thread, a helical bore).
    Helix(HelixChoice),
}
