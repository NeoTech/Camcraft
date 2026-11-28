//! Refusals: each Klartext function outside the first slice is refused by name, with its line and block number; the program frame is checked.

use camcraft_gcode::{GcodeError, KlartextOptions};

use super::{parse, pgm, refusal};

#[test]
fn klartext_functions_outside_the_slice_are_refused_by_name_and_block() {
    let cases: [(&str, &str); 17] = [
        ("RND R5", "RND"),
        ("CHF 2", "CHF"),
        ("LP PR+10 PA+0 R0 F100", "LP"),
        ("CP IPA+90 DR+", "CP"),
        ("L X+5 RL F100", "RL"),
        ("L X+5 RR F100", "RR"),
        ("LBL 1", "LBL"),
        ("CALL LBL 1 REP 2", "CALL"),
        ("APPR LT X+0 Y+0 LEN10 RL F100", "APPR"),
        ("L A+90 FMAX", "axis A"),
        ("L X+5 FAUTO", "FAUTO"),
        ("L X+5 FMAX M91", "M91"),
        ("M136", "M136"),
        ("FN 1: Q1 = +1 + +2", "FN 1"),
        ("Q1 = Q2 + 1", "Q1 = Q2+1"),
        ("TOOL CALL 1 X S100", "TOOL CALL X"),
        ("C X+10 Y+0 Z-5 DR+ F100", "helix"),
    ];
    for (line, code) in cases {
        let nc = pgm(&["L X+0 Y+0 Z+10 R0 F100", "CC X+5 Y+0", line]);
        let e = refusal(&nc);
        match &e {
            GcodeError::UnsupportedCode { code: c, .. } => assert_eq!(c, code, "{line}"),
            other => panic!("{line}: expected an unsupported-code refusal, got {other}"),
        }
        assert_eq!(e.line(), 4, "{line}");
        assert!(e.to_string().contains("block 3:"), "{line}: {e}");
    }
}

#[test]
fn klartext_inch_programs_and_unknown_words_are_refused() {
    let e = refusal("0 BEGIN PGM T INCH\n1 END PGM T INCH\n");
    assert!(
        matches!(&e, GcodeError::UnsupportedCode { code, line: 1, .. } if code == "BEGIN PGM INCH"),
        "{e}"
    );
    let e = refusal(&pgm(&["FOO X+1"]));
    assert!(matches!(&e, GcodeError::UnknownWord { line: 2, .. }), "{e}");
    let e = refusal(&pgm(&["L X+1 KK FMAX"]));
    assert!(matches!(&e, GcodeError::UnknownWord { line: 2, .. }), "{e}");
}

#[test]
fn klartext_program_frame_is_checked() {
    let syntax_line = |nc: &str| match refusal(nc) {
        GcodeError::Syntax { line, .. } => line,
        other => panic!("expected a syntax refusal, got {other} for\n{nc}"),
    };
    // Motion before BEGIN PGM, a block after END PGM, a missing END PGM, a name mismatch.
    assert_eq!(
        syntax_line("0 L X+1 FMAX\n1 BEGIN PGM T MM\n2 END PGM T MM\n"),
        1
    );
    assert_eq!(
        syntax_line("0 BEGIN PGM T MM\n1 END PGM T MM\n2 L X+1 FMAX\n"),
        3
    );
    assert_eq!(syntax_line("0 BEGIN PGM T MM\n1 L X+1 FMAX\n"), 2);
    assert_eq!(syntax_line("0 BEGIN PGM T MM\n1 END PGM U MM\n"), 2);
    // Motion after M30.
    assert_eq!(syntax_line(&pgm(&["M30", "L X+1 FMAX"])), 3);
    // A feed move without any F.
    let e = refusal(&pgm(&["L X+1 Y+0"]));
    assert!(matches!(e, GcodeError::InvalidModal { line: 2, .. }), "{e}");
}

#[test]
fn klartext_arc_preconditions_are_refused() {
    let invalid_arc = |body: &[&str]| match refusal(&pgm(body)) {
        GcodeError::InvalidArc { .. } => {}
        other => panic!("expected an invalid arc, got {other} for {body:?}"),
    };
    invalid_arc(&["L X+10 Y+0 Z+0 F100", "C X+0 Y+10 DR+"]); // no CC
    invalid_arc(&["L X+10 Y+0 Z+0 F100", "CC X+0 Y+0", "C X+0 Y+10"]); // no DR
    invalid_arc(&["L X+10 Y+0 Z+0 F100", "CC X+0 Y+0", "C X+0 Y+12 DR+"]); // radius mismatch
    invalid_arc(&["L X+10 Y+0 Z+0 F100", "CR X+0 Y+50 R+10 DR+"]); // chord longer than 2R
    invalid_arc(&["L X+0 Y+0 Z+0 F100", "CT X+10 Y+0"]); // pure Z before CT: no tangent
    invalid_arc(&["L X+0 Y+0 Z+0 F100", "L X+10", "CT X+20 Y+0"]); // CT end on the tangent line
    invalid_arc(&["L X+10 Y+0 Z+0 F100", "CC X+0", "C X+0 Y+10 DR+"]); // CC with one axis
}

#[test]
fn klartext_m140_mb_max_needs_the_traverse_end() {
    let nc = pgm(&["L M140 MB MAX"]);
    let e = parse(&nc, &KlartextOptions::default()).expect_err("no traverse end");
    assert!(
        matches!(e, GcodeError::MissingReferencePosition { line: 2, .. }),
        "{e}"
    );
    let low = KlartextOptions {
        tool_axis_max_z: Some(-5.0),
        ..KlartextOptions::default()
    };
    let e = parse(&nc, &low).expect_err("traverse end below the tool");
    assert!(
        matches!(e, GcodeError::UnsupportedCode { line: 2, .. }),
        "{e}"
    );
    let bad = KlartextOptions {
        tool_axis_max_z: Some(f64::NAN),
        ..KlartextOptions::default()
    };
    assert!(matches!(
        parse(&nc, &bad),
        Err(GcodeError::InvalidOptions { .. })
    ));
}
