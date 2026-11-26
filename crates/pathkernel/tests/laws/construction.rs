//! Finding 3 of the #4123 review, the whole class: no public type of the
//! kernel can hold an invalid value. Each constructor refuses by name.

use super::common::{square, square_region, tool};
use camcraft_pathkernel::entry::{BAD_EXIT, Exit};
use camcraft_pathkernel::frame::{BAD_PLANE, BAD_POINT, BAD_SPOT, BAD_TOOL, NOT_POSITIVE};
use camcraft_pathkernel::guide::{BAD_OFFSET, Circle, GuideChoice, RingRef, RingsChoice, RowsChoice, TraceOffset, ZERO_COUNT};
use camcraft_pathkernel::link::{BAD_STAY, Hop, LinkPolicy, LinkTally, USER_RETRACT};
use camcraft_pathkernel::load::{BAD_LOAD, BAD_TROCHOID, ConstantLoad, LoadControl, Reach, Trochoid};
use camcraft_pathkernel::order::OrderChoice;
use camcraft_pathkernel::recipe::RINGS_REACH_NEEDS_RINGS;
use camcraft_pathkernel::region::{BAD_BOUNDS, Bounds, EMPTY_CENTER, FLAT_EDGE, KernelRegion, MaterialSide, NO_EDGE, PlanarRegion};
use camcraft_pathkernel::verdict::{BAD_GAP, FALLBACK_SAME, INCOMPLETE_NO_GAP, NO_REASON};
use camcraft_pathkernel::{
    AxisControl, Fallback, Gap, Gaps, Loop, Note, PlanePoint, Point2, Positive, Recipe, Refusal, Region, Spot, Stage, ToolShape,
    Verdict, WorkPlane,
};
use camcraft_pathkernel::entry::{EntryChoice, EntryKind};

const NAN: f64 = f64::NAN;
const INF: f64 = f64::INFINITY;

#[test]
fn a_work_plane_needs_a_finite_origin_and_unit_axes_at_right_angles() {
    assert!(WorkPlane::setup_xy(5.0).is_ok());
    for (o, n, x) in [
        ([NAN, 0.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]),
        ([0.0; 3], [0.0; 3], [1.0, 0.0, 0.0]),
        ([0.0; 3], [0.0, 0.0, 2.0], [1.0, 0.0, 0.0]),
        ([0.0; 3], [0.0, 0.0, 1.0], [0.0, 0.6, 0.8]),
        ([0.0; 3], [0.0, 0.0, INF], [1.0, 0.0, 0.0]),
    ] {
        assert_eq!(WorkPlane::planar(o, n, x).unwrap_err().reason(), BAD_PLANE, "{o:?} {n:?} {x:?}");
    }
    let s = std::f64::consts::FRAC_1_SQRT_2;
    assert_eq!(WorkPlane::planar([0.0; 3], [0.0, s, s], [1.0, 0.0, 0.0]).unwrap().normal(), [0.0, s, s]);
}

#[test]
fn a_tool_disc_needs_a_radius_a_corner_inside_it_and_a_flute() {
    assert!(ToolShape::disc(4.0, 4.0, false, 20.0).is_ok());
    for (r, c, f) in [(0.0, 0.0, 20.0), (4.0, 4.5, 20.0), (4.0, -0.1, 20.0), (4.0, 0.0, 0.0), (INF, 0.0, 20.0), (NAN, 0.0, 1.0)] {
        assert_eq!(ToolShape::disc(r, c, true, f).unwrap_err().reason(), BAD_TOOL, "{r} {c} {f}");
    }
    let t = ToolShape::disc(3.0, 0.5, false, 12.0).unwrap();
    assert_eq!((t.radius(), t.can_plunge()), (3.0, false));
}

#[test]
fn spots_scalars_and_points_are_finite() {
    assert_eq!(Spot::new(Stage::Link, Point2::new(NAN, 0.0)).unwrap_err().reason(), BAD_SPOT);
    for v in [0.0, -1.0, NAN, INF] {
        assert_eq!(Positive::new(Stage::Load, v).unwrap_err().reason(), NOT_POSITIVE, "{v}");
    }
    assert_eq!(PlanePoint::new(Point2::new(0.0, 0.0), NAN).unwrap_err().reason(), BAD_POINT);
    assert_eq!(PlanePoint::flat(Point2::new(INF, 0.0)).unwrap_err().reason(), BAD_POINT);
    assert_eq!(AxisControl::default(), AxisControl::Normal);
}

#[test]
fn verdict_records_name_their_reason() {
    let p = Point2::new(1.0, 2.0);
    assert_eq!(Fallback::new(Stage::Order, "inside", "inside", "x").unwrap_err().reason(), FALLBACK_SAME);
    assert_eq!(Fallback::new(Stage::Order, "inside", "zigzag", " ").unwrap_err().reason(), NO_REASON);
    assert_eq!(Fallback::new(Stage::Order, "", "zigzag", "r").unwrap_err().reason(), NO_REASON);
    for (a, at) in [(0.0, p), (-1.0, p), (NAN, p), (1.0, Point2::new(INF, 0.0))] {
        assert_eq!(Gap::new(a, at).unwrap_err().reason(), BAD_GAP);
    }
    assert_eq!(Gaps::new(Stage::Guide, vec![]).unwrap_err().reason(), INCOMPLETE_NO_GAP);
    assert_eq!(Verdict::incomplete(Stage::Guide, 1, vec![]).unwrap_err().reason(), INCOMPLETE_NO_GAP);
    assert_eq!(Note::new(Stage::Link, "  ").unwrap_err().reason(), NO_REASON);
    // A refusal is never silent about why, and never carries a place that is not finite.
    let r = Refusal::new(Stage::Region, " ").at(Point2::new(NAN, 0.0)).fix(" ");
    assert_eq!((r.reason(), r.place(), r.fix_text()), (NO_REASON, None, None));
    assert_eq!(Refusal::new(Stage::Region, "empty").fix("select a loop").to_string(), "Region: empty (select a loop)");
}

#[test]
fn an_incomplete_verdict_keeps_a_fallback_named_and_is_not_postable() {
    let gap = Gap::new(0.2, Point2::new(0.0, 0.0)).unwrap();
    let fb = Fallback::new(Stage::Entry, "helix", "ramp", "no room").unwrap();
    let v = Verdict::incomplete(Stage::Guide, 1, vec![gap]).unwrap().with_fallback(fb.clone());
    assert!(!v.postable());
    let Verdict::Incomplete { notes, gaps, .. } = v else { panic!("still incomplete") };
    assert!(notes[0].text().contains("ramp instead of helix"));
    assert_eq!(gaps.list().len(), 1);
    let Verdict::Done { fallbacks, .. } = Verdict::done(()).with_fallback(fb) else { panic!("done") };
    assert_eq!(fallbacks.len(), 1);
}

#[test]
fn bounds_and_regions_are_checked() {
    assert!(Bounds::new(None, Region::empty(), 0.0, -5.0).is_ok());
    assert_eq!(Bounds::new(None, Region::empty(), -5.0, 0.0).unwrap_err().reason(), BAD_BOUNDS);
    assert_eq!(Bounds::new(None, Region::empty(), NAN, 0.0).unwrap_err().reason(), BAD_BOUNDS);
    let mk = |c: Region, walls: Vec<Loop>, edge: Vec<Loop>| {
        PlanarRegion::new(c, MaterialSide::Inside, walls, edge, Region::empty(), Region::empty(), tool(2.0, true))
    };
    assert_eq!(mk(Region::empty(), vec![square(0.0, 0.0, 10.0)], vec![]).unwrap_err().reason(), EMPTY_CENTER);
    assert_eq!(mk(square_region(0.0, 0.0, 10.0), vec![], vec![]).unwrap_err().reason(), NO_EDGE);
    let line = Loop::new(vec![Point2::new(0.0, 0.0), Point2::new(1.0, 0.0), Point2::new(2.0, 0.0)]);
    if let Ok(line) = line {
        assert_eq!(mk(square_region(0.0, 0.0, 10.0), vec![line], vec![]).unwrap_err().reason(), FLAT_EDGE);
    }
    let r = mk(square_region(0.0, 0.0, 10.0), vec![], vec![square(-1.0, -1.0, 12.0)]).unwrap();
    let k = KernelRegion::Planar(r);
    let p = k.planar().unwrap();
    assert_eq!((p.stock_edge().len(), p.tool().radius()), (1, 2.0));
    assert!((p.center().area() - 100.0).abs() < 1e-9);
}

#[test]
fn guide_and_load_choices_are_checked() {
    assert!(RingsChoice::new(RingRef::Part, 0.0, 1.0, None, None).is_ok());
    assert_eq!(RingsChoice::new(RingRef::Part, 0.0, 0.0, None, None).unwrap_err().reason(), NOT_POSITIVE);
    assert_eq!(RingsChoice::new(RingRef::Part, NAN, 1.0, None, None).unwrap_err().reason(), BAD_OFFSET);
    assert_eq!(RingsChoice::new(RingRef::Part, 0.0, 1.0, None, Some(0)).unwrap_err().reason(), ZERO_COUNT);
    assert_eq!(RowsChoice::new(NAN, 1.0, false).unwrap_err().reason(), BAD_OFFSET);
    assert_eq!(RowsChoice::new(0.0, -1.0, false).unwrap_err().reason(), NOT_POSITIVE);
    assert_eq!(TraceOffset::new(INF).unwrap_err().reason(), BAD_OFFSET);
    assert!(Circle::new(Point2::new(0.0, 0.0), 0.0).is_err());
    assert!(Circle::new(Point2::new(NAN, 0.0), 1.0).is_err());
    assert!(Trochoid::new(6.0, 1.0).is_ok());
    for (w, p) in [(1.0, 1.0), (0.0, 0.0), (NAN, 1.0), (6.0, -1.0)] {
        assert_eq!(Trochoid::new(w, p).unwrap_err().reason(), BAD_TROCHOID, "{w} {p}");
    }
    assert!(ConstantLoad::new(0.3, Reach::Stock).is_ok());
    for v in [0.0, -0.1, 1.5, NAN] {
        assert_eq!(LoadControl::constant(v, Reach::Rings).unwrap_err().reason(), BAD_LOAD, "{v}");
    }
}

#[test]
fn a_recipe_refuses_choices_that_cannot_go_together() {
    let make = |guide: GuideChoice, load: LoadControl| {
        let plane = WorkPlane::setup_xy(0.0).unwrap();
        let entry = EntryChoice::new(EntryKind::Helix, 2.0, vec![]).unwrap();
        Recipe::new(plane, AxisControl::Normal, guide, load, OrderChoice::default(), LinkPolicy::default(), entry)
    };
    let rings = GuideChoice::Rings(RingsChoice::new(RingRef::Boundary, 0.0, 2.0, None, None).unwrap());
    let rows = GuideChoice::Rows(RowsChoice::new(0.0, 2.0, true).unwrap());
    let on_rings = LoadControl::constant(0.4, Reach::Rings).unwrap();
    let r = make(rings, on_rings).unwrap();
    assert_eq!((r.axis(), r.link()), (AxisControl::Normal, LinkPolicy::StayWhereProved));
    let e = make(rows, on_rings).unwrap_err();
    assert_eq!((e.stage(), e.reason()), (Stage::Recipe, RINGS_REACH_NEEDS_RINGS));
    assert!(make(rows, LoadControl::constant(0.4, Reach::Stock).unwrap()).is_ok());
}

#[test]
fn hops_and_exits_are_checked_and_lifts_counted() {
    assert_eq!(Hop::stay(vec![]).unwrap_err().reason(), BAD_STAY);
    assert_eq!(Hop::stay(vec![Point2::new(NAN, 0.0)]).unwrap_err().reason(), BAD_SPOT);
    assert_eq!(Hop::lift(" ").unwrap_err().reason(), NO_REASON);
    let mut t = LinkTally::default();
    t.record(Hop::lift(USER_RETRACT).unwrap());
    t.record(Hop::lift("crosses a planned cut").unwrap());
    t.record(Hop::lift(USER_RETRACT).unwrap());
    t.record(Hop::stay(vec![Point2::new(0.0, 0.0)]).unwrap());
    assert_eq!((t.lifts(), t.by_reason()[USER_RETRACT]), (3, 2));
    assert!(Exit::new(None, 5.0).is_ok());
    assert_eq!(Exit::new(None, -1.0).unwrap_err().reason(), BAD_EXIT);
    assert_eq!(Exit::new(None, NAN).unwrap_err().reason(), BAD_EXIT);
    assert_eq!(Exit::new(Some(vec![]), 1.0).unwrap_err().reason(), BAD_EXIT);
}
