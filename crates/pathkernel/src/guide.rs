//! Stage 2, Guide curves (K2): the guide choice, the curve type and the
//! canonical guide set with its named stretches. The choices are in
//! `guide/choice.rs`, the planners in the files of `guide/`: rings and rows
//! (#4176, first part); the medial line, the trace and the helix (second
//! part).
//! Provenance: our own design record (`RECOMMENDED.md`, `## #4024`)
//! "Stage 2: Guide curves"; contour-parallel and direction-parallel paths as
//! in Held (1991).

use std::cmp::Ordering;

use crate::frame::PlanePoint;
use crate::region::KernelRegion;
use crate::verdict::{Refusal, Stage, Verdict};
use crate::{KERNEL_TOL, finite};

mod all;
mod choice;
mod cover;
pub mod helix;
mod lead;
pub mod mark;
pub mod medial;
pub mod rings;
pub mod rows;
pub mod trace;

pub use all::Guides;
pub use choice::{
    BAD_OFFSET, Circle, GuideChoice, Hand, HelixChoice, MAX_TURNS, RingRef, RingsChoice, RowsChoice, TOO_MANY_TURNS, TraceOffset,
    ZERO_COUNT,
};
pub use cover::{BAD_TOL, GUIDE_GEOMETRY, GUIDE_GEOMETRY_INVALID, NOT_AN_AREA, NOT_PLANAR, STEP_ABOVE_DIAMETER};
pub use helix::HelixPlanner;
pub use lead::{HELIX_NEEDS_LEAD, Lead, NOT_ON_HELIX};
pub use mark::{Mark, MarkKind, Span};
pub use medial::MedialPlanner;
pub use rings::RingPlanner;
pub use rows::RowPlanner;
pub use trace::TracePlanner;

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
/// circle where it has one, closed or open, its index `k`, its family,
/// and the lead of a helix ([`Curve::helix`]; none for other curves).
#[derive(Clone, Debug, PartialEq)]
pub struct Curve {
    pts: Vec<PlanePoint>,
    arc: Option<Circle>,
    closed: bool,
    k: u32,
    family: Family,
    lead: Option<Lead>,
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
    /// A curve of any family but [`Family::Helix`] (see [`Curve::helix`]).
    pub fn new(pts: Vec<PlanePoint>, arc: Option<Circle>, closed: bool, k: u32, family: Family) -> Result<Self, Refusal> {
        if family == Family::Helix {
            return Err(Refusal::new(Stage::Guide, HELIX_NEEDS_LEAD));
        }
        Self::checked(pts, arc, closed, k, family, None)
    }

    fn checked(pts: Vec<PlanePoint>, arc: Option<Circle>, closed: bool, k: u32, family: Family, lead: Option<Lead>) -> Result<Self, Refusal> {
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
        Ok(Self { pts, arc, closed, k, family, lead })
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
    /// The pitch and the hand of a helix.
    #[must_use]
    pub fn lead(&self) -> Option<Lead> {
        self.lead
    }

    /// This curve with the points `pts`, made again through the constructor
    /// of its family ([`Curve::helix`] for a helix, [`Curve::new`] else)
    /// with its index, closure, circle and lead. The one way for a later
    /// stage to restart or reverse a curve: it keeps all the data of the
    /// family, or the constructor refuses the points (#4176). The caller
    /// maps a refusal to a reason of its own stage.
    pub(crate) fn with_points(&self, pts: Vec<PlanePoint>) -> Result<Self, Refusal> {
        match (self.family, self.arc, self.lead) {
            (Family::Helix, Some(circle), Some(lead)) => Self::helix(circle, lead, pts, self.k),
            (Family::Helix, ..) => Err(Refusal::new(Stage::Guide, HELIX_NEEDS_LEAD)),
            _ => Self::checked(pts, self.arc, self.closed, self.k, self.family, None),
        }
    }

    /// True when the curve goes down along the plane normal from its first
    /// point to its last (a helix as it cuts).
    #[must_use]
    pub(crate) fn descends(&self) -> bool {
        self.pts.first().zip(self.pts.last()).is_some_and(|(a, b)| b.depth() < a.depth() - KERNEL_TOL)
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
            .then_with(|| lead::cmp_lead(self.lead, o.lead))
    }

    /// True when `other` is this curve with another start (closed: a cyclic
    /// shift) or in the other direction, or both - and nothing else: the
    /// same index, family, closure and circle, the same points in the same
    /// cyclic sequence within [`KERNEL_TOL`] (law 3).
    #[must_use]
    pub fn same_cut(&self, other: &Self) -> bool {
        self.turn_of(other).is_some()
    }

    /// How `other` runs against this curve when it is the same cut
    /// ([`Curve::same_cut`]): `Some(false)` in the same direction,
    /// `Some(true)` reversed, `None` when it is not the same cut. A curve
    /// that matches both ways reads as the same direction.
    #[must_use]
    pub(crate) fn turn_of(&self, other: &Self) -> Option<bool> {
        let arc_same = match (self.arc, other.arc) {
            (None, None) => true,
            (Some(a), Some(b)) => a.near(&b),
            _ => false,
        };
        let lead_same = lead::same_lead(self.lead, other.lead);
        if self.k != other.k || self.family != other.family || self.closed != other.closed || !arc_same || !lead_same {
            return None;
        }
        let n = self.pts.len();
        if n != other.pts.len() {
            return None;
        }
        let rev: Vec<PlanePoint> = self.pts.iter().rev().copied().collect();
        let matches = |seq: &[PlanePoint], s: usize| (0..n).all(|j| seq[(s + j) % n].near(&other.pts[j]));
        let starts = if self.closed { n } else { 1 };
        if (0..starts).any(|s| matches(&self.pts, s)) {
            return Some(false);
        }
        (0..starts).any(|s| matches(&rev, s)).then_some(true)
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
