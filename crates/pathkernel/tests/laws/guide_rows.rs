//! The row guide planner (#4176, K2): parallel rows at the angle, spaced
//! at the stepover or less, clipped to the centre area; a closing ring on
//! request; uncovered area named; each refusal named. Also the marks of a
//! guide set: each mark is checked against its curve.

use super::common::curve;
use super::guide_kit::{all_inside, center, circle, marks, p, pocket, rect, same_set, xy_of};
use camcraft_pathkernel::guide::mark::{BAD_STRETCH, MARK_NO_CURVE, MARK_OFF_CURVE};
use camcraft_pathkernel::guide::rows::{NOT_ROWS, TOO_MANY_ROWS};
use camcraft_pathkernel::guide::{
    Family, GuideChoice, GuidePlanner, GuideSet, Mark, MarkKind, RingRef, RingsChoice, RowPlanner, RowsChoice,
    STEP_ABOVE_DIAMETER,
};
use camcraft_pathkernel::{KernelRegion, PlanePoint, Refusal, Verdict};

const TOL: f64 = 1e-3;

fn rows(angle: f64, step: f64, closing: bool) -> GuideChoice {
    GuideChoice::Rows(RowsChoice::new(angle, step, closing).unwrap())
}

fn plan(r: &KernelRegion, g: &GuideChoice) -> Verdict<GuideSet> {
    RowPlanner.guide(r, g, TOL).unwrap()
}

fn refused(r: Result<Verdict<GuideSet>, Refusal>) -> &'static str {
    r.unwrap_err().reason()
}

/// A 40 by 30 pocket with a tool of radius 3: the centre is 34 by 24.
fn box_pocket() -> KernelRegion {
    pocket(rect(0.0, 0.0, 40.0, 30.0), vec![], 3.0)
}

#[test]
fn rows_run_at_the_angle_spaced_evenly_at_the_stepover_or_less() {
    let r = box_pocket();
    let v = plan(&r, &rows(0.0, 4.0, false));
    assert!(v.postable(), "{v:?}");
    let set = v.out().unwrap();
    // 24 mm across at 4 mm: 7 rows on y = 3, 7, ... 27, each from x 3 to 37.
    assert_eq!(set.curves().len(), 7);
    for (i, c) in set.curves().iter().enumerate() {
        let pts = xy_of(c);
        assert_eq!((c.k() as usize, c.closed(), c.family(), pts.len()), (i, false, Family::Row, 2));
        let y = 3.0 + 4.0 * i as f64;
        assert!(pts.iter().all(|q| (q.y - y).abs() < 1e-6), "row {i} on y = {y}");
        assert!((pts[0].x - 3.0).abs() < 1e-3 && (pts[1].x - 37.0).abs() < 1e-3, "row {i} spans the centre");
    }
    assert!(all_inside(set, center(&r), TOL));
}

#[test]
fn rows_at_an_angle_with_a_closing_ring_cover_a_disc() {
    let r = pocket(circle(0.0, 0.0, 20.0, 256), vec![], 3.0);
    let v = plan(&r, &rows(30.0, 4.0, true));
    assert!(v.postable(), "{v:?}");
    let set = v.out().unwrap();
    let (dx, dy) = (30f64.to_radians().cos(), 30f64.to_radians().sin());
    for c in set.curves().iter().filter(|c| c.family() == Family::Row) {
        let q = xy_of(c);
        let (ux, uy) = (q[1].x - q[0].x, q[1].y - q[0].y);
        assert!((ux * dy - uy * dx).abs() < 1e-6 * ux.hypot(uy), "each row runs at 30 degrees");
    }
    let closing = marks(set, MarkKind::Closing);
    assert_eq!(closing.len(), 1);
    let ring = &set.curves()[closing[0].curve()];
    assert_eq!((ring.family(), ring.closed(), ring.k()), (Family::Ring, true, 0));
    assert!(all_inside(set, center(&r), TOL));
}

#[test]
fn rows_without_a_closing_ring_name_the_area_that_they_leave() {
    // At a step of the tool diameter the caps of a disc between the row
    // ends stay uncut: the result is incomplete with each gap named.
    let r = pocket(circle(0.0, 0.0, 20.0, 256), vec![], 3.0);
    let v = plan(&r, &rows(0.0, 6.0, false));
    let Verdict::Incomplete { gaps, .. } = &v else { panic!("an incomplete verdict: {v:?}") };
    let total: f64 = gaps.list().iter().map(|g| g.area_mm2()).sum();
    assert!(total > 0.1, "the gaps are above the 0.1 mm2 guard: {total}");
    let closed = plan(&r, &rows(0.0, 6.0, true));
    assert!(closed.postable(), "the closing ring cuts the caps: {closed:?}");
}

#[test]
fn the_same_region_and_choice_give_the_same_rows_from_any_pick_order() {
    let g = rows(15.0, 3.0, true);
    let a = plan(&box_pocket(), &g);
    let mut turned = rect(0.0, 0.0, 40.0, 30.0);
    turned.rotate_left(2);
    turned.reverse();
    let b = plan(&pocket(turned, vec![], 3.0), &g);
    assert!(same_set(a.out().unwrap(), b.out().unwrap(), TOL), "law 1: same rows within 1 um");
    assert_eq!(plan(&box_pocket(), &g), a);
}

#[test]
fn each_row_refusal_is_named() {
    let r = box_pocket();
    let ring = GuideChoice::Rings(RingsChoice::new(RingRef::Boundary, 0.0, 2.0, None, None).unwrap());
    assert_eq!(refused(RowPlanner.guide(&r, &ring, TOL)), NOT_ROWS);
    assert_eq!(refused(RowPlanner.guide(&r, &rows(0.0, 6.5, false), TOL)), STEP_ABOVE_DIAMETER);
    assert_eq!(refused(RowPlanner.guide(&r, &rows(0.0, 1e-4, false), TOL)), TOO_MANY_ROWS);
}

#[test]
fn a_mark_is_checked_against_its_curve() {
    let c = || curve(&[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)], true, 0, Family::Ring);
    let on = |xy: &[(f64, f64)]| xy.iter().map(|&(x, y)| PlanePoint::flat(p(x, y)).unwrap()).collect::<Vec<_>>();
    assert_eq!(Mark::part(0, MarkKind::FullWidth, on(&[(1.0, 0.0)])).unwrap_err().reason(), BAD_STRETCH);
    assert_eq!(Mark::part(0, MarkKind::FullWidth, on(&[(1.0, 0.0), (1.0, 0.0)])).unwrap_err().reason(), BAD_STRETCH);
    let off = Mark::part(0, MarkKind::FullWidth, on(&[(1.0, 0.0), (1.0, 0.5)])).unwrap();
    assert_eq!(GuideSet::with_marks(vec![c()], vec![off]).unwrap_err().reason(), MARK_OFF_CURVE);
    assert_eq!(GuideSet::with_marks(vec![c()], vec![Mark::whole(1, MarkKind::Fill)]).unwrap_err().reason(), MARK_NO_CURVE);
    // A stretch on the closing segment is on the curve; the mark follows
    // its curve into the canonical order.
    let late = curve(&[(0.0, 5.0), (3.0, 5.0)], false, 4, Family::Ring);
    let set = GuideSet::with_marks(vec![late, c()], vec![Mark::part(1, MarkKind::FullWidth, on(&[(8.0, 8.0), (5.0, 5.0)])).unwrap(), Mark::whole(0, MarkKind::Fill)]).unwrap();
    assert_eq!(set.marks().len(), 2);
    assert_eq!((set.marks()[0].curve(), set.marks()[0].kind()), (0, MarkKind::FullWidth));
    assert_eq!((set.marks()[1].curve(), set.marks()[1].kind()), (1, MarkKind::Fill));
    assert_eq!(set.curves()[1].k(), 4);
}
