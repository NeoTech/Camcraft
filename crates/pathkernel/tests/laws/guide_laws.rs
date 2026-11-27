//! The guide planners of #4176 (second part) registered with the law kit:
//! each planner of an area (medial line, exact circle rings, helix) runs
//! through `kit::run` with the stand-in stages of the other stages and
//! must hold L1 to L4. The trace has no area, so the kit's stages cannot
//! run it (the entry stage plans an area); its L1 to L4 are in
//! `guide_trace.rs`.

use camcraft_pathkernel::entry::EntryKind;
use camcraft_pathkernel::guide::{
    Circle, GuideChoice, GuidePlanner, Guides, Hand, HelixChoice, HelixPlanner, MedialPlanner, RingPlanner, RingRef, RingsChoice,
};
use camcraft_pathkernel::link::LinkPolicy;
use camcraft_pathkernel::{Bounds, KernelRegion, Loop, MaterialSide, Point2, Refusal, Region, Selection, region};

use super::guide_kit::{circle, tool, xy};
use super::kit::{Case, Law, LawVerdict, Planners, assert_held, run};
use super::l1_guide::check_l1;
use super::l2_sides::check_l2;
use super::l3_order::check_l3;
use super::l4_fallback::check_l4;
use super::standins::{ProvedEntry, ReadLoad, ShapeReader, SideOrder, StraightLink, recipe};

fn inside(outer: Vec<Point2>, m: MaterialSide, r: f64) -> Result<KernelRegion, Refusal> {
    let sel = Selection::face(outer, vec![])?;
    let b = Bounds::new(None, Region::empty(), 0.0, -5.0)?;
    region(&sel, m, &b, xy(), tool(r), 0.0)
}

/// A slot of half-width 5 round (0, 0)-(40, 0); tool radius 2.
pub fn slot_region(m: MaterialSide) -> Result<KernelRegion, Refusal> {
    use std::f64::consts::{FRAC_PI_2, PI};
    let half = |cx: f64, a0: f64| (0..=64).map(move |i| a0 + PI * f64::from(i) / 64.0).map(move |a| Point2::new(cx + 5.0 * a.cos(), 5.0 * a.sin()));
    inside(half(40.0, -FRAC_PI_2).chain(half(0.0, FRAC_PI_2)).collect(), m, 2.0)
}

/// A hole of radius 15 at (5, 5); tool radius 3: the centre is a disc of
/// radius 12.
pub fn hole_region(m: MaterialSide) -> Result<KernelRegion, Refusal> {
    inside(circle(5.0, 5.0, 15.0, 512), m, 3.0)
}

fn shape(v: Vec<Point2>) -> Region {
    Region::new(vec![Loop::new(v).unwrap()]).unwrap()
}

/// L1 to L4 of `guide` on `region` with `choice`, through the kit.
pub fn laws(guide: Box<dyn GuidePlanner>, name: &'static str, region: fn(MaterialSide) -> Result<KernelRegion, Refusal>, stock: Region, choice: GuideChoice) -> Vec<LawVerdict> {
    let c = Case {
        name,
        region,
        material: MaterialSide::Inside,
        recipe: recipe(choice, EntryKind::Helix, LinkPolicy::StayWhereProved).unwrap(),
        stock: stock.clone(),
        reader: Box::new(ShapeReader(stock)),
        no_split: false,
        recorded: None,
    };
    let r = region(MaterialSide::Inside).unwrap();
    let mut v = vec![check_l1(guide.as_ref(), &r, &choice, &[&Guides])];
    let p = Planners { guide, order: Box::new(SideOrder), load: Box::new(ReadLoad { blind: false }), link: Box::new(StraightLink), entry: Box::new(ProvedEntry) };
    let auto = run(&p, &c, Default::default()).unwrap();
    v.push(check_l2(region, &SideOrder, auto.guide.out().unwrap(), c.recipe.order()));
    v.push(check_l3(&p, &c));
    v.push(check_l4(&auto, &c.recipe));
    v
}

fn all_held(v: &[LawVerdict]) {
    let laws: Vec<Law> = v.iter().map(|x| x.law).collect();
    assert_eq!(laws, vec![Law::L1, Law::L2, Law::L3, Law::L4]);
    v.iter().for_each(assert_held);
}

#[test]
fn the_medial_planner_holds_l1_to_l4() {
    let stock = shape(super::guide_kit::rect(-6.0, -6.0, 46.0, 6.0));
    all_held(&laws(Box::new(MedialPlanner), "slot", slot_region, stock, GuideChoice::Medial));
}

#[test]
fn the_circle_rings_hold_l1_to_l4() {
    let c = Circle::new(Point2::new(5.0, 5.0), 12.0).unwrap();
    let g = GuideChoice::Rings(RingsChoice::new(RingRef::Boundary, 0.0, 3.0, Some(c), None).unwrap());
    all_held(&laws(Box::new(RingPlanner), "hole", hole_region, shape(circle(5.0, 5.0, 15.0, 512)), g));
}

#[test]
fn the_helix_planner_holds_l1_to_l4() {
    // Less than one turn: the stand-in load stage of the kit names each
    // reading above its limit by its place, and a helix of several turns
    // passes each place once per turn (the multi-turn helix is pinned in
    // `guide_helix.rs`).
    let c = Circle::new(Point2::new(5.0, 5.0), 10.0).unwrap();
    let g = GuideChoice::Helix(HelixChoice::new(c, 1.5, 1.2, Hand::Right).unwrap());
    all_held(&laws(Box::new(HelixPlanner), "hole", hole_region, shape(circle(5.0, 5.0, 15.0, 512)), g));
}
