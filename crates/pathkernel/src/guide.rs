//! Stage 2, Guide curves (K2): the guide choice, the curve type and the
//! canonical guide set with its named stretches. The planners are in the
//! files of `guide/`: rings and rows (#4176, first part); the medial line,
//! the trace and the helix follow.
//! Provenance: our own design record `docs/decisions/4024-path-kernel-design.md`
//! "Stage 2: Guide curves"; contour-parallel and direction-parallel paths as
//! in Held (1991).

use std::cmp::Ordering;

use crate::frame::{PlanePoint, Positive, Spot};
use crate::region::KernelRegion;
use crate::verdict::{Refusal, Stage, Verdict};
use crate::{KERNEL_TOL, finite};

mod all;
mod cover;
pub mod mark;
pub mod rings;
pub mod rows;

pub use all::{FAMILY_NOT_YET, Guides};
pub use cover::{BAD_TOL, GUIDE_GEOMETRY, NOT_PLANAR, STEP_ABOVE_DIAMETER};
pub use mark::{Mark, MarkKind, Span};
pub use rings::RingPlanner;
pub use rows::RowPlanner;

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
}

/// The finite offset of a trace guide.
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

/// The family of a curve, for the canonical sort.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Family {
    Ring,
    Row,
    Medial,
    Trace,
    Helix,
}

/// One guide curve: points with a depth along the plane normal, an exact
/// circle where it has one, closed or open, its index `k` and its family.
#[derive(Clone, Debug, PartialEq)]
pub struct Curve {
    pts: Vec<PlanePoint>,
    arc: Option<Circle>,
    closed: bool,
    k: u32,
    family: Family,
}

/// Refusal reason: a curve with too few points.
pub const BAD_CURVE: &str = "a curve needs two or more points when open, three or more when closed";
/// Refusal reason: a curve of zero length.
pub const ZERO_LENGTH: &str = "a curve must have a length above the kernel tolerance";
/// Refusal reason: a closed curve whose last point repeats its first.
pub const REPEATED_START: &str = "a closed curve must not repeat its first point at the end: the closure is implicit";

/// The total order of two points: y, then x, then depth.
fn cmp_pt(a: &PlanePoint, b: &PlanePoint) -> Ordering {
    let (p, q) = (a.at(), b.at());
    p.y.total_cmp(&q.y).then(p.x.total_cmp(&q.x)).then(a.depth().total_cmp(&b.depth()))
}

/// The lexicographic order of two point sequences.
fn cmp_seq(a: &[PlanePoint], b: &[PlanePoint]) -> Ordering {
    a.iter().zip(b).map(|(p, q)| cmp_pt(p, q)).find(|o| o.is_ne()).unwrap_or_else(|| a.len().cmp(&b.len()))
}

fn cmp_arc(a: Option<Circle>, b: Option<Circle>) -> Ordering {
    let key = |c: Option<Circle>| c.map(|c| [c.center().y, c.center().x, c.radius()]);
    match (key(a), key(b)) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
        (Some(x), Some(y)) => x.iter().zip(&y).map(|(p, q)| p.total_cmp(q)).find(|o| o.is_ne()).unwrap_or(Ordering::Equal),
    }
}

impl Curve {
    pub fn new(pts: Vec<PlanePoint>, arc: Option<Circle>, closed: bool, k: u32, family: Family) -> Result<Self, Refusal> {
        let need = if closed { 3 } else { 2 };
        if pts.len() < need {
            return Err(Refusal::new(Stage::Guide, BAD_CURVE));
        }
        if closed && pts.first().zip(pts.last()).is_some_and(|(a, b)| a.near(b)) {
            return Err(Refusal::new(Stage::Guide, REPEATED_START));
        }
        let length: f64 = pts.windows(2).map(|w| (w[1].at().x - w[0].at().x).hypot(w[1].at().y - w[0].at().y)).sum();
        if !finite(&[length]) || length <= KERNEL_TOL {
            return Err(Refusal::new(Stage::Guide, ZERO_LENGTH));
        }
        Ok(Self { pts, arc, closed, k, family })
    }
    #[must_use]
    pub fn pts(&self) -> &[PlanePoint] {
        &self.pts
    }
    #[must_use]
    pub fn arc(&self) -> Option<Circle> {
        self.arc
    }
    #[must_use]
    pub fn closed(&self) -> bool {
        self.closed
    }
    #[must_use]
    pub fn k(&self) -> u32 {
        self.k
    }
    #[must_use]
    pub fn family(&self) -> Family {
        self.family
    }

    /// The canonical form: a closed curve starts at its lowest point (least
    /// y, then x, then depth; among equal lowest points, the start that gives
    /// the least sequence); an open curve runs from its lesser end.
    fn canonical(mut self) -> Self {
        if self.closed {
            let n = self.pts.len();
            let rot = |s: usize| -> Vec<PlanePoint> { self.pts[s..].iter().chain(&self.pts[..s]).copied().collect() };
            let low = self.pts.iter().min_by(|a, b| cmp_pt(a, b)).copied();
            let best = (0..n)
                .filter(|&i| low.is_some_and(|l| cmp_pt(&self.pts[i], &l).is_eq()))
                .min_by(|&a, &b| cmp_seq(&rot(a), &rot(b)));
            self.pts.rotate_left(best.unwrap_or(0));
        } else {
            let rev: Vec<PlanePoint> = self.pts.iter().rev().copied().collect();
            if cmp_seq(&rev, &self.pts).is_lt() {
                self.pts = rev;
            }
        }
        self
    }

    /// The total order of the guide set: `k`, family, open before closed,
    /// the point sequence, then the circle.
    fn cmp_set(&self, o: &Self) -> Ordering {
        self.k
            .cmp(&o.k)
            .then(self.family.cmp(&o.family))
            .then(self.closed.cmp(&o.closed))
            .then_with(|| cmp_seq(&self.pts, &o.pts))
            .then_with(|| cmp_arc(self.arc, o.arc))
    }

    /// True when `other` is this curve with another start (closed: a cyclic
    /// shift) or in the other direction, or both - and nothing else: the
    /// same index, family, closure and circle, the same points in the same
    /// cyclic sequence within [`KERNEL_TOL`] (law 3).
    #[must_use]
    pub fn same_cut(&self, other: &Self) -> bool {
        let arc_same = match (self.arc, other.arc) {
            (None, None) => true,
            (Some(a), Some(b)) => a.near(&b),
            _ => false,
        };
        if self.k != other.k || self.family != other.family || self.closed != other.closed || !arc_same {
            return false;
        }
        let n = self.pts.len();
        if n != other.pts.len() {
            return false;
        }
        let rev: Vec<PlanePoint> = self.pts.iter().rev().copied().collect();
        let matches = |seq: &[PlanePoint], s: usize| (0..n).all(|j| seq[(s + j) % n].near(&other.pts[j]));
        let starts = if self.closed { n } else { 1 };
        (0..starts).any(|s| matches(&self.pts, s) || matches(&rev, s))
    }
}

/// A set of guide curves: no order and no start point (they are stage 4),
/// and the stretches that the planner names ([`Mark`]). The constructors
/// make the set canonical with a total order: the same curves in any input
/// order and from any start give the same set (law 1).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GuideSet {
    curves: Vec<Curve>,
    marks: Vec<Mark>,
}

impl GuideSet {
    #[must_use]
    pub fn new(curves: Vec<Curve>) -> Self {
        let mut curves: Vec<Curve> = curves.into_iter().map(Curve::canonical).collect();
        curves.sort_by(Curve::cmp_set);
        Self { curves, marks: Vec::new() }
    }

    /// The canonical set with its marks. Each mark names a curve by its
    /// index in `curves`; the set moves it to the canonical index. Refused
    /// for a mark on a curve that is not in `curves`, or a marked stretch
    /// with a point off its curve.
    pub fn with_marks(curves: Vec<Curve>, marks: Vec<Mark>) -> Result<Self, Refusal> {
        let mut tagged: Vec<(usize, Curve)> = curves.into_iter().map(Curve::canonical).enumerate().collect();
        tagged.sort_by(|a, b| a.1.cmp_set(&b.1));
        let mut at = vec![0usize; tagged.len()];
        for (new, (old, _)) in tagged.iter().enumerate() {
            at[*old] = new;
        }
        let curves: Vec<Curve> = tagged.into_iter().map(|(_, c)| c).collect();
        let mut out = Vec::with_capacity(marks.len());
        for m in marks {
            let i = at.get(m.curve()).copied().ok_or(Refusal::new(Stage::Guide, mark::MARK_NO_CURVE))?;
            let c = curves.get(i).ok_or(Refusal::new(Stage::Guide, mark::MARK_NO_CURVE))?;
            out.push(m.onto(i, c)?);
        }
        out.sort_by(Mark::cmp_set);
        Ok(Self { curves, marks: out })
    }
    #[must_use]
    pub fn curves(&self) -> &[Curve] {
        &self.curves
    }
    /// The named stretches, in the canonical order of the set.
    #[must_use]
    pub fn marks(&self) -> &[Mark] {
        &self.marks
    }
}

/// A guide planner: a pure function of the region and the choice (law 1).
pub trait GuidePlanner {
    fn guide(&self, r: &KernelRegion, g: &GuideChoice, tol: f64) -> Result<Verdict<GuideSet>, Refusal>;
}
