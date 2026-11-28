//! L3 (#4027): the start side changes only the order and the start point.
//! The check runs the stages for Auto, Inside and Outside. For each side the
//! ordered curves must be the guide curves as an undirected multiset, and
//! the guide output and the ordered multiset must be the same for each
//! side (within 1 um). The kernel's `OrderedSet` already refuses a set that
//! is not a permutation of its input; this check also catches an order
//! stage that gives the set another input.
//!
//! #4406: a start side that refuses where another side plans is a broken
//! measure (the side changed more than the order). When each side refuses,
//! or a run has no curve to order, L3 is refused (not measured, never held).

use camcraft_pathkernel::guide::{Curve, GuideSet};
use camcraft_pathkernel::order::{OrderChoice, OrderPlanner, OrderedSet, StartSide};
use camcraft_pathkernel::region::KernelRegion;
use camcraft_pathkernel::{PlanePoint, Point2, Refusal, Region, Verdict};

use super::kit::{Case, Law, LawVerdict, MATCH_TOL, Measure, Planners, Run, assert_broken, assert_held, judge, run, set_gap};
use super::standins::{SideOrder, good, order_curves, resolve, square_case};

const SIDES: [StartSide; 3] = [StartSide::Auto, StartSide::Inside, StartSide::Outside];

fn guide_curves(r: &Run) -> Vec<&Curve> {
    r.guide.out().map(|g| g.curves().iter().collect()).unwrap_or_default()
}

fn order_curves_of(r: &Run) -> Vec<&Curve> {
    r.ordered.out().map(|o| o.curves().iter().collect()).unwrap_or_default()
}

/// L3 over the planners `p` on the case `c`.
pub fn check_l3(p: &Planners, c: &Case) -> LawVerdict {
    let runs: Vec<(StartSide, Result<Run, Refusal>)> = SIDES.iter().map(|&s| (s, run(p, c, s))).collect();
    let mut m = Vec::new();
    let ok: Vec<&Run> = runs.iter().filter_map(|(_, r)| r.as_ref().ok()).collect();
    if ok.is_empty() {
        for (s, r) in &runs {
            if let Err(e) = r {
                m.push(Measure::refusal(format!("start side {s:?}"), e));
            }
        }
        return judge(Law::L3, m);
    }
    if ok.len() != runs.len() {
        let refused: Vec<String> = runs.iter().filter_map(|(s, r)| r.as_ref().err().map(|e| format!("{s:?} ({})", e.reason()))).collect();
        m.push(Measure::count(refused.len(), format!("refused for {}", refused.join(", ")), None));
    }
    for r in &ok {
        if order_curves_of(r).is_empty() {
            m.push(Measure::unread(format!("start side {:?}", r.side), "the run has no curve to order"));
        }
        let (g, at) = set_gap(&order_curves_of(r), &guide_curves(r));
        m.push(Measure::new(g, MATCH_TOL, format!("start side {:?}: the ordered curves against the guide, {at}", r.side), None));
    }
    if let Some((first, rest)) = ok.split_first() {
        for r in rest {
            let (g, at) = set_gap(&guide_curves(first), &guide_curves(r));
            m.push(Measure::new(g, MATCH_TOL, format!("guide, {:?} against {:?}: {at}", first.side, r.side), None));
            let (g, at) = set_gap(&order_curves_of(first), &order_curves_of(r));
            m.push(Measure::new(g, MATCH_TOL, format!("ordered set, {:?} against {:?}: {at}", first.side, r.side), None));
        }
    }
    judge(Law::L3, m)
}

/// The guide set of `g` with each point moved by `dx` and with the last
/// `drop` curves left out: another input for the ordered set.
fn other_input(g: &GuideSet, dx: f64, drop: usize) -> Result<GuideSet, Refusal> {
    let keep = g.curves().len().saturating_sub(drop);
    let moved = g.curves()[..keep]
        .iter()
        .map(|c| {
            let pts =
                c.pts().iter().map(|p| PlanePoint::new(Point2::new(p.at().x + dx, p.at().y), p.depth())).collect::<Result<Vec<_>, _>>()?;
            Curve::new(pts, c.arc(), c.closed(), c.k(), c.family())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(GuideSet::new(moved))
}

/// Breaks L3: for the Outside start it moves the curves 0.1 mm, or drops
/// one, and gives the moved set to the ordered set as its input.
struct SideMoves {
    dx: f64,
    drop: usize,
}

impl OrderPlanner for SideMoves {
    fn order(&self, g: GuideSet, r: &KernelRegion, o: &OrderChoice, s: Option<&Region>) -> Result<Verdict<OrderedSet>, Refusal> {
        if o.start != StartSide::Outside {
            return SideOrder.order(g, r, o, s);
        }
        let other = other_input(&g, self.dx, self.drop)?;
        Ok(Verdict::done(order_curves(&other, other.curves().to_vec(), resolve(o.start), o.direction)?))
    }
}

#[test]
fn l3_holds_for_an_order_that_only_permutes_restarts_and_reverses() {
    assert_held(&check_l3(&good(), &square_case().unwrap()));
}

#[test]
fn l3_breaks_for_an_order_that_moves_the_curves_for_one_side() {
    let p = Planners { order: Box::new(SideMoves { dx: 0.1, drop: 0 }), ..good() };
    let v = check_l3(&p, &square_case().unwrap());
    assert_broken(&v, Law::L3, "Outside");
    assert!((v.worst.value - 0.1).abs() < 1e-6, "{v}");
}

#[test]
fn l3_breaks_for_an_order_that_drops_a_curve_for_one_side() {
    let p = Planners { order: Box::new(SideMoves { dx: 0.0, drop: 1 }), ..good() };
    let v = check_l3(&p, &square_case().unwrap());
    assert_broken(&v, Law::L3, "Outside");
    assert!(v.worst.value.is_infinite(), "{v}");
}
