//! The `heidenhain.cps` script suite (#926): the post binary on every recorded
//! edge-case stream (`recorded/streams/*.cnc`), its program or its named
//! refusal compared with the golden in `recorded/golden/heidenhain/`. No app, no
//! toolpath generation: the inputs are recorded, and their generators keep
//! them current (`recorded_streams.rs`, app's `recorded_post_streams.rs`).
//!
//! Regenerate after an intentional change, then read the diff:
//! `WRITE_GOLDEN=1 cargo test -p camcraft-post --test script_goldens script_heidenhain::`
//!
//! Gate map (M4.4): covers assets/posts/heidenhain.cps, camcraft_post::post::engine,
//! camcraft_post::post::cnc, the post binary.

use crate::recorded;

post_suite!("heidenhain.cps");

// #4085: the DIN/ISO post on the Fehlmann Picomax 55, whose installation
// states a manual tool change at the upper change position. The program
// halts at M06 there; a T block alone does not stop.
mod picomax {
    use super::recorded;
    use camcraft_post::{
        machine::parse_mch,
        post::{engine, machine},
    };
    use serde_json::Value;

    const STREAM: &str = include_str!("../klartext/contour.cnc");
    const MCH: &str = include_str!("../../../../assets/machines/fehlmann-picomax55.mch");

    fn picomax() -> Value {
        machine::mirror(&parse_mch(MCH).expect("the Picomax machine file"))
    }

    fn script(stem: &str) -> String {
        std::fs::read_to_string(recorded::repo().join("assets/posts").join(format!("{stem}.cps")))
            .expect("shipped post")
    }

    fn two_tools() -> String {
        let second = STREAM.replace("TOOL 2 FLAT-6", "TOOL 3 FLAT-6").replace("Arc contour", "Second contour");
        format!("{STREAM}{second}")
    }

    /// The block bodies without their N numbers.
    fn bodies(nc: &str) -> Vec<&str> {
        nc.lines()
            .filter_map(|l| l.strip_prefix('N').and_then(|l| l.split_once(' ')).map(|(_, b)| b.trim_end_matches('*')))
            .collect()
    }

    fn is_motion(b: &str) -> bool {
        ["G00", "G01", "G02", "G03"].iter().any(|g| b.split_whitespace().next() == Some(g))
    }

    #[test]
    fn din_iso_retracts_and_stops_before_the_second_tool_moves() {
        let out = engine::run(&script("heidenhain"), &picomax(), &two_tools()).expect("posts");
        assert!(out.warnings.is_empty(), "{:?}", out.warnings);
        recorded::compare("heidenhain.cps", "picomax_two_tools", "nc", &out.program);
        let b = bodies(&out.program);
        let t2 = b.iter().position(|l| l.starts_with("T2 G17")).expect("first tool call");
        let t3 = b.iter().position(|l| l.starts_with("T3 G17")).expect("second tool call");
        // The first change: the tool in the spindle is unknown, so a stop and no move.
        // M6, then M0: the halt does not depend on the M06 machine parameter (#4114).
        let first_m6 = b[..t2].iter().rposition(|l| *l == "M6").expect("M6 before the first tool");
        let first_stop = b[..t2].iter().rposition(|l| *l == "M0").expect("a stop before the first tool");
        assert!(first_m6 < first_stop, "{b:?}");
        assert!(!b[..t2].iter().any(|l| is_motion(l)), "no move before the first tool call: {b:?}");
        assert!(b[first_stop + 1..t2].iter().all(|l| l.starts_with(';')), "{b:?}");
        // The second change: spindle off, the M92 retract under the old tool call, the stop,
        // then the new tool call - and no move with the old tool after the stop.
        let between = &b[t2..t3];
        let off = between.iter().rposition(|l| *l == "M5").expect("spindle stops");
        let retract = between.iter().position(|l| *l == "G00 G90 Z+0 M92").expect("M92 retract");
        let m6 = between.iter().rposition(|l| *l == "M6").expect("M6");
        let stop = between.iter().rposition(|l| *l == "M0").expect("M0 stop");
        assert!(off < retract && retract < m6 && m6 < stop, "{between:?}");
        assert!(!between[retract + 1..].iter().any(|l| is_motion(l)), "{between:?}");
        assert!(between[stop + 1..].iter().all(|l| l.starts_with(';')), "{between:?}");
        assert_eq!(b.iter().filter(|l| **l == "M6").count(), 2, "{b:?}");
        assert_eq!(b.iter().filter(|l| **l == "M0").count(), 2, "{b:?}");
    }

    /// The M92 retract and the stop are verified for one installation only
    /// (#4114): a manual-retract-top machine with no controller identity, a
    /// different identity, or a different machine model is refused by name.
    #[test]
    fn din_iso_refuses_a_manual_change_machine_it_was_not_verified_for() {
        type Edit = fn(&mut Value);
        let cases: [(&str, Edit); 4] = [
            ("no identity", |f| {
                f["controller"].as_object_mut().expect("controller").remove("identity");
            }),
            ("other NC software", |f| f["controller"]["identity"]["nc_software"] = "340490-05".into()),
            ("other control", |f| f["controller"]["identity"]["model"] = "TNC 426".into()),
            ("other machine", |f| f["general"]["model"] = "Knee Mill (iTNC 530)".into()),
        ];
        for (what, edit) in cases {
            let mut file: Value = serde_json::from_str(MCH).expect("JSON");
            edit(&mut file);
            let other = machine::mirror(&parse_mch(&file.to_string()).expect("machine"));
            let why = engine::run(&script("heidenhain"), &other, &two_tools()).err().map(|e| e.to_string()).unwrap_or_default();
            assert!(
                why.contains("Heidenhain TNC: the manual-retract-top tool change")
                    && why.contains("is verified only on the Fehlmann Picomax 55 (iTNC 530) with iTNC 530 NC 340490-03"),
                "{what}: {why}"
            );
        }
    }

    #[test]
    fn the_same_tool_again_is_no_change_and_writes_no_stop() {
        let again = STREAM.replace("Arc contour", "Second contour");
        let out = engine::run(&script("heidenhain"), &picomax(), &format!("{STREAM}{again}")).expect("posts");
        assert!(out.warnings.is_empty(), "{:?}", out.warnings);
        let b = bodies(&out.program);
        assert_eq!(b.iter().filter(|l| **l == "M6").count(), 1, "{b:?}");
        assert!(!b.contains(&"G00 G90 Z+0 M92"), "{b:?}");
    }

    #[test]
    fn an_automatic_change_writes_no_stop() {
        let mut file: Value = serde_json::from_str(MCH).expect("JSON");
        file["tooling"]["tool_change"] = "changer".into();
        let changer = machine::mirror(&parse_mch(&file.to_string()).expect("machine"));
        let out = engine::run(&script("heidenhain"), &changer, &two_tools()).expect("posts");
        let b = bodies(&out.program);
        assert!(!b.contains(&"M6") && !b.iter().any(|l| l.contains("M92")), "{b:?}");
    }

    /// The engine law, over every shipped post on the one shipped manual-change
    /// machine: only a post that declares the mode passes the law.
    #[test]
    fn every_shipped_post_without_the_manual_change_is_refused_on_the_picomax() {
        let dir = recorded::repo().join("assets/posts");
        let mut stems: Vec<String> = std::fs::read_dir(&dir)
            .expect("assets/posts")
            .filter_map(|e| e.ok()?.file_name().to_str()?.strip_suffix(".cps").map(str::to_owned))
            .collect();
        stems.sort();
        let handles = ["heidenhain", "heidenhain-klartext"];
        for stem in &stems {
            let result = engine::run(&script(stem), &picomax(), &two_tools());
            if handles.contains(&stem.as_str()) {
                let out = result.unwrap_or_else(|e| panic!("{stem}: {e}"));
                assert!(out.warnings.is_empty(), "{stem}: {:?}", out.warnings);
            } else {
                let why = result.err().map(|e| e.to_string()).unwrap_or_default();
                assert!(
                    why.contains("the machine Fehlmann Picomax 55 (iTNC 530) changes tools by hand")
                        && why.contains("does not declare manualToolChange"),
                    "{stem}: {why}"
                );
            }
        }
        // A plain manual change states no change position: the DIN/ISO post is refused too.
        let mut file: Value = serde_json::from_str(MCH).expect("JSON");
        file["tooling"]["tool_change"] = "manual".into();
        let manual = machine::mirror(&parse_mch(&file.to_string()).expect("machine"));
        let why = engine::run(&script("heidenhain"), &manual, &two_tools()).err().map(|e| e.to_string()).unwrap_or_default();
        assert!(why.contains("the Heidenhain TNC post handles only the manual tool change modes manual-retract-top"), "{why}");
    }

    /// #4226: the G62 filter mode (P01) and the T range, by profile
    /// (docs/decisions/4226-g62-smoothing.md).
    mod g62 {
        use super::*;

        fn smoothed(tol: &str, mode: &str) -> String {
            STREAM.replace("PARAM Coolant off\n", &format!("PARAM Coolant off\nPARAM Smoothing {tol}\nPARAM SmoothingMode {mode}\n"))
        }

        /// The Picomax file with an edit; a stated control other than the
        /// Picomax's takes the automatic change (the manual change is verified
        /// for the Picomax control only).
        fn with(edit: impl Fn(&mut Value)) -> Value {
            let mut file: Value = serde_json::from_str(MCH).expect("JSON");
            edit(&mut file);
            machine::mirror(&parse_mch(&file.to_string()).expect("machine"))
        }

        fn control(model: &str, nc: &str) -> Value {
            with(|f| {
                f["tooling"]["tool_change"] = "changer".into();
                f["controller"]["identity"] = serde_json::json!({ "model": model, "nc_software": nc });
            })
        }

        fn g62_lines(m: &Value, tol: &str, mode: &str) -> (Vec<String>, Vec<String>) {
            let out = engine::run(&script("heidenhain"), m, &smoothed(tol, mode)).unwrap_or_else(|e| panic!("{tol} {mode}: {e}"));
            let lines = bodies(&out.program).into_iter().map(str::trim).filter(|b| b.starts_with("G62")).map(str::to_owned).collect();
            (lines, out.warnings)
        }

        fn refusal(m: &Value, tol: &str) -> String {
            engine::run(&script("heidenhain"), m, &smoothed(tol, "finish")).err().map(|e| e.to_string()).unwrap_or_default()
        }

        #[test]
        fn picomax_g62_stays_t_only_without_option_2() {
            let (lines, warnings) = g62_lines(&picomax(), "0.02", "finish");
            assert_eq!(lines, ["G62 T0.02", "G62"]);
            assert!(warnings.is_empty(), "{warnings:?}");
            let (lines, warnings) = g62_lines(&picomax(), "0.02", "rough");
            assert_eq!(lines, ["G62 T0.02", "G62"]);
            assert!(
                warnings.iter().any(|w| w.contains("smoothing mode rough is not written")
                    && w.contains("only with software option 2 (HSC machining)")),
                "{warnings:?}"
            );
        }

        #[test]
        fn g62_writes_the_finish_or_rough_mode_where_the_control_takes_it() {
            let opt2 = with(|f| f["controller"]["identity"]["installed_options"] = serde_json::json!(["2"]));
            let tnc640 = control("TNC 640", "340595-08");
            for m in [&opt2, &tnc640, &Value::Null] {
                let (fin, w) = g62_lines(m, "0.02", "finish");
                assert_eq!(fin, ["G62 T0.02 P01 0 P02 0", "G62"]);
                assert!(w.is_empty(), "{w:?}");
                let (rough, _) = g62_lines(m, "0.1", "rough");
                assert_eq!(rough, ["G62 T0.1 P01 1 P02 0", "G62"]);
            }
            // TNC 426/430: no documented P01 - T only, and a rough ask is told.
            let (rough, w) = g62_lines(&control("TNC 430", "280476-01"), "0.1", "rough");
            assert_eq!(rough, ["G62 T0.1", "G62"]);
            assert!(w.iter().any(|w| w.contains("has no documented G62 P01")), "{w:?}");
        }

        #[test]
        fn g62_refuses_a_t_outside_the_documented_range_by_name() {
            let tnc640_08 = control("TNC 640", "340595-08");
            assert_eq!(g62_lines(&tnc640_08, "10", "finish").0[0], "G62 T10 P01 0 P02 0");
            let why = refusal(&tnc640_08, "10.001");
            assert!(why.contains("smoothing tolerance T10.001 mm is above the largest T of TNC 640 NC 340595-08") && why.contains("(10 mm)"), "{why}");
            // 34059x-07 documents 0 to 99999.9999.
            assert_eq!(g62_lines(&control("TNC 640", "340595-07"), "12", "finish").0[0], "G62 T12 P01 0 P02 0");
            // The Picomax manual states no range: the narrowest documented one (10 mm).
            assert!(refusal(&picomax(), "12").contains("above the largest T of iTNC 530 NC 340490-03"));
            // A T that rounds to 0 is the builder's tolerance on the control.
            let why = refusal(&Value::Null, "0.0004");
            assert!(why.contains("writes T0 at 0.001 mm") && why.contains("refused, not posted"), "{why}");
            // TNC 410: no cycle 32.
            let why = refusal(&control("TNC 410", "286060-01"), "0.02");
            assert!(why.contains("has no tolerance cycle (cycle 32 / G62)"), "{why}");
        }
    }
}
