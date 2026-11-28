//! L7 (#4027): a move that migrates keeps its output. The check compares
//! the cuts of the kernel's plan, in order and point by point, with the
//! recorded old output of the case (within 1 um). A case with no record is
//! a breach, so that no kind moves without one.
//!
//! #4406: the record comes from a source that the run under test does not
//! make. Before, the holds test compared the kernel with its own output of
//! the same run, so it could not break. For a real kind, the record is the
//! old planner's output: its golden files are in `camcraft-motion`
//! (`tests/pass_start_levels/kernel_baseline/`, `tests/golden/kernel_*`),
//! and the old-against-new comparison is at the motion level
//! (`tests/kernel_laws/`): this crate cannot depend on motion. In this
//! crate, the record of a case is a file recorded once and kept in
//! `tests/laws/records/`; the check reads it and does not make it again.
//! An empty record and a refused run are not measured, never held.

use camcraft_pathkernel::guide::{Curve, Family};
use camcraft_pathkernel::{PlanePoint, Point2, Refusal, Stage};
use camcraft_pathkernel::order::Direction;
use camcraft_pathkernel::plan::Plan;

use super::kit::{Law, LawVerdict, MATCH_TOL, Measure, Planners, Run, assert_broken, assert_held, assert_refused, judge, plan_cuts, run, seq_gap};
use super::standins::{RingGuide, good, square_case};

/// L7 of the kernel plan `plan` against the record `recorded`.
pub fn check_l7(recorded: Option<&[Curve]>, plan: &Result<Plan, Refusal>) -> LawVerdict {
    let Some(old) = recorded else {
        return judge(Law::L7, vec![Measure::new(f64::INFINITY, MATCH_TOL, "the case has no recorded old output", None)]);
    };
    let old: Vec<&Curve> = old.iter().collect();
    let m = match plan {
        Ok(p) if old.is_empty() && plan_cuts(p).is_empty() => Measure::unread("the record", "the recorded old output has no cut"),
        Err(e) if old.is_empty() => Measure::refusal("no old cut, and the kernel", e),
        Ok(p) => {
            let (g, at) = seq_gap(&old, &plan_cuts(p));
            Measure::new(g, MATCH_TOL, at, None)
        }
        Err(e) => Measure::new(f64::INFINITY, MATCH_TOL, format!("the old planner cut, the kernel refused: {}", e.reason()), None),
    };
    judge(Law::L7, vec![m])
}

/// The folder of the recorded outputs of the stand-in cases.
const RECORDS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/laws/records");

/// Refusal reason of the kit: a record file that does not read.
pub const BAD_RECORD: &str = "the record file does not read";

/// The recorded cuts of the file `name` (in `tests/laws/records/`): one line
/// for each cut, `<k> <closed 0|1> x,y,depth x,y,depth ...`; a line that
/// starts with `#` is a comment. All the cuts are of the ring family.
pub fn recorded(name: &str) -> Result<Vec<Curve>, Refusal> {
    let bad = || Refusal::new(Stage::Recipe, BAD_RECORD);
    let text = std::fs::read_to_string(format!("{RECORDS}/{name}")).map_err(|_| bad())?;
    let mut out = Vec::new();
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
        let mut words = line.split_whitespace();
        let k: u32 = words.next().and_then(|w| w.parse().ok()).ok_or_else(bad)?;
        let closed = match words.next() {
            Some("1") => true,
            Some("0") => false,
            _ => return Err(bad()),
        };
        let pts = words
            .map(|w| {
                let v: Vec<f64> = w.split(',').map(str::parse).collect::<Result<_, _>>().map_err(|_| bad())?;
                let [x, y, d] = v[..] else { return Err(bad()) };
                PlanePoint::new(Point2::new(x, y), d)
            })
            .collect::<Result<Vec<_>, _>>()?;
        out.push(Curve::new(pts, None, closed, k, Family::Ring)?);
    }
    Ok(out)
}

fn record() -> Vec<Curve> {
    recorded("square_pocket.txt").expect("the recorded square pocket")
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
fn l7_holds_against_the_recorded_file() {
    // The record was made once and is read from its file: a change of the
    // stand-in output, or of the kernel types under it, breaks this test.
    let v = l7(&good(), false);
    assert_held(&v);
    assert_eq!(record().len(), 10, "the record has the 10 rings of the square pocket");
}

#[test]
fn l7_is_not_held_for_an_empty_record_or_a_refused_run() {
    let r = run(&good(), &square_case().unwrap(), Default::default()).map(|r| r.plan);
    // The old planner cut nothing, and the kernel cuts: the output changed.
    assert_broken(&check_l7(Some(&[]), &r), Law::L7, "0 cuts against");
    let refused = Err(Refusal::new(Stage::Guide, "the guide refused"));
    assert_refused(&check_l7(Some(&[]), &refused), Law::L7, "the guide refused");
    assert_broken(&check_l7(Some(&record()), &refused), Law::L7, "the kernel refused");
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
