//! The K3 pins (#4026): the kernel's load stage, registered with the law
//! kit and run with the ring planner of K2 and the order and link stages
//! of K4, on the disc pocket, the pocket with an island and a pocket of
//! two lobes joined by a neck (a full-width stretch). For each mode: None
//! names every cut above the limit and keeps the curves; constant
//! engagement leaves no unnamed cut above the limit and states the worst
//! one; trochoidal leaves no full-width feed in the marked stretch. The
//! negative pins: an unnamed cut above the limit, a limit that is not a
//! number or not above 0, the stock reach, and a stretch with no room for
//! a loop (named, never out of the area).

use camcraft_pathkernel::entry::{EntryChoice, EntryKind};
use camcraft_pathkernel::guide::{GuideChoice, GuidePlanner, MarkKind, RingPlanner, RingRef, RingsChoice, Span};
use camcraft_pathkernel::link::{LinkPolicy, LinkStage};
use camcraft_pathkernel::load::{
    BAD_LOAD, Controlled, LoadControl, LoadPlanner, LoadStage, NO_CONTROL, NO_ROOM, PLAN_TOL, Reach, Reading, STOCK_REACH_ELSEWHERE, Trochoid,
    UNNAMED_OVERLOAD,
};
use camcraft_pathkernel::order::{OrderChoice, OrderPlanner, OrderStage};
use camcraft_pathkernel::{AxisControl, KernelRegion, Loop, MaterialSide, Point2, Recipe, Refusal, Region, WorkPlane};

use super::grid::walk;
use super::guide_kit::{outline, pocket, rect};
use super::kit::{Case, Planners, Run, assert_held, dist, outside, planar, poly, poly_dist, run};
use super::l6_load::check_l6;
use super::order_link::{disc, stock_box};
use super::standins::{ProvedEntry, ShapeReader, island_region, sq};

fn area(pts: Vec<Point2>) -> Region {
    Region::new(vec![Loop::new(pts).unwrap()]).unwrap()
}

/// Two 20 x 20 lobes joined by a neck 6 mm wide and 12 mm long.
fn neck(_m: MaterialSide) -> Result<KernelRegion, Refusal> {
    Ok(pocket(outline(&[rect(-26.0, -10.0, -6.0, 10.0), rect(6.0, -10.0, 26.0, 10.0), rect(-7.0, -3.0, 7.0, 3.0)]), vec![], 2.0))
}

/// A slot 4.1 mm wide for a tool of radius 2: no room for a loop.
fn slot(_m: MaterialSide) -> Result<KernelRegion, Refusal> {
    Ok(pocket(rect(-15.0, -2.05, 15.0, 2.05), vec![], 2.0))
}

fn recipe(step: f64, load: LoadControl) -> Recipe {
    let guide = GuideChoice::Rings(RingsChoice::new(RingRef::Boundary, 0.0, step, None, None).unwrap());
    let entry = EntryChoice::new(EntryKind::Helix, 2.0, vec![]).unwrap();
    Recipe::new(WorkPlane::setup_xy(0.0).unwrap(), AxisControl::Normal, guide, load, OrderChoice::default(), LinkPolicy::StayWhereProved, entry).unwrap()
}

/// One fixture: its name, region, ring step and nominal load (step / D), and its stock.
struct Fixture {
    name: &'static str,
    region: fn(MaterialSide) -> Result<KernelRegion, Refusal>,
    step: f64,
    load: f64,
    stock: Region,
}

fn fixtures() -> Vec<Fixture> {
    vec![
        Fixture { name: "disc pocket", region: disc, step: 2.0, load: 0.25, stock: stock_box() },
        Fixture { name: "pocket with an island", region: island_region, step: 0.8, load: 0.2, stock: Region::new(vec![sq(0.0, 0.0, 30.0).unwrap()]).unwrap() },
        Fixture { name: "two lobes and a neck", region: neck, step: 1.6, load: 0.4, stock: area(rect(-30.0, -15.0, 30.0, 15.0)) },
    ]
}

fn case(f: &Fixture, load: LoadControl) -> Case {
    let reader = Box::new(ShapeReader(f.stock.clone()));
    Case { name: f.name, region: f.region, material: MaterialSide::Inside, recipe: recipe(f.step, load), stock: f.stock.clone(), reader, no_split: false, recorded: None }
}

/// The kernel stages with the load stage that states `limit`.
fn kernel(limit: f64) -> Planners {
    Planners {
        guide: Box::new(RingPlanner),
        order: Box::new(OrderStage),
        load: Box::new(LoadStage::new(limit).unwrap()),
        link: Box::new(LinkStage),
        entry: Box::new(ProvedEntry),
    }
}

/// The reads of a run on the check grid above `cap`: (all, unnamed), and the worst.
fn over(r: &Run, c: &Case, cap: f64) -> (usize, usize, f64) {
    let ctl = r.controlled.out().unwrap();
    let tool_r = planar(&r.region).unwrap().tool().radius();
    let w = walk(&r.plan, &c.stock, tool_r).unwrap();
    let hot: Vec<_> = w.cuts.iter().filter(|k| k.load > cap).collect();
    let unnamed = hot.iter().filter(|k| !ctl.over().iter().any(|o| dist(o.at(), k.at) <= tool_r)).count();
    (hot.len(), unnamed, w.cuts.iter().map(|k| k.load).fold(0.0, f64::max))
}

fn notes(r: &Run) -> Vec<String> {
    match &r.controlled {
        camcraft_pathkernel::Verdict::Done { notes, .. } => notes.iter().map(|n| n.text().to_string()).collect(),
        _ => vec![],
    }
}

/// Each curve of the controlled set is read: the clean-up rings too (#3849).
fn each_curve_read(ctl: &Controlled) -> bool {
    ctl.set().curves().iter().all(|c| ctl.readings().iter().any(|rd| poly_dist(rd.at(), &poly(c)) <= 1e-6))
}

#[test]
fn k3_none_keeps_the_curves_and_names_every_cut_above_the_limit() {
    for f in fixtures() {
        let c = case(&f, LoadControl::None);
        let r = run(&kernel(f.load), &c, Default::default()).unwrap();
        let ctl = r.controlled.out().unwrap();
        assert_eq!(ctl.set(), r.ordered.out().unwrap(), "{}: None is the identity", f.name);
        assert!(each_curve_read(ctl), "{}: each curve is read", f.name);
        let (hot, unnamed, worst) = over(&r, &c, f.load * (1.0 + PLAN_TOL));
        eprintln!("{} None: {} records, {hot} hot reads ({unnamed} unnamed), worst {worst:.3}; {:?}", f.name, ctl.over().len(), notes(&r));
        assert!(ctl.over().iter().all(|o| o.reason() == NO_CONTROL), "{}", f.name);
        assert!(!ctl.over().is_empty() && !notes(&r).is_empty(), "{}: the first ring is a full-width cut, named on the status line", f.name);
        assert_held(&check_l6(&Ok(r), &c));
    }
}

#[test]
fn k3_constant_engagement_leaves_no_unnamed_cut_above_the_limit() {
    for f in fixtures() {
        let ce = case(&f, LoadControl::constant(f.load, Reach::Rings).unwrap());
        let none = case(&f, LoadControl::None);
        let (a, b) = (run(&kernel(f.load), &none, Default::default()).unwrap(), run(&kernel(f.load), &ce, Default::default()).unwrap());
        let cap = f.load * (1.0 + PLAN_TOL);
        let (hot0, _, worst0) = over(&a, &none, cap);
        let (hot1, unnamed, worst1) = over(&b, &ce, cap);
        let ctl = b.controlled.out().unwrap();
        let (n0, n1) = (a.controlled.out().unwrap().set().curves().len(), ctl.set().curves().len());
        eprintln!("{}: None {n0} curves, {hot0} hot reads, worst {worst0:.3}; constant engagement {n1} curves, {hot1} hot reads ({unnamed} unnamed), worst {worst1:.3}", f.name);
        for n in notes(&b) {
            eprintln!("  {n}");
        }
        assert!(each_curve_read(ctl), "{}: each curve is read", f.name);
        assert!(hot1 <= hot0, "{}: constant engagement does not add load", f.name);
        assert!(ctl.over().is_empty() || notes(&b).iter().any(|n| n.contains("the worst")), "{}: the worst cut is stated", f.name);
        assert_held(&check_l6(&Ok(b), &ce));
    }
}

#[test]
fn k3_trochoidal_leaves_no_full_width_feed_in_the_marked_stretch() {
    let f = &fixtures()[2];
    let region = neck(MaterialSide::Inside).unwrap();
    let gs = RingPlanner.guide(&region, &recipe(f.step, LoadControl::None).guide(), 1e-3).unwrap().out().unwrap().clone();
    let stretch: Vec<Point2> = gs
        .marks()
        .iter()
        .filter(|m| m.kind() == MarkKind::FullWidth)
        .flat_map(|m| match m.span() {
            Span::Part(pts) => pts.iter().map(|q| q.at()).collect::<Vec<_>>(),
            Span::Whole => poly(&gs.curves()[m.curve()]),
        })
        .collect();
    assert!(!stretch.is_empty(), "the neck has a full-width stretch");
    let c = case(f, LoadControl::Trochoidal(Trochoid::new(2.0, 0.8).unwrap()));
    let r = run(&kernel(f.load), &c, Default::default()).unwrap();
    let tool_r = planar(&r.region).unwrap().tool().radius();
    let w = walk(&r.plan, &c.stock, tool_r).unwrap();
    let in_stretch: Vec<_> = w.cuts.iter().filter(|k| poly_dist(k.at, &stretch) <= 0.5).collect();
    let worst = in_stretch.iter().map(|k| k.load).fold(0.0, f64::max);
    eprintln!("trochoidal: {} reads in the stretch, worst {worst:.3}; {:?}", in_stretch.len(), notes(&r));
    assert!(!in_stretch.is_empty());
    assert!(worst < 0.99, "no full-width feed in the marked stretch: {worst}");
    let center = planar(&r.region).unwrap().center().clone();
    for cv in r.controlled.out().unwrap().set().curves() {
        assert!(poly(cv).iter().all(|q| outside(&center, *q) <= 1e-6), "each loop stays in the cutter-centre area");
    }
    assert_held(&check_l6(&Ok(r), &c));
}

#[test]
fn k3_a_stretch_with_no_room_for_a_loop_stays_and_is_named() {
    let f = Fixture { name: "slot", region: slot, step: 0.1, load: 0.4, stock: area(rect(-20.0, -5.0, 20.0, 5.0)) };
    let c = case(&f, LoadControl::Trochoidal(Trochoid::new(2.0, 0.8).unwrap()));
    let r = run(&kernel(f.load), &c, Default::default()).unwrap();
    let ctl = r.controlled.out().unwrap();
    assert_eq!(ctl.set(), r.ordered.out().unwrap(), "no loop fits: the curves stay");
    assert!(ctl.over().iter().any(|o| o.reason() == NO_ROOM), "the full-width slot is named: {:?}", ctl.over().first());
    assert_held(&check_l6(&Ok(r), &c));
}

#[test]
fn k3_an_unnamed_cut_above_the_limit_and_a_bad_limit_are_refused() {
    let f = &fixtures()[0];
    let region = disc(MaterialSide::Inside).unwrap();
    let gs = RingPlanner.guide(&region, &recipe(f.step, LoadControl::None).guide(), 1e-3).unwrap().out().unwrap().clone();
    let os = OrderStage.order(gs, &region, &OrderChoice::default(), None).unwrap().out().unwrap().clone();
    let hot = vec![Reading::new(Point2::new(0.0, 0.0), 1.0).unwrap()];
    assert_eq!(Controlled::new(os.clone(), hot, vec![], 0.25).unwrap_err().reason(), UNNAMED_OVERLOAD);
    for bad in [f64::NAN, -0.1, 0.0, 1.5] {
        assert_eq!(LoadStage::new(bad).unwrap_err().reason(), BAD_LOAD, "{bad}");
        assert_eq!(LoadControl::constant(bad, Reach::Rings).unwrap_err().reason(), BAD_LOAD, "{bad}");
    }
    let reader = ShapeReader(stock_box());
    let stage = LoadStage::default();
    let stock = LoadControl::constant(0.25, Reach::Stock).unwrap();
    assert_eq!(stage.control(os.clone(), &region, &reader, &stock, 4.0).unwrap_err().reason(), STOCK_REACH_ELSEWHERE);
    assert!(stage.control(os, &region, &reader, &LoadControl::None, f64::NAN).is_err(), "a tool radius that is not a number");
}
