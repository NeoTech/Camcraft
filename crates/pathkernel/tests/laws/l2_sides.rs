//! L2 (#4027): the material side and the start side are separate
//! parameters. At compile level the region stage takes no start side and
//! the order stage takes no material side (the trait signatures). The check
//! reads the behaviour: flipping the material side changes the centre area,
//! and the order stage gives the same sequence for two regions that differ
//! only in their material side.

use camcraft_pathkernel::guide::GuideSet;
use camcraft_pathkernel::order::{OrderChoice, OrderPlanner, OrderedSet, StartSide};
use camcraft_pathkernel::region::{KernelRegion, MaterialSide};
use camcraft_pathkernel::{Refusal, Region, Verdict};

use super::kit::{Law, LawVerdict, MATCH_TOL, Measure, assert_broken, assert_held, dist, judge, planar, rebuild, seq_gap};
use super::standins::{RingGuide, SideOrder, order_curves, resolve, rings_choice, square_region};

/// The compile-level half: a region stage is a function of the material
/// side alone. This line stops compiling if a start side is added.
const _REGION_STAGE: fn(MaterialSide) -> Result<KernelRegion, Refusal> = square_region;

/// The area of the symmetric difference of two regions.
fn sym_diff(a: &Region, b: &Region) -> f64 {
    let one = |x: &Region, y: &Region| x.difference(y).map_or(f64::INFINITY, |d| d.area().abs());
    one(a, b) + one(b, a)
}

fn ordered(o: &dyn OrderPlanner, g: &GuideSet, r: &KernelRegion, oc: &OrderChoice) -> Option<OrderedSet> {
    o.order(g.clone(), r, oc, None).ok().and_then(|v| v.out().cloned())
}

/// L2 over the region stage `region` and the order stage `order` on the
/// guide set `guide`, for each start side with the direction of `choice`.
pub fn check_l2(
    region: fn(MaterialSide) -> Result<KernelRegion, Refusal>,
    order: &dyn OrderPlanner,
    guide: &GuideSet,
    choice: OrderChoice,
) -> LawVerdict {
    let mut m = Vec::new();
    match (region(MaterialSide::Inside), region(MaterialSide::Outside)) {
        (Ok(a), Ok(b)) => {
            let d = match (planar(&a), planar(&b)) {
                (Ok(pa), Ok(pb)) => sym_diff(pa.center(), pb.center()),
                _ => 0.0,
            };
            let same = usize::from(d <= MATCH_TOL);
            m.push(Measure::count(same, format!("region: Inside and Outside centres differ by {d:.4} mm2"), None));
        }
        (Err(e), _) | (_, Err(e)) => m.push(Measure::new(0.0, 0.0, format!("region: one side refused ({})", e.reason()), None)),
    }
    let Ok(inside) = region(MaterialSide::Inside) else {
        return judge(Law::L2, m);
    };
    let flipped = planar(&inside).and_then(|p| rebuild(p, MaterialSide::Outside, p.center().clone(), p.part_walls().to_vec()));
    let Ok(flipped) = flipped else {
        return judge(Law::L2, m);
    };
    for side in [StartSide::Auto, StartSide::Inside, StartSide::Outside] {
        let oc = OrderChoice { start: side, direction: choice.direction };
        let place = format!("order, start side {side:?}: the material side changed");
        match (ordered(order, guide, &inside, &oc), ordered(order, guide, &flipped, &oc)) {
            (Some(a), Some(b)) => {
                let (x, y): (Vec<_>, Vec<_>) = (a.curves().iter().collect(), b.curves().iter().collect());
                let (g, at) = seq_gap(&x, &y);
                m.push(Measure::new(g.max(dist(a.start(), b.start())), MATCH_TOL, format!("{place} {at}"), Some(a.start())));
            }
            (None, None) => m.push(Measure::new(0.0, MATCH_TOL, format!("{place}: refused both times"), None)),
            _ => m.push(Measure::new(f64::INFINITY, MATCH_TOL, format!("{place}: refused one time"), None)),
        }
    }
    judge(Law::L2, m)
}

/// Breaks L2: the centre does not follow the material side.
fn blind_region(_m: MaterialSide) -> Result<KernelRegion, Refusal> {
    square_region(MaterialSide::Inside)
}

/// Breaks L2: the order reads the material side and starts outside for it.
struct MaterialOrder;

impl OrderPlanner for MaterialOrder {
    fn order(&self, g: GuideSet, r: &KernelRegion, o: &OrderChoice, _s: Option<&Region>) -> Result<Verdict<OrderedSet>, Refusal> {
        let side = match planar(r)?.material() {
            MaterialSide::Outside => StartSide::Outside,
            MaterialSide::Inside => resolve(o.start),
        };
        Ok(Verdict::done(order_curves(&g, g.curves().to_vec(), side, o.direction)?))
    }
}

fn guide() -> GuideSet {
    use camcraft_pathkernel::guide::GuidePlanner;
    let r = square_region(MaterialSide::Inside).unwrap();
    RingGuide { extra: 0.0 }.guide(&r, &rings_choice().unwrap(), 1e-3).unwrap().out().unwrap().clone()
}

#[test]
fn l2_holds_for_a_region_and_an_order_that_keep_the_sides_apart() {
    assert_held(&check_l2(square_region, &SideOrder, &guide(), OrderChoice::default()));
}

#[test]
fn l2_breaks_for_a_region_that_ignores_the_material_side() {
    assert_broken(&check_l2(blind_region, &SideOrder, &guide(), OrderChoice::default()), Law::L2, "region: Inside and Outside");
}

#[test]
fn l2_breaks_for_an_order_that_reads_the_material_side() {
    let v = check_l2(square_region, &MaterialOrder, &guide(), OrderChoice::default());
    assert_broken(&v, Law::L2, "order, start side");
    assert!(!v.worst.place.contains("Outside"), "the Outside start reads the same for both sides: {v}");
    assert!(v.worst.value > 1.0, "{v}");
}
