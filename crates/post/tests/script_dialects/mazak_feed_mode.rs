//! #4343: the Mazak post states the feed mode. Every F word it writes is a
//! feed per minute (mm/min or in/min), and a control can be left in G95
//! (feed per revolution) by its power-on setting, an earlier program or
//! MDI. So each section preamble writes G94 before the section's first F
//! word, in a metric and in an inch program, and every recorded Mazak
//! golden that feeds states G94 first.
//!
//! Provenance: RS-274 / ISO 6983 (G94 feed per minute, G95 feed per
//! revolution, both modal); our own synthetic streams and recorded goldens.
//!
//! Gate map (#4343): covers assets/posts/mazak.cps (startPreamble),
//! camcraft_post::post::engine.

use camcraft_post::post::engine;
use std::path::Path;

fn root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../")
}

fn read(rel: &str) -> String {
    let p = root().join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{rel}: {e}")).replace("\r\n", "\n")
}

/// Two milling sections with their own tools: each feeds at F300.
fn stream() -> String {
    let section = |op: &str, tool: u32| {
        format!(
            "PARAM OperationName {op}
PARAM SpindleSpeed 3000.000000
PARAM RetractPlane 20.000000
TOOL {tool} MILL-{tool}
SECTION
Rapid X 10000 Y 10000 Z 45000
Rapid X 10000 Y 10000 Z 5000
Feed X 10000 Y 10000 Z -1000 F 300000
Feed X 30000 Y 10000 Z -1000 F 300000
Rapid X 30000 Y 10000 Z 45000
ENDSECTION
"
        )
    };
    format!(
        "; HSMW-CNC v1
PARAM WorkOffset G54
PARAM OriginX 0.000000
PARAM OriginY 0.000000
PARAM OriginZ 0.000000
PARAM ClearancePlane 45.000000
{}{}",
        section("A", 1),
        section("B", 2)
    )
}

/// The words of a block, its comment dropped.
fn words(line: &str) -> Vec<&str> {
    line.split('(').next().unwrap_or("").split_whitespace().collect()
}

/// Each F word comes after a G94 that no G95 cancelled since; returns the
/// number of section preambles (blocks with G17) that state G94.
fn assert_feed_per_minute(program: &str, what: &str) -> usize {
    let mut g94 = false;
    let mut preambles = 0;
    for line in program.lines() {
        let ws = words(line);
        if ws.contains(&"G95") {
            g94 = false;
        }
        if ws.contains(&"G94") {
            g94 = true;
        }
        if ws.contains(&"G17") && ws.contains(&"G94") {
            preambles += 1;
        }
        let feeds = ws.iter().any(|w| w.len() > 1 && w.starts_with('F') && w[1..].parse::<f64>().is_ok());
        assert!(!feeds || g94, "{what}: `{line}` feeds before any G94:\n{program}");
    }
    preambles
}

#[test]
fn every_mazak_section_states_g94_before_its_first_feed() {
    let mch: serde_json::Value =
        serde_json::from_str(&read("assets/machines/3axis-vmc.mch")).expect("machine json");
    let cps = read("assets/posts/mazak.cps");
    for (what, options) in [("metric", serde_json::Value::Null), ("inch", serde_json::json!({ "Units": "inch" }))] {
        let out = engine::run_with(&cps, &mch, &stream(), &options).expect("mazak posts");
        assert!(
            out.program.lines().any(|l| words(l) == ["G17", "G90", "G94", "G54", "G40", "G49", "G80"]),
            "{what}: the preamble states G94 with its modal words:\n{}",
            out.program
        );
        assert_eq!(assert_feed_per_minute(&out.program, what), 2, "{what}: both sections state G94:\n{}", out.program);
    }
}

/// The recorded goldens: each Mazak program that feeds states G94 first.
#[test]
fn every_recorded_mazak_program_feeds_per_minute() {
    let dir = root().join("crates/post/tests/recorded/golden/mazak");
    let mut seen = 0;
    for e in std::fs::read_dir(&dir).expect("mazak goldens") {
        let p = e.expect("entry").path();
        if p.extension().is_some_and(|x| x == "nc") {
            let text = std::fs::read_to_string(&p).expect("golden").replace("\r\n", "\n");
            assert_feed_per_minute(&text, &p.display().to_string());
            seen += 1;
        }
    }
    assert!(seen >= 5, "the recorded Mazak goldens were read: {seen}");
}
