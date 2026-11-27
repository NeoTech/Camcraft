//! The trace guide planner (#4176, second part): the picked chains at the
//! choice's offset, open and closed, the keep-out area, each refusal named
//! through the public API, and L1 to L4 on the chain region (the kit's
//! stages plan an area, so the laws are read here on the guide and the
//! stand-in order stage).

use super::guide_kit::{p, pocket, rect, seg, xy_of};
use super::kit::{Law, MATCH_TOL, Measure, assert_held, judge, set_gap};
use super::standins::{SideOrder, order_curves, resolve};
use camcraft_pathkernel::guide::trace::{NOT_A_CHAIN, NOT_TRACE, TRACE_COLLAPSES, TRACE_KEPT_OUT};
use camcraft_pathkernel::guide::{Family, GuideChoice, GuidePlanner, GuideSet, Guides, TraceOffset, TracePlanner};
use camcraft_pathkernel::order::{OrderChoice, OrderPlanner, StartSide};
use camcraft_pathkernel::{
    Chain, KernelRegion, Loop, Point2, Refusal, Region, Selection, ToolShape, Verdict, WorkPlane, chain_region,
};

const TOL: f64 = 1e-3;

fn chains(chains: Vec<Chain>, keep_out: Region) -> KernelRegion {
    let sel = Selection::new(chains).unwrap();
    chain_region(&sel, WorkPlane::setup_xy(0.0).unwrap(), ToolShape::disc(1.0, 0.0, true, 10.0).unwrap(), keep_out).unwrap()
}

fn trace(d: f64) -> GuideChoice {
    GuideChoice::Trace(TraceOffset::new(d).unwrap())
}

fn plan(r: &KernelRegion, d: f64) -> Result<Verdict<GuideSet>, Refusal> {
    TracePlanner.guide(r, &trace(d), TOL)
}

fn done(v: &Verdict<GuideSet>) -> &GuideSet {
    assert!(v.postable(), "a done verdict: {v:?}");
    v.out().unwrap()
}

/// An open chain with one left turn: (0, 0) - (20, 0) - (20, 10).
fn ell() -> Chain {
    Chain::open(vec![p(0.0, 0.0), p(20.0, 0.0), p(20.0, 10.0)]).unwrap()
}

fn square(x0: f64, y0: f64, s: f64, ccw: bool) -> Chain {
    let mut v = rect(x0, y0, x0 + s, y0 + s);
    if !ccw {
        v.reverse();
    }
    Chain::closed(v).unwrap()
}

fn dist_to(c: &[Point2], q: Point2) -> f64 {
    c.windows(2).map(|w| seg(q, w[0], w[1])).fold(f64::INFINITY, f64::min)
}

#[test]
fn an_open_chain_is_traced_on_it_to_its_left_and_to_its_right() {
    let r = chains(vec![ell()], Region::empty());
    let path = [p(0.0, 0.0), p(20.0, 0.0), p(20.0, 10.0)];
    let on = plan(&r, 0.0).unwrap();
    let on = done(&on);
    assert_eq!(on.curves().len(), 1);
    assert_eq!((on.curves()[0].family(), on.curves()[0].closed()), (Family::Trace, false));
    assert_eq!(xy_of(&on.curves()[0]), path.to_vec());
    for d in [2.0, -2.0] {
        let v = plan(&r, d).unwrap();
        let set = done(&v);
        assert_eq!(set.curves().len(), 1, "{set:?}");
        let pts = xy_of(&set.curves()[0]);
        assert!(pts.iter().all(|q| (dist_to(&path, *q) - 2.0).abs() < 1e-2), "offset {d}: {pts:?}");
        // Left of the first leg is +y, right is -y.
        assert!(pts.iter().filter(|q| q.x < 15.0).all(|q| (q.y - d).abs() < 1e-6), "offset {d}: {pts:?}");
    }
}

#[test]
fn a_closed_chain_is_traced_inside_for_a_positive_offset() {
    // The same square in either direction: left of the counter-clockwise
    // square is its inside.
    for ccw in [true, false] {
        let r = chains(vec![square(0.0, 0.0, 10.0, ccw)], Region::empty());
        let v = plan(&r, 2.0).unwrap();
        let inner = done(&v);
        assert_eq!(inner.curves().len(), 1);
        assert!(inner.curves()[0].closed());
        let mut xs: Vec<f64> = xy_of(&inner.curves()[0]).iter().map(|q| q.x).collect();
        xs.sort_by(f64::total_cmp);
        assert!((xs[0] - 2.0).abs() < 1e-6 && (xs[xs.len() - 1] - 8.0).abs() < 1e-6, "{xs:?}");
        let v = plan(&r, -2.0).unwrap();
        let outer = done(&v);
        let pts = xy_of(&outer.curves()[0]);
        assert!(pts.iter().all(|q| q.x >= -2.0 - 1e-6 && q.x <= 12.0 + 1e-6), "the grown square with round corners");
        assert!(pts.len() > 8, "round corners");
    }
}

#[test]
fn the_keep_out_area_cuts_the_trace_and_the_note_says_so() {
    // A keep-out square over the middle of the first leg; the tool radius 1
    // keeps the trace one more millimetre away.
    let keep = Region::new(vec![Loop::new(rect(8.0, -1.0, 12.0, 1.0)).unwrap()]).unwrap();
    let r = chains(vec![ell()], keep);
    let v = plan(&r, 0.0).unwrap();
    let set = done(&v);
    assert_eq!(set.curves().len(), 2, "{set:?}");
    assert!(set.curves().iter().flat_map(xy_of).all(|q| q.x <= 7.0 + 1e-3 || q.x >= 13.0 - 1e-3));
    let Verdict::Done { notes, .. } = &v else { unreachable!() };
    assert!(notes.iter().any(|n| n.text().contains("keep-out")), "{notes:?}");
}

#[test]
fn each_refusal_of_the_trace_is_named() {
    let r = chains(vec![square(0.0, 0.0, 4.0, true)], Region::empty());
    assert_eq!(plan(&r, 3.0).unwrap_err().reason(), TRACE_COLLAPSES);
    assert_eq!(TracePlanner.guide(&r, &GuideChoice::Medial, TOL).unwrap_err().reason(), NOT_TRACE);
    let area = pocket(rect(0.0, 0.0, 10.0, 10.0), vec![], 1.0);
    assert_eq!(plan(&area, 0.0).unwrap_err().reason(), NOT_A_CHAIN);
    let all = Region::new(vec![Loop::new(rect(-50.0, -50.0, 50.0, 50.0)).unwrap()]).unwrap();
    assert_eq!(plan(&chains(vec![ell()], all), 0.0).unwrap_err().reason(), TRACE_KEPT_OUT);
    assert!(TraceOffset::new(f64::NAN).is_err());
    // An area planner refuses the chains by name.
    let row = GuideChoice::Rows(camcraft_pathkernel::guide::RowsChoice::new(0.0, 1.0, false).unwrap());
    assert_eq!(Guides.guide(&r, &row, TOL).unwrap_err().reason(), camcraft_pathkernel::guide::NOT_AN_AREA);
}

/// The chain region of the downstream pin: one open chain, no keep-out.
fn ell_region(_m: camcraft_pathkernel::MaterialSide) -> Result<KernelRegion, Refusal> {
    Ok(chains(vec![ell()], Region::empty()))
}

#[test]
fn each_later_stage_refuses_the_chain_region_of_a_trace_by_name() {
    // The trace guide plans on a chain region, but the later stages on this
    // branch plan an area only. Each stage that reads the region must refuse
    // the chain region by a named reason: never a panic, never a plan. The
    // follow-up #4236 makes these stages accept it; then this table changes
    // with it. The order stage refuses it; the link stage lifts for a named
    // reason (it proves no in-plane link on it). The load stage has no
    // planner on this branch yet (a trait only, K3).
    use camcraft_pathkernel::entry::{self, ArcChoice, EntryChoice, EntryKind, EntryPlanner, EntryStage, Heights};
    use camcraft_pathkernel::link::{Hop, Level, LinkPolicy, LinkProof, LinkStage, LinkTally, NOT_PLANAR_LINK};
    use camcraft_pathkernel::order::{ORDER_NOT_PLANAR, OrderStage};
    use camcraft_pathkernel::plan::Cut;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use super::kit::{self, Case, Planners};
    use super::standins::{ProvedEntry, ReadLoad, ShapeReader, StraightLink, recipe};

    let r = ell_region(camcraft_pathkernel::MaterialSide::Inside).unwrap();
    let set = done(&plan(&r, 0.0).unwrap()).clone();
    let cut = Cut::new(set.curves()[0].clone(), None).unwrap();
    let stage = EntryStage::new(Heights::new(5.0, 2.0, 10.0).unwrap(), ArcChoice::new(5.0, 90.0).unwrap(), 0.01).unwrap();
    let choice = EntryChoice::new(EntryKind::Plunge, 1.0, vec![]).unwrap();
    let case = Case {
        name: "trace of an open chain",
        region: ell_region,
        material: camcraft_pathkernel::MaterialSide::Inside,
        recipe: recipe(trace(0.0), EntryKind::Plunge, LinkPolicy::StayWhereProved).unwrap(),
        stock: Region::empty(),
        reader: Box::new(ShapeReader(Region::empty())),
        no_split: false,
        recorded: None,
    };
    let planners = Planners {
        guide: Box::new(TracePlanner),
        order: Box::new(SideOrder),
        load: Box::new(ReadLoad { blind: false }),
        link: Box::new(StraightLink),
        entry: Box::new(ProvedEntry),
    };
    // Each call gives the named reason of its refusal (or lift), or none
    // when the stage accepts the chain region.
    type Call<'a> = Box<dyn Fn() -> Option<&'static str> + 'a>;
    let named = |r: Result<(), Refusal>| r.err().map(|e| e.reason());
    let order = OrderChoice { start: StartSide::Auto, direction: Default::default() };
    let cuts = [cut.clone()];
    let table: Vec<(&str, &str, Call<'_>)> = vec![
        ("order stage", ORDER_NOT_PLANAR, Box::new(|| named(OrderStage.order(set.clone(), &r, &order, None).map(|_| ())))),
        (
            "link stage",
            NOT_PLANAR_LINK,
            Box::new(|| match LinkStage.link(&cut, &cut, &r, Level::new(&cuts, &cuts), LinkPolicy::StayWhereProved, &mut LinkTally::default()) {
                Hop::Lift(why) => Some(why.get()),
                Hop::Stay(_) => None,
            }),
        ),
        ("entry stage: entry", entry::NOT_PLANAR, Box::new(|| named(stage.entry(&cut, &r, &Region::empty(), &choice).map(|_| ())))),
        ("entry stage: exit", entry::NOT_PLANAR, Box::new(|| named(stage.exit(&cut, &r, &Region::empty()).map(|_| ())))),
        ("law kit: planar", kit::NOT_PLANAR, Box::new(|| named(kit::planar(&r).map(|_| ())))),
        ("law kit: run", kit::NOT_PLANAR, Box::new(|| named(kit::run(&planners, &case, StartSide::Auto).map(|_| ())))),
    ];
    for (what, reason, call) in table {
        match catch_unwind(AssertUnwindSafe(&call)) {
            Err(_) => panic!("{what}: panics on the chain region of a trace"),
            Ok(None) => panic!("{what}: accepts the chain region of a trace (a finding of #4236)"),
            Ok(Some(got)) => assert_eq!(got, reason, "{what}"),
        }
    }
}

fn curves(v: &Verdict<GuideSet>) -> Vec<&camcraft_pathkernel::guide::Curve> {
    v.out().map(|g| g.curves().iter().collect()).unwrap_or_default()
}

#[test]
fn the_trace_holds_l1_to_l4() {
    // A square and an open chain, traced to their left.
    let base = vec![square(30.0, 0.0, 10.0, true), ell()];
    let r = chains(base.clone(), Region::empty());
    let g = trace(1.5);
    let one = TracePlanner.guide(&r, &g, TOL).unwrap();
    // L1: a second call, the chains in the other order with the closed one
    // started at its next vertex, and the guide stage as one planner.
    let mut turned = rect(30.0, 0.0, 40.0, 10.0);
    turned.rotate_left(1);
    let other = chains(vec![ell(), Chain::closed(turned).unwrap()], Region::empty());
    let mut m = Vec::new();
    for (what, v) in [
        ("a second call", TracePlanner.guide(&r, &g, TOL).unwrap()),
        ("the chains reordered and restarted", TracePlanner.guide(&other, &g, TOL).unwrap()),
        ("the guide stage", Guides.guide(&r, &g, TOL).unwrap()),
    ] {
        let (gap, at) = set_gap(&curves(&one), &curves(&v));
        m.push(Measure::new(gap, MATCH_TOL, format!("{what}: {at}"), None));
    }
    assert_held(&judge(Law::L1, m));
    // L2: the chain region takes no material side and no start side, and
    // the guide takes no start side (compile level).
    const _REGION: fn(&Selection, WorkPlane, ToolShape, Region) -> Result<KernelRegion, Refusal> = chain_region;
    type Stage = fn(&TracePlanner, &KernelRegion, &GuideChoice, f64) -> Result<Verdict<GuideSet>, Refusal>;
    let _: Stage = <TracePlanner as GuidePlanner>::guide;
    // L3: each start side orders the same curves, and nothing else.
    let set = one.out().unwrap();
    let mut m = Vec::new();
    for side in [StartSide::Auto, StartSide::Inside, StartSide::Outside] {
        let oc = OrderChoice { start: side, direction: Default::default() };
        let o = SideOrder.order(set.clone(), &r, &oc, None).unwrap();
        let o = o.out().unwrap();
        let (gap, at) = set_gap(&o.curves().iter().collect::<Vec<_>>(), &curves(&one));
        m.push(Measure::new(gap, MATCH_TOL, format!("start side {side:?}: {at}"), None));
        let again = order_curves(set, set.curves().to_vec(), resolve(side), oc.direction).unwrap();
        let (gap, at) = set_gap(&again.curves().iter().collect::<Vec<_>>(), &curves(&one));
        m.push(Measure::new(gap, MATCH_TOL, format!("start side {side:?} again: {at}"), None));
    }
    assert_held(&judge(Law::L3, m));
    // L4: every curve is of the family asked, so no fallback is owed.
    let other = set.curves().iter().filter(|c| c.family() != Family::Trace).count();
    assert_held(&judge(Law::L4, vec![Measure::count(other, "guide: curves of another family than the trace", None)]));
}
