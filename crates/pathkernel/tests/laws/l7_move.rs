//! L7 (#4027): a move that migrates keeps its output. The check compares
//! the cuts of the kernel's plan, in order and point by point, with the
//! recorded old output of the case (within 1 um). The comparison harness of
//! `camcraft-app` gives the record from the golden `.cl` of the old planner;
//! a case with no record is a breach, so that no kind moves without one.

use camcraft_pathkernel::Refusal;
use camcraft_pathkernel::guide::Curve;
use camcraft_pathkernel::order::Direction;
use camcraft_pathkernel::plan::Plan;

use super::kit::{Law, LawVerdict, MATCH_TOL, Measure, Planners, Run, assert_broken, assert_held, judge, plan_cuts, run, seq_gap};
use super::standins::{RingGuide, good, square_case};

/// L7 of the kernel plan `plan` against the record `recorded`.
pub fn check_l7(recorded: Option<&[Curve]>, plan: &Result<Plan, Refusal>) -> LawVerdict {
    let Some(old) = recorded else {
        return judge(Law::L7, vec![Measure::new(f64::INFINITY, MATCH_TOL, "the case has no recorded old output", None)]);
    };
    let old: Vec<&Curve> = old.iter().collect();
    let m = match plan {
        Ok(p) => {
            let (g, at) = seq_gap(&old, &plan_cuts(p));
            Measure::new(g, MATCH_TOL, at, None)
        }
        Err(e) if old.is_empty() => Measure::new(0.0, MATCH_TOL, format!("no old cuts, refused: {}", e.reason()), None),
        Err(e) => Measure::new(f64::INFINITY, MATCH_TOL, format!("the old planner cut, the kernel refused: {}", e.reason()), None),
    };
    judge(Law::L7, vec![m])
}

fn record() -> Vec<Curve> {
    let r = run(&good(), &square_case().unwrap(), Default::default()).unwrap();
    plan_cuts(&r.plan).into_iter().cloned().collect()
}

fn l7(p: &Planners, conventional: bool) -> LawVerdict {
    let mut c = square_case().unwrap();
    if conventional {
        let o = c.recipe.order();
        let flip = camcraft_pathkernel::order::OrderChoice { direction: Direction::Conventional, ..o };
        c.recipe = camcraft_pathkernel::Recipe::new(
            *c.recipe.plane(),
            c.recipe.axis(),
            c.recipe.guide(),
            c.recipe.load(),
            flip,
            c.recipe.link(),
            c.recipe.entry().clone(),
        )
        .unwrap();
    }
    let rec = record();
    check_l7(Some(&rec), &run(p, &c, Default::default()).map(|r: Run| r.plan))
}

#[test]
fn l7_holds_for_the_same_output() {
    let v = l7(&good(), false);
    assert_held(&v);
    assert_eq!(v.worst.value, 0.0, "{v}");
}

#[test]
fn l7_breaks_for_a_moved_ring() {
    let v = l7(&Planners { guide: Box::new(RingGuide { extra: 0.01 }), ..good() }, false);
    assert_broken(&v, Law::L7, "cut");
}

#[test]
fn l7_breaks_for_another_direction() {
    let v = l7(&good(), true);
    assert_broken(&v, Law::L7, "cut");
    assert!(v.worst.value > 1.0, "a reversed ring: {v}");
}

#[test]
fn l7_breaks_without_a_record() {
    let r = run(&good(), &square_case().unwrap(), Default::default()).map(|r| r.plan);
    assert_broken(&check_l7(None, &r), Law::L7, "no recorded old output");
}
