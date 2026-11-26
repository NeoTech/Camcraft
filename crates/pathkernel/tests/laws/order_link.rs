//! The K4 pins (#4177): the kernel's order stage and link stage, registered
//! with the law kit and run with the ring planner of K2 on the disc of
//! #3934 (as a pocket and with the material round it), the dog bone of
//! #3741 with the material round it, and a pocket with an island. L3 and
//! the link rule hold; each lift has a named reason and the lift count of
//! each case is pinned. The negative pins: a link across an island is
//! refused, a lift with no reason is refused, an order that moves a curve
//! is refused.

use camcraft_pathkernel::entry::EntryKind;
use camcraft_pathkernel::guide::{Curve, GuideChoice, GuidePlanner, MarkKind, RingPlanner, RingRef, RingsChoice, Span};
use camcraft_pathkernel::link::{CUTS_PART, Hop, Level, LinkPolicy, LinkProof, LinkStage, LinkTally, USER_RETRACT};
use camcraft_pathkernel::load::{Controlled, LoadControl, LoadPlanner, StockRead};
use camcraft_pathkernel::order::{Direction, NOT_A_PERMUTATION, OrderChoice, OrderPlanner, OrderStage, OrderedSet, StartSide};
use camcraft_pathkernel::plan::Cut;
use camcraft_pathkernel::verdict::NO_REASON;
use camcraft_pathkernel::{Bounds, KernelRegion, MaterialSide, PlanePoint, Point2, Refusal, Region, Selection, Verdict, region};

use super::guide_kit::{circle, outline, pocket, rect, tool, xy};
use super::kit::{Case, Planners, assert_held, run};
use super::l3_order::check_l3;
use super::link_rule::check_link;
use super::standins::{ProvedEntry, ShapeReader, island_region, recipe};

/// The tool radius and the ring step of the #4154 fixtures.
const TOOL_R: f64 = 4.0;
const STEP: f64 = 2.0;

/// The load stage as the identity: L3 and the link rule read no load.
struct Identity;

impl LoadPlanner for Identity {
    fn control(&self, s: OrderedSet, _r: &KernelRegion, _st: &dyn StockRead, _lc: &LoadControl, _tool_r: f64) -> Result<Verdict<Controlled>, Refusal> {
        Ok(Verdict::done(Controlled::new(s, vec![], vec![], 0.5)?))
    }
}

/// The kernel's order and link stages with the ring planner of K2.
pub fn kernel() -> Planners {
    Planners {
        guide: Box::new(RingPlanner),
        order: Box::new(OrderStage),
        load: Box::new(Identity),
        link: Box::new(LinkStage),
        entry: Box::new(ProvedEntry),
    }
}

pub fn stock_box() -> Region {
    Region::new(vec![camcraft_pathkernel::Loop::new(rect(-40.0, -25.0, 40.0, 25.0)).unwrap()]).unwrap()
}

fn round(part: Vec<Point2>, m: MaterialSide) -> Result<KernelRegion, Refusal> {
    let sel = Selection::face(part, vec![])?;
    let b = Bounds::new(Some(stock_box()), Region::empty(), 0.0, -5.0)?;
    region(&sel, m, &b, xy(), tool(TOOL_R), 0.0)
}

/// The disc of radius 12 in the 80 x 50 stock box.
pub fn disc(m: MaterialSide) -> Result<KernelRegion, Refusal> {
    round(circle(0.0, 0.0, 12.0, 128), m)
}

/// Two discs of radius 12, 30 mm apart, joined by a neck 8 mm wide.
pub fn dog_bone(m: MaterialSide) -> Result<KernelRegion, Refusal> {
    round(outline(&[circle(-15.0, 0.0, 12.0, 128), circle(15.0, 0.0, 12.0, 128), rect(-15.0, -4.0, 15.0, 4.0)]), m)
}

fn rings(reference: RingRef, step: f64) -> GuideChoice {
    GuideChoice::Rings(RingsChoice::new(reference, 0.0, step, None, None).unwrap())
}

fn case(name: &'static str, region: fn(MaterialSide) -> Result<KernelRegion, Refusal>, material: MaterialSide, guide: GuideChoice, no_split: bool) -> Case {
    let stock = stock_box();
    let recipe = recipe(guide, EntryKind::Helix, LinkPolicy::StayWhereProved).unwrap();
    Case { name, region, material, recipe, reader: Box::new(ShapeReader(stock.clone())), stock, no_split, recorded: None }
}

/// The four K4 cases, with the material side that each one cuts.
pub fn cases() -> Vec<Case> {
    vec![
        case("disc pocket", disc, MaterialSide::Inside, rings(RingRef::Boundary, STEP), true),
        case("disc, material round it", disc, MaterialSide::Outside, rings(RingRef::Part, STEP), false),
        case("dog bone, material round it", dog_bone, MaterialSide::Outside, rings(RingRef::Part, STEP), false),
        case("pocket with an island", island_region, MaterialSide::Inside, rings(RingRef::Boundary, 0.8), false),
    ]
}

const SIDES: [StartSide; 3] = [StartSide::Auto, StartSide::Inside, StartSide::Outside];

#[test]
fn k4_order_obeys_l3_with_the_ring_planner() {
    for c in cases() {
        eprintln!("{}:", c.name);
        assert_held(&check_l3(&kernel(), &c));
    }
}

/// The measured lifts of each case and start side (Auto, Inside,
/// Outside), each with its reason. The record's target is 0 on the disc.
/// The lifts of the island pocket are named in
/// `docs/decisions/4177-kernel-order-link.md`: Inside starts in the four
/// separate deepest corners and crosses the strips between them; Outside
/// must cross the rings still planned between the wall and the island.
const LIFTS: [(&str, [u32; 3]); 4] = [
    ("disc pocket", [0, 0, 0]),
    ("disc, material round it", [0, 0, 0]),
    ("dog bone, material round it", [0, 0, 0]),
    ("pocket with an island", [7, 7, 5]),
];

#[test]
fn k4_link_rule_holds_and_each_lift_is_named() {
    let mut read = Vec::new();
    for c in cases() {
        let mut row = [0u32; 3];
        for (j, side) in SIDES.iter().enumerate() {
            let r = run(&kernel(), &c, *side);
            let v = check_link(&r, &c);
            eprintln!("{} {side:?}: {v}", c.name);
            assert_held(&v);
            let r = r.unwrap();
            eprintln!("  lifts {:?}", r.tally.by_reason());
            row[j] = r.tally.lifts();
        }
        read.push((c.name, row));
    }
    assert_eq!(read, LIFTS.to_vec(), "the lift counts (Auto, Inside, Outside)");
}

fn cut(xy: &[(f64, f64)]) -> Cut {
    let pts = xy.iter().map(|&(x, y)| PlanePoint::flat(Point2::new(x, y)).unwrap()).collect();
    Cut::new(Curve::new(pts, None, false, 0, camcraft_pathkernel::guide::Family::Ring).unwrap(), None).unwrap()
}

#[test]
fn a_link_across_the_island_is_refused_with_the_part_reason() {
    // The island pocket's centre: the square 2..28 less the square 8..22.
    let r = island_region(MaterialSide::Inside).unwrap();
    let from = cut(&[(4.0, 4.0), (4.0, 15.0)]);
    let to = cut(&[(26.0, 15.0), (26.0, 26.0)]);
    let (made, planned) = ([from.clone()], [to.clone()]);
    let mut tally = LinkTally::default();
    let hop = LinkStage.link(&from, &to, &r, Level::new(&made, &planned), LinkPolicy::StayWhereProved, &mut tally);
    let Hop::Lift(why) = hop else { panic!("a link across the island must not stay: {hop:?}") };
    assert_eq!(why.get(), CUTS_PART);
    assert_eq!(tally.by_reason().get(CUTS_PART), Some(&1));
}

#[test]
fn a_link_round_the_island_stays_on_cut_ground_only() {
    // The outer wall ring is cut: the link walks along it, round the island.
    let r = island_region(MaterialSide::Inside).unwrap();
    let wall = Cut::new(Curve::new(rect(2.0, 2.0, 28.0, 28.0).into_iter().map(|q| PlanePoint::flat(q).unwrap()).collect(), None, true, 0, camcraft_pathkernel::guide::Family::Ring).unwrap(), None).unwrap();
    let from = cut(&[(4.0, 4.0), (4.0, 15.0)]);
    let to = cut(&[(26.5, 15.0), (26.5, 26.0)]);
    let (made, planned) = ([wall, from.clone()], [to.clone()]);
    let mut tally = LinkTally::default();
    let hop = LinkStage.link(&from, &to, &r, Level::new(&made, &planned), LinkPolicy::StayWhereProved, &mut tally);
    let Hop::Stay(path) = hop else { panic!("a proved walk round the island stays: {hop:?}") };
    let center = r.planar().unwrap().center();
    let pts = path.points();
    assert!(pts.windows(2).all(|w| (0..=20).all(|i| {
        let t = f64::from(i) / 20.0;
        let q = Point2::new(w[0].x + t * (w[1].x - w[0].x), w[0].y + t * (w[1].y - w[0].y));
        center.contains(q) || super::kit::outside(center, q) < 1e-6
    })), "the walk stays in the centre area: {pts:?}");
    assert_eq!(tally.lifts(), 0);
}

#[test]
fn a_lift_with_no_reason_is_refused_and_a_retract_is_named() {
    assert_eq!(Hop::lift("").unwrap_err().reason(), NO_REASON);
    assert_eq!(Hop::lift("   ").unwrap_err().reason(), NO_REASON);
    let r = island_region(MaterialSide::Inside).unwrap();
    let (from, to) = (cut(&[(4.0, 4.0), (4.0, 15.0)]), cut(&[(4.0, 16.0), (4.0, 20.0)]));
    let (made, planned) = ([from.clone()], [to.clone()]);
    let mut tally = LinkTally::default();
    let hop = LinkStage.link(&from, &to, &r, Level::new(&made, &planned), LinkPolicy::Retract, &mut tally);
    assert!(matches!(hop, Hop::Lift(why) if why.get() == USER_RETRACT), "{hop:?}");
    let hop = LinkStage.link(&from, &to, &r, Level::new(&made, &planned), LinkPolicy::StayWhereProved, &mut tally);
    assert!(matches!(hop, Hop::Stay(_)), "a short proved hop stays: {hop:?}");
    assert_eq!(tally.lifts(), 1);
}

fn ordered(r: &KernelRegion, g: &GuideChoice, start: StartSide, direction: Direction) -> (camcraft_pathkernel::guide::GuideSet, OrderedSet) {
    let set = RingPlanner.guide(r, g, 1e-3).unwrap().out().unwrap().clone();
    let o = OrderStage.order(set.clone(), r, &OrderChoice { start, direction }, None).unwrap().out().unwrap().clone();
    (set, o)
}

#[test]
fn an_order_that_moves_a_curve_is_refused() {
    let r = disc(MaterialSide::Inside).unwrap();
    let (set, o) = ordered(&r, &rings(RingRef::Boundary, STEP), StartSide::Inside, Direction::Climb);
    let mut curves = o.curves().to_vec();
    let moved = curves[1].pts().iter().map(|q| PlanePoint::new(Point2::new(q.at().x + 0.1, q.at().y), q.depth()).unwrap()).collect();
    curves[1] = Curve::new(moved, None, true, curves[1].k(), curves[1].family()).unwrap();
    assert_eq!(OrderedSet::new(&set, curves, o.start(), StartSide::Inside).unwrap_err().reason(), NOT_A_PERMUTATION);
}

/// The signed area of a closed curve: above 0 when it runs counter-clockwise.
fn turn(c: &Curve) -> f64 {
    let p: Vec<Point2> = c.pts().iter().map(|q| q.at()).collect();
    p.iter().zip(p.iter().cycle().skip(1)).map(|(a, b)| a.x * b.y - b.x * a.y).sum::<f64>() / 2.0
}

#[test]
fn the_side_orders_the_rings_and_climb_keeps_the_material_on_the_right() {
    let pocket_r = disc(MaterialSide::Inside).unwrap();
    let g = rings(RingRef::Boundary, STEP);
    let (_, inside) = ordered(&pocket_r, &g, StartSide::Inside, Direction::Climb);
    let ks: Vec<u32> = inside.curves().iter().map(Curve::k).collect();
    assert!(ks.windows(2).all(|w| w[0] >= w[1]), "Inside cuts a pocket from the middle out: {ks:?}");
    assert!(inside.curves().iter().all(|c| turn(c) > 0.0), "climb from the middle out runs counter-clockwise");
    let (_, outside) = ordered(&pocket_r, &g, StartSide::Outside, Direction::Climb);
    assert!(outside.curves().windows(2).all(|w| w[0].k() <= w[1].k()), "Outside cuts a pocket from the wall in");
    assert!(outside.curves().iter().all(|c| turn(c) < 0.0), "climb from the wall in runs clockwise");
    let (_, conv) = ordered(&pocket_r, &g, StartSide::Inside, Direction::Conventional);
    assert!(conv.curves().iter().all(|c| turn(c) < 0.0), "conventional reverses each ring");
    assert_eq!(inside.resolved(), StartSide::Inside);
    let band = disc(MaterialSide::Outside).unwrap();
    let g = rings(RingRef::Part, STEP);
    let (_, auto) = ordered(&band, &g, StartSide::Auto, Direction::Climb);
    assert_eq!(auto.resolved(), StartSide::Outside, "Auto starts outside when the stock edge has air beyond it");
    let top = auto.curves().iter().map(Curve::k).max().unwrap();
    assert_eq!(auto.curves()[0].k(), top, "Outside starts at the stock edge");
    assert!(auto.curves().iter().filter(|c| c.closed()).all(|c| turn(c) < 0.0), "climb round the part runs clockwise");
    let (_, inner) = ordered(&band, &g, StartSide::Inside, Direction::Climb);
    assert_eq!(inner.curves()[0].k(), 0, "Inside starts at the part wall");
    assert!(inner.curves().iter().filter(|c| c.closed()).all(|c| turn(c) > 0.0));
}

#[test]
fn the_marks_go_with_their_curves_and_a_full_width_stretch_is_cut_last() {
    // The waist of the K2 pins: ring 0 runs through a neck narrower than
    // two steps; each strand of the neck is a named full-width stretch.
    let shape = outline(&[circle(-25.0, 0.0, 15.0, 192), circle(25.0, 0.0, 15.0, 192), rect(-12.0, -4.0, 12.0, 4.0)]);
    let r = pocket(shape, vec![], 2.0);
    let g = rings(RingRef::Boundary, 3.0);
    for side in [StartSide::Inside, StartSide::Outside] {
        let (set, o) = ordered(&r, &g, side, Direction::Climb);
        assert_eq!(o.marks().len(), set.marks().len(), "{side:?}: no mark is lost");
        for m in o.marks() {
            let c = &o.curves()[m.curve()];
            let twin = set.marks().iter().filter(|n| n.kind() == m.kind() && set.curves()[n.curve()].same_cut(c)).count();
            assert!(twin > 0, "{side:?}: the mark stays on its curve");
        }
        // Each curve with full-width stretches starts where its longest
        // stretch ends: the heavy cut comes last on that curve.
        let d = |a: Point2, b: Point2| (a.x - b.x).hypot(a.y - b.y);
        for (i, c) in o.curves().iter().enumerate() {
            let ends: Vec<(f64, Point2)> = o
                .marks()
                .iter()
                .filter(|m| m.curve() == i && m.kind() == MarkKind::FullWidth)
                .filter_map(|m| match m.span() {
                    Span::Part(pts) => Some((pts.windows(2).map(|w| d(w[0].at(), w[1].at())).sum(), pts.last()?.at())),
                    Span::Whole => None,
                })
                .collect();
            let Some((_, end)) = ends.iter().copied().max_by(|a, b| a.0.total_cmp(&b.0)) else { continue };
            let seg = c.pts().windows(2).map(|w| d(w[0].at(), w[1].at())).fold(0.0, f64::max);
            assert!(c.closed() && d(c.pts()[0].at(), end) <= seg, "{side:?}: curve {i} starts where its full-width stretch ends");
        }
    }
}

