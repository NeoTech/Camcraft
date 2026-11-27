//! #4406: a refusal is never held. Each law check is run on stand-ins that
//! always refuse: the law must be reported as refused (not measured), never
//! as held. The law runner of the registry must fail a kind whose stage
//! refuses a case that the kind does not register as an expected refusal.
//! (The stand-ins that break each law are in the law files.)
//! Provenance: our own design record (RECOMMENDED.md "#4024 - Path kernel
//! K0", "The laws as checks"; "a refusal is a legal result, not a pass").

use camcraft_pathkernel::entry::{Entry, EntryChoice, EntryPlanner, Exit};
use camcraft_pathkernel::guide::{GuideChoice, GuidePlanner, GuideSet};
use camcraft_pathkernel::load::{Controlled, LoadControl, LoadPlanner, StockRead};
use camcraft_pathkernel::order::{OrderChoice, OrderPlanner, OrderedSet};
use camcraft_pathkernel::plan::Cut;
use camcraft_pathkernel::region::{KernelRegion, MaterialSide};
use camcraft_pathkernel::{Refusal, Region, Stage, Verdict};

use super::kit::{Case, Law, Outcome, Planners, assert_broken, assert_refused, run};
use super::l1_guide::check_l1;
use super::l2_sides::check_l2;
use super::l3_order::check_l3;
use super::l5_entry::check_l5;
use super::l6_load::check_l6;
use super::link_rule::check_link;
use super::registry::{ExpectedRefusal, check_all, law_failures, stand_in_kind};
use super::standins::{ProvedEntry, RingGuide, SideOrder, good, rings_choice, square_case, square_region};

/// The reason of each always-refusing stand-in: one for each stage, so that
/// a verdict names the stage that refused.
const NO_REGION: &str = "the stand-in region stage always refuses";
const NO_GUIDE: &str = "the stand-in guide stage always refuses";
const NO_ORDER: &str = "the stand-in order stage always refuses";
const NO_LOAD: &str = "the stand-in load stage always refuses";
const NO_ENTRY: &str = "the stand-in entry stage always refuses";
const NO_EXIT: &str = "the stand-in exit always refuses";

fn no_region(_m: MaterialSide) -> Result<KernelRegion, Refusal> {
    Err(Refusal::new(Stage::Region, NO_REGION))
}

/// Refuses the Outside material side only (an L2 breaker).
fn no_outside(m: MaterialSide) -> Result<KernelRegion, Refusal> {
    if m == MaterialSide::Outside { no_region(m) } else { square_region(m) }
}

struct NoGuide;

impl GuidePlanner for NoGuide {
    fn guide(&self, _r: &KernelRegion, _g: &GuideChoice, _tol: f64) -> Result<Verdict<GuideSet>, Refusal> {
        Ok(Verdict::Refused(Refusal::new(Stage::Guide, NO_GUIDE)))
    }
}

struct NoOrder;

impl OrderPlanner for NoOrder {
    fn order(&self, _g: GuideSet, _r: &KernelRegion, _o: &OrderChoice, _s: Option<&Region>) -> Result<Verdict<OrderedSet>, Refusal> {
        Err(Refusal::new(Stage::Order, NO_ORDER))
    }
}

struct NoLoad;

impl LoadPlanner for NoLoad {
    fn control(&self, _s: OrderedSet, _r: &KernelRegion, _st: &dyn StockRead, _l: &LoadControl, _t: f64) -> Result<Verdict<Controlled>, Refusal> {
        Ok(Verdict::Refused(Refusal::new(Stage::Load, NO_LOAD)))
    }
}

/// Refuses each entry (`exit: false`) or each exit (`exit: true`).
struct NoEntry {
    exit: bool,
}

impl EntryPlanner for NoEntry {
    fn entry(&self, first: &Cut, r: &KernelRegion, cleared: &Region, e: &EntryChoice) -> Result<Verdict<Entry>, Refusal> {
        if self.exit { ProvedEntry.entry(first, r, cleared, e) } else { Err(Refusal::new(Stage::Entry, NO_ENTRY)) }
    }
    fn exit(&self, last: &Cut, r: &KernelRegion, cleared: &Region) -> Result<Exit, Refusal> {
        if self.exit { Err(Refusal::new(Stage::Entry, NO_EXIT)) } else { ProvedEntry.exit(last, r, cleared) }
    }
}

/// Each stage planner refuses (the link proof has no refusal: it lifts).
fn refusing() -> Planners {
    Planners { guide: Box::new(NoGuide), order: Box::new(NoOrder), load: Box::new(NoLoad), entry: Box::new(NoEntry { exit: false }), ..good() }
}

fn guide() -> GuideSet {
    let r = square_region(MaterialSide::Inside).unwrap();
    RingGuide { extra: 0.0 }.guide(&r, &rings_choice().unwrap(), 1e-3).unwrap().out().unwrap().clone()
}

fn case() -> Case {
    square_case().unwrap()
}

#[test]
fn l1_is_refused_for_a_guide_that_always_refuses() {
    let r = square_region(MaterialSide::Inside).unwrap();
    // Two refusals with the same reason do not match: the law is not measured.
    assert_refused(&check_l1(&NoGuide, &r, &rings_choice().unwrap(), &[&NoGuide]), Law::L1, NO_GUIDE);
    // Curves against a refusal of a shared guide break the law.
    assert_broken(&check_l1(&RingGuide { extra: 0.0 }, &r, &rings_choice().unwrap(), &[&NoGuide]), Law::L1, "a refusal against curves");
}

#[test]
fn l2_is_refused_for_a_region_or_an_order_that_always_refuses() {
    assert_refused(&check_l2(no_region, &SideOrder, &guide(), OrderChoice::default()), Law::L2, NO_REGION);
    assert_refused(&check_l2(square_region, &NoOrder, &guide(), OrderChoice::default()), Law::L2, "refused both times");
}

#[test]
fn l2_breaks_for_a_region_that_refuses_one_material_side() {
    // The Inside leg still runs: the region measure is the broken one.
    let v = check_l2(no_outside, &SideOrder, &guide(), OrderChoice::default());
    assert_broken(&v, Law::L2, "region: the Outside side refused");
}

#[test]
fn l3_l5_l6_and_the_link_rule_are_refused_for_a_refused_run() {
    let c = case();
    for (p, why) in [
        (Planners { guide: Box::new(NoGuide), ..good() }, NO_GUIDE),
        (Planners { order: Box::new(NoOrder), ..good() }, NO_ORDER),
        (Planners { load: Box::new(NoLoad), ..good() }, NO_LOAD),
        (Planners { entry: Box::new(NoEntry { exit: false }), ..good() }, NO_ENTRY),
        (Planners { entry: Box::new(NoEntry { exit: true }), ..good() }, NO_EXIT),
    ] {
        assert_refused(&check_l3(&p, &c), Law::L3, why);
        let r = run(&p, &c, Default::default());
        assert_refused(&check_l5(&r, &c), Law::L5, why);
        assert_refused(&check_l6(&r, &c), Law::L6, why);
        assert_refused(&check_link(&r, &c), Law::Link, why);
    }
}

#[test]
fn no_law_is_held_on_an_always_refusing_planner_set() {
    let kr = stand_in_kind();
    let recorded = kr.cases.into_iter().next().and_then(|c| c.recorded);
    let with_region = Case { recorded: recorded.clone(), ..case() };
    let no_region_case = Case { region: no_region, recorded, ..case() };
    for (c, p) in [(&with_region, refusing()), (&no_region_case, refusing()), (&no_region_case, good())] {
        let v = check_all(&p, c);
        assert_eq!(v.len(), 8, "each law and the link rule");
        for x in &v {
            eprintln!("{x}");
            assert!(!x.held(), "{}: a law held on a refused run: {x}", c.name);
            // L7 compares with the old output: a refusal where the old
            // planner cut is a breach, not a refusal.
            let want = if x.law == Law::L7 { Outcome::Broken } else { x.outcome.clone() };
            assert_eq!(x.outcome, want, "{x}");
            if x.law != Law::L7 {
                assert!(x.refused().is_some(), "{x}");
            }
        }
    }
}

#[test]
fn the_law_runner_fails_a_kind_whose_stage_refuses_a_case() {
    let refusing_kind = || {
        let mut kr = stand_in_kind();
        kr.planners = Planners { entry: Box::new(NoEntry { exit: false }), ..good() };
        kr
    };
    let f = law_failures("refusing", &refusing_kind());
    assert!(!f.is_empty(), "a refused case must fail the runner");
    assert!(f.iter().all(|x| x.contains("an unexpected refusal") || x.contains("BROKEN")), "{f:#?}");
    // The kind registers the refusal by name for one case: that case
    // passes, and the other case still fails.
    let mut kr = refusing_kind();
    kr.refusals = vec![ExpectedRefusal { case: "square pocket", reason: NO_ENTRY }];
    let f = law_failures("refusing", &kr);
    assert!(f.iter().all(|x| !x.contains("/ square pocket: an unexpected")), "{f:#?}");
    assert!(f.iter().any(|x| x.contains("/ pocket with an island: an unexpected refusal")), "{f:#?}");
    // A registered reason that is not the reason given does not pass.
    let mut kr = refusing_kind();
    kr.refusals = vec![ExpectedRefusal { case: "square pocket", reason: NO_GUIDE }];
    assert!(law_failures("refusing", &kr).iter().any(|x| x.contains("/ square pocket: an unexpected refusal")));
}

/// The record of the square pocket is the stand-in kind's, so the L7 of a
/// refused run breaks (the old planner cut, the kernel refused); the
/// expected refusal does not hide a broken law.
#[test]
fn an_expected_refusal_does_not_hide_a_broken_law() {
    let mut kr = stand_in_kind();
    kr.planners = Planners { entry: Box::new(NoEntry { exit: false }), ..good() };
    kr.refusals = vec![ExpectedRefusal { case: "square pocket", reason: NO_ENTRY }, ExpectedRefusal { case: "pocket with an island", reason: NO_ENTRY }];
    let f = law_failures("refusing", &kr);
    assert!(!f.is_empty() && f.iter().all(|x| x.contains("L7 BROKEN")), "{f:#?}");
}
