//! Polar blocks about the pole CC: LP lines with absolute and incremental polar coordinates, CP arcs (PA and IPA, both directions), the CP helix, and their refusals.

use camcraft_cl::cl::MoveKind;
use camcraft_gcode::GcodeError;

use super::{assert_to, moves, near, ok, pgm, refusal};

const H: f64 = 8.660_254_037_844_386; // 10 sin(60 deg)

#[test]
fn klartext_lp_reaches_polar_points_about_the_pole() {
    let p = ok(&pgm(&[
        "CC X+0 Y+0",
        "L X+10 Y+0 Z-1 R0 F100",
        "LP PR+10 PA+60",
        "LP IPA+60",
        "LP PR+5",
        "LP IPR+1 IPA+60",
        "LP PA-90 Z+2 FMAX",
    ]));
    let m = moves(&p);
    assert_to(m[1], 5.0, H, -1.0);
    assert_to(m[2], -5.0, H, -1.0);
    // PR alone keeps the last programmed polar angle (120 deg).
    assert_to(m[3], -2.5, H / 2.0, -1.0);
    // IPR and IPA add to the last programmed polar coordinates.
    assert_to(m[4], -6.0, 0.0, -1.0);
    // PA alone keeps the last programmed polar radius (6); FMAX is a rapid.
    assert_to(m[5], 0.0, -6.0, 2.0);
    assert_eq!((m[5].kind, m[5].feed), (MoveKind::Rapid, None));
    assert!(
        m[1..5]
            .iter()
            .all(|m| m.arc.is_none() && m.feed == Some(100.0))
    );
}

#[test]
fn klartext_cp_arcs_about_the_pole_in_both_directions() {
    let p = ok(&pgm(&[
        "CC X+0 Y+0",
        "L X+10 Y+0 Z-1 R0 F100",
        "CP PA+90 DR+",
        "CP IPA-180 DR-",
        "CP PA+0 DR+ F300",
        "CT X+0 Y+10",
    ]));
    let m = moves(&p);
    assert_eq!(m.len(), 6, "line, quarter, half in two pieces, quarter, CT");
    assert_to(m[1], 0.0, 10.0, -1.0);
    assert_to(m[2], 10.0, 0.0, -1.0);
    assert_to(m[3], 0.0, -10.0, -1.0);
    assert_to(m[4], 10.0, 0.0, -1.0);
    for (mv, ccw) in [(m[1], true), (m[2], false), (m[3], false), (m[4], true)] {
        let a = mv.arc.expect("arc");
        assert!(
            near(a.center.x, 0.0) && near(a.center.y, 0.0) && a.ccw == ccw,
            "{a:?}"
        );
    }
    assert_eq!(m[4].feed, Some(300.0));
    // CT continues the tangent at the end of the CP arc: same circle.
    let a = m[5].arc.expect("CT arc");
    assert!(
        a.ccw && near(a.center.x, 0.0) && near(a.center.y, 0.0),
        "{a:?}"
    );
    assert_eq!(
        p.line_for_move(0, 3),
        Some(5),
        "both pieces map to the CP block"
    );
}

#[test]
fn klartext_cp_with_iz_is_a_helix_split_into_quarter_turns() {
    let p = ok(&pgm(&[
        "CC X+0 Y+0",
        "L X+10 Y+0 Z-1 R0 F100",
        "CP IPA+720 IZ-4 DR+",
    ]));
    let m = moves(&p);
    assert_eq!(m.len(), 9, "two turns are eight quarter arcs");
    assert_to(m[1], 0.0, 10.0, -1.5);
    assert_to(m[2], -10.0, 0.0, -2.0);
    assert_to(m[8], 10.0, 0.0, -5.0);
    assert!(m[1..].iter().all(|m| m.arc.is_some_and(|a| a.ccw)));
}

#[test]
fn klartext_polar_blocks_refuse_what_is_not_exact() {
    let base = "L X+10 Y+0 Z-1 R0 F100";
    let kind = |body: &[&str]| refusal(&pgm(body));
    let e = kind(&[base, "LP PR+5 PA+0"]);
    assert!(
        matches!(e, GcodeError::InvalidModal { line: 3, .. }),
        "no pole: {e}"
    );
    let e = kind(&["CC X+0 Y+0", base, "LP X+5 PA+0"]);
    assert!(
        matches!(e, GcodeError::Syntax { line: 4, .. }),
        "X in LP: {e}"
    );
    let e = kind(&["CC X+0 Y+0", base, "LP PR+5 IPR+1"]);
    assert!(
        matches!(e, GcodeError::Syntax { line: 4, .. }),
        "PR and IPR: {e}"
    );
    let e = kind(&["CC X+0 Y+0", base, "LP PR-5 PA+0"]);
    assert!(
        matches!(e, GcodeError::InvalidModal { line: 4, .. }),
        "negative PR: {e}"
    );
    let e = kind(&["CC X+10 Y+0", base, "LP PR+5"]);
    assert!(
        matches!(e, GcodeError::InvalidModal { line: 4, .. }),
        "PR without PA after a Cartesian move: {e}"
    );
    for cp in [
        "CP IPA+90 DR-",
        "CP IPA-90 DR+",
        "CP PA+0 DR+",
        "CP IPA+200000 DR+",
        "CP IPA+0 DR+",
        "CP PA+90",
        "CP DR+",
        "CP PA+90 DR+ FMAX",
    ] {
        let e = kind(&["CC X+0 Y+0", base, cp]);
        assert!(
            matches!(e, GcodeError::InvalidArc { line: 4, .. }),
            "{cp}: {e}"
        );
    }
    let e = kind(&["CC X+10 Y+0", base, "CP IPA+90 DR+"]);
    assert!(
        matches!(e, GcodeError::InvalidArc { line: 4, .. }),
        "CP at the pole: {e}"
    );
    let e = kind(&["CC X+0 Y+0", base, "CP PR+5 PA+90 DR+"]);
    assert!(
        matches!(e, GcodeError::Syntax { line: 4, .. }),
        "PR in CP: {e}"
    );
    for (line, code) in [("LP PR+5 PA+0 RL", "RL"), ("CP IPA+90 DR+ RR", "RR")] {
        let e = kind(&["CC X+0 Y+0", base, line]);
        assert!(
            matches!(&e, GcodeError::UnsupportedCode { code: c, line: 4, .. } if c == code),
            "{line}: {e}"
        );
    }
}

/// The refusal text of a program that must be refused as `InvalidModal` on `line`.
fn modal_refusal(body: &[&str], line: usize) -> String {
    let e = refusal(&pgm(body));
    assert!(
        matches!(e, GcodeError::InvalidModal { line: l, .. } if l == line),
        "{body:?}: {e}"
    );
    e.to_string()
}

#[test]
fn klartext_lp_refuses_a_polar_coordinate_that_it_does_not_know() {
    let start = ["CC X+0 Y+0", "L X+10 Y+0 Z-1 R0 F100", "LP PR+10 PA+0"];
    // A Cartesian move or a moved pole: the last programmed PR / PA is not known.
    for (tail, what) in [
        (&["L X+0 Y+10", "LP PR+5"][..], "angle"),
        (&["L X+0 Y+10", "LP PA+0"][..], "radius"),
        (&["L X+0 Y+10", "LP IPR+1 PA+0"][..], "radius"),
        (&["L X+0 Y+10", "LP PR+5 IPA+10"][..], "angle"),
        (&["CC X+5 Y+0", "LP PA+90"][..], "radius"),
        (&["C X+0 Y+10 DR+", "LP PR+5"][..], "angle"),
    ] {
        let body: Vec<&str> = start.iter().chain(tail).copied().collect();
        let e = modal_refusal(&body, 6);
        assert!(
            e.contains(&format!(
                "the last polar {what} about this pole is not known"
            )),
            "{body:?}: {e}"
        );
    }
    // The first LP after the CC block knows no polar coordinate.
    let e = modal_refusal(&["CC X+0 Y+0", "L X+10 Y+0 Z-1 R0 F100", "LP PR+10"], 4);
    assert!(e.contains("LP without PA"), "{e}");
    let e = modal_refusal(&["CC X+0 Y+0", "L X+10 Y+0 Z-1 R0 F100", "LP IPA+90"], 4);
    assert!(e.contains("LP without PR"), "{e}");
}

#[test]
fn klartext_lp_keeps_the_last_polar_coordinates_where_they_are_known() {
    // A Z-only move, the same pole again and an M block keep PR and PA.
    let p = ok(&pgm(&[
        "CC X+0 Y+0",
        "L X+10 Y+0 Z-1 R0 F100",
        "LP PR+10 PA+0",
        "L Z-2",
        "CC X+0 Y+0",
        "M8",
        "LP PA+90",
    ]));
    assert_to(moves(&p)[3], 0.0, 10.0, -2.0);
    // CP keeps PR; CP PA sets PA, and CP IPA adds to a known PA.
    let p = ok(&pgm(&[
        "CC X+0 Y+0",
        "L X+10 Y+0 Z-1 R0 F100",
        "LP PR+10 PA+0",
        "CP PA+90 DR+",
        "CP IPA+90 DR+",
        "LP IPA+90",
        "LP PR+5",
    ]));
    let m = moves(&p);
    assert_to(m[4], 0.0, -10.0, -1.0);
    assert_to(m[5], 0.0, -5.0, -1.0);
    // After a Cartesian move, CP PA sets the angle, but the radius stays unknown.
    let after_l = ["CC X+0 Y+0", "L X+10 Y+0 Z-1 R0 F100", "CP PA+90 DR+"];
    let body: Vec<&str> = after_l.iter().copied().chain(["LP PR+5"]).collect();
    assert_to(moves(&ok(&pgm(&body)))[2], 0.0, 5.0, -1.0);
    let body: Vec<&str> = after_l.iter().copied().chain(["LP PA+180"]).collect();
    assert!(modal_refusal(&body, 5).contains("LP without PR"));
    // CP IPA from an unknown start angle leaves the angle unknown.
    let e = modal_refusal(
        &[
            "CC X+0 Y+0",
            "L X+10 Y+0 Z-1 R0 F100",
            "CP IPA+90 DR+",
            "LP PR+5",
        ],
        5,
    );
    assert!(e.contains("LP without PA"), "{e}");
    // PR 0 is the pole: no angle is needed, and none is known after it.
    let p = ok(&pgm(&["CC X+0 Y+0", "L X+10 Y+0 Z-1 R0 F100", "LP PR+0"]));
    assert_to(moves(&p)[1], 0.0, 0.0, -1.0);
    let body = ["CC X+0 Y+0", "L X+10 Y+0 Z-1 R0 F100", "LP PR+0", "LP PR+5"];
    assert!(modal_refusal(&body, 5).contains("LP without PA"));
}

#[test]
fn klartext_lp_angle_out_of_range_is_refused() {
    let e = modal_refusal(
        &[
            "CC X+0 Y+0",
            "L X+10 Y+0 Z-1 R0 F100",
            "LP PR+10 PA+99999",
            "LP IPA+99999",
            "LP IPA+99999",
        ],
        6,
    );
    assert!(e.contains("out of range"), "{e}");
}
