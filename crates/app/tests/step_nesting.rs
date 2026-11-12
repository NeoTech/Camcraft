//! #1075: a STEP record nested thousands of parentheses deep is refused by
//! name before ruststep's recursive parser can overflow the stack - on the
//! model import path and on the tool-body coordinate-system read. The deep
//! inputs are built here from box.step, not committed as fixtures.
//!
//! Gate map: covers app::geometry::import::step_depth, step and step_csys.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use camcraft_app::geometry::import::step_csys::read_tool_csys;
use camcraft_app::geometry::import::step_depth::{max_depth, MAX_NESTING};
use camcraft_app::geometry::import::{self, Format};

fn models() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/models")
}

/// box.step with one extra record `#999999` nested `depth` deep, put just
/// before the DATA section's ENDSEC; and the 1-based line it starts on.
fn box_with_deep_record(depth: usize, crlf: bool) -> (Vec<u8>, usize) {
    let text = std::fs::read_to_string(models().join("box.step")).expect("box.step reads");
    let text = text.replace("\r\n", "\n");
    let at = text.rfind("ENDSEC;").expect("box.step has a DATA ENDSEC");
    let line = text[..at].matches('\n').count() + 1;
    let inner = depth.saturating_sub(1);
    let record = format!(
        "#999999 = DEEP_LIST({}1.0{});\n",
        "(".repeat(inner),
        ")".repeat(inner)
    );
    let mut out = format!("{}{record}{}", &text[..at], &text[at..]);
    if crlf {
        out = out.replace('\n', "\r\n");
    }
    (out.into_bytes(), line)
}

fn refusal(data: &[u8]) -> String {
    match import::parse_model(data, Format::Step) {
        Ok(_) => String::from("<imported>"),
        Err(e) => e.to_string(),
    }
}

#[test]
fn a_record_nested_5000_deep_is_refused_by_name_quickly() {
    let (data, line) = box_with_deep_record(5000, false);
    let start = Instant::now();
    let msg = refusal(&data);
    let took = start.elapsed();
    assert!(msg.contains(&format!("deeper than {MAX_NESTING}")), "{msg}");
    assert!(msg.contains("entity #999999"), "{msg}");
    assert!(msg.contains(&format!("line {line}")), "{msg}");
    assert!(took < Duration::from_secs(1), "refusal took {took:?}");
}

#[test]
fn crlf_and_lf_files_name_the_same_line() {
    let (lf, line) = box_with_deep_record(5000, false);
    let (crlf, _) = box_with_deep_record(5000, true);
    let a = refusal(&lf);
    assert!(a.contains(&format!("line {line}")), "{a}");
    assert_eq!(a, refusal(&crlf));
}

#[test]
fn a_record_at_the_cap_still_imports() {
    let (data, _) = box_with_deep_record(MAX_NESTING, false);
    assert_eq!(max_depth(&data), MAX_NESTING);
    let mesh = import::parse_model(&data, Format::Step).expect("a record at the cap imports");
    let plain = import::load_model(&models().join("box.step")).expect("box.step imports");
    assert_eq!(
        mesh.facets.len(),
        plain.facets.len(),
        "the dead record must not change the mesh"
    );
    let (over, _) = box_with_deep_record(MAX_NESTING + 1, false);
    assert!(
        refusal(&over).contains("deeper than"),
        "one past the cap is refused"
    );
}

#[test]
fn the_tool_body_csys_read_is_guarded_too() {
    let (data, line) = box_with_deep_record(5000, false);
    let msg = read_tool_csys(&data)
        .err()
        .map(|e| e.to_string())
        .unwrap_or_default();
    assert!(
        msg.contains("entity #999999") && msg.contains(&format!("line {line}")),
        "{msg}"
    );
}

/// Every committed STEP fixture nests far below the cap (they reach 3).
#[test]
fn every_step_fixture_is_under_the_cap() {
    let mut seen = 0;
    for entry in std::fs::read_dir(models()).expect("testdata/models lists") {
        let path = entry.expect("dir entry").path();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase);
        if !matches!(ext.as_deref(), Some("step" | "stp")) {
            continue;
        }
        let depth = max_depth(&std::fs::read(&path).expect("fixture reads"));
        println!("{}: max depth {depth}", path.display());
        assert!(
            depth <= 8,
            "{} nests {depth} deep - re-check the cap's margin",
            path.display()
        );
        assert!(depth < MAX_NESTING);
        seen += 1;
    }
    assert!(seen >= 5, "found only {seen} STEP fixtures");
}
