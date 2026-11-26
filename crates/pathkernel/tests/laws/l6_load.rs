//! L6 (#4027): a cut above the engagement limit is prevented or named. The
//! kernel's `Controlled` already refuses a reading above the limit that its
//! over-limit list does not name. This check reads each cut again on the
//! check's own stock grid, as the stock is at that step of the plan: each
//! read above `limit x (1 + PLAN_TOL)` (plus the grid's read error) must
//! have an over-limit record within one tool radius of its place.

use camcraft_pathkernel::Refusal;
use camcraft_pathkernel::load::PLAN_TOL;

use super::grid::{GRID_TOL, walk};
use super::kit::{Case, Law, LawVerdict, Measure, Planners, Run, assert_broken, assert_held, dist, judge, planar, refused_held, run};
use super::l5_entry::NO_STOCK;
use super::standins::{LIMIT, ReadLoad, good, island_case, square_case};

/// L6 over one run of the case `c`, or over its refusal.
pub fn check_l6(run: &Result<Run, Refusal>, c: &Case) -> LawVerdict {
    let r = match run {
        Ok(r) => r,
        Err(e) => return refused_held(Law::L6, e),
    };
    let (Some(ctl), Ok(tool_r)) = (r.controlled.out(), planar(&r.region).map(|p| p.tool().radius())) else {
        return judge(Law::L6, vec![Measure::new(f64::INFINITY, 0.0, "no load output or no planar region", None)]);
    };
    let Some(w) = walk(&r.plan, &c.stock, tool_r) else {
        return judge(Law::L6, vec![Measure::new(f64::INFINITY, 0.0, NO_STOCK, None)]);
    };
    let cap = ctl.limit() * (1.0 + PLAN_TOL) + GRID_TOL;
    let mut m: Vec<Measure> = w
        .cuts
        .iter()
        .map(|k| {
            let named = ctl.over().iter().any(|o| dist(o.at(), k.at) <= tool_r);
            let v = if named { 0.0 } else { k.load };
            Measure::new(v, cap, format!("cut {}: ae/D {:.3}, named: {named}", k.index, k.load), Some(k.at))
        })
        .collect();
    let top = w.cuts.iter().map(|k| k.load).fold(0.0, f64::max);
    m.push(Measure::new(0.0, cap, format!("the highest read ae/D is {top:.3}"), None));
    judge(Law::L6, m)
}

fn l6(p: &Planners, c: &Case) -> LawVerdict {
    check_l6(&run(p, c, Default::default()), c)
}

#[test]
fn l6_holds_when_each_full_width_cut_is_named() {
    for c in [square_case().unwrap(), island_case().unwrap()] {
        let v = l6(&good(), &c);
        assert_held(&v);
        assert!((v.worst.limit - (LIMIT * (1.0 + PLAN_TOL) + GRID_TOL)).abs() < 1e-12, "{}: {v}", c.name);
    }
}

#[test]
fn l6_breaks_for_a_load_stage_that_reads_a_slot_as_a_side_cut() {
    let v = l6(&Planners { load: Box::new(ReadLoad { blind: true }), ..good() }, &square_case().unwrap());
    assert_broken(&v, Law::L6, "named: false");
    assert!(v.worst.value > 0.9, "a full-width slot: {v}");
}
