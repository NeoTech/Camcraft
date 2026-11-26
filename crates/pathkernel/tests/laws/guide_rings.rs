//! The ring guide planner (#4176, K2): the rings are the level sets of the
//! distance to the walls, spaced by the choice's stepover; a full-width
//! stretch is named (#3788); a small piece is kept; each refusal is named.

use super::guide_kit::{around, center, circle, marks, mark_pts, outline, p, pocket, rect, same_set, to_edge, to_set, all_inside};
use camcraft_pathkernel::guide::rings::{
    BAND_NOT_YET, CIRCLE_NOT_YET, NOT_RINGS, NO_RING, OFFSET_BELOW_CENTER, TOO_MANY_RINGS, WALL_STRIP_UNCUT,
};
use camcraft_pathkernel::guide::{
    BAD_TOL, Circle, FAMILY_NOT_YET, Family, GuideChoice, GuidePlanner, GuideSet, Guides, MarkKind, RingPlanner, RingRef,
    RingsChoice, RowsChoice, STEP_ABOVE_DIAMETER, Span,
};
use camcraft_pathkernel::{KernelRegion, Refusal, Verdict};

const TOL: f64 = 1e-3;

fn rings(reference: RingRef, first: f64, step: f64, count: Option<u32>) -> GuideChoice {
    GuideChoice::Rings(RingsChoice::new(reference, first, step, None, count).unwrap())
}

fn plan(r: &KernelRegion, g: &GuideChoice) -> Verdict<GuideSet> {
    RingPlanner.guide(r, g, TOL).unwrap()
}

fn done(v: &Verdict<GuideSet>) -> &GuideSet {
    assert!(v.postable(), "a done verdict: {v:?}");
    v.out().unwrap()
}

fn refused(r: Result<Verdict<GuideSet>, Refusal>) -> &'static str {
    r.unwrap_err().reason()
}

/// A disc pocket of radius 20 with a tool of radius 3: the centre is a
/// disc of radius 17.
fn disc() -> KernelRegion {
    pocket(circle(0.0, 0.0, 20.0, 256), vec![], 3.0)
}

/// Two lobes (radius 15, centres 50 mm apart) joined by a neck 8 mm wide.
/// With a tool of radius 2 the neck of the centre is 4 mm wide: narrower
/// than two steps of 3 mm.
fn waist() -> KernelRegion {
    let shape = outline(&[circle(-25.0, 0.0, 15.0, 192), circle(25.0, 0.0, 15.0, 192), rect(-12.0, -4.0, 12.0, 4.0)]);
    pocket(shape, vec![], 2.0)
}

#[test]
fn the_guide_stage_takes_no_start_side() {
    // Laws 2 and 3 in the type system: a region, a guide choice and a
    // tolerance; no start side, no order, no link.
    type Stage = fn(&RingPlanner, &KernelRegion, &GuideChoice, f64) -> Result<Verdict<GuideSet>, Refusal>;
    let _: Stage = <RingPlanner as GuidePlanner>::guide;
}

#[test]
fn disc_rings_are_level_sets_spaced_by_the_stepover() {
    let r = disc();
    let set = plan(&r, &rings(RingRef::Boundary, 0.0, 4.0, None));
    let set = done(&set);
    // Radii 17, 13, 9, 5, 1: one closed ring each, index 0 to 4.
    assert_eq!(set.curves().len(), 5);
    for (k, c) in set.curves().iter().enumerate() {
        assert_eq!((c.k() as usize, c.closed(), c.family()), (k, true, Family::Ring));
        let worst = c.pts().iter().map(|q| (to_edge(center(&r), q.at()) - 4.0 * k as f64).abs()).fold(0.0, f64::max);
        assert!(worst < 0.01, "ring {k} is 4 mm per index from the wall: off by {worst}");
    }
    assert!(all_inside(set, center(&r), TOL));
    assert!(marks(set, MarkKind::Fill).is_empty(), "a step under the tool diameter covers the disc");
}

#[test]
fn the_same_region_and_choice_give_the_same_curves_from_any_pick_order() {
    let g = rings(RingRef::Boundary, 0.0, 4.0, None);
    let a = plan(&disc(), &g);
    let mut turned = circle(0.0, 0.0, 20.0, 256);
    turned.rotate_left(77);
    turned.reverse();
    let b = plan(&pocket(turned, vec![], 3.0), &g);
    assert!(same_set(a.out().unwrap(), b.out().unwrap(), TOL), "law 1: same curves within 1 um");
    assert_eq!(plan(&disc(), &g), a, "law 1: a pure function");
}

#[test]
fn part_and_boundary_rings_agree_where_there_is_no_stock_edge() {
    let r = waist();
    let a = plan(&r, &rings(RingRef::Boundary, 0.0, 3.0, None));
    let b = plan(&r, &rings(RingRef::Part, 0.0, 3.0, None));
    assert_eq!(a, b);
}

#[test]
fn the_full_width_stretch_through_a_waist_is_named() {
    let r = waist();
    let v = plan(&r, &rings(RingRef::Boundary, 0.0, 3.0, None));
    let set = done(&v);
    // Ring 0 runs through the neck; ring 1 is one loop in each lobe. The
    // two mouths of the neck, which neither ring 0 nor ring 1 reaches, get
    // one clean-up ring each, with the index of ring 1.
    let fills: Vec<usize> = marks(set, MarkKind::Fill).iter().map(|m| m.curve()).collect();
    let regular = |k: u32| set.curves().iter().enumerate().filter(|(i, c)| c.k() == k && !fills.contains(i)).count();
    assert_eq!((regular(0), regular(1)), (1, 2));
    assert_eq!(fills.len(), 2);
    for i in &fills {
        let c = &set.curves()[*i];
        assert_eq!(c.k(), 1);
        assert!(c.pts().iter().all(|q| (q.at().x.abs() - 11.5).abs() < 1.0 && q.at().y.abs() < 2.0), "a clean-up ring in a mouth");
    }
    let wide = marks(set, MarkKind::FullWidth);
    let named: Vec<_> = wide.iter().filter(|m| set.curves()[m.curve()].k() == 0 && matches!(m.span(), Span::Part(_))).collect();
    // Each strand of ring 0 in the neck is one named stretch. It ends where
    // the clean-up rings of the mouths come within the tool diameter.
    assert_eq!(named.len(), 2, "each strand of ring 0 in the neck is one named stretch");
    for m in &named {
        let pts = mark_pts(set, m);
        let (lo, hi) = pts.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), q| (a.min(q.x), b.max(q.x)));
        assert!(lo < -6.0 && hi > 6.0, "the stretch runs along the neck to the mouths: x {lo} to {hi}");
        assert!(pts.iter().all(|q| (q.y.abs() - 2.0).abs() < 0.05), "on a strand of the neck");
        assert!(pts.iter().all(|q| q.x.abs() < 11.0), "the stretch stays in the neck: {:?}", pts.first());
    }
    // Far from the neck, ring 0 has ring 1 within the tool diameter: not named.
    assert!(to_set(set, p(-37.0, 0.0)) < 1.0);
    assert!(!named.iter().flat_map(|m| mark_pts(set, m)).any(|q| q.x < -20.0));
}

#[test]
fn a_step_above_the_tool_radius_gets_clean_up_rings_and_covers_the_area() {
    // Rings at depth 0, 6 and 12 leave the core of radius 5 uncut beyond 3 mm.
    let r = disc();
    let v = plan(&r, &rings(RingRef::Boundary, 0.0, 6.0, None));
    let set = done(&v);
    let fills = marks(set, MarkKind::Fill);
    assert!(!fills.is_empty(), "the core gets a clean-up ring");
    for m in fills {
        let c = &set.curves()[m.curve()];
        assert_eq!((c.k(), c.closed()), (3, true), "the clean-up ring sorts after the deepest ring");
    }
    assert!(all_inside(set, center(&r), TOL));
}

#[test]
fn a_small_piece_of_the_area_is_kept() {
    // A 6.1 mm square with a tool of radius 3 leaves a 0.1 mm square of
    // centre (0.01 mm2): it is a ring, not dropped (the #2232 sliver drop
    // is not ported).
    let r = pocket(rect(0.0, 0.0, 40.0, 30.0), vec![], 3.0);
    let small = pocket(rect(50.0, 0.0, 56.1, 6.1), vec![], 3.0);
    let s = plan(&small, &rings(RingRef::Boundary, 0.0, 2.0, None));
    let s = done(&s);
    assert_eq!(s.curves().len(), 1);
    assert!(s.curves()[0].pts().iter().all(|q| (q.at().x - 53.05).abs() < 0.06 && (q.at().y - 3.05).abs() < 0.06));
    assert!(plan(&r, &rings(RingRef::Boundary, 0.0, 2.0, None)).postable());
}

#[test]
fn a_ring_count_gives_that_many_rings_and_no_clean_up() {
    let v = plan(&disc(), &rings(RingRef::Boundary, 0.0, 6.0, Some(1)));
    let set = done(&v);
    assert_eq!(set.curves().len(), 1);
    assert!(marks(set, MarkKind::Fill).is_empty());
}

#[test]
fn rings_round_a_part_end_at_the_stock_edge() {
    // A 20 mm square part in a 60 by 40 stock, tool radius 4: the rings
    // round the part are cut open where they meet the stock outline.
    let r = around(rect(0.0, 0.0, 20.0, 20.0), rect(-20.0, -10.0, 40.0, 30.0), 4.0);
    let v = plan(&r, &rings(RingRef::Part, 0.0, 2.0, None));
    let set = done(&v);
    assert!(set.curves().iter().any(|c| !c.closed()), "rings meet the stock edge");
    assert!(set.curves().iter().any(|c| c.closed() && c.k() == 0), "the first ring goes round the part");
    assert!(all_inside(set, center(&r), TOL));
    // Within a tool radius of the stock outline the disc is in air: no
    // full-width stretch there.
    let off_air = |q: &camcraft_pathkernel::Point2| q.x > -16.0 - TOL && q.x < 36.0 + TOL && q.y > -6.0 - TOL && q.y < 26.0 + TOL;
    assert!(marks(set, MarkKind::FullWidth).iter().flat_map(|m| mark_pts(set, m)).all(|q| off_air(&q)));
}

#[test]
fn each_ring_refusal_is_named() {
    let r = disc();
    let row = GuideChoice::Rows(RowsChoice::new(0.0, 2.0, false).unwrap());
    assert_eq!(refused(RingPlanner.guide(&r, &row, TOL)), NOT_RINGS);
    let circ = Circle::new(p(0.0, 0.0), 17.0).unwrap();
    let g = GuideChoice::Rings(RingsChoice::new(RingRef::Boundary, 0.0, 2.0, Some(circ), None).unwrap());
    assert_eq!(refused(RingPlanner.guide(&r, &g, TOL)), CIRCLE_NOT_YET);
    assert_eq!(refused(RingPlanner.guide(&r, &rings(RingRef::Band, 0.0, 2.0, None), TOL)), BAND_NOT_YET);
    assert_eq!(refused(RingPlanner.guide(&r, &rings(RingRef::Boundary, -0.5, 2.0, None), TOL)), OFFSET_BELOW_CENTER);
    assert_eq!(refused(RingPlanner.guide(&r, &rings(RingRef::Boundary, 18.0, 2.0, Some(1)), TOL)), NO_RING);
    assert_eq!(refused(RingPlanner.guide(&r, &rings(RingRef::Boundary, 3.5, 2.0, None), TOL)), WALL_STRIP_UNCUT);
    assert_eq!(refused(RingPlanner.guide(&r, &rings(RingRef::Boundary, 0.0, 6.5, None), TOL)), STEP_ABOVE_DIAMETER);
    assert_eq!(refused(RingPlanner.guide(&r, &rings(RingRef::Boundary, 0.0, 1e-3, None), TOL)), TOO_MANY_RINGS);
    for bad in [f64::NAN, 0.0, 1e-9, f64::INFINITY] {
        assert_eq!(refused(RingPlanner.guide(&r, &rings(RingRef::Boundary, 0.0, 2.0, None), bad)), BAD_TOL);
    }
}

#[test]
fn the_guide_stage_sends_each_family_to_its_planner_or_refuses_it_by_name() {
    let r = disc();
    let g = rings(RingRef::Boundary, 0.0, 4.0, None);
    assert_eq!(Guides.guide(&r, &g, TOL).unwrap(), plan(&r, &g));
    assert_eq!(refused(Guides.guide(&r, &GuideChoice::Medial, TOL)), FAMILY_NOT_YET);
}

#[test]
fn a_full_width_stretch_on_an_open_ring_round_a_part_is_named() {
    // A U-shaped part with a bay 6 mm wide and 16 mm deep that opens to the
    // stock edge, tool radius 1, step 1.5. Ring 1 enters the bay (its
    // strands 1 mm apart) and is cut open at the stock edge; ring 2 does not
    // enter the bay. The two strands and their turn are one named stretch.
    let part = vec![p(-20.0, -10.0), p(-3.0, -10.0), p(-3.0, 6.0), p(3.0, 6.0), p(3.0, -10.0), p(20.0, -10.0), p(20.0, 10.0), p(-20.0, 10.0)];
    let r = around(part, rect(-30.0, -12.0, 30.0, 18.0), 1.0);
    let v = plan(&r, &rings(RingRef::Part, 0.0, 1.5, None));
    let set = done(&v);
    let bay: Vec<_> = marks(set, MarkKind::FullWidth)
        .into_iter()
        .filter(|m| !set.curves()[m.curve()].closed() && matches!(m.span(), Span::Part(_)))
        .collect();
    assert_eq!(bay.len(), 1, "one named stretch on an open ring");
    assert_eq!(set.curves()[bay[0].curve()].k(), 1);
    let pts = mark_pts(set, bay[0]);
    assert!(pts.iter().all(|q| q.x.abs() <= 0.5 + TOL && q.y < 4.0), "the stretch is in the bay");
    let (lo, hi) = pts.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), q| (a.min(q.y), b.max(q.y)));
    assert!(lo < -9.0 && hi > 3.0, "the stretch runs the depth of the bay: y {lo} to {hi}");
}
