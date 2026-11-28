//! The exact circle rings and the helix guide (#4176, second part): the
//! rings of a hole as exact circles with a chord count from the tolerance
//! (#3756), the helix of a thread on its circle at its pitch and hand, and
//! each refusal named through the public API.

use super::guide_kit::{marks, p, pocket, rect, xy_of};
use super::guide_laws::hole_region;
use camcraft_pathkernel::guide::helix::{CIRCLE_NOT_REGION, HELIX_OUTSIDE, NOT_HELIX};
use camcraft_pathkernel::guide::rings::{NO_RING, WALL_STRIP_UNCUT};
use camcraft_pathkernel::guide::{
    Circle, Curve, Family, GuideChoice, GuidePlanner, GuideSet, Guides, HELIX_NEEDS_LEAD, Hand, HelixChoice, HelixPlanner, Lead,
    MarkKind, NOT_ON_HELIX, RingPlanner, RingRef, RingsChoice, TOO_MANY_TURNS,
};
use camcraft_pathkernel::{MaterialSide, PlanePoint, Point2, Refusal, Verdict};

const TOL: f64 = 1e-3;

fn hole_rings(first: f64, step: f64, count: Option<u32>) -> GuideChoice {
    let c = Circle::new(p(5.0, 5.0), 12.0).unwrap();
    GuideChoice::Rings(RingsChoice::new(RingRef::Boundary, first, step, Some(c), count).unwrap())
}

fn done(v: &Verdict<GuideSet>) -> &GuideSet {
    assert!(v.postable(), "a done verdict: {v:?}");
    v.out().unwrap()
}

fn refused(r: Result<Verdict<GuideSet>, Refusal>) -> &'static str {
    r.unwrap_err().reason()
}

fn radii(set: &GuideSet) -> Vec<f64> {
    let mut v: Vec<f64> = set.curves().iter().map(|c| c.arc().unwrap().radius()).collect();
    v.sort_by(|a, b| b.total_cmp(a));
    v
}

#[test]
fn the_rings_of_a_hole_are_exact_circles_at_the_stepover() {
    // Centre radius 12 (hole 15, tool 3), step 3: rings 12, 9, 6, 3; the
    // last ring's disc reaches the centre.
    let r = hole_region(MaterialSide::Inside).unwrap();
    let v = RingPlanner.guide(&r, &hole_rings(0.0, 3.0, None), TOL).unwrap();
    let set = done(&v);
    assert_eq!(radii(set), vec![12.0, 9.0, 6.0, 3.0]);
    for c in set.curves() {
        let arc = c.arc().unwrap();
        assert_eq!((c.family(), c.closed()), (Family::Ring, true));
        assert!((arc.center().x - 5.0).abs() < 1e-12 && (arc.center().y - 5.0).abs() < 1e-12);
        // The carrier polygon: every vertex on the circle, the sagitta within the tolerance.
        let n = c.pts().len() as f64;
        assert!(xy_of(c).iter().all(|q| ((q.x - 5.0).hypot(q.y - 5.0) - arc.radius()).abs() < 1e-9));
        assert!(arc.radius() * (1.0 - (std::f64::consts::PI / n).cos()) <= TOL, "{n} chords for {}", arc.radius());
        assert!(arc.radius() * (1.0 - (std::f64::consts::PI / (n - 1.0)).cos()) > TOL || n == 8.0, "the least chord count");
    }
    let ks: Vec<u32> = set.curves().iter().map(Curve::k).collect();
    assert_eq!(ks, vec![0, 1, 2, 3]);
}

#[test]
fn a_last_ring_far_from_the_centre_gets_a_named_clean_up_circle() {
    // Step 6: rings 12 and 6. The disc (radius 3) of ring 6 leaves the
    // centre uncut, so a clean-up circle of radius 3 is added and marked.
    let r = hole_region(MaterialSide::Inside).unwrap();
    let v = RingPlanner.guide(&r, &hole_rings(0.0, 6.0, None), TOL).unwrap();
    let set = done(&v);
    assert_eq!(radii(set), vec![12.0, 6.0, 3.0]);
    let fill = marks(set, MarkKind::Fill);
    assert_eq!(fill.len(), 1);
    assert_eq!(set.curves()[fill[0].curve()].arc().unwrap().radius(), 3.0);
}

#[test]
fn a_ring_count_gives_that_many_circles_and_names_the_uncut_area() {
    // Bore with one ring: the wall ring only; the centre stays uncut and
    // the note says so.
    let r = hole_region(MaterialSide::Inside).unwrap();
    let v = RingPlanner.guide(&r, &hole_rings(0.0, 3.0, Some(1)), TOL).unwrap();
    assert_eq!(radii(done(&v)), vec![12.0]);
    let Verdict::Done { notes, .. } = &v else { unreachable!() };
    assert!(notes.iter().any(|n| n.text().contains("as the choice asks")), "{notes:?}");
}

#[test]
fn each_refusal_of_the_circle_rings_is_named() {
    let r = hole_region(MaterialSide::Inside).unwrap();
    assert_eq!(refused(RingPlanner.guide(&r, &hole_rings(3.5, 3.0, None), TOL)), WALL_STRIP_UNCUT);
    assert_eq!(refused(RingPlanner.guide(&r, &hole_rings(12.0, 3.0, Some(1)), TOL)), NO_RING);
    // A circle that is not the circle of the area, and an area that is not a disc.
    let off = GuideChoice::Rings(RingsChoice::new(RingRef::Boundary, 0.0, 3.0, Some(Circle::new(p(5.0, 5.0), 11.0).unwrap()), None).unwrap());
    assert_eq!(refused(RingPlanner.guide(&r, &off, TOL)), CIRCLE_NOT_REGION);
    let square = pocket(rect(-10.0, -10.0, 10.0, 10.0), vec![], 3.0);
    assert_eq!(refused(RingPlanner.guide(&square, &hole_rings(0.0, 3.0, None), TOL)), CIRCLE_NOT_REGION);
}

fn thread(r: f64, hand: Hand) -> GuideChoice {
    GuideChoice::Helix(HelixChoice::new(Circle::new(p(5.0, 5.0), r).unwrap(), 1.5, 6.0, hand).unwrap())
}

#[test]
fn a_helix_turns_on_its_circle_at_its_pitch() {
    let r = hole_region(MaterialSide::Inside).unwrap();
    for hand in [Hand::Right, Hand::Left] {
        let v = HelixPlanner.guide(&r, &thread(10.0, hand), TOL).unwrap();
        let set = done(&v);
        assert_eq!(set.curves().len(), 1);
        let c = &set.curves()[0];
        assert_eq!((c.family(), c.closed(), c.k()), (Family::Helix, false, 0));
        let lead = c.lead().unwrap();
        assert_eq!((lead.pitch(), lead.hand()), (1.5, hand));
        assert_eq!(c.arc().unwrap().radius(), 10.0);
        let depths: Vec<f64> = c.pts().iter().map(PlanePoint::depth).collect();
        let (lo, hi) = depths.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), d| (a.min(*d), b.max(*d)));
        assert!((lo + 6.0).abs() < 1e-12 && hi.abs() < 1e-12, "from the level down by the drop: {lo} {hi}");
        // Four turns: the ends are on the first axis of the plane.
        let ends: Vec<Point2> = [c.pts()[0], c.pts()[c.pts().len() - 1]].iter().map(|q| q.at()).collect();
        assert!(ends.iter().all(|q| (q.x - 15.0).abs() < 1e-9 && (q.y - 5.0).abs() < 1e-9), "{ends:?}");
    }
}

#[test]
fn each_refusal_of_the_helix_is_named() {
    let r = hole_region(MaterialSide::Inside).unwrap();
    assert_eq!(refused(HelixPlanner.guide(&r, &thread(12.5, Hand::Right), TOL)), HELIX_OUTSIDE);
    assert_eq!(refused(HelixPlanner.guide(&r, &hole_rings(0.0, 3.0, None), TOL)), NOT_HELIX);
    let c = Circle::new(p(0.0, 0.0), 5.0).unwrap();
    assert_eq!(HelixChoice::new(c, 1e-4, 10.0, Hand::Right).unwrap_err().reason(), TOO_MANY_TURNS);
    assert!(HelixChoice::new(c, 0.0, 10.0, Hand::Right).is_err());
    assert!(HelixChoice::new(c, 1.0, f64::NAN, Hand::Right).is_err());
    // A helix curve is valid by construction.
    let flat = |x: f64, y: f64, d: f64| PlanePoint::new(p(x, y), d).unwrap();
    let pts = vec![flat(5.0, 0.0, 0.0), flat(0.0, 5.0, 0.0)];
    assert_eq!(Curve::new(pts.clone(), Some(c), false, 0, Family::Helix).unwrap_err().reason(), HELIX_NEEDS_LEAD);
    let lead = Lead::new(2.0, Hand::Right).unwrap();
    assert_eq!(Curve::helix(c, lead, pts, 0).unwrap_err().reason(), NOT_ON_HELIX);
    let good = vec![flat(5.0, 0.0, 0.0), flat(0.0, 5.0, 0.5)];
    assert!(Curve::helix(c, lead, good.clone(), 0).is_ok());
    assert_eq!(Curve::helix(c, Lead::new(2.0, Hand::Left).unwrap(), good, 0).unwrap_err().reason(), NOT_ON_HELIX);
    let off = vec![flat(5.1, 0.0, 0.0), flat(0.0, 5.0, 0.5)];
    assert_eq!(Curve::helix(c, lead, off, 0).unwrap_err().reason(), NOT_ON_HELIX);
}

#[test]
fn the_guide_stage_sends_a_helix_to_the_helix_planner() {
    let r = hole_region(MaterialSide::Inside).unwrap();
    let g = thread(10.0, Hand::Right);
    assert_eq!(Guides.guide(&r, &g, TOL).unwrap(), HelixPlanner.guide(&r, &g, TOL).unwrap());
    let g = hole_rings(0.0, 3.0, None);
    assert_eq!(Guides.guide(&r, &g, TOL).unwrap(), RingPlanner.guide(&r, &g, TOL).unwrap());
}
