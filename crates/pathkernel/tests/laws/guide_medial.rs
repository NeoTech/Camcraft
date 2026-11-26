//! The medial guide planner (#4176, second part): the centre line of the
//! cutter-centre area, its curves inside the area, and each refusal named
//! through the public API.

use super::guide_kit::{all_inside, center, circle, outline, pocket, rect, xy_of};
use super::guide_laws::slot_region;
use camcraft_pathkernel::guide::medial::{MEDIAL_TOO_FINE, NARROW_NECK, NOT_MEDIAL, NO_MEDIAL};
use camcraft_pathkernel::guide::{BAD_TOL, Family, GuideChoice, GuidePlanner, GuideSet, MedialPlanner, NOT_AN_AREA, RowsChoice};
use camcraft_pathkernel::{KernelRegion, MaterialSide, Refusal, Region, Selection, ToolShape, Verdict, WorkPlane, chain_region};

const TOL: f64 = 1e-3;

fn medial(r: &KernelRegion) -> Result<Verdict<GuideSet>, Refusal> {
    MedialPlanner.guide(r, &GuideChoice::Medial, TOL)
}

fn done(v: &Verdict<GuideSet>) -> &GuideSet {
    assert!(v.postable(), "a done verdict: {v:?}");
    v.out().unwrap()
}

fn refused(r: Result<Verdict<GuideSet>, Refusal>) -> &'static str {
    r.unwrap_err().reason()
}

#[test]
fn the_centre_line_of_a_slot_is_its_axis() {
    // A slot of half-width 5 round (0, 0)-(40, 0), tool radius 2: the
    // centre area is a slot of half-width 3 round the same segment.
    let r = slot_region(MaterialSide::Inside).unwrap();
    let v = medial(&r).unwrap();
    let set = done(&v);
    assert_eq!(set.curves().len(), 1, "{set:?}");
    let c = &set.curves()[0];
    assert_eq!((c.family(), c.k(), c.closed()), (Family::Medial, 0, false));
    let pts = xy_of(c);
    assert!(pts.iter().all(|q| q.y.abs() <= 2.0 * TOL), "{pts:?}");
    let (x0, x1) = pts.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), q| (a.min(q.x), b.max(q.x)));
    assert!(x0.abs() <= 0.1 && (x1 - 40.0).abs() <= 0.1, "the line runs between the centres of the two ends: {x0} {x1}");
    assert!(all_inside(set, center(&r), TOL));
    // The tool of radius 2 along the line leaves the two strips of the
    // centre area beside it uncut: the note says so.
    let Verdict::Done { notes, .. } = &v else { unreachable!() };
    assert!(notes.iter().any(|n| n.text().contains("does not clear the area")), "{notes:?}");
}

#[test]
fn a_channel_round_an_island_has_a_closed_centre_line() {
    let r = pocket(circle(0.0, 0.0, 20.0, 256), vec![circle(0.0, 0.0, 10.0, 256).into_iter().rev().collect()], 2.0);
    let set = medial(&r).unwrap();
    let set = done(&set);
    let closed: Vec<_> = set.curves().iter().filter(|c| c.closed()).collect();
    assert_eq!(closed.len(), 1, "{set:?}");
    assert!(xy_of(closed[0]).iter().all(|q| (q.x.hypot(q.y) - 15.0).abs() <= 0.01));
}

#[test]
fn an_l_shaped_channel_has_one_line_round_its_corner() {
    // An L of two legs 8 mm wide; tool radius 2: the centre is an L of
    // legs 4 mm wide. Its axis: the two leg centre lines and the branches
    // into the corners of the centre area.
    let shape = outline(&[rect(0.0, 0.0, 40.0, 8.0), rect(0.0, 0.0, 8.0, 40.0)]);
    let r = pocket(shape, vec![], 2.0);
    let set = medial(&r).unwrap();
    let set = done(&set);
    assert!(all_inside(set, center(&r), TOL));
    let on_x = set.curves().iter().flat_map(xy_of).any(|q| (q.y - 4.0).abs() < 2.0 * TOL && q.x > 30.0);
    let on_y = set.curves().iter().flat_map(xy_of).any(|q| (q.x - 4.0).abs() < 2.0 * TOL && q.y > 30.0);
    assert!(on_x && on_y, "{set:?}");
}

#[test]
fn the_same_area_gives_the_same_centre_line() {
    let r = slot_region(MaterialSide::Inside).unwrap();
    assert_eq!(medial(&r).unwrap(), medial(&r).unwrap());
}

#[test]
fn each_refusal_of_the_medial_planner_is_named() {
    let slot = slot_region(MaterialSide::Inside).unwrap();
    let rows = GuideChoice::Rows(RowsChoice::new(0.0, 2.0, false).unwrap());
    assert_eq!(refused(MedialPlanner.guide(&slot, &rows, TOL)), NOT_MEDIAL);
    for bad in [f64::NAN, 0.0, 1e-9] {
        assert_eq!(refused(MedialPlanner.guide(&slot, &GuideChoice::Medial, bad)), BAD_TOL);
    }
    // A round area has a centre point, not a line.
    let disc = pocket(circle(0.0, 0.0, 10.0, 256), vec![], 2.0);
    assert_eq!(refused(medial(&disc)), NO_MEDIAL);
    // Two squares joined by a neck that the tool of radius 2 leaves only
    // 0.8 um wide: the centre line runs through a point where the area
    // touches itself.
    let neck = outline(&[rect(0.0, 0.0, 10.0, 10.0), rect(10.0, 3.0, 14.0, 7.0008), rect(14.0, 0.0, 24.0, 10.0)]);
    let pinch = pocket(neck, vec![], 2.0);
    assert_eq!(refused(medial(&pinch)), NARROW_NECK);
    // Too fine: a large area at the kernel tolerance.
    let big = pocket(circle(0.0, 0.0, 60.0, 64), vec![], 2.0);
    assert_eq!(refused(MedialPlanner.guide(&big, &GuideChoice::Medial, 1e-6)), MEDIAL_TOO_FINE);
    // The chains of a trace have no area.
    let sel = Selection::points(vec![camcraft_pathkernel::Point2::new(0.0, 0.0), camcraft_pathkernel::Point2::new(10.0, 0.0)], false).unwrap();
    let chains = chain_region(&sel, WorkPlane::setup_xy(0.0).unwrap(), ToolShape::disc(1.0, 0.0, true, 10.0).unwrap(), Region::empty()).unwrap();
    assert_eq!(refused(medial(&chains)), NOT_AN_AREA);
}
