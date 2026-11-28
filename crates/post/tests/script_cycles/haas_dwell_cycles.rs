//! Haas G82/G89 modal dwell regression, from the public Haas G82, G89 and G04 documentation (#2350).
//! #1977: the reverse mill spindle (M4) and the right-hand tap under it, from
//! the Haas Mill NGC Operator's Manual 2025 (M03/M04 p. 436, G84 and G74 pp. 224, 371).
//! Gate map: covers assets/posts/haas.cps, camcraft_post::post::engine and camcraft_post::post::cnc.

use camcraft_post::cl::{CycleKind, MoveKind};
use camcraft_post::post::{
    cnc::{self, CycleRec, MoveRec, Record},
    engine,
};
use serde_json::json;
use std::path::Path;

fn read(rel: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../")
            .join(rel),
    )
    .expect("fixture")
}

/// One open cycle group: back-to-back holes of `kind` with the given
/// dwells (seconds), each at its own XY so the group stays open and the
/// later holes would post as bare X Y blocks if the signature matched.
fn stream(kind: CycleKind, dwells: &[f64]) -> String {
    let mut records = vec![
        Record::Param {
            name: "SpindleSpeed".into(),
            value: json!(900.0),
        },
        Record::Param {
            name: "ClearancePlane".into(),
            value: json!(10.0),
        },
        Record::Param {
            name: "RetractPlane".into(),
            value: json!(10.0),
        },
        Record::Tool {
            number: 4,
            name: "DRILL".into(),
        },
        Record::Section { begin: true },
        Record::Move(MoveRec {
            kind: MoveKind::Rapid,
            x: 10.0,
            y: 10.0,
            z: 2.0,
            feed: None,
            center: None,
            ccw: false,
            plane: Default::default(),
            abc: None,
        }),
    ];
    for &dwell in dwells {
        records.push(Record::Cycle(CycleRec {
            kind,
            x: 10.0,
            y: 10.0,
            r_z: 2.0,
            bottom_z: -12.0,
            entry_z: 0.0,
            peck: 0.0,
            dwell,
            first_move: 1,
            feed: Some(120.0),
        }));
    }
    records.push(Record::Section { begin: false });
    cnc::serialize(&records)
}

fn run(kind: CycleKind, dwells: &[f64]) -> engine::PostOutput {
    let machine = serde_json::from_str(&read("assets/machines/3axis-vmc.mch")).expect("machine");
    engine::run(
        &read("assets/posts/haas.cps"),
        &machine,
        &stream(kind, dwells),
    )
    .expect("post engine")
}

/// The P word of every hole: the cycle block's P, or `None` for a hole
/// posted as a bare X Y block inside the open group (it runs the modal P).
fn hole_dwells(program: &str, g: &str) -> Vec<Option<String>> {
    let mut out = Vec::new();
    let mut open = false;
    for line in program.lines() {
        let words: Vec<&str> = line.split_whitespace().collect();
        if words.contains(&g) {
            open = true;
            out.push(
                words
                    .iter()
                    .find(|w| w.starts_with('P'))
                    .map(|w| w.to_string()),
            );
        } else if words.contains(&"G80") {
            open = false;
        } else if open
            && !words.is_empty()
            && words
                .iter()
                .all(|w| w.starts_with('X') || w.starts_with('Y'))
        {
            out.push(None);
        }
    }
    out
}

fn pins(kind: CycleKind, g: &str, dwells: &[f64], want: &[&str]) {
    let out = run(kind, dwells);
    assert!(out.warnings.is_empty(), "{g}: {:?}", out.warnings);
    let got = hole_dwells(&out.program, g);
    let want: Vec<Option<String>> = want.iter().map(|p| Some(p.to_string())).collect();
    assert_eq!(
        got, want,
        "{g}: every hole must post its own dwell P, never inherit the modal one:\n{}",
        out.program
    );
}

#[test]
fn haas_g82_rewrites_p_when_the_dwell_changes() {
    pins(CycleKind::Drill, "G82", &[0.5, 0.75], &["P500", "P750"]);
}

#[test]
fn haas_g89_rewrites_p_when_the_dwell_changes() {
    pins(CycleKind::BackBore, "G89", &[0.5, 0.75], &["P500", "P750"]);
}

#[test]
fn haas_g89_zero_dwell_writes_p0() {
    // A G89 opened after a dwelling G89 without P would reuse the old
    // modal P on Haas; P0 clears it.
    pins(CycleKind::BackBore, "G89", &[0.5, 0.0], &["P500", "P0"]);
    pins(CycleKind::BackBore, "G89", &[0.0], &["P0"]);
}

#[test]
fn haas_same_dwell_keeps_the_open_group() {
    for (kind, g) in [(CycleKind::Drill, "G82"), (CycleKind::BackBore, "G89")] {
        let out = run(kind, &[0.5, 0.5]);
        assert!(out.warnings.is_empty(), "{g}: {:?}", out.warnings);
        let got = hole_dwells(&out.program, g);
        // The unchanged dwell rides the open group as a bare X Y hole.
        assert_eq!(
            got,
            vec![Some("P500".to_string()), None],
            "{g}:\n{}",
            out.program
        );
    }
}

/// #1977: two milling sections, the first with `PARAM MillSpindle reverse`,
/// and optionally a G84 tap in the reversed section.
fn reverse_stream(tap: bool) -> String {
    let cut = "Rapid X 10000 Y 10000 Z 45000\nRapid X 10000 Y 10000 Z 5000\nFeed X 10000 Y 10000 Z -1000 F 300000\nRapid X 10000 Y 10000 Z 45000\n";
    let first = if tap {
        "Rapid X 0 Y 0 Z 45000\nRapid X 0 Y 0 Z 20000\nCYCLE TAP X 0 Y 0 Z 8000 R 20000 ENTRY 0 Q 0 P 0 F 600000\nPlunge X 0 Y 0 Z 8000 F 600000\nPlunge X 0 Y 0 Z 20000 F 600000\nRapid X 0 Y 0 Z 45000\n"
    } else {
        cut
    };
    format!(
        "; HSMW-CNC v1\nPARAM WorkOffset G54\nPARAM ClearancePlane 45.000000\n\
         PARAM OperationName Rev1\nPARAM SpindleSpeed 600.000000\nPARAM RetractPlane 20.000000\nPARAM MillSpindle reverse\nTOOL 2 MILL-2\nSECTION\n{first}ENDSECTION\n\
         PARAM OperationName Fwd2\nPARAM SpindleSpeed 3000.000000\nPARAM RetractPlane 20.000000\nTOOL 3 MILL-3\nSECTION\n{cut}ENDSECTION\n"
    )
}

fn run_text(stream: &str) -> engine::PostOutput {
    let machine = serde_json::from_str(&read("assets/machines/3axis-vmc.mch")).expect("machine");
    engine::run(&read("assets/posts/haas.cps"), &machine, stream).expect("post engine")
}

#[test]
fn haas_reverse_mill_spindle_starts_m4_for_its_section_only() {
    let out = run_text(&reverse_stream(false));
    assert!(out.warnings.is_empty(), "the post declares millSpindleReverse: {:?}", out.warnings);
    let starts: Vec<&str> = out.program.lines().filter(|l| l.starts_with('S')).collect();
    assert_eq!(starts, ["S600 M4", "S3000 M3"], "{}", out.program);
}

#[test]
fn haas_refuses_a_right_hand_tap_under_a_reverse_spindle() {
    let out = run_text(&reverse_stream(true));
    assert!(
        out.warnings.iter().any(|w| w.contains("G84") && w.contains("M4")),
        "named refusal: {:?}",
        out.warnings
    );
    assert!(!out.program.contains("G84"), "no right-hand tap block: {}", out.program);
}
