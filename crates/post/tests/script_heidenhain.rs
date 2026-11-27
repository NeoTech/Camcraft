//! The `heidenhain.cps` script suite (#926): the post binary on every recorded
//! edge-case stream (`recorded/streams/*.cnc`), its program or its named
//! refusal compared with the golden in `recorded/golden/heidenhain/`. No app, no
//! toolpath generation: the inputs are recorded, and their generators keep
//! them current (`recorded_streams.rs`, app's `recorded_post_streams.rs`).
//!
//! Regenerate after an intentional change, then read the diff:
//! `WRITE_GOLDEN=1 cargo test -p camcraft-post --test script_heidenhain`
//!
//! Gate map (M4.4): covers assets/posts/heidenhain.cps, camcraft_post::post::engine,
//! camcraft_post::post::cnc, the post binary.

#[macro_use]
mod recorded;

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

    const STREAM: &str = include_str!("klartext/contour.cnc");
    const MCH: &str = include_str!("../../../assets/machines/fehlmann-picomax55.mch");

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
}
