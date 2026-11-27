//! Cycles: CYCL DEF 200 / 203 / 205 called by M99 and CYCL CALL, CYCL DEF 247 presets, and the cycle refusals by name.

use camcraft_cl::cl::{CycleKind, MoveKind};
use camcraft_gcode::{GcodeError, WorkOffset};

use super::{assert_to, moves, ok, pgm, refusal};

const DEF_200: [&str; 9] = [
    "CYCL DEF 200 DRILLING ~",
    "  Q200=2 ;SET-UP CLEARANCE ~",
    "  Q201=-10 ;DEPTH ~",
    "  Q206=150 ;FEED RATE FOR PLNGNG ~",
    "  Q202=4 ;PLUNGING DEPTH ~",
    "  Q210=0 ;DWELL TIME AT TOP ~",
    "  Q203=+0 ;SURFACE COORDINATE ~",
    "  Q204=50 ;2ND SET-UP CLEARANCE ~",
    "  Q211=0.5 ;DWELL TIME AT DEPTH",
];

#[test]
fn klartext_cycle_200_pecks_from_the_surface_and_ends_at_the_2nd_clearance() {
    let mut body: Vec<&str> = vec!["TOOL CALL 4 Z S2000", "M3"];
    body.extend(DEF_200);
    body.push("L X+10 Y+20 Z+60 R0 FMAX M99");
    let p = ok(&pgm(&body));
    let m = moves(&p);
    let z: Vec<(MoveKind, f64)> = m[1..].iter().map(|m| (m.kind, m.to.z)).collect();
    assert_eq!(
        z,
        vec![
            (MoveKind::Rapid, 2.0),
            (MoveKind::Plunge, -4.0),
            (MoveKind::Rapid, 2.0),
            (MoveKind::Plunge, -8.0),
            (MoveKind::Rapid, 2.0),
            (MoveKind::Plunge, -10.0),
            (MoveKind::Rapid, 50.0),
        ]
    );
    assert!(m[1..].iter().all(|mv| mv.to.x == 10.0 && mv.to.y == 20.0));
    assert_eq!(m[2].feed, Some(150.0));
    let tp = &p.sections[0].toolpath;
    assert_eq!(tp.cycles.len(), 1);
    let c = &tp.cycles[0];
    assert_eq!(
        (c.kind, c.r_z, c.bottom_z, c.peck, c.dwell, c.first_move),
        (CycleKind::Deep, 2.0, -10.0, 4.0, 0.5, 2)
    );
    // Every cycle move maps to the M99 block (block 4 on line 13, after 8 continuation lines).
    assert!(p.sections[0].move_lines.iter().all(|l| *l == 13));
}

#[test]
fn klartext_cycles_203_and_205_without_chip_breaking_drill_at_cycl_call() {
    let p = ok(&pgm(&[
        "CYCL DEF 203 UNIVERSAL DRILLING Q200=2 Q201=-6 Q206=100 Q202=10 Q210=0 Q203=-1 Q204=0 Q212=0 Q213=0 Q205=0 Q211=0 Q208=500 Q256=0.2",
        "L X+1 Y+1 Z+20 R0 FMAX",
        "CYCL CALL",
        "CYCL DEF 205 UNIVERSAL PECKING Q200=1 Q201=-9 Q206=80 Q202=3 Q203=+0 Q204=0 Q212=0 Q205=0 Q258=0.2 Q259=0.2 Q257=0 Q256=0.2 Q211=0 Q379=0 Q253=750 Q208=99999",
        "L X+5 Y+5 FMAX",
        "CYCL CALL M3",
    ]));
    let tp = &p.sections.last().expect("a section").toolpath;
    let kinds: Vec<_> = p
        .sections
        .iter()
        .flat_map(|s| s.toolpath.cycles.iter().map(|c| c.kind))
        .collect();
    assert_eq!(kinds, vec![CycleKind::Drill, CycleKind::Deep]);
    let m = moves(&p);
    // 203: one plunge from Q203+Q200 = 1 to Q203+Q201 = -7, back to 1 (Q204 is lower).
    assert_to(m[1], 1.0, 1.0, 1.0);
    assert_to(m[2], 1.0, 1.0, -7.0);
    assert_to(m[3], 1.0, 1.0, 1.0);
    // 205: pecks at -3, -6, -9 from the surface.
    let pecks: Vec<f64> = tp
        .moves
        .iter()
        .filter(|m| m.kind == MoveKind::Plunge)
        .map(|m| m.to.z)
        .collect();
    assert_eq!(pecks, vec![-3.0, -6.0, -9.0]);
}

#[test]
fn klartext_cycle_247_selects_the_preset_as_work_offset() {
    let p = ok(&pgm(&[
        "CYCL DEF 247 DATUM SETTING ~",
        "  Q339=+4",
        "L X+0 Y+0 Z+5 R0 FMAX",
    ]));
    assert_eq!(p.sections[0].work_offset, WorkOffset::G57);
}

fn unsupported_code(e: &GcodeError) -> &str {
    match e {
        GcodeError::UnsupportedCode { code, .. } => code,
        other => panic!("expected an unsupported-code refusal, got {other}"),
    }
}

#[test]
fn klartext_unsupported_cycles_and_cycle_options_are_refused_by_name() {
    let cases: [(&str, &str); 6] = [
        ("CYCL DEF 1.0 PECKING", "CYCL DEF 1.0"),
        ("CYCL DEF 262 THREAD MILLING Q335=10", "CYCL DEF 262"),
        (
            "CYCL DEF 200 DRILLING Q200=2 Q201=-5 Q206=100 Q202=5 Q203=0 Q204=0 Q210=1",
            "CYCL DEF 200 Q210",
        ),
        (
            "CYCL DEF 203 X Q200=2 Q201=-5 Q206=100 Q202=5 Q203=0 Q204=0 Q213=2",
            "CYCL DEF 203 Q213",
        ),
        (
            "CYCL DEF 205 X Q200=2 Q201=-5 Q206=100 Q202=5 Q203=0 Q204=0 Q395=1",
            "CYCL DEF 205 Q395",
        ),
        ("CYCL CALL PAT F100", "CYCL CALL PAT"),
    ];
    for (line, code) in cases {
        let e = refusal(&pgm(&["L X+0 Y+0 Z+10 R0 FMAX", line]));
        assert_eq!(unsupported_code(&e), code, "{line}");
        assert_eq!(e.line(), 3, "{line}");
        assert!(e.to_string().contains("block 2:"), "{e}");
    }
}

#[test]
fn klartext_invalid_cycle_definitions_and_calls_are_refused() {
    let cases = [
        "CYCL DEF 200 DRILLING Q200=2 Q206=100 Q202=5 Q203=0 Q204=0",
        "CYCL DEF 200 DRILLING Q200=2 Q201=+5 Q206=100 Q202=5 Q203=0 Q204=0",
        "CYCL DEF 200 DRILLING Q200=2 Q201=-5 Q206=100 Q202=0 Q203=0 Q204=0",
        "CYCL DEF 200 DRILLING Q200=2 Q201=-5 Q206=100 Q202=5 Q203=0 Q204=0 Q339=1",
        "CYCL DEF 200 DRILLING Q200=2 Q201=-1000 Q206=100 Q202=0.0001 Q203=0 Q204=0",
        "CYCL CALL",
    ];
    for line in cases {
        let mut body = vec!["L X+0 Y+0 Z+10 R0 FMAX", line];
        if line.starts_with("CYCL DEF") {
            body.push("CYCL CALL");
        }
        let e = refusal(&pgm(&body));
        assert!(matches!(e, GcodeError::InvalidCycle { .. }), "{line}: {e}");
    }
}
