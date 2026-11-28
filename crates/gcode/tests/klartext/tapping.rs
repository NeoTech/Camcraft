//! Tapping cycles 206 / 207 and the sub-block cycles 9 (dwell) and 32 (tolerance): the motion they give and the refusals that name the block.

use camcraft_cl::cl::{CycleKind, MoveKind};
use camcraft_gcode::GcodeError;

use super::{assert_to, moves, near, ok, pgm, refusal};

fn cycles(p: &camcraft_gcode::ParsedProgram) -> Vec<camcraft_cl::cl::Cycle> {
    p.sections
        .iter()
        .flat_map(|s| s.toolpath.cycles.iter().copied())
        .collect()
}

#[test]
fn klartext_cycle_207_taps_at_s_times_pitch_and_feeds_out_to_the_set_up_clearance() {
    let p = ok(&pgm(&[
        "TOOL CALL 4 Z S500",
        "M3",
        "L X+5 Y+5 Z+10 R0 FMAX",
        "CYCL DEF 207 RIGID TAPPING NEW ~",
        "  Q200=+1 ~",
        "  Q201=-8 ~",
        "  Q239=+1.25 ~",
        "  Q203=+5 ~",
        "  Q204=+3",
        "CYCL CALL",
    ]));
    let m = moves(&p);
    assert_eq!(m.len(), 5, "{m:#?}");
    assert_to(m[1], 5.0, 5.0, 6.0);
    assert_eq!(m[1].kind, MoveKind::Rapid);
    // Feed in and out at S x pitch = 500 x 1.25 = 625 mm/min.
    assert_to(m[2], 5.0, 5.0, -3.0);
    assert_to(m[3], 5.0, 5.0, 6.0);
    for f in [m[2], m[3]] {
        assert_eq!(f.kind, MoveKind::Feed);
        assert!(near(f.feed.expect("tap feed"), 625.0), "{f:?}");
    }
    // The 2nd set-up clearance (surface + 3) is higher: rapid to it.
    assert_to(m[4], 5.0, 5.0, 8.0);
    assert_eq!(m[4].kind, MoveKind::Rapid);
    let c = cycles(&p);
    assert_eq!(c.len(), 1);
    assert_eq!(c[0].kind, CycleKind::Tap);
    assert!(near(c[0].r_z, 6.0) && near(c[0].bottom_z, -3.0) && near(c[0].entry_z, 5.0));
    assert!(near(c[0].dwell, 0.0) && near(c[0].peck, 0.0));
}

#[test]
fn klartext_cycle_206_feeds_at_q206_with_the_bottom_dwell_at_each_m99() {
    let p = ok(&pgm(&[
        "TOOL CALL 5 Z S300",
        "M3",
        "L X+0 Y+0 Z+10 R0 FMAX",
        "CYCL DEF 206 TAPPING NEW Q200=2 Q201=-10 Q206=150 Q211=0.25 Q203=+0 Q204=0",
        "L X+10 Y+0 R0 FMAX M99",
        "L X+20 Y+0 R0 FMAX M99",
    ]));
    let c = cycles(&p);
    assert_eq!(c.len(), 2);
    for (cy, x) in c.iter().zip([10.0, 20.0]) {
        assert_eq!(cy.kind, CycleKind::FloatingTap);
        assert!(near(cy.xy.x, x) && near(cy.dwell, 0.25), "{cy:?}");
    }
    let m = moves(&p);
    // Hole 1: XY at Z10, rapid to Z2, feed to Z-10 and back to Z2 (the 2nd
    // clearance is lower, so no last rapid). Hole 2: XY at Z2, feed in and out.
    let z: Vec<f64> = m.iter().map(|m| m.to.z).collect();
    assert_eq!(z, vec![10.0, 10.0, 2.0, -10.0, 2.0, 2.0, -10.0, 2.0]);
    for f in m.iter().filter(|m| m.kind != MoveKind::Rapid) {
        assert!(near(f.feed.expect("feed"), 150.0), "{f:?}");
    }
}

#[test]
fn klartext_cycle_207_needs_a_spindle_speed_at_the_call() {
    for tool in ["TOOL CALL 4 Z", "TOOL CALL 4 Z S0"] {
        let e = refusal(&pgm(&[
            tool,
            "L X+0 Y+0 Z+10 R0 FMAX",
            "CYCL DEF 207 RIGID TAPPING NEW Q200=1 Q201=-8 Q239=1 Q203=0 Q204=1",
            "CYCL CALL",
        ]));
        assert!(matches!(e, GcodeError::InvalidCycle { line: 5, .. }), "{tool}: {e}");
        assert!(e.to_string().contains("block 4:"), "{e}");
    }
}

#[test]
fn klartext_tapping_cycle_definitions_are_checked_by_name_and_block() {
    let left = refusal(&pgm(&[
        "CYCL DEF 207 RIGID TAPPING NEW Q200=1 Q201=-8 Q239=-1 Q203=0 Q204=1",
    ]));
    assert!(
        matches!(&left, GcodeError::UnsupportedCode { code, line: 2, .. } if code == "CYCL DEF 207 Q239"),
        "{left}"
    );
    let cases = [
        "CYCL DEF 207 RIGID TAPPING NEW Q200=1 Q201=-8 Q239=0 Q203=0 Q204=1",
        "CYCL DEF 207 RIGID TAPPING NEW Q200=1 Q201=-8 Q239=1 Q203=0 Q204=1 Q206=100",
        "CYCL DEF 207 RIGID TAPPING NEW Q200=1 Q201=-8 Q203=0 Q204=1",
        "CYCL DEF 207 RIGID TAPPING NEW Q200=1 Q201=+8 Q239=1 Q203=0 Q204=1",
        "CYCL DEF 206 TAPPING NEW Q200=1 Q201=-8 Q211=0 Q203=0 Q204=1",
        "CYCL DEF 206 TAPPING NEW Q200=-1 Q201=-8 Q206=100 Q203=0 Q204=1",
        "CYCL DEF 206 TAPPING NEW Q200=1 Q201=-8 Q206=100 Q203=0 Q204=1 Q202=2",
    ];
    for line in cases {
        let e = refusal(&pgm(&[line]));
        assert!(matches!(e, GcodeError::InvalidCycle { line: 2, .. }), "{line}: {e}");
        assert!(e.to_string().contains("block 1:"), "{e}");
    }
    // Bad feed and dwell values are refused on the definition line.
    for line in [
        "CYCL DEF 206 TAPPING NEW Q200=1 Q201=-8 Q206=0 Q203=0 Q204=1",
        "CYCL DEF 206 TAPPING NEW Q200=1 Q201=-8 Q206=100 Q211=-1 Q203=0 Q204=1",
        "CYCL DEF 206 TAPPING NEW Q200=1 Q201=-8 Q206=100 Q211=99999999 Q203=0 Q204=1",
    ] {
        assert_eq!(refusal(&pgm(&[line])).line(), 2, "{line}");
    }
}

#[test]
fn klartext_cycle_9_dwells_where_it_is_defined() {
    let p = ok(&pgm(&[
        "L X+0 Y+0 Z+10 R0 FMAX",
        "L Z-2 R0 F100",
        "CYCL DEF 9.0 DWELL TIME",
        "CYCL DEF 9.1 DWELL 0.3",
        "FN 0: Q5 = +1.5",
        "CYCL DEF 9.0 VERWEILZEIT",
        "CYCL DEF 9.1 V.ZEIT Q5",
        "L Z+10 R0 FMAX",
    ]));
    let d: Vec<(usize, f64, Option<usize>)> = p
        .dwells
        .iter()
        .map(|d| (d.line, d.seconds, d.after_move))
        .collect();
    assert_eq!(d, vec![(5, 0.3, Some(1)), (8, 1.5, Some(1))]);
    assert_eq!(moves(&p).len(), 3);
}

#[test]
fn klartext_cycle_32_is_checked_and_leaves_the_path_unchanged() {
    let path = ["L X+0 Y+0 Z+5 R0 FMAX", "L X+10 R0 F500", "L Y+10"];
    let plain = ok(&pgm(&path));
    let smoothed = ok(&pgm(&[
        "CYCL DEF 32.0 TOLERANCE",
        "CYCL DEF 32.1 T0.01",
        "CYCL DEF 32.2 HSC-MODE:1 TA0.5",
        path[0],
        path[1],
        "CYCL DEF 32.0 TOLERANCE",
        "CYCL DEF 32.1 T0.05",
        path[2],
        "CYCL DEF 32.0 TOLERANCE",
        "CYCL DEF 32.1",
    ]));
    let a: Vec<_> = moves(&plain).into_iter().map(|m| (m.kind, m.to, m.feed)).collect();
    let b: Vec<_> = moves(&smoothed).into_iter().map(|m| (m.kind, m.to, m.feed)).collect();
    assert_eq!(a, b);
    assert!(smoothed.dwells.is_empty() && smoothed.stops.is_empty());
}

#[test]
fn klartext_sub_block_cycles_must_be_complete_and_in_order() {
    let syntax: [&[&str]; 9] = [
        &["CYCL DEF 9.0 DWELL TIME", "L X+1 R0 FMAX"],
        &["CYCL DEF 9.1 DWELL 1"],
        &["CYCL DEF 9.0 DWELL TIME"],
        &["CYCL DEF 9.0 DWELL TIME", "CYCL DEF 9.0 DWELL TIME"],
        &["CYCL DEF 9.0 DWELL TIME", "CYCL DEF 9.2 DWELL 1"],
        &["CYCL DEF 9.0 DWELL TIME", "CYCL DEF 9.1"],
        &["CYCL DEF 9.0 DWELL 1"],
        &["CYCL DEF 32.0 TOLERANCE", "CYCL DEF 32.2 HSC-MODE:0"],
        &["CYCL DEF 32.0 TOLERANCE", "CYCL DEF 32.1 T0.01", "CYCL DEF 32.2 HSC-MODE:2"],
    ];
    for body in syntax {
        let e = refusal(&pgm(body));
        assert!(matches!(e, GcodeError::Syntax { .. }), "{body:?}: {e}");
        assert!(e.to_string().contains("block "), "{body:?}: {e}");
    }
    let other: [&[&str]; 5] = [
        &["CYCL DEF 9.0 DWELL TIME", "CYCL DEF 9.1 DWELL -1"],
        &["CYCL DEF 32.0 TOLERANCE", "CYCL DEF 32.1 T-0.1"],
        &["CYCL DEF 32.0 TOLERANCE", "CYCL DEF 32.1 X5"],
        &["CYCL DEF 32.0 TOLERANCE", "CYCL DEF 32.1 T0.01", "CYCL DEF 32.2 HSC-MODE:0 TA400"],
        &["CYCL DEF 32.0 TOLERANCE", "CYCL DEF 32.1 T0.01", "CYCL DEF 32.2 HSC-MODE:0 Q5"],
    ];
    for body in other {
        let e = refusal(&pgm(body));
        assert_eq!(e.line(), body.len() + 1, "{body:?}: {e}");
    }
    // Other sub-block cycles stay refused by name.
    for (line, code) in [
        ("CYCL DEF 10.0 ROTATION", "CYCL DEF 10.0"),
        ("CYCL DEF 7.0 DATUM SHIFT", "CYCL DEF 7.0"),
    ] {
        let e = refusal(&pgm(&[line]));
        assert!(
            matches!(&e, GcodeError::UnsupportedCode { code: c, line: 2, .. } if c == code),
            "{line}: {e}"
        );
    }
}
