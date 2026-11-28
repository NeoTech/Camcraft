//! #4134: the control's wire offset spans over a wire-centre path. The
//! programmed contour of each pass is the part profile, the side follows
//! the stated die / punch and the walk, each pass states its own radius,
//! and what the control cannot engage on is refused by name.
//!
//! The wire and gap below are SYNTHETIC - a plausible size, from no
//! machine's manual.

use super::super::{cut_pass, wire_passes, WirePass};
use super::*;
use crate::geo2d::Loop;

const K: f64 = 0.165;

fn params(side: WireSide, start: (f64, f64)) -> super::super::WireParams {
    super::super::WireParams {
        wire_diameter: 0.25,
        gap: 0.04,
        condition: "C001".into(),
        side,
        start: Some(Point2::new(start.0, start.1)),
        plane_z: 0.0,
        tabs: None,
        stock: None,
        clearance: 1.0,
        sharp_corners: true,
        overlap: 0.0,
    }
}

fn square(h: f64) -> Loop {
    Loop::new(vec![Point2::new(-h, -h), Point2::new(h, -h), Point2::new(h, h), Point2::new(-h, h)]).expect("square")
}

/// The distance from `p` to the boundary of the square of half side `h`.
fn off_square(p: Point3, h: f64) -> f64 {
    let (x, y) = (p.x.abs(), p.y.abs());
    if x <= h && y <= h {
        (h - x).min(h - y)
    } else {
        (x - h).max(0.0).hypot((y - h).max(0.0))
    }
}

fn ring_of(span: &CompSpan) -> &[Move] {
    &span.programmed[1..span.programmed.len() - 1]
}

#[test]
fn a_die_square_is_programmed_on_its_profile_under_g41() {
    let mut path = cut_pass(&[square(5.0)], &[], &params(WireSide::Die, (0.0, 0.0)), false).expect("the die cuts");
    let centre = path.moves.clone();
    wire_comp_spans(&mut path, WireSide::Die, Compensation::Control, &[K]).expect("the offset is programmed");
    assert_eq!(path.moves, centre, "the wire-centre moves stay the path's truth");
    let [span] = path.comps.as_slice() else { panic!("one span: {:?}", path.comps) };
    assert_eq!(span.side, CompSide::Left, "a die walked counter-clockwise has the wire on the left (G41)");
    assert!(!span.wear && (span.radius - K).abs() < 1e-12);
    assert_eq!((span.first, span.last), (0, path.moves.len() - 1));
    for m in ring_of(span) {
        assert!(off_square(m.from, 5.0) < 1e-9 && off_square(m.to, 5.0) < 1e-9, "on the profile: {m:?}");
    }
    // Every corner of the opening is programmed.
    for (x, y) in [(5.0, 5.0), (-5.0, 5.0), (-5.0, -5.0), (5.0, -5.0)] {
        assert!(ring_of(span).iter().any(|m| (m.to.x - x).abs() < 1e-9 && (m.to.y - y).abs() < 1e-9), "corner {x} {y}");
    }
    let lead_in = span.programmed[0];
    assert_eq!(lead_in.from, centre[0].from);
    assert!((lead_in.to.x - 5.0).abs() < 1e-9 && lead_in.to.y.abs() < 1e-9, "{lead_in:?}");
}

#[test]
fn a_punch_square_is_programmed_on_its_profile_under_g42() {
    let mut path = cut_pass(&[square(5.0)], &[], &params(WireSide::Punch, (-12.0, 0.0)), false).expect("the punch cuts");
    wire_comp_spans(&mut path, WireSide::Punch, Compensation::Control, &[K]).expect("programmed");
    let [span] = path.comps.as_slice() else { panic!("one span") };
    assert_eq!(span.side, CompSide::Right, "a punch walked counter-clockwise has the wire on the right (G42)");
    for m in ring_of(span) {
        assert!(off_square(m.to, 5.0) < 1e-9, "on the profile: {m:?}");
    }
}

#[test]
fn a_reversed_skim_takes_the_other_side() {
    let mut path = cut_pass(&[square(5.0)], &[], &params(WireSide::Die, (0.0, 0.0)), true).expect("cuts");
    wire_comp_spans(&mut path, WireSide::Die, Compensation::Control, &[K]).expect("programmed");
    assert_eq!(path.comps[0].side, CompSide::Right, "a clockwise die pass has the wire on the right (G42)");
}

#[test]
fn each_pass_states_its_own_radius() {
    let rough = params(WireSide::Die, (0.0, 0.0));
    let skim = WirePass { condition: "C002".into(), wire_diameter: 0.25, gap: 0.01, reverse: false };
    let mut path = wire_passes(&[square(5.0)], &[], &rough, &[skim]).expect("two passes");
    let mut one = path.clone();
    wire_comp_spans(&mut path, WireSide::Die, Compensation::Control, &[K, 0.135]).expect("programmed");
    let radii: Vec<f64> = path.comps.iter().map(|c| c.radius).collect();
    assert_eq!(radii, [K, 0.135]);
    for c in &path.comps {
        for m in ring_of(c) {
            assert!(off_square(m.to, 5.0) < 1e-9, "each pass programs the same profile: {m:?}");
        }
    }
    let err = wire_comp_spans(&mut one, WireSide::Die, Compensation::Control, &[K]).expect_err("one radius, two passes");
    assert!(err.to_string().contains("compensation") && err.to_string().contains("after the last"), "{err}");
}

#[test]
fn the_wear_span_programs_the_wire_centre_moves() {
    let mut path = cut_pass(&[square(5.0)], &[], &params(WireSide::Die, (0.0, 0.0)), false).expect("cuts");
    wire_comp_spans(&mut path, WireSide::Die, Compensation::Wear, &[K]).expect("programmed");
    let span = &path.comps[0];
    assert!(span.wear && span.programmed == path.moves && span.side == CompSide::Left);
}

#[test]
fn computer_compensation_records_nothing() {
    let mut path = cut_pass(&[square(5.0)], &[], &params(WireSide::Die, (0.0, 0.0)), false).expect("cuts");
    let before = path.clone();
    wire_comp_spans(&mut path, WireSide::Die, Compensation::Computer, &[K]).expect("nothing to do");
    assert_eq!(path, before);
}

fn feed(a: (f64, f64), b: (f64, f64)) -> Move {
    Move::new(MoveKind::Feed, Point3::new(a.0, a.1, 0.0), Point3::new(b.0, b.1, 0.0), None)
}

#[test]
fn a_circle_is_programmed_as_arcs_at_the_part_radius() {
    let r = 2.335;
    let c = Point2::new(0.0, 0.0);
    let moves = vec![
        feed((0.0, 0.0), (r, 0.0)),
        feed((r, 0.0), (-r, 0.0)).with_arc(c, true),
        feed((-r, 0.0), (r, 0.0)).with_arc(c, true),
        feed((r, 0.0), (0.0, 0.0)),
    ];
    let mut path = Toolpath { moves, ..Toolpath::default() };
    wire_comp_spans(&mut path, WireSide::Die, Compensation::Control, &[K]).expect("programmed");
    let span = &path.comps[0];
    assert_eq!(span.side, CompSide::Left);
    for m in ring_of(span) {
        let a = m.arc.expect("an arc stays an arc");
        assert!(a.ccw && (dist(xy(m.to), a.center) - (r + K)).abs() < 1e-9, "{m:?}");
    }
}

#[test]
fn a_lead_in_shorter_than_the_offset_is_refused_by_name() {
    // The lead-in arrives along the first cut: the part contour's start
    // stands only 0.115 mm from the start hole.
    let moves = vec![
        feed((0.0, -0.05), (0.0, 0.0)),
        feed((0.0, 0.0), (10.0, 0.0)),
        feed((10.0, 0.0), (10.0, 10.0)),
        feed((10.0, 10.0), (0.0, 10.0)),
        feed((0.0, 10.0), (0.0, 0.0)),
        feed((0.0, 0.0), (0.0, -0.05)),
    ];
    let mut path = Toolpath { moves, ..Toolpath::default() };
    let err = wire_comp_spans(&mut path, WireSide::Die, Compensation::Control, &[K]).expect_err("too short");
    let said = err.to_string();
    assert!(said.contains("control compensation") && said.contains("lead-in at least the offset radius"), "{said}");
}

#[test]
fn an_arc_lead_and_an_open_run_are_refused_by_name() {
    let moves = vec![
        feed((0.0, 0.0), (2.0, 0.0)).with_arc(Point2::new(1.0, 0.0), true),
        feed((2.0, 0.0), (2.0, 2.0)),
        feed((2.0, 2.0), (0.0, 0.0)),
    ];
    let mut path = Toolpath { moves, ..Toolpath::default() };
    let err = wire_comp_spans(&mut path.clone(), WireSide::Die, Compensation::Control, &[K]).expect_err("arc lead");
    assert!(err.to_string().contains("straight lead-in and lead-out"), "{err}");
    let err = wire_comp_spans(&mut path, WireSide::Left, Compensation::Wear, &[K]).expect_err("open run");
    assert!(err.to_string().contains("wear compensation is stated on an open run"), "{err}");
}
