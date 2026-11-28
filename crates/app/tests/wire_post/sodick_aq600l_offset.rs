//! #4134: the Wire Contour's control-side wire offset, posted with the
//! Sodick LN2W post on the AQ600L. The square (rough and two skims) and
//! the 2-axis block post with the offset on as goldens: the part contour
//! under `G41` / `G42 Hnnn` for each pass, `G40` on each lead-out. The
//! wire laws still judge the wire-centre path, the wear mode posts the
//! wire-centre path under the offset, and what the offset cannot engage
//! on is refused at the command door by name.
//! Provenance: our own tests over our own test parts; the owner's rules
//! on #4076 (H from H001 by pass, C001 with H001, G40 on the lead-out).
//! Gate map (#4134): covers app::operation (wire_comp, generate),
//! app::commands (wire_gate), app::posting, sodick-ln2w.cps.

use super::*;
use camcraft_app::links::Compensation;

fn offset(mut op: WireContourOp, mode: Compensation) -> WireContourOp {
    op.compensation = mode;
    op
}

fn square_with(mode: Compensation) -> World {
    let op = offset(wire(WireSide::Die, (0.0, 0.0), 10.0, 0.0, &["C002", "C903"], true), mode);
    cut("edm-test-square.step", (40.0, 40.0), 10.0, within((-5.0, 5.0), (-5.0, 5.0)), op)
}

fn two_axis_with(mode: Compensation) -> World {
    let op = offset(wire(WireSide::Die, (-13.0, -3.0), 20.0, 0.0, &[], true), mode);
    cut("edm-test-2axis.step", (60.0, 40.0), 20.0, within((-24.0, -4.0), (-6.0, 0.0)), op)
}

/// Done means: the square posts with the control's offset - the part
/// contour (the 10 mm opening itself, not the wire centre at 4.835),
/// `G41 H001` after `C001`, `H002` and `H003` for the skims with their own
/// C words, and `G40` on each pass's lead-out.
#[test]
fn the_ln2w_square_posts_the_part_contour_with_the_controls_offset() {
    let world = square_with(Compensation::Control);
    let clause = posting::wire_clause(&world);
    assert!(clause.problems.is_empty(), "the wire laws judge the wire-centre path: {:?}", clause.problems);
    let text = post(&world, LN2W, "square-offset").expect("the square posts with the offset");
    golden("edm_test_square_offset.nc", &text);
    let w = words(&text);
    assert_eq!(&w[2..10], ["G00 X0. Y0. U0. V0.", "Z0.", "T91", "T84", "T94", "C001", "G41 H001", "G01 X5. Y0."], "{text}");
    for (h, c) in [("G41 H002", "C002"), ("G41 H003", "C903")] {
        let at = w.iter().position(|l| *l == h).unwrap_or_else(|| panic!("{h}: {text}"));
        assert_eq!(w[at - 1], c, "the skim's C word comes before its register: {text}");
    }
    assert_eq!(w.iter().filter(|l| **l == "G40 G01 X0. Y0.").count(), 3, "G40 on each pass's lead-out: {text}");
    assert!(w.contains(&"G01 X5. Y5.") && !w.iter().any(|l| l.contains("4.835")), "the part contour, not the wire centre: {text}");
    assert!(text.contains("SET H001 AT THE CONTROL") && text.contains("THE STREAM STATES 0.165"), "{text}");
}

/// Done means: the 2-axis block's holes post as arcs of the part's own
/// radius under the offset (2.5 mm, not the wire centre's 2.335 mm).
#[test]
fn the_ln2w_two_axis_block_posts_the_part_arcs_with_the_controls_offset() {
    let world = two_axis_with(Compensation::Control);
    assert!(posting::wire_clause(&world).problems.is_empty());
    let text = post(&world, LN2W, "2axis-offset").expect("the 2-axis block posts with the offset");
    golden("edm_test_2axis_offset.nc", &text);
    let w = words(&text);
    assert_eq!(&w[7..10], ["C001", "G41 H001", "G01 X-13. Y-0.5"], "{text}");
    assert!(w.iter().any(|l| l.starts_with("G03 ") && l.ends_with("J-2.5")), "the hole at the part radius: {text}");
    assert_eq!(&w[w.len() - 3..], ["G40 G01 X-13. Y-3.", "T90", "M02"], "{text}");
}

/// The wear mode posts the wire-centre path itself under the offset; the
/// register holds only the wear delta.
#[test]
fn the_ln2w_square_posts_the_wire_centre_path_with_a_wear_offset() {
    let text = post(&square_with(Compensation::Wear), LN2W, "square-wear").expect("the square posts with a wear offset");
    let w = words(&text);
    assert_eq!(&w[7..10], ["C001", "G41 H001", "G01 X4.835 Y0."], "{text}");
    assert!(text.contains("HOLDS THE WEAR DELTA ONLY"), "{text}");
}

/// A post that does not write the offset refuses it by name; no program
/// with the part contour and no offset comes out.
#[test]
fn the_mark_21_post_refuses_the_controls_offset_by_name() {
    let err = post(&square_with(Compensation::Control), MARK21, "square-offset-mark21").expect_err("refused");
    assert!(err.contains("control-side wire offset"), "{err}");
}

/// What the offset cannot engage on is refused at the command door.
#[test]
fn the_offset_with_a_taper_or_an_open_run_is_refused_at_the_door() {
    let world = square();
    let tapered = offset(wire(WireSide::Die, (0.0, 0.0), 10.0, 1.0, &[], true), Compensation::Control);
    let open = offset(wire(WireSide::Left, (0.0, 0.0), 10.0, 0.0, &[], true), Compensation::Wear);
    for (op, said) in [(tapered, "control compensation is stated with a 1.000 degree taper"), (open, "wear compensation is stated with an open run")] {
        let src = SrcRef::Entity("edm-test-square".into());
        let why = CreateOperation::new(&world, "Cut2", "EDM-W025", src, OpParams::WireContour(op)).err().expect("refused").to_string();
        assert!(why.contains(said), "{why}");
    }
}

/// The closing overlap (#3216) rides the offset too: the control's part
/// contour continues past the seam by the stated length, on the part's
/// own edge, and `G40` goes on the lead-out after it.
#[test]
fn the_closing_overlap_is_programmed_on_the_part_contour_under_the_offset() {
    let mut op = offset(wire(WireSide::Die, (0.0, 0.0), 10.0, 0.0, &[], true), Compensation::Control);
    op.overlap = 2.0;
    let world = cut("edm-test-square.step", (40.0, 40.0), 10.0, within((-5.0, 5.0), (-5.0, 5.0)), op);
    let clause = posting::wire_clause(&world);
    assert!(clause.problems.is_empty(), "the wire laws judge the wire-centre path: {:?}", clause.problems);
    let text = post(&world, LN2W, "square-offset-overlap").expect("the square posts with the offset and the overlap");
    let w = words(&text);
    let out = w.iter().position(|l| *l == "G40 G01 X0. Y0.").unwrap_or_else(|| panic!("G40 on the lead-out: {text}"));
    assert_eq!(&w[out - 2..out], ["G01 X5. Y0.", "G01 X5. Y2."], "the overlap past the seam is on the part contour: {text}");
    assert!(!w.iter().any(|l| l.contains("4.835")), "the part contour, not the wire centre: {text}");
}
