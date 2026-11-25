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
pub const NOT_A_PERMUTATION: &str = "the order stage may only permute, restart and reverse the guide curves";
/// Refusal reason: Auto was not resolved to Inside or Outside.
pub const UNRESOLVED: &str = "the order stage must resolve Auto to Inside or Outside";
/// Refusal reason: a start point that is not finite.
pub const BAD_START: &str = "the start point is not a finite number";

impl OrderedSet {
    /// The ordered set made from `input`. Refused unless `curves` is a
    /// permutation of the input curves (each the same point set, same `k`
    /// and family), `resolved` is not Auto and `start` is finite.
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

/// True when `out` holds each curve of `input` once, as the same point set.
fn is_permutation(input: &[Curve], out: &[Curve]) -> bool {
    if input.len() != out.len() {
        return false;
    }
    let mut used = vec![false; input.len()];
    out.iter().all(|c| {
        let hit = input.iter().enumerate().find(|(i, g)| {
            !used[*i] && g.k() == c.k() && g.family() == c.family() && g.closed() == c.closed() && g.same_points(c)
        });
        hit.map(|(i, _)| used[i] = true).is_some()
    })
}

/// An order planner. It reads no material side (law 2).
pub trait OrderPlanner {
    fn order(&self, g: GuideSet, r: &KernelRegion, o: &OrderChoice, stock: Option<&Region>) -> Result<Verdict<OrderedSet>, Refusal>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::PlanePoint;
    use crate::guide::Family;

    fn curve(xy: &[(f64, f64)], k: u32) -> Curve {
        let pts = xy.iter().map(|&(x, y)| PlanePoint::flat(Point2::new(x, y))).collect();
        Curve::new(pts, None, true, k, Family::Ring).unwrap()
    }

    fn set() -> (GuideSet, Curve, Curve) {
        let a = curve(&[(0.0, 0.0), (4.0, 0.0), (4.0, 4.0)], 0);
        let b = curve(&[(1.0, 1.0), (3.0, 1.0), (3.0, 3.0)], 1);
        (GuideSet::new(vec![a.clone(), b.clone()]), a, b)
    }

    #[test]
    fn a_permutation_with_reversed_curves_is_accepted() {
        let (g, a, _) = set();
        let b_rev = curve(&[(3.0, 3.0), (3.0, 1.0), (1.0, 1.0)], 1);
        let o = OrderedSet::new(&g, vec![b_rev, a], Point2::new(3.0, 3.0), StartSide::Inside).unwrap();
        assert_eq!((o.curves().len(), o.resolved()), (2, StartSide::Inside));
    }

    #[test]
    fn a_changed_dropped_or_doubled_curve_is_refused() {
        let (g, a, b) = set();
        let moved = curve(&[(1.0, 1.0), (3.0, 1.0), (3.0, 3.5)], 1);
        let p = Point2::new(0.0, 0.0);
        assert_eq!(OrderedSet::new(&g, vec![a.clone(), moved], p, StartSide::Outside).unwrap_err().reason, NOT_A_PERMUTATION);
        assert!(OrderedSet::new(&g, vec![a.clone()], p, StartSide::Outside).is_err());
        assert!(OrderedSet::new(&g, vec![a.clone(), a.clone()], p, StartSide::Outside).is_err());
        let relabelled = curve(&[(1.0, 1.0), (3.0, 1.0), (3.0, 3.0)], 7);
        assert!(OrderedSet::new(&g, vec![a.clone(), relabelled], p, StartSide::Outside).is_err());
        assert!(OrderedSet::new(&g, vec![b, a], p, StartSide::Outside).is_ok());
    }

    #[test]
    fn auto_must_be_resolved_and_the_start_finite() {
        let (g, a, b) = set();
        let p = Point2::new(0.0, 0.0);
        assert_eq!(OrderedSet::new(&g, vec![a.clone(), b.clone()], p, StartSide::Auto).unwrap_err().reason, UNRESOLVED);
        assert_eq!(OrderedSet::new(&g, vec![a, b], Point2::new(f64::NAN, 0.0), StartSide::Inside).unwrap_err().reason, BAD_START);
    }
}
