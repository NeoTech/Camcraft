//! #4088: the machine file states the highest work offset its control holds
//! (`controller.work_offset_max`), and the post engine refuses a higher one by
//! setup name. Our own design, on the installation-fact pattern of #3181.
//! Covers camcraft_post::machine::installation, camcraft_post::post::work_offset
//! and assets/machines/fehlmann-picomax55.mch.

use camcraft_post::{
    machine::parse_mch,
    post::{engine, machine},
};
use serde_json::{Value, json};

const PICOMAX: &str = include_str!("../../../../assets/machines/fehlmann-picomax55.mch");
const POSTS: [(&str, &str); 2] = [
    ("heidenhain.cps", include_str!("../../../../assets/posts/heidenhain.cps")),
    ("heidenhain-klartext.cps", include_str!("../../../../assets/posts/heidenhain-klartext.cps")),
];
const STREAM: &str = include_str!("../klartext/contour.cnc");

/// The warnings and the error of one post at work offset `n`, as one text.
fn post(cps: &str, input: &Value, n: u32) -> Result<engine::PostOutput, String> {
    let stream = STREAM.replace("PARAM WorkOffset 2", &format!("PARAM WorkOffset {n}"));
    assert_ne!(stream, STREAM, "the recorded stream states WorkOffset 2");
    engine::run(cps, input, &stream).map_err(|e| e.to_string())
}

fn refusal(cps: &str, input: &Value, n: u32) -> String {
    match post(cps, input, n) {
        Ok(out) => out.warnings.join("\n"),
        Err(e) => e,
    }
}

/// The Picomax preset table on the delivery CD has rows 0..5 (PRESET.PR), so
/// preset 5 posts and preset 6 is refused by setup name, by both Heidenhain
/// posts, through the raw file and through the parsed mirror.
#[test]
fn heidenhain_posts_refuse_a_preset_above_the_machine_preset_table() {
    let file: Value = serde_json::from_str(PICOMAX).expect("machine file");
    assert_eq!(file["controller"]["work_offset_max"], 5, "the Picomax states its last preset row");
    let parsed = parse_mch(PICOMAX).expect("machine reads");
    assert_eq!(parsed.work_offset_max, Some(5));
    let mirrored = machine::mirror(&parsed);
    for (name, cps) in POSTS {
        for input in [&file, &mirrored] {
            let at_limit = post(cps, input, 5).unwrap_or_else(|e| panic!("{name}: preset 5 must post: {e}"));
            assert!(at_limit.warnings.is_empty(), "{name}: {:?}", at_limit.warnings);
            let over = refusal(cps, input, 6);
            assert!(
                over.contains("setup Setup1: uses work offset 6") && over.contains("controller.work_offset_max"),
                "{name}: {over}"
            );
        }
    }
}

/// A machine that states no maximum keeps the post's own limit (6 for the ISO
/// Heidenhain post), and a maximum above the post's own limit does not raise it.
#[test]
fn an_unstated_or_larger_machine_maximum_keeps_the_post_limit() {
    let (name, cps) = POSTS[0];
    let out = post(cps, &Value::Null, 6).unwrap_or_else(|e| panic!("{name}: preset 6 must post: {e}"));
    assert!(out.warnings.is_empty(), "{name}: {:?}", out.warnings);
    let mut file: Value = serde_json::from_str(PICOMAX).expect("machine file");
    file["controller"]["work_offset_max"] = json!(99);
    let mirrored = machine::mirror(&parse_mch(&file.to_string()).expect("machine reads"));
    for input in [&file, &mirrored] {
        let at_six = post(cps, input, 6).unwrap_or_else(|e| panic!("{name}: preset 6 must post: {e}"));
        assert!(at_six.warnings.is_empty(), "{name}: {:?}", at_six.warnings);
        let over = refusal(cps, input, 7);
        assert!(over.contains("work offsets 1-6 only"), "{name}: {over}");
    }
}

/// A malformed maximum is refused by name at the machine reader and at the raw
/// engine, never read as no limit.
#[test]
fn a_malformed_work_offset_max_is_refused_by_name() {
    for bad in [json!(0), json!(-1), json!(2.5), json!("5"), json!(256)] {
        let mut file: Value = serde_json::from_str(PICOMAX).expect("machine file");
        file["controller"]["work_offset_max"] = bad.clone();
        let read = parse_mch(&file.to_string()).expect_err("the reader refuses it");
        assert!(read.contains("controller.work_offset_max"), "{bad}: {read}");
        let raw = refusal(POSTS[0].1, &file, 1);
        assert!(raw.contains("controller.work_offset_max"), "{bad}: {raw}");
    }
}
