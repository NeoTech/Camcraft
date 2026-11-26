//! Named stretches of a guide set (#4176): a guide planner names a stretch
//! that a later stage must act on, in place of a silent choice. The ring
//! planner names a full-width stretch (#3788, the merge of two lobes) and
//! a clean-up ring; the row planner names its closing ring.
//! Provenance: our own design record `docs/decisions/4024-path-kernel-design.md`
//! "Stage 2: Guide curves" and the Prime Directive (a full-width cut is
//! named, not hidden); `docs/decisions/4176-kernel-guides-rings-rows.md`.

use std::cmp::Ordering;

use super::{Curve, cmp_seq};
use crate::frame::PlanePoint;
use crate::verdict::{Refusal, Stage};
use crate::{KERNEL_TOL, seg_dist};

/// What a mark names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum MarkKind {
    /// The tool disc on this stretch overlaps the disc of no deeper curve
    /// and no air: when the deeper curves are cut first, the tool cuts
    /// here at full width (180 degrees). The load stage (K3) and the order
    /// and link stage (K4) act on it.
    FullWidth,
    /// A clean-up ring: it cuts an area that the regular rings leave, so
    /// that the swept discs cover the cutter-centre area.
    Fill,
    /// The closing ring that a row choice asks for.
    Closing,
}

/// The part of a curve that a mark names.
#[derive(Clone, Debug, PartialEq)]
pub enum Span {
    /// The whole curve.
    Whole,
    /// A stretch of the curve: two or more points on it, in its direction.
    Part(Vec<PlanePoint>),
}

/// A named stretch of curve `curve` of its guide set.
#[derive(Clone, Debug, PartialEq)]
pub struct Mark {
    curve: usize,
    kind: MarkKind,
    span: Span,
}

/// Refusal reason: a stretch with fewer than two points or no length.
pub const BAD_STRETCH: &str = "a marked stretch needs two or more points and a length above the kernel tolerance";
/// Refusal reason: a mark on a curve that the set does not have.
pub const MARK_NO_CURVE: &str = "a mark names a curve that the guide set does not have";
/// Refusal reason: a marked stretch with a point off its curve.
pub const MARK_OFF_CURVE: &str = "a marked stretch has a point that is not on its curve";

impl Mark {
    /// A mark on the whole of curve `curve`.
    #[must_use]
    pub fn whole(curve: usize, kind: MarkKind) -> Self {
        Self { curve, kind, span: Span::Whole }
    }

    /// A mark on a stretch of curve `curve`. Refused for fewer than two
    /// points or a stretch with no length; [`super::GuideSet::with_marks`]
    /// checks that each point is on the curve.
    pub fn part(curve: usize, kind: MarkKind, pts: Vec<PlanePoint>) -> Result<Self, Refusal> {
        let length: f64 = pts.windows(2).map(|w| (w[1].at().x - w[0].at().x).hypot(w[1].at().y - w[0].at().y)).sum();
        if pts.len() < 2 || !length.is_finite() || length <= KERNEL_TOL {
            return Err(Refusal::new(Stage::Guide, BAD_STRETCH));
        }
        Ok(Self { curve, kind, span: Span::Part(pts) })
    }
    /// The index of the marked curve in [`super::GuideSet::curves`].
    #[must_use]
    pub fn curve(&self) -> usize {
        self.curve
    }
    #[must_use]
    pub fn kind(&self) -> MarkKind {
        self.kind
    }
    #[must_use]
    pub fn span(&self) -> &Span {
        &self.span
    }

    /// This mark on curve index `i` of a list that a planner builds.
    pub(crate) fn renumber(self, i: usize) -> Self {
        Self { curve: i, ..self }
    }

    /// This mark moved to curve index `i`, checked against that curve.
    pub(super) fn onto(self, i: usize, c: &Curve) -> Result<Self, Refusal> {
        if let Span::Part(pts) = &self.span
            && !pts.iter().all(|p| on_curve(c, p))
        {
            return Err(Refusal::new(Stage::Guide, MARK_OFF_CURVE).at(pts.first().map_or_else(Default::default, PlanePoint::at)));
        }
        Ok(Self { curve: i, ..self })
    }

    /// The total order of the marks of a set: curve, kind, the whole curve
    /// before a stretch, then the points.
    pub(super) fn cmp_set(&self, o: &Self) -> Ordering {
        self.curve.cmp(&o.curve).then(self.kind.cmp(&o.kind)).then_with(|| match (&self.span, &o.span) {
            (Span::Whole, Span::Whole) => Ordering::Equal,
            (Span::Whole, Span::Part(_)) => Ordering::Less,
            (Span::Part(_), Span::Whole) => Ordering::Greater,
            (Span::Part(a), Span::Part(b)) => cmp_seq(a, b),
        })
    }
}

/// True when `p` is on curve `c` (a segment of it, the closing one of a
/// closed curve included) within [`KERNEL_TOL`] in place and depth.
fn on_curve(c: &Curve, p: &PlanePoint) -> bool {
    let pts = c.pts();
    let close = c.closed().then(|| (pts.last(), pts.first()));
    let segs = pts.windows(2).map(|w| (&w[0], &w[1])).chain(close.into_iter().filter_map(|(a, b)| a.zip(b)));
    let flat = pts.iter().all(|q| (q.depth() - p.depth()).abs() <= KERNEL_TOL);
    flat && segs.into_iter().any(|(a, b)| seg_dist(p.at(), a.at(), b.at()) <= KERNEL_TOL)
}
