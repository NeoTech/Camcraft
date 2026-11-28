//! Path blocks: L with FMAX and modal F, incremental and Q-parameter values, CC + C, CR, CT arcs, tool calls, sections and M functions.

use camcraft_cl::cl::MoveKind;
use camcraft_gcode::{SpindleDir, StopKind};

use super::{assert_to, moves, near, ok, pgm};

#[test]
fn klartext_fmax_is_blockwise_and_f_is_modal() {
    let p = ok(&pgm(&[
        "L X+10 Y+0 Z+5 R0 FMAX",
        "L Z-1 F100",
        "L X+20",
        "L Y+5 FMAX",
        "L Y+10",
    ]));
    let m = moves(&p);
    let kinds: Vec<_> = m.iter().map(|m| (m.kind, m.feed)).collect();
    assert_eq!(
        kinds,
        vec![
            (MoveKind::Rapid, None),
            (MoveKind::Feed, Some(100.0)),
            (MoveKind::Feed, Some(100.0)),
            (MoveKind::Rapid, None),
            (MoveKind::Feed, Some(100.0)),
        ]
    );
    assert_to(m[4], 20.0, 10.0, -1.0);
    assert_eq!(p.line_for_move(0, 2), Some(4));
}

#[test]
fn klartext_incremental_words_and_literal_q_parameters() {
    let p = ok(&pgm(&[
        "FN 0: Q1 = +5",
        "Q2 = -2.5",
        "Q3=+200",
        "L X+Q1 Y-Q1 Z+10 R0 FMAX",
        "L IX+Q2 IY+1 IZ-Q1 FQ3",
    ]));
    let m = moves(&p);
    assert_to(m[0], 5.0, -5.0, 10.0);
    assert_to(m[1], 2.5, -4.0, 5.0);
    assert_eq!(m[1].feed, Some(200.0));
}

#[test]
fn klartext_cc_and_c_arcs_with_both_directions_and_a_full_circle() {
    let p = ok(&pgm(&[
        "L X+10 Y+0 Z-1 R0 F100",
        "CC X+0 Y+0",
        "C X+0 Y+10 DR+ F300",
        "C X+10 Y+0 DR-",
        "CC IX-5 IY+0",
        "C X+10 Y+0 DR+",
    ]));
    let m = moves(&p);
    assert_eq!(m.len(), 5, "the full circle is two arc moves");
    let a = m[1].arc.expect("arc");
    assert!(a.ccw && near(a.center.x, 0.0) && near(a.center.y, 0.0));
    assert_eq!(m[1].feed, Some(300.0));
    assert!(!m[2].arc.expect("arc").ccw);
    for mv in &m[3..] {
        let a = mv.arc.expect("arc");
        assert!(a.ccw && near(a.center.x, 5.0) && near(a.center.y, 0.0));
    }
    assert_to(m[3], 0.0, 0.0, -1.0);
    assert_to(m[4], 10.0, 0.0, -1.0);
}

#[test]
fn klartext_cr_radius_sign_picks_the_minor_or_major_arc() {
    let p = ok(&pgm(&[
        "L X+0 Y+10 Z-1 R0 F100",
        "CR X+10 Y+0 R+10 DR+",
        "CR X+0 Y+10 R-10 DR-",
    ]));
    let m = moves(&p);
    let a = m[1].arc.expect("arc");
    assert!(
        a.ccw && near(a.center.x, 10.0) && near(a.center.y, 10.0),
        "{a:?}"
    );
    let b = m[2].arc.expect("arc");
    assert!(
        !b.ccw && near(b.center.x, 0.0) && near(b.center.y, 0.0),
        "{b:?}"
    );
}

#[test]
fn klartext_ct_continues_the_tangent_of_the_previous_element() {
    let p = ok(&pgm(&[
        "L X+0 Y+0 Z-1 R0 F100",
        "L X+10 Y+0",
        "CT X+20 Y+10",
        "CT X+10 Y+20",
    ]));
    let m = moves(&p);
    for mv in &m[2..] {
        let a = mv.arc.expect("arc");
        assert!(
            a.ccw && near(a.center.x, 10.0) && near(a.center.y, 10.0),
            "{a:?}"
        );
    }
    assert_to(m[3], 10.0, 20.0, -1.0);
}

#[test]
fn klartext_tool_calls_by_number_and_name_split_sections() {
    let p = ok(&pgm(&[
        "TOOL CALL 3 Z S1000",
        "M3",
        "L X+0 Y+0 Z+10 R0 FMAX",
        "TOOL CALL \"Drill 5\" Z S2000 F250",
        "L X+5 Y+0 Z+10 R0 FMAX",
        "L Z-2",
        "TOOL CALL \"FLAT-6\" Z S3000",
        "L X+6 FMAX",
        "TOOL CALL Z S3500",
        "L X+7 FMAX",
    ]));
    let s = &p.sections;
    assert_eq!(s.len(), 4);
    assert_eq!(
        (
            s[0].tool_number,
            s[0].tool_name.as_deref(),
            s[0].spindle_rpm
        ),
        (Some(3), None, Some(1000.0))
    );
    assert_eq!(
        (
            s[1].tool_number,
            s[1].tool_name.as_deref(),
            s[1].spindle_rpm
        ),
        (None, Some("Drill 5"), Some(2000.0))
    );
    assert_eq!(s[1].toolpath.moves[1].feed, Some(250.0));
    assert_eq!(s[2].tool_name.as_deref(), Some("FLAT-6"));
    assert_eq!(
        (s[3].tool_name.as_deref(), s[3].spindle_rpm),
        (Some("FLAT-6"), Some(3500.0))
    );
    assert_eq!(p.tools, vec![3]);
}

#[test]
fn klartext_m_functions_set_spindle_coolant_and_stops() {
    let p = ok(&pgm(&[
        "TOOL CALL 1 Z S1000",
        "L X+0 Y+0 Z+10 R0 FMAX M13",
        "L X+5 FMAX M0",
        "STOP",
        "L X+6 FMAX M4",
        "L X+7 FMAX M5 M9",
        "L M140 MB 20",
        "M30",
    ]));
    let s = &p.sections;
    assert!(s[0].coolant.flood && s[0].spindle_dir == SpindleDir::Cw);
    assert_eq!(p.stops.len(), 2);
    assert_eq!(p.stops[0].kind, StopKind::ProgramStop);
    assert_eq!(p.stops[0].line, 4);
    let ccw = s
        .iter()
        .find(|x| x.spindle_dir == SpindleDir::Ccw)
        .expect("an M4 section");
    assert!(!ccw.coolant.flood, "M0 switched the coolant off");
    let m = moves(&p);
    assert_to(m[m.len() - 1], 7.0, 0.0, 30.0);
}

#[test]
fn klartext_stop_with_m0_or_m1_records_one_program_stop() {
    let p = ok(&pgm(&[
        "TOOL CALL 1 Z S1000",
        "L X+0 Y+0 Z+10 R0 FMAX M13",
        "STOP M0",
        "L X+5 FMAX M3",
        "STOP M1",
        "M30",
    ]));
    let kinds: Vec<_> = p.stops.iter().map(|s| (s.kind, s.line)).collect();
    assert_eq!(kinds.len(), 2, "one stop per STOP block: {kinds:?}");
    assert!(
        p.stops.iter().all(|s| s.kind == StopKind::ProgramStop),
        "STOP is unconditional, M1 does not weaken it: {kinds:?}"
    );
    assert_ne!(p.stops[0].line, p.stops[1].line);
    let off = p
        .sections
        .iter()
        .any(|x| x.spindle_dir == SpindleDir::Off || !x.coolant.flood);
    assert!(off, "STOP M0 keeps the M0 spindle stop and coolant off");
}
