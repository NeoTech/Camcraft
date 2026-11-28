//! #4342: Mazak probe branches have one unique N-word target, including numbered programs.
//! Provenance: the issue's Mazak EIA/ISO branch behavior and our recorded probe stream.
//! Gate map: covers assets/posts/mazak.cps (nextSeq, block, alarmUnless), camcraft_post::post::engine.

use crate::recorded;
use camcraft_post::post::engine;
use serde_json::{json, Value};
use std::collections::BTreeSet;

const MAZAK: &str = include_str!("../../../../assets/posts/mazak.cps");

fn post(stream: &str, options: Value) -> String {
    let output =
        engine::run_with(MAZAK, &Value::Null, stream, &options).expect("Mazak probe posts");
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    output.program
}

fn branch_targets(program: &str) -> Vec<u64> {
    program
        .lines()
        .filter_map(|line| line.split_once("GOTO"))
        .map(|(_, rest)| {
            rest.chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
                .parse()
                .expect("numeric GOTO target")
        })
        .collect()
}

fn assert_unique_numbers(program: &str) {
    let mut seen = BTreeSet::new();
    for line in program.lines() {
        for number in line
            .split_whitespace()
            .filter_map(|word| word.strip_prefix('N')?.parse::<u64>().ok())
        {
            assert!(
                seen.insert(number),
                "N{number} is used more than once:\n{program}"
            );
        }
    }
}

fn assert_branch_targets(program: &str) {
    let targets = branch_targets(program);
    assert!(
        !targets.is_empty(),
        "the fixture must emit probe alarm branches"
    );
    for target in targets {
        let label = format!("N{target}");
        assert_eq!(
            program.lines().filter(|line| *line == label).count(),
            1,
            "GOTO{target} must have exactly one standalone {label} target:\n{program}"
        );
    }
    for line in program.lines() {
        let count = line
            .split_whitespace()
            .filter(|word| {
                word.strip_prefix('N')
                    .is_some_and(|n| n.parse::<u64>().is_ok())
            })
            .count();
        assert!(count <= 1, "a block must not have two N words: {line}");
    }
}

#[test]
fn every_block_probe_labels_are_unambiguous_jump_targets() {
    let stream = recorded::input("probe_t7_bore_g55.cnc");
    let program = post(&stream, json!({ "showSequenceNumbers": "every-block" }));
    assert_branch_targets(&program);
    assert_unique_numbers(&program);
}

#[test]
fn every_block_probe_matches_its_golden() {
    let stream = recorded::input("probe_t7_bore_g55.cnc");
    let program = post(&stream, json!({ "showSequenceNumbers": "every-block" }));
    recorded::compare("mazak.cps", "probe_every_block", "nc", &program);
}

#[test]
fn nonadvancing_sequence_options_are_refused_by_name() {
    let stream = recorded::input("probe_t7_bore_g55.cnc");
    for increment in [0.0, -1.0, 0.1] {
        let options =
            json!({ "showSequenceNumbers": "every-block", "sequenceNumberIncrement": increment });
        let error = engine::run_with(MAZAK, &Value::Null, &stream, &options)
            .expect_err("nonpositive increment refuses");
        assert!(
            error
                .to_string()
                .contains("sequence number increment must be positive"),
            "{error}"
        );
    }
    let options = json!({ "showSequenceNumbers": "every-block", "sequenceNumberStart": 1e100 });
    let error = engine::run_with(MAZAK, &Value::Null, &stream, &options)
        .expect_err("nonadvancing number refuses");
    assert!(
        error
            .to_string()
            .contains("cannot advance sequence number safely"),
        "{error}"
    );
}

#[test]
fn probe_labels_never_collide_with_numbered_blocks() {
    let stream = recorded::input("probe_t7_bore_g55.cnc");
    for (mode, start, increment) in [
        ("every-block", 1, 1000),
        ("every-block", 8999, 1),
        ("every-block", 9000, 1),
        ("every-block", 9001, 1),
        ("every-block", 999999, 10000),
        ("tool-change", 9001, 1),
        ("op-start", 9001, 1),
        ("off", 9001, 1),
    ] {
        let program = post(
            &stream,
            json!({ "showSequenceNumbers": mode, "sequenceNumberStart": start, "sequenceNumberIncrement": increment }),
        );
        assert_unique_numbers(&program);
        assert_branch_targets(&program);
    }
}

#[test]
fn probe_labels_remain_unique_across_sections_and_word_spacing() {
    let stream = recorded::input("probe_t7_bore_g55.cnc");
    let two_sections = format!("{stream}\n{stream}");
    for separated in [false, true] {
        let program = post(
            &two_sections,
            json!({
                "showSequenceNumbers": "every-block",
                "sequenceNumberStart": 8999,
                "sequenceNumberIncrement": 1,
                "separateWordsWithSpace": separated
            }),
        );
        assert_eq!(
            branch_targets(&program).len(),
            2 * branch_targets(&post(&stream, Value::Null)).len()
        );
        assert_unique_numbers(&program);
        assert_branch_targets(&program);
    }
}
