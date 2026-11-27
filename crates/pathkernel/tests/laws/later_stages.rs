//! Each guide family through each later stage (#4176, the review of PR
//! #4244): a stage that restarts, reverses, splits or replaces a curve goes
//! through the constructor of that curve's family and keeps its data (the
//! circle, the lead, the marks), or it refuses by a reason of its own
//! stage. Never a panic, never a refusal that carries the reason of a
//! constructor or of another stage. The chain region of a trace is refused
//! by name in `guide_trace.rs`; here it is one row of the table too.

use std::panic::{AssertUnwindSafe, catch_unwind};

use super::guide_kit::{p, pocket, rect};
use super::guide_laws::{hole_region, slot_region};
use super::standins::ShapeReader;
use camcraft_pathkernel::entry::{ArcChoice, EntryChoice, EntryKind, EntryPlanner, EntryStage, Heights};
use camcraft_pathkernel::guide::mark::{BAD_STRETCH, MARK_NO_CURVE, MARK_OFF_CURVE};
use camcraft_pathkernel::guide::{
    BAD_CURVE, Circle, Curve, Family, GuideChoice, GuidePlanner, GuideSet, HELIX_NEEDS_LEAD, Hand, HelixChoice, HelixPlanner, MedialPlanner,
    NOT_ON_HELIX, REPEATED_START, RingPlanner, RingRef, RingsChoice, RowPlanner, RowsChoice, TraceOffset, TracePlanner, ZERO_LENGTH,
};
use camcraft_pathkernel::link::{Hop, Level, LinkPolicy, LinkProof, LinkStage, LinkTally};
use camcraft_pathkernel::load::{LoadControl, LoadPlanner, LoadStage, Reach, Trochoid};
use camcraft_pathkernel::order::{
    Direction, HELIX_HAND_SETS_DIRECTION, NOT_A_PERMUTATION, OrderChoice, OrderPlanner, OrderStage, OrderedSet, StartSide,
};
use camcraft_pathkernel::plan::Cut;
use camcraft_pathkernel::{
    Chain, KernelRegion, Loop, MaterialSide, Region, Selection, Stage, ToolShape, Verdict, WorkPlane, chain_region,
};

const TOL: f64 = 1e-3;

/// The reasons of the curve, set and mark constructors: never the reason
/// of a later stage.
const CONSTRUCTOR: [&str; 8] = [HELIX_NEEDS_LEAD, NOT_ON_HELIX, BAD_CURVE, ZERO_LENGTH, REPEATED_START, MARK_NO_CURVE, MARK_OFF_CURVE, BAD_STRETCH];

/// One row: a guide family on a region where it is valid.
struct Row {
    name: &'static str,
    region: KernelRegion,
    set: GuideSet,
    tool_r: f64,
}

fn row(name: &'static str, r: KernelRegion, g: &dyn GuidePlanner, c: GuideChoice, tool_r: f64) -> Row {
    let v = g.guide(&r, &c, TOL).unwrap_or_else(|e| panic!("{name}: the guide plans: {e:?}"));
    let set = v.out().unwrap_or_else(|| panic!("{name}: a guide set")).clone();
    assert!(!set.curves().is_empty(), "{name}: curves");
    Row { name, region: r, set, tool_r }
}

fn helix(hand: Hand, drop: f64) -> GuideChoice {
    GuideChoice::Helix(HelixChoice::new(Circle::new(p(5.0, 5.0), 10.0).unwrap(), 1.5, drop, hand).unwrap())
}

fn rows() -> Vec<Row> {
    let square = || pocket(rect(-10.0, -10.0, 10.0, 10.0), vec![], 2.0);
    let hole = || hole_region(MaterialSide::Inside).unwrap();
    let ell = Chain::open(vec![p(0.0, 0.0), p(20.0, 0.0), p(20.0, 10.0)]).unwrap();
    let chains = chain_region(&Selection::new(vec![ell]).unwrap(), WorkPlane::setup_xy(0.0).unwrap(), ToolShape::disc(1.0, 0.0, true, 10.0).unwrap(), Region::empty()).unwrap();
    let boundary = GuideChoice::Rings(RingsChoice::new(RingRef::Boundary, 0.0, 2.0, None, None).unwrap());
    let circles = GuideChoice::Rings(RingsChoice::new(RingRef::Boundary, 0.0, 3.0, Some(Circle::new(p(5.0, 5.0), 12.0).unwrap()), None).unwrap());
    vec![
        row("rings of a pocket", square(), &RingPlanner, boundary, 2.0),
        row("exact circle rings of a hole", hole(), &RingPlanner, circles, 3.0),
        row("rows of a pocket", square(), &RowPlanner, GuideChoice::Rows(RowsChoice::new(0.0, 2.0, false).unwrap()), 2.0),
        row("medial line of a slot", slot_region(MaterialSide::Inside).unwrap(), &MedialPlanner, GuideChoice::Medial, 2.0),
        row("trace of an open chain", chains, &TracePlanner, GuideChoice::Trace(TraceOffset::new(0.0).unwrap()), 1.0),
        // The review's case (red before the fix: HELIX_NEEDS_LEAD).
        row("right-hand helix of a hole", hole(), &HelixPlanner, helix(Hand::Right, 1.2), 3.0),
        row("left-hand helix of a hole, four turns", hole(), &HelixPlanner, helix(Hand::Left, 6.0), 3.0),
    ]
}

/// What a cell gives: a result whose data the check read, or the refusal.
enum Got {
    Kept,
    Refused(Stage, &'static str),
}

/// True when each curve of `out` is one of `input` (a permutation, law 3:
/// the same index, family, closure, circle and lead; restarted or
/// reversed only).
fn permutation(input: &[Curve], out: &[Curve]) -> bool {
    let mut used = vec![false; input.len()];
    input.len() == out.len()
        && out.iter().all(|c| match (0..input.len()).find(|&i| !used[i] && input[i].same_cut(c)) {
            Some(i) => {
                used[i] = true;
                true
            }
            None => false,
        })
}

/// The data of a family that a later stage must keep: each curve of
/// `input` with a circle or a lead is in `out` unchanged (restarted or
/// reversed only), and each helix of `out` has its circle and its lead
/// and runs down its lead.
fn family_data_kept(input: &[Curve], out: &[Curve]) -> Result<(), String> {
    for c in input.iter().filter(|c| c.arc().is_some() || c.lead().is_some()) {
        if !out.iter().any(|o| c.same_cut(o)) {
            return Err(format!("a curve of family {:?} lost its circle or its lead", c.family()));
        }
    }
    for h in out.iter().filter(|c| c.family() == Family::Helix) {
        let (a, b) = (h.pts().first().unwrap().depth(), h.pts().last().unwrap().depth());
        if h.lead().is_none() || h.arc().is_none() || b >= a {
            return Err(format!("a helix without its data or not down its lead: {a} to {b}"));
        }
    }
    Ok(())
}

/// The ordered set of a row for the stages after the order: the order
/// stage's own when it plans the row, else the guide set as it is.
fn ordered(r: &Row) -> OrderedSet {
    match OrderStage.order(r.set.clone(), &r.region, &OrderChoice::default(), None) {
        Ok(v) => v.out().unwrap().clone(),
        Err(_) => {
            let start = r.set.curves()[0].pts()[0].at();
            OrderedSet::new(&r.set, r.set.curves().to_vec(), start, StartSide::Inside).unwrap()
        }
    }
}

#[test]
fn each_guide_family_keeps_its_data_or_is_refused_by_the_stage_in_each_later_stage() {
    let stock = ShapeReader(Region::new(vec![Loop::new(rect(-60.0, -60.0, 60.0, 60.0)).unwrap()]).unwrap());
    let entry = EntryStage::new(Heights::new(5.0, 2.0, 10.0).unwrap(), ArcChoice::new(5.0, 90.0).unwrap(), 0.01).unwrap();
    let plunge = EntryChoice::new(EntryKind::Plunge, 1.0, vec![]).unwrap();
    let mut checked = 0;
    for row in rows() {
        let r = &row;
        type Cell<'a> = (String, Stage, Box<dyn Fn() -> Result<Got, String> + 'a>);
        let mut cells: Vec<Cell<'_>> = Vec::new();
        for start in [StartSide::Auto, StartSide::Inside, StartSide::Outside] {
            for direction in [Direction::Climb, Direction::Conventional] {
                let o = OrderChoice { start, direction };
                cells.push((
                    format!("order {start:?} {direction:?}"),
                    Stage::Order,
                    Box::new(move || match OrderStage.order(r.set.clone(), &r.region, &o, None) {
                        Err(e) => Ok(Got::Refused(e.stage(), e.reason())),
                        Ok(v) => {
                            let out = v.out().ok_or("no ordered set")?;
                            if !permutation(r.set.curves(), out.curves()) || out.marks().len() != r.set.marks().len() {
                                return Err("not a permutation of the curves and their marks".into());
                            }
                            family_data_kept(r.set.curves(), out.curves()).map(|()| Got::Kept)
                        }
                    }),
                ));
            }
        }
        let set = ordered(r);
        let modes = [
            ("none", LoadControl::None),
            ("constant engagement on the rings", LoadControl::constant(0.25, Reach::Rings).unwrap()),
            ("constant engagement on the stock", LoadControl::constant(0.25, Reach::Stock).unwrap()),
            ("trochoidal", LoadControl::Trochoidal(Trochoid::new(2.0, 0.8).unwrap())),
        ];
        for (mode, lc) in modes {
            let (set, stock) = (set.clone(), &stock);
            let tool_r = r.tool_r;
            cells.push((
                format!("load {mode}"),
                Stage::Load,
                Box::new(move || match LoadStage::default().control(set.clone(), &r.region, stock, &lc, tool_r) {
                    Err(e) => Ok(Got::Refused(e.stage(), e.reason())),
                    Ok(v) => family_data_kept(set.curves(), v.out().ok_or("no controlled set")?.set().curves()).map(|()| Got::Kept),
                }),
            ));
        }
        let cuts: Vec<Cut> = set.curves().iter().map(|c| Cut::new(c.clone(), None).unwrap()).collect();
        let (first, last) = (cuts[0].clone(), cuts[cuts.len() - 1].clone());
        let next = cuts.get(1).cloned().unwrap_or_else(|| first.clone());
        {
            let (first, next, cuts) = (first.clone(), next.clone(), cuts.clone());
            cells.push((
                "link".into(),
                Stage::Link,
                Box::new(move || match LinkStage.link(&first, &next, &r.region, Level::new(&cuts[..1], &cuts[1..]), LinkPolicy::StayWhereProved, &mut LinkTally::default()) {
                    Hop::Stay(_) => Ok(Got::Kept),
                    Hop::Lift(why) => Ok(Got::Refused(Stage::Link, why.get())),
                }),
            ));
        }
        {
            let (entry, first) = (&entry, first.clone());
            let plunge = &plunge;
            cells.push((
                "entry".into(),
                Stage::Entry,
                Box::new(move || match entry.entry(&first, &r.region, &Region::empty(), plunge) {
                    Err(e) => Ok(Got::Refused(e.stage(), e.reason())),
                    Ok(_) => Ok(Got::Kept),
                }),
            ));
        }
        {
            let entry = &entry;
            cells.push((
                "exit".into(),
                Stage::Exit,
                Box::new(move || match entry.exit(&last, &r.region, &Region::empty()) {
                    Err(e) => Ok(Got::Refused(e.stage(), e.reason())),
                    Ok(_) => Ok(Got::Kept),
                }),
            ));
        }
        for (what, stage, cell) in &cells {
            let at = format!("{} / {what}", r.name);
            match catch_unwind(AssertUnwindSafe(cell)) {
                Err(_) => panic!("{at}: panics"),
                Ok(Err(lost)) => panic!("{at}: {lost}"),
                Ok(Ok(Got::Kept)) => {}
                Ok(Ok(Got::Refused(by, why))) => {
                    // The entry stage names an exit refusal by the exit stage too.
                    let own = by == *stage || (*stage == Stage::Exit && by == Stage::Entry) || (*stage == Stage::Entry && by == Stage::Exit);
                    assert!(own, "{at}: refused by the {by:?} stage, not its own: {why}");
                    assert!(!CONSTRUCTOR.contains(&why), "{at}: refused by a constructor's reason: {why}");
                    assert!(*stage == Stage::Order || why != NOT_A_PERMUTATION, "{at}: refused by the order stage's reason: {why}");
                }
            }
            checked += 1;
        }
    }
    assert_eq!(checked, 7 * (6 + 4 + 3));
}

#[test]
fn the_order_stage_runs_a_helix_down_its_lead_and_names_a_direction_that_its_hand_does_not_give() {
    // The review's case: a hole (Inside), a helix on Circle(5, 5; 10) with a
    // right hand, the default order choice (Auto, climb). Before the fix the
    // stage refused it with HELIX_NEEDS_LEAD. A right hand turns clockwise
    // as it descends: in a hole that is conventional, so climb is named.
    let r = hole_region(MaterialSide::Inside).unwrap();
    for (hand, direction, named) in [
        (Hand::Right, Direction::Climb, true),
        (Hand::Right, Direction::Conventional, false),
        (Hand::Left, Direction::Climb, false),
        (Hand::Left, Direction::Conventional, true),
    ] {
        let set = HelixPlanner.guide(&r, &helix(hand, 1.2), TOL).unwrap().out().unwrap().clone();
        let o = OrderChoice { start: StartSide::Auto, direction };
        let v = OrderStage.order(set.clone(), &r, &o, None).unwrap_or_else(|e| panic!("{hand:?} {direction:?}: {e:?}"));
        let Verdict::Done { out, notes, .. } = &v else { panic!("{hand:?} {direction:?}: a done verdict") };
        let [h] = out.curves() else { panic!("one helix") };
        assert!(set.curves()[0].same_cut(h), "{hand:?} {direction:?}: the same circle, lead and points");
        assert_eq!(h.lead(), set.curves()[0].lead());
        assert!(h.pts().last().unwrap().depth() < h.pts()[0].depth(), "{hand:?} {direction:?}: down its lead");
        let texts: Vec<&str> = notes.iter().map(|n| n.text()).collect();
        assert_eq!(texts.contains(&HELIX_HAND_SETS_DIRECTION), named, "{hand:?} {direction:?}: {texts:?}");
    }
}
