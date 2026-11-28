//! L1 (#4027): the same region and the same guide choice give the same
//! curves. The check calls the guide stage twice, once more on the same
//! region given another way (loops reversed, each started at its next
//! vertex), and through each other kind that shares the guide choice. The
//! canonical sets must match within 1 um (Hausdorff per curve, same count).
//! A refusal never matches (#4406): when the first call refuses, L1 is not
//! measured (refused, with the reason); when a later call refuses against
//! curves, L1 is broken. Two refusals are never compared by their reason
//! text, so stand-ins that share one reason cannot pass as equal.

use std::cell::Cell;

use camcraft_pathkernel::guide::{GuideChoice, GuidePlanner, GuideSet};
use camcraft_pathkernel::region::{KernelRegion, MaterialSide};
use camcraft_pathkernel::{Point2, Refusal, Verdict};

use super::kit::{GUIDE_TOL, Law, LawVerdict, MATCH_TOL, Measure, assert_broken, assert_held, judge, permuted, planar, set_gap};
use super::standins::{RingGuide, loop_curve, rings, rings_choice, square_region};

type GuideResult = Result<Verdict<GuideSet>, Refusal>;

/// The curves of a guide result (none for a refusal).
fn curves(r: &GuideResult) -> Option<Vec<&camcraft_pathkernel::guide::Curve>> {
    r.as_ref().ok().and_then(Verdict::out).map(|g| g.curves().iter().collect())
}

fn refusal(r: &GuideResult) -> Option<&Refusal> {
    match r {
        Err(e) | Ok(Verdict::Refused(e)) => Some(e),
        Ok(_) => None,
    }
}

/// The gap of a later guide result `b` against the first result `a`, which
/// gave curves: a refusal of `b` is an infinite gap.
fn gap(a: &[&camcraft_pathkernel::guide::Curve], b: &GuideResult, what: &str) -> Measure {
    match curves(b) {
        Some(y) => {
            let (g, place) = set_gap(a, &y);
            Measure::new(g, MATCH_TOL, format!("{what}: {place}"), None)
        }
        None => {
            let why = refusal(b).map_or("no output", Refusal::reason);
            Measure::new(f64::INFINITY, MATCH_TOL, format!("{what}: a refusal against curves ({why})"), None)
        }
    }
}

/// L1 over `guide` on `region` with `choice`; `others` are the guide
/// planners of other kinds that share the choice.
pub fn check_l1(guide: &dyn GuidePlanner, region: &KernelRegion, choice: &GuideChoice, others: &[&dyn GuidePlanner]) -> LawVerdict {
    let first = guide.guide(region, choice, GUIDE_TOL);
    let Some(base) = curves(&first) else {
        return match refusal(&first) {
            Some(e) => judge(Law::L1, vec![Measure::refusal("the first call", e)]),
            None => judge(Law::L1, vec![Measure::unread("the first call", super::kit::NO_OUTPUT)]),
        };
    };
    if base.is_empty() {
        return judge(Law::L1, vec![Measure::unread("the first call", "the guide gave no curves")]);
    }
    let base = &base;
    let mut m = vec![gap(base, &guide.guide(region, choice, GUIDE_TOL), "a second call")];
    match planar(region).and_then(permuted) {
        Ok(turned) => m.push(gap(base, &guide.guide(&turned, choice, GUIDE_TOL), "the loops permuted and restarted")),
        Err(e) => m.push(Measure::new(f64::INFINITY, MATCH_TOL, format!("the permuted region: {}", e.reason()), None)),
    }
    for (i, o) in others.iter().enumerate() {
        m.push(gap(base, &o.guide(region, choice, GUIDE_TOL), &format!("shared guide {i}")));
    }
    judge(Law::L1, m)
}

/// Breaks L1 one way: each call moves the rings a little (hidden state).
struct Drifting(Cell<u32>);

impl GuidePlanner for Drifting {
    fn guide(&self, r: &KernelRegion, g: &GuideChoice, tol: f64) -> GuideResult {
        let n = self.0.get();
        self.0.set(n + 1);
        RingGuide { extra: 0.01 * f64::from(n) }.guide(r, g, tol)
    }
}

/// Breaks L1 the other way: the first ring has a seam step at the first
/// vertex of the first part wall, so the curves depend on how the loops
/// are given.
struct Seam;

impl GuidePlanner for Seam {
    fn guide(&self, r: &KernelRegion, g: &GuideChoice, _tol: f64) -> GuideResult {
        let p = planar(r)?;
        let GuideChoice::Rings(rc) = g else {
            return RingGuide { extra: 0.0 }.guide(r, g, GUIDE_TOL);
        };
        let mut curves = rings(p.center(), rc.first_offset(), rc.stepover(), rc.count())?;
        if let (Some(w), Some(first)) = (p.part_walls().first().and_then(|l| l.points().first()), curves.first().cloned()) {
            let seam = camcraft_pathkernel::Loop::new(
                first
                    .pts()
                    .iter()
                    .map(|q| {
                        if (q.at().x - w.x).abs() < 3.0 && (q.at().y - w.y).abs() < 3.0 {
                            Point2::new(q.at().x + 0.5, q.at().y + 0.5)
                        } else {
                            q.at()
                        }
                    })
                    .collect(),
            );
            if let Ok(l) = seam {
                curves[0] = loop_curve(&l, first.k())?;
            }
        }
        Ok(Verdict::done(GuideSet::new(curves)))
    }
}

fn region() -> KernelRegion {
    square_region(MaterialSide::Inside).expect("fixture")
}

#[test]
fn l1_holds_for_a_pure_guide_and_a_shared_guide() {
    let (g, other) = (RingGuide { extra: 0.0 }, RingGuide { extra: 0.0 });
    let v = check_l1(&g, &region(), &rings_choice().unwrap(), &[&other]);
    assert_held(&v);
    assert_eq!(v.worst.value, 0.0, "{v}");
}

#[test]
fn l1_breaks_for_a_guide_with_hidden_state() {
    let v = check_l1(&Drifting(Cell::new(0)), &region(), &rings_choice().unwrap(), &[]);
    // Each later call drifts more: the third call (the permuted region) is the worst.
    assert_broken(&v, Law::L1, "curve");
    assert!(v.worst.value >= 0.01 - MATCH_TOL, "{v}");
}

#[test]
fn l1_breaks_for_a_guide_that_reads_the_loop_start() {
    let v = check_l1(&Seam, &region(), &rings_choice().unwrap(), &[]);
    assert_broken(&v, Law::L1, "permuted");
}

#[test]
fn l1_breaks_when_two_kinds_with_one_guide_choice_differ() {
    let v = check_l1(&RingGuide { extra: 0.0 }, &region(), &rings_choice().unwrap(), &[&RingGuide { extra: 0.05 }]);
    assert_broken(&v, Law::L1, "shared guide 0");
}
