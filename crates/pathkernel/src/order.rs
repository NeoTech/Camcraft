//! Stage 4, Order (K4): the start side, the direction and the ordered set.
//! The ordered set's constructor checks law 3: the output curves are the
//! input curves, permuted, with a start point and a direction. Nothing else
//! changes. K4 adds `order()`.
//! Provenance: our own design record `docs/decisions/4024-path-kernel-design.md`
//! "Stage 4: Order"; climb and conventional milling as in any machining
//! handbook.

use crate::guide::{Curve, GuideSet};
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

/// The ordered curves, the first start point, and the side that Auto
/// resolved to (never Auto).
#[derive(Clone, Debug, PartialEq)]
pub struct OrderedSet {
    curves: Vec<Curve>,
    start: Point2,
    resolved: StartSide,
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
    pub fn new(input: &GuideSet, curves: Vec<Curve>, start: Point2, resolved: StartSide) -> Result<Self, Refusal> {
        if resolved == StartSide::Auto {
            return Err(Refusal::new(Stage::Order, UNRESOLVED));
        }
        if !finite(&[start.x, start.y]) {
            return Err(Refusal::new(Stage::Order, BAD_START));
        }
        if !is_permutation(input.curves(), &curves) {
            return Err(Refusal::new(Stage::Order, NOT_A_PERMUTATION));
        }
        if curves.first().and_then(|c| c.pts().first()).is_some_and(|p| !crate::near(p.at(), start)) {
            return Err(Refusal::new(Stage::Order, START_OFF_CURVE).at(start));
        }
        Ok(Self { curves, start, resolved })
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
}

/// True when `out` holds each curve of `input` once, each only restarted
/// or reversed ([`Curve::same_cut`]).
fn is_permutation(input: &[Curve], out: &[Curve]) -> bool {
    if input.len() != out.len() {
        return false;
    }
    let mut used = vec![false; input.len()];
    out.iter().all(|c| {
        let hit = input.iter().enumerate().find(|(i, g)| !used[*i] && g.same_cut(c));
        hit.map(|(i, _)| used[i] = true).is_some()
    })
}

/// An order planner. It reads no material side (law 2).
pub trait OrderPlanner {
    fn order(&self, g: GuideSet, r: &KernelRegion, o: &OrderChoice, stock: Option<&Region>) -> Result<Verdict<OrderedSet>, Refusal>;
}
