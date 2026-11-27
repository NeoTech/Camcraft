//! Stage 4, Order (K4): the start side, the direction and the ordered set.
//! The ordered set's constructor checks law 3: the output curves are the
//! input curves, permuted, with a start point and a direction. Nothing else
//! changes. The marks of the guide set go with their curves. The planner
//! is in `order/` (#4177).
//! Provenance: our own design record `docs/decisions/4024-path-kernel-design.md`
//! "Stage 4: Order"; climb and conventional milling as in any machining
//! handbook.

mod chain;
mod side;
mod stage;

pub use stage::{ORDER_NOT_PLANAR, OrderStage};

use crate::guide::{Curve, GuideSet, Mark};
use crate::region::KernelRegion;
use crate::verdict::{Refusal, Stage, Verdict};
use crate::{Point2, Region, finite};

/// Where the passes start. The material side is not here: it is stage 1.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum StartSide {
    #[default]
    Auto,
    Inside,
    Outside,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Direction {
    #[default]
    Climb,
    Conventional,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct OrderChoice {
    pub start: StartSide,
    pub direction: Direction,
}

/// The ordered curves, the first start point, the side that Auto
/// resolved to (never Auto), and the marks of the guide set, each on its
/// curve's new index.
#[derive(Clone, Debug, PartialEq)]
pub struct OrderedSet {
    curves: Vec<Curve>,
    start: Point2,
    resolved: StartSide,
    marks: Vec<Mark>,
}

/// Refusal reason: the order stage changed a curve, dropped one or added one.
pub const NOT_A_PERMUTATION: &str = "the order stage may only permute the guide curves, restart a closed curve and reverse a curve";
/// Refusal reason: Auto was not resolved to Inside or Outside.
pub const UNRESOLVED: &str = "the order stage must resolve Auto to Inside or Outside";
/// Refusal reason: a start point that is not the first point of the first curve.
pub const START_OFF_CURVE: &str = "the start point must be the first point of the first ordered curve";
/// Refusal reason: a start point that is not finite.
pub const BAD_START: &str = "the start point is not a finite number";

impl OrderedSet {
    /// The ordered set made from `input`. Refused unless `curves` is a
    /// permutation of the input curves, each only restarted (closed) or
    /// reversed, with its index, family and circle; `resolved` is not Auto; and `start`
    /// is the first point of the first curve (any finite point for no curve).
    /// Each mark of `input` moves with its curve: no planner can drop or
    /// move a mark (#4177).
    pub fn new(input: &GuideSet, curves: Vec<Curve>, start: Point2, resolved: StartSide) -> Result<Self, Refusal> {
        if resolved == StartSide::Auto {
            return Err(Refusal::new(Stage::Order, UNRESOLVED));
        }
        if !finite(&[start.x, start.y]) {
            return Err(Refusal::new(Stage::Order, BAD_START));
        }
        let Some(map) = permutation(input.curves(), &curves) else {
            return Err(Refusal::new(Stage::Order, NOT_A_PERMUTATION));
        };
        if curves.first().and_then(|c| c.pts().first()).is_some_and(|p| !crate::near(p.at(), start)) {
            return Err(Refusal::new(Stage::Order, START_OFF_CURVE).at(start));
        }
        let mut marks = Vec::with_capacity(input.marks().len());
        for m in input.marks() {
            let hit = map.iter().enumerate().find(|(_, (i, _))| *i == m.curve());
            let Some((j, &(_, reversed))) = hit else {
                return Err(Refusal::new(Stage::Order, NOT_A_PERMUTATION));
            };
            marks.push(m.clone().follow(j, reversed));
        }
        marks.sort_by(Mark::cmp_set);
        Ok(Self { curves, start, resolved, marks })
    }
    #[must_use]
    pub fn curves(&self) -> &[Curve] {
        &self.curves
    }
    #[must_use]
    pub fn start(&self) -> Point2 {
        self.start
    }
    #[must_use]
    pub fn resolved(&self) -> StartSide {
        self.resolved
    }
    /// The named stretches of the guide set, each on the index of its
    /// curve in [`OrderedSet::curves`] (a stretch of a reversed curve runs
    /// in the new direction). The load stage (K3) reads them.
    #[must_use]
    pub fn marks(&self) -> &[Mark] {
        &self.marks
    }
}

/// For each curve of `out`, the index of its input curve and whether it is
/// reversed; `None` unless `out` holds each curve of `input` once, each
/// only restarted or reversed ([`Curve::same_cut`]).
fn permutation(input: &[Curve], out: &[Curve]) -> Option<Vec<(usize, bool)>> {
    if input.len() != out.len() {
        return None;
    }
    let mut used = vec![false; input.len()];
    let mut map = Vec::with_capacity(out.len());
    for c in out {
        let (i, rev) = input.iter().enumerate().filter(|(i, _)| !used[*i]).find_map(|(i, g)| g.turn_of(c).map(|r| (i, r)))?;
        used[i] = true;
        map.push((i, rev));
    }
    Some(map)
}

/// An order planner. It reads no material side (law 2).
pub trait OrderPlanner {
    fn order(&self, g: GuideSet, r: &KernelRegion, o: &OrderChoice, stock: Option<&Region>) -> Result<Verdict<OrderedSet>, Refusal>;
}
