//! Hurco WinMax (NC) ISNC post processor tests (#2573).
//!
//! Provenance: clean-room specification from Hurco WinMax Mill NC
//! Programming documentation (704-0116-101, support.hurco.com ISNC
//! dialect) and public Hurco VM10i machine specifications.
//!
//! Gate map: covers assets/posts/hurco-winmax.cps,
//! assets/machines/hurco-vm10i.mch, post::post::engine.

use camcraft_post::cl::{CycleKind, MoveKind};
use camcraft_post::machine::parse_mch;
use camcraft_post::post::{
    cnc::{self, CycleRec, MoveRec, Record},
    engine, machine, ProgramId,
};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn repo(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../").join(rel)
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

fn cps() -> String {
    read("assets/posts/hurco-winmax.cps")
}

fn vm10i() -> Value {
    let m = parse_mch(&read("assets/machines/hurco-vm10i.mch")).expect("hurco-vm10i.mch parses");
    machine::mirror(&m)
}

fn vmc() -> Value {
    serde_json::from_str(&read("assets/machines/3axis-vmc.mch")).expect("3axis-vmc.mch")
}

/// Strip `( ... )` comments, checking balance and clean comment characters.
fn strip_comments(line: &str) -> String {
    let mut out = String::new();
    let mut depth = 0usize;
    for ch in line.chars() {
        assert!(ch.is_ascii(), "non-ASCII char in WinMax NC line: {line}");
        match ch {
            '(' => depth += 1,
            ')' => {
                assert!(depth > 0, "unbalanced ')' in line: {line}");
                depth -= 1;
            }
            _ if depth == 0 => out.push(ch),
            _ => {}
        }
    }
    assert_eq!(depth, 0, "unbalanced '(' in line: {line}");
    out
}

/// Validate that every emitted word belongs to the documented Hurco WinMax ISNC set.
pub fn assert_isnc_words(program: &str) {
    let allowed_g = [
        "G0", "G00", "G1", "G01", "G2", "G02", "G3", "G03", "G4", "G04", "G10", "G17", "G18",
        "G19", "G20", "G21", "G28", "G31", "G40", "G41", "G42", "G43", "G49", "G53", "G54",
        "G54.1", "G55", "G56", "G57", "G58", "G59", "G68", "G69", "G73", "G80", "G81", "G82",
        "G83", "G84", "G84.2", "G84.3", "G85", "G86", "G89", "G90", "G91", "G94", "G98", "G99",
    ];
    let allowed_m = [
        "M0", "M00", "M1", "M01", "M2", "M02", "M3", "M03", "M4", "M04", "M5", "M05", "M6",
        "M06", "M7", "M07", "M8", "M08", "M9", "M09", "M30",
    ];
    let lines: Vec<&str> = program.lines().collect();
    assert!(lines.len() >= 3, "program too short");
    assert_eq!(lines.first().copied(), Some("%"), "missing leading %");
    assert_eq!(lines.last().copied(), Some("%"), "missing trailing %");

    for (idx, raw) in lines.iter().enumerate() {
        let code = strip_comments(raw);
        let trimmed = code.trim();
        if trimmed.is_empty() || trimmed == "%" {
            continue;
        }
        if trimmed.starts_with("IF[") || trimmed.starts_with('#') {
            assert!(
                trimmed.starts_with("IF[#506") || trimmed.starts_with("IF[ABS[") || trimmed.starts_with("#3000=") || trimmed.starts_with("#1"),
                "unexpected macro line {}: {raw}",
                idx + 1
            );
            continue;
        }
        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        let has_dwell = tokens.iter().any(|t| matches!(*t, "G4" | "G04" | "G82" | "G89"));
        let has_ext_or_g10 = tokens.iter().any(|t| matches!(*t, "G54.1" | "G10"));

        for w in tokens {
            if let Some(rest) = w.strip_prefix('G') {
                assert!(allowed_g.contains(&w), "line {}: undocumented G-code {w}: {raw}", idx + 1);
                let _ = rest;
            } else if w.starts_with('M') {
                assert!(allowed_m.contains(&w), "line {}: undocumented M-code {w}: {raw}", idx + 1);
            } else if let Some(n) = w.strip_prefix('O') {
                let v: u32 = n.parse().unwrap_or_else(|_| panic!("bad O word {w}"));
                assert!((1..=9999).contains(&v), "line {}: O word out of 1..9999: {w}", idx + 1);
            } else if let Some(n) = w.strip_prefix('N') {
                let _: u32 = n.parse().unwrap_or_else(|_| panic!("bad N word {w}"));
            } else if let Some(p) = w.strip_prefix('P') {
                if has_dwell {
                    assert!(p.contains('.'), "line {}: dwell P must include decimal seconds: {raw}", idx + 1);
                    let _: f64 = p.parse().unwrap_or_else(|_| panic!("bad dwell P {w}"));
                } else {
                    assert!(has_ext_or_g10, "line {}: unexpected P word {w}: {raw}", idx + 1);
                    let v: u32 = p.parse().unwrap_or_else(|_| panic!("bad integer P {w}"));
                    assert!((1..=93).contains(&v), "line {}: G54.1/G10 P out of range: {w}", idx + 1);
                }
            } else if let Some(l) = w.strip_prefix('L') {
                assert!(tokens_contain_g10(trimmed) && l == "2", "line {}: unexpected L word {w}: {raw}", idx + 1);
            } else if let Some(val) = w.strip_prefix(['X', 'Y', 'Z', 'I', 'J', 'K', 'R', 'Q']) {
                if has_ext_or_g10 && val.starts_with('[') {
                    assert!(val.ends_with(']'), "line {}: malformed G10 macro arg {w}: {raw}", idx + 1);
                } else {
                    assert!(val.contains('.'), "line {}: coordinate word {w} missing decimal point: {raw}", idx + 1);
                    let _: f64 = val.parse().unwrap_or_else(|_| panic!("bad coord {w}"));
                }
            } else if let Some(val) = w.strip_prefix(['F', 'S']) {
                let _: f64 = val.parse().unwrap_or_else(|_| panic!("bad rate {w}"));
            } else if let Some(val) = w.strip_prefix(['T', 'H', 'D']) {
                let _: u32 = val.parse().unwrap_or_else(|_| panic!("bad int word {w}"));
            } else {
                panic!("line {}: unrecognized WinMax ISNC word {w}: {raw}", idx + 1);
            }
        }
    }
}

fn tokens_contain_g10(line: &str) -> bool {
    line.split_whitespace().any(|t| t == "G10")
}

fn cycle_stream(kind: CycleKind, dwell: f64, peck: f64, wcs: i64, spindle_ccw: bool) -> String {
    let mut recs = vec![
        Record::Param { name: "SpindleSpeed".into(), value: json!(1000.0) },
        Record::Param { name: "ClearancePlane".into(), value: json!(15.0) },
        Record::Param { name: "RetractPlane".into(), value: json!(2.0) },
        Record::Param { name: "WorkOffset".into(), value: json!(wcs) },
    ];
    if spindle_ccw {
        recs.push(Record::Param { name: "MillSpindle".into(), value: json!("reverse") });
    }
    recs.extend([
        Record::Tool { number: 3, name: "Cycled Tool".into() },
        Record::Section { begin: true },
        Record::Move(MoveRec {
            kind: MoveKind::Rapid,
            x: 10.0,
            y: 20.0,
            z: 2.0,
            feed: None,
            center: None,
            ccw: false,
            plane: Default::default(),
            abc: None,
        }),
        Record::Cycle(CycleRec {
            kind,
            x: 10.0,
            y: 20.0,
            r_z: 2.0,
            bottom_z: -12.0,
            entry_z: 0.0,
            peck,
            dwell,
            first_move: 1,
            feed: Some(250.0),
        }),
        Record::Section { begin: false },
    ]);
    cnc::serialize(&recs)
}

#[test]
fn posts_vm10i_contour_pocket_drill_peck_tap_two_tools_two_offsets() {
    let stream = "; HSMW-CNC v1
PARAM Coolant flood
PARAM WorkOffset 1
PARAM ClearancePlane 25
TOOL 1 6mm Endmill
PARAM SpindleSpeed 8000
SECTION
Rapid X 0 Y 0 Z 5000
Feed X 20000 Y 0 Z -2000 F 600000
Arc CW X 20000 Y 10000 Z -2000 F 600000 I 20000 J 5000
Rapid X 20000 Y 10000 Z 25000
ENDSECTION
TOOL 2 6mm Drill
PARAM WorkOffset 1
PARAM ClearancePlane 25
PARAM RetractPlane 2
PARAM SpindleSpeed 3000
SECTION
Rapid X 10000 Y 10000 Z 2000
CYCLE DRILL X 10000 Y 10000 Z -10000 R 2000 F 240000 AT 4
CYCLE DEEP X 30000 Y 10000 Z -15000 R 2000 Q 3000 F 180000 AT 4
ENDSECTION
TOOL 3 M6 Tap
PARAM WorkOffset 2
PARAM ClearancePlane 25
PARAM RetractPlane 2
PARAM SpindleSpeed 600
SECTION
Rapid X 15000 Y 15000 Z 2000
CYCLE TAP X 15000 Y 15000 Z -12000 R 2000 F 600000 AT 5
ENDSECTION
";
    let out = engine::run(&cps(), &vm10i(), stream).expect("posts multi-op VM10i stream");
    assert!(out.warnings.is_empty(), "warnings: {:?}", out.warnings);
    let p = &out.program;
    assert_isnc_words(p);
    assert!(p.starts_with("%\nO0001 ("), "header:\n{p}");
    assert!(p.contains("\nG21\n"), "metric ISNC G21:\n{p}");
    assert!(p.contains("G90 G17 G94 G54 G40 G49 G80\n"), "safe start:\n{p}");
    assert!(!p.contains("G91.1"), "WinMax ISNC has no G91.1:\n{p}");
    assert!(p.contains("T1 M6\n") && p.contains("G43 H1\n"), "T1 tool change:\n{p}");
    assert!(p.contains("T2 M6\n") && p.contains("G43 H2\n"), "T2 tool change:\n{p}");
    assert!(p.contains("T3 M6\n") && p.contains("G43 H3\n"), "T3 tool change:\n{p}");
    assert!(p.contains("G55\n") || p.contains("G55 "), "second offset G55:\n{p}");
    assert!(p.contains("G02 X20. Y10. I0. J5."), "incremental arc center I0 J5:\n{p}");
    assert!(p.contains("G81 ") && p.contains("G83 ") && p.contains("Q3.") && p.contains("G84.2 "), "cycles:\n{p}");
    assert!(p.contains("G91 G28 Z0.") && p.contains("G28 X0. Y0.") && p.contains("M30\n%"), "footer:\n{p}");
}

fn assert_refused(res: Result<engine::PostOutput, camcraft_post::post::PostError>, needle: &str, ctx: &str) {
    match res {
        Err(e) => assert!(e.to_string().to_lowercase().contains(&needle.to_lowercase()), "{ctx}: {e}"),
        Ok(out) => {
            assert!(!out.warnings.is_empty(), "{ctx}: expected refusal warning, got Ok:\n{}", out.program);
            assert!(
                out.warnings.iter().any(|w| w.to_lowercase().contains(&needle.to_lowercase())),
                "{ctx}: warnings {:?} missing '{needle}'",
                out.warnings
            );
        }
    }
}

#[test]
fn inch_mode_and_end_units_write_isnc_g20_g21() {
    let stream = "; HSMW-CNC v1
PARAM SpindleSpeed 4000
PARAM ClearancePlane 25.4
TOOL 1 EM-INCH
SECTION
Rapid X 25400 Y 50800 Z 25400
Feed X 50800 Y 50800 Z -6350 F 508000
ENDSECTION
";
    let out = engine::run_with(&cps(), &vm10i(), stream, &json!({ "Units": "inch" })).expect("inch stream");
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    assert_isnc_words(&out.program);
    assert!(out.program.contains("\nG20\n"), "inch ISNC G20:\n{}", out.program);
    assert!(out.program.contains("X2. Y2."), "inch converted coords:\n{}", out.program);
    assert!(!out.program.contains("G70") && !out.program.contains("G71"), "never BNC G70/G71");

    let end_g21 = engine::run_with(&cps(), &vm10i(), stream, &json!({ "Units": "inch", "EndUnits": "mm" })).expect("endUnits mm");
    assert!(end_g21.warnings.is_empty(), "{:?}", end_g21.warnings);
    assert_isnc_words(&end_g21.program);
    assert!(end_g21.program.contains("\nG21\nM30\n"), "endUnits G21 before M30:\n{}", end_g21.program);
}

#[test]
fn dwell_p_uses_decimal_seconds_on_g82_and_g89() {
    for (kind, g, dwell, want) in [
        (CycleKind::Spot, "G82", 1.0, "P1."),
        (CycleKind::Drill, "G82", 0.5, "P0.5"),
        (CycleKind::BackBore, "G89", 0.25, "P0.25"),
        (CycleKind::Ream, "G89", 0.75, "P0.75"),
    ] {
        let out = engine::run(&cps(), &vm10i(), &cycle_stream(kind, dwell, 0.0, 1, false)).expect("posts");
        assert!(out.warnings.is_empty(), "{:?}", out.warnings);
        assert_isnc_words(&out.program);
        assert!(out.program.contains(&format!("{g} ")) && out.program.contains(want), "{g} {want}:\n{}", out.program);
    }
}

#[test]
fn tapping_modes_and_rigid_tapping_guard() {
    let rh = engine::run(&cps(), &vm10i(), &cycle_stream(CycleKind::Tap, 0.0, 0.0, 1, false)).expect("RH rigid tap");
    assert!(rh.warnings.is_empty(), "{:?}", rh.warnings);
    assert_isnc_words(&rh.program);
    assert!(rh.program.contains("G84.2 "), "RH rigid tap G84.2:\n{}", rh.program);

    let lh = engine::run(&cps(), &vm10i(), &cycle_stream(CycleKind::Tap, 0.0, 0.0, 1, true)).expect("LH rigid tap");
    assert!(lh.warnings.is_empty(), "{:?}", lh.warnings);
    assert_isnc_words(&lh.program);
    assert!(lh.program.contains("G84.3 "), "LH rigid tap G84.3:\n{}", lh.program);

    let fl = engine::run(&cps(), &vm10i(), &cycle_stream(CycleKind::FloatingTap, 0.0, 0.0, 1, false)).expect("floating tap");
    assert!(fl.warnings.is_empty(), "{:?}", fl.warnings);
    assert_isnc_words(&fl.program);
    assert!(fl.program.contains("G84 ") && !fl.program.contains("G84.2"), "floating tap G84:\n{}", fl.program);

    let mut no_rigid = vmc();
    no_rigid["rigidTapping"] = json!(false);
    let res = engine::run(&cps(), &no_rigid, &cycle_stream(CycleKind::Tap, 0.0, 0.0, 1, false));
    assert_refused(res, "rigid", "must refuse rigid tap when machine.rigidTapping is false");
    let fl_ok = engine::run(&cps(), &no_rigid, &cycle_stream(CycleKind::FloatingTap, 0.0, 0.0, 1, false))
        .expect("floating tap remains allowed when rigidTapping is false");
    assert!(fl_ok.warnings.is_empty(), "{:?}", fl_ok.warnings);
}

#[test]
fn extended_work_offsets_and_safe_retract_and_o_word() {
    for (wcs, want) in [(1, "G54"), (6, "G59"), (7, "G54.1 P1"), (99, "G54.1 P93")] {
        let out = engine::run(&cps(), &vm10i(), &cycle_stream(CycleKind::Drill, 0.0, 0.0, wcs, false)).expect("wcs");
        assert!(out.warnings.is_empty(), "{:?}", out.warnings);
        assert_isnc_words(&out.program);
        assert!(out.program.contains(want), "wcs {wcs} -> {want}:\n{}", out.program);
    }
    let res_100 = engine::run(&cps(), &vm10i(), &cycle_stream(CycleKind::Drill, 0.0, 0.0, 100, false));
    assert_refused(res_100, "99", "wcs > 99 refused");

    let mut g53_mch = vm10i();
    g53_mch["safeRetractZ"] = json!(0.0);
    let g53 = engine::run_with(&cps(), &g53_mch, &cycle_stream(CycleKind::Drill, 0.0, 0.0, 1, false), &json!({ "safeRetract": "G53" }))
        .expect("G53 retract");
    assert!(g53.warnings.is_empty(), "{:?}", g53.warnings);
    assert_isnc_words(&g53.program);
    assert!(g53.program.contains("G53 G00 Z0.\n"), "G53 retract:\n{}", g53.program);

    let mut no_retract_z = vm10i();
    no_retract_z["safeRetractZ"] = Value::Null;
    let res_g53 = engine::run_with(&cps(), &no_retract_z, &cycle_stream(CycleKind::Drill, 0.0, 0.0, 1, false), &json!({ "safeRetract": "G53" }));
    assert_refused(res_g53, "safeRetractZ", "G53 without safeRetractZ refused");

    let o42 = engine::run_program_with(
        &cps(),
        &vm10i(),
        &cycle_stream(CycleKind::Drill, 0.0, 0.0, 1, false),
        &json!({}),
        &ProgramId { number: 42, numbered: true, comment: String::new() },
    )
    .expect("O0042");
    assert!(o42.program.contains("\nO0042 ("));
    let res_o = engine::run_program_with(
        &cps(),
        &vm10i(),
        &cycle_stream(CycleKind::Drill, 0.0, 0.0, 1, false),
        &json!({}),
        &ProgramId { number: 10000, numbered: true, comment: String::new() },
    );
    assert_refused(res_o, "O0001..O9999", "O10000 refused");
}

#[test]
fn cycles_refuse_illegal_dwell_and_tilt_and_zero_spindle() {
    for (kind, peck) in [
        (CycleKind::Peck, 2.0),
        (CycleKind::Deep, 3.0),
        (CycleKind::Tap, 0.0),
        (CycleKind::FloatingTap, 0.0),
        (CycleKind::Bore, 0.0),
    ] {
        let res = engine::run(&cps(), &vm10i(), &cycle_stream(kind, 0.001, peck, 1, false));
        assert_refused(res, "dwell", &format!("{kind:?} illegal dwell refused"));
    }

    let tilted = "; HSMW-CNC v1
PARAM SpindleSpeed 3000
PARAM ClearancePlane 20
TOOL 1 EM
SECTION
Rapid X 0 Y 0 Z 20000
Rapid X 10000 Y 0 Z 20000 A 45.0 B 0.0 C 0.0
ENDSECTION
";
    let res_tilt = engine::run(&cps(), &vm10i(), tilted);
    assert_refused(res_tilt, "Hurco WinMax (NC) post", "tilted 3-axis move refused");
}

#[test]
fn comp_rotation_and_probing_and_smoothing() {
    let stream = "; HSMW-CNC v1
PARAM OperationName CompRot
PARAM SpindleSpeed 5000
PARAM ClearancePlane 10
PARAM WcsYaw 15.0
PARAM WcsRotationMode control
PARAM Smoothing 0.05
TOOL 1 EM6
SECTION
Rapid X -5000 Y 0 Z 10000
Plunge X -5000 Y 0 Z -2000 F 200000
COMP LEFT 3000
Feed X 0 Y 0 Z -2000 F 400000
Feed X 20000 Y 0 Z -2000 F 400000
COMP OFF
Feed X 25000 Y 0 Z -2000 F 400000
ENDSECTION
";
    let out = engine::run_with(&cps(), &vm10i(), stream, &json!({ "WcsRotation": "control" })).expect("comp + rot + smooth");
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    assert_isnc_words(&out.program);
    assert!(out.program.contains("G68 X0. Y0. R15.\n"), "G68:\n{}", out.program);
    assert!(out.program.contains("G41 ") && out.program.contains("D1 ") && out.program.contains("G40 "), "comp:\n{}", out.program);
    assert!(out.program.contains("G69\n"), "G69:\n{}", out.program);
    assert!(out.program.contains("(SMOOTHING NOT WRITTEN - G05.2/G05.3 REQUIRE NC OPTION)"), "smooth note:\n{}", out.program);

    let probe_ok = read("crates/post/tests/recorded/streams/probe.cnc");
    let p_out = engine::run(&cps(), &vm10i(), &probe_ok).expect("probe.cnc posts");
    assert!(p_out.warnings.is_empty(), "{:?}", p_out.warnings);
    assert_isnc_words(&p_out.program);
    assert!(p_out.program.contains("G31 ") && p_out.program.contains("G10 L2 P2 "), "probe:\n{}", p_out.program);

    let probe_bad = probe_ok.replacen("FOUND SET 1", "FOUND SET 7", 1);
    let res_probe = engine::run(&cps(), &vm10i(), &probe_bad);
    assert_refused(res_probe, "not one of G54..G59", "FOUND SET 7 refused");
}

#[test]
fn every_hurco_winmax_golden_writes_only_documented_isnc_words() {
    let mut dirs = vec![
        repo("crates/post/tests/recorded/golden/hurco-winmax"),
        repo("crates/app/tests/golden/nc/hurco-winmax"),
    ];
    let mut checked = 0usize;
    while let Some(d) = dirs.pop() {
        for entry in std::fs::read_dir(&d).unwrap_or_else(|e| panic!("{}: {e}", d.display())).flatten() {
            let p = entry.path();
            if p.extension().is_some_and(|x| x == "nc") {
                assert_isnc_words(&std::fs::read_to_string(&p).expect("golden .nc"));
                checked += 1;
            }
        }
    }
    let synth = repo("crates/app/tests/golden/nc/synth_hurco-winmax.nc");
    assert_isnc_words(&std::fs::read_to_string(&synth).expect("synth_hurco-winmax.nc"));
    checked += 1;
    assert!(checked >= 20, "expected >= 20 Hurco WinMax .nc goldens, checked {checked}");
}
