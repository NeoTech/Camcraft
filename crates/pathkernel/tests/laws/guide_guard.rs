//! The completeness guard of the guide planners (#4176 review of PR #4189):
//! the guard reads the WHOLE cutter-centre area of the region, so no choice
//! of parameters can hide an uncut area. A done verdict means that at most
//! 0.1 mm2 of the centre area stays uncut, measured here on its own (the
//! area less the swept discs along every curve of the set); an incomplete
//! verdict names more than that; any other result is a named refusal.

use super::guide_kit::{center, circle, pocket, rect, segs};
use camcraft_pathkernel::guide::rings::{NO_RING, WALL_STRIP_UNCUT};
use camcraft_pathkernel::guide::{
    GuideChoice, GuidePlanner, GuideSet, RingPlanner, RingRef, RingsChoice, RowPlanner, RowsChoice,
    STEP_ABOVE_DIAMETER,
};
use camcraft_pathkernel::{KernelRegion, Point2, Region, Verdict};

const TOL: f64 = 1e-3;
/// The guard's limit (record "Stage 2").
const LIMIT: f64 = 0.1;
/// The flattening allowance of the independent measure: 10 um on the
/// tool radius (the planner's own guard uses 6 um).
const SAG: f64 = 0.01;

/// The area of the centre that the tool discs along the set leave uncut.
fn uncut(r: &KernelRegion, set: &GuideSet, rad: f64) -> f64 {
    let walks: Vec<Vec<Point2>> = set
        .curves()
        .iter()
        .map(|c| {
            let s = segs(c);
            let mut w: Vec<Point2> = s.iter().map(|(a, _)| *a).collect();
            if let Some((_, b)) = s.last() {
                w.push(*b);
            }
            w
        })
        .collect();
    let swept = Region::sweep(&walks, rad + SAG).unwrap();
    center(r).difference(&swept).unwrap().area()
}

/// Checks one result of a guide planner against the guard's law.
fn holds(r: &KernelRegion, v: &Verdict<GuideSet>, rad: f64, what: &str) {
    match v {
        Verdict::Done { out, .. } => {
            let left = uncut(r, out, rad);
            assert!(left <= LIMIT, "{what}: done with {left:.3} mm2 uncut");
        }
        Verdict::Incomplete { gaps, .. } => {
            let named: f64 = gaps.list().iter().map(|g| g.area_mm2()).sum();
            assert!(named > LIMIT, "{what}: incomplete with only {named:.3} mm2 named");
        }
        Verdict::Refused(e) => panic!("{what}: a refused verdict in place of an error: {e}"),
    }
}

/// A disc pocket of radius 20, and a 60 by 40 pocket round a round island
/// of radius 8.
fn shapes(rad: f64) -> [(&'static str, KernelRegion); 2] {
    [
        ("disc", pocket(circle(0.0, 0.0, 20.0, 256), vec![], rad)),
        ("island", pocket(rect(0.0, 0.0, 60.0, 40.0), vec![circle(30.0, 20.0, 8.0, 128)], rad)),
    ]
}

#[test]
fn a_first_offset_above_the_tool_radius_is_refused_not_left_uncut() {
    // The review case: a disc pocket, tool radius 3, first offset 5, step 2.
    // Before the fix the verdict was done with about 201 mm2 uncut.
    let r = pocket(circle(0.0, 0.0, 20.0, 256), vec![], 3.0);
    let g = GuideChoice::Rings(RingsChoice::new(RingRef::Boundary, 5.0, 2.0, None, None).unwrap());
    assert_eq!(RingPlanner.guide(&r, &g, TOL).unwrap_err().reason(), WALL_STRIP_UNCUT);
    // With a ring count the choice asks for rings, not a cleared area: done,
    // and the uncut area is named in a note.
    let g = GuideChoice::Rings(RingsChoice::new(RingRef::Boundary, 5.0, 2.0, None, Some(2)).unwrap());
    let v = RingPlanner.guide(&r, &g, TOL).unwrap();
    let Verdict::Done { notes, out, .. } = &v else { panic!("done: {v:?}") };
    assert_eq!(out.curves().len(), 2);
    assert!(notes.iter().any(|n| n.text().contains("uncut, as the choice asks")), "{notes:?}");
}

/// The ring grid on one shape and one reference: tool radius, first offset
/// and stepover. Each result is done within the limit, incomplete with more
/// than the limit named, or refused by name for a first offset above the
/// tool radius.
fn ring_grid(shape: usize, reference: RingRef) {
    for rad in [2.0, 3.0] {
        let (name, r) = &shapes(rad)[shape];
        for d0 in [0.0, 0.5 * rad, rad, 1.5 * rad] {
            for step in [0.5 * rad, rad, 1.5 * rad, 2.0 * rad] {
                let what = format!("{name} r {rad} {reference:?} first {d0} step {step}");
                let g = GuideChoice::Rings(RingsChoice::new(reference, d0, step, None, None).unwrap());
                match RingPlanner.guide(r, &g, TOL) {
                    Ok(v) => holds(r, &v, rad, &what),
                    Err(e) => {
                        assert!(d0 > rad, "{what}: refused: {e}");
                        assert_eq!(e.reason(), WALL_STRIP_UNCUT, "{what}");
                    }
                }
            }
        }
    }
}

#[test]
fn done_rings_leave_at_most_the_limit_uncut_on_a_disc() {
    ring_grid(0, RingRef::Boundary);
}

#[test]
fn done_part_rings_leave_at_most_the_limit_uncut_on_a_disc() {
    ring_grid(0, RingRef::Part);
}

#[test]
fn done_rings_leave_at_most_the_limit_uncut_round_an_island() {
    ring_grid(1, RingRef::Boundary);
}

#[test]
fn done_part_rings_leave_at_most_the_limit_uncut_round_an_island() {
    ring_grid(1, RingRef::Part);
}

#[test]
fn done_rows_leave_at_most_the_limit_uncut_for_each_choice() {
    for rad in [1.5, 3.0] {
        for (name, r) in shapes(rad) {
            for angle in [0.0, 30.0, 90.0] {
                for step in [0.5 * rad, rad, 2.0 * rad] {
                    for closing in [false, true] {
                        let what = format!("{name} r {rad} angle {angle} step {step} closing {closing}");
                        let g = GuideChoice::Rows(RowsChoice::new(angle, step, closing).unwrap());
                        holds(&r, &RowPlanner.guide(&r, &g, TOL).unwrap(), rad, &what);
                    }
                }
            }
        }
    }
}

#[test]
fn the_guard_refusals_stay_named() {
    let r = pocket(circle(0.0, 0.0, 20.0, 256), vec![], 3.0);
    let rings = |d0: f64, step: f64| GuideChoice::Rings(RingsChoice::new(RingRef::Boundary, d0, step, None, Some(1)).unwrap());
    assert_eq!(RingPlanner.guide(&r, &rings(18.0, 2.0), TOL).unwrap_err().reason(), NO_RING);
    assert_eq!(RingPlanner.guide(&r, &rings(0.0, 7.0), TOL).unwrap_err().reason(), STEP_ABOVE_DIAMETER);
}
