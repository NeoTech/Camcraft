//! The link rule (#4023 stage 5, #4027): stay at depth; lift only with a
//! named reason. The check counts the lifts of the plan against the link
//! tally (in total and by reason), counts the lifts on a fixture with no
//! split ring (there must be none), and reads each stay-down hop - from
//! the end of its cut, along its points, to the start of the next cut - for
//! a crossing out of the centre area and the air.

use std::collections::BTreeMap;

use camcraft_pathkernel::link::{Hop, LinkPolicy, LinkProof, LinkTally};
use camcraft_pathkernel::plan::{Cut, Step};
use camcraft_pathkernel::region::KernelRegion;
use camcraft_pathkernel::{Point2, Refusal};

use super::grid::samples;
use super::kit::{
    Case, Law, LawVerdict, MATCH_TOL, Measure, Planners, Run, assert_broken, assert_held, judge, outside, planar, refused_held, run,
};
use super::standins::{LEAVES_AREA, end_of, good, island_case, square_case, start_of};

/// L-link over one run of the case `c`, or over its refusal.
pub fn check_link(run: &Result<Run, Refusal>, c: &Case) -> LawVerdict {
    let r = match run {
        Ok(r) => r,
        Err(e) => return refused_held(Law::Link, e),
    };
    let steps = r.plan.steps();
    let mut by_reason: BTreeMap<&'static str, u32> = BTreeMap::new();
    for s in steps {
        if let Step::Link(Hop::Lift(why)) = s {
            *by_reason.entry(why.get()).or_insert(0) += 1;
        }
    }
    let lifts: u32 = by_reason.values().sum();
    let tally = r.tally.lifts();
    let mut m =
        vec![Measure::new(f64::from(lifts.abs_diff(tally)), 0.0, format!("lifts: {lifts} in the plan, {tally} in the tally"), None)];
    let odd = by_reason.iter().filter(|(k, v)| r.tally.by_reason().get(*k) != Some(*v)).count();
    m.push(Measure::count(odd, format!("lift reasons: {by_reason:?} in the plan, {:?} in the tally", r.tally.by_reason()), None));
    if c.no_split {
        m.push(Measure::new(f64::from(lifts), 0.0, format!("lifts on a fixture with no split ring: {by_reason:?}"), None));
    }
    let Ok(p) = planar(&r.region) else {
        return judge(Law::Link, m);
    };
    for (i, s) in steps.iter().enumerate() {
        let Step::Link(Hop::Stay(path)) = s else {
            continue;
        };
        let cut = |j: Option<usize>| match j.and_then(|j| steps.get(j)) {
            Some(Step::Cut(c)) => Some(c),
            _ => None,
        };
        let mut line: Vec<Point2> = cut(i.checked_sub(1)).map(end_of).into_iter().collect();
        line.extend(path.points());
        line.extend(cut(i.checked_add(1)).map(start_of));
        let worst = samples(&line, 0.05)
            .iter()
            .map(|(q, _)| (outside(p.center(), *q).min(outside(p.air(), *q)), *q))
            .max_by(|a, b| a.0.total_cmp(&b.0));
        if let Some((d, q)) = worst {
            m.push(Measure::new(d, MATCH_TOL, format!("stay link at step {i}: a crossing out of the area"), Some(q)));
        }
    }
    judge(Law::Link, m)
}

/// Breaks the link rule one of three ways.
enum BadLink {
    /// Lifts and does not count the lift.
    Silent,
    /// Lifts (counted) where the straight hop could stay.
    Always,
    /// Stays at depth across the island.
    Across,
}

impl LinkProof for BadLink {
    fn link(&self, _from: &Cut, _to: &Cut, _r: &KernelRegion, _planned: &[Cut], _policy: LinkPolicy, tally: &mut LinkTally) -> Hop {
        let lift = || Hop::lift(LEAVES_AREA).expect("a named lift");
        match self {
            Self::Silent => lift(),
            Self::Always => tally.record(lift()),
            // The island of the fixture is the square from (10, 10) to (20, 20).
            Self::Across => Hop::stay(vec![Point2::new(15.0, 15.0)]).expect("a stay hop"),
        }
    }
}

fn link(p: &Planners, c: &Case) -> LawVerdict {
    check_link(&run(p, c, Default::default()), c)
}

#[test]
fn link_rule_holds_for_counted_lifts_and_proved_stays() {
    assert_held(&link(&good(), &square_case().unwrap()));
    assert_held(&link(&good(), &island_case().unwrap()));
}

#[test]
fn link_rule_breaks_for_an_uncounted_lift() {
    assert_broken(&link(&Planners { link: Box::new(BadLink::Silent), ..good() }, &island_case().unwrap()), Law::Link, "in the tally");
}

#[test]
fn link_rule_breaks_for_a_lift_on_a_fixture_with_no_split_ring() {
    let v = link(&Planners { link: Box::new(BadLink::Always), ..good() }, &square_case().unwrap());
    assert_broken(&v, Law::Link, "no split ring");
}

#[test]
fn link_rule_breaks_for_a_stay_across_the_island() {
    let v = link(&Planners { link: Box::new(BadLink::Across), ..good() }, &island_case().unwrap());
    assert_broken(&v, Law::Link, "a crossing out of the area");
    assert!(v.worst.value > 1.0, "{v}");
}
