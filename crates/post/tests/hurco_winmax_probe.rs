//! Provenance: Hurco public NCPP Variable Summary, #3020 contact status and
//! #2501..#2799 work offsets. These tests evaluate emitted probe safety branches.
//! Gate map (M4.4): covers assets/posts/hurco-winmax.cps, camcraft_post::post::engine.

use camcraft_post::post::engine;
use serde_json::json;
use std::path::Path;

fn read(path: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path),
    )
    .expect("test input exists")
}

fn post(stream: &str, inch: bool) -> String {
    let out = engine::run_with(
        &read("assets/posts/hurco-winmax.cps"),
        &json!({}),
        stream,
        &json!({ "Units": if inch { "inch" } else { "mm" } }),
    )
    .expect("probe stream posts");
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    out.program
}

/// Execute the safety branch immediately after one G31. The status is the
/// control's #3020 result, independent of the skip coordinate or desired point.
fn contact_alarm<'a>(lines: &'a [&str], index: usize, status: u8) -> Option<&'a str> {
    let condition = lines[index + 1];
    let predicate = condition
        .strip_prefix("IF[#3020EQ")
        .expect("G31 must check documented contact status");
    let (expected, label) = predicate
        .split_once("]GOTO")
        .expect("contact branch target");
    let expected: u8 = expected.parse().expect("numeric contact status");
    let alarm = lines[index + 2];
    assert!(alarm.starts_with("#3000="), "missing alarm: {alarm}");
    assert_eq!(
        lines[index + 3],
        format!("N{label}"),
        "branch must skip only its alarm"
    );
    (status != expected).then_some(alarm)
}

#[test]
fn contact_status_checks_every_touch_and_guard_in_mm_and_inch() {
    let stream = read("crates/post/tests/recorded/streams/probe.cnc");
    let kinds: Vec<bool> = stream
        .lines()
        .filter_map(|line| {
            if line.starts_with("TOUCH ") {
                Some(true)
            } else if line.starts_with("GUARD ") {
                Some(false)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(kinds.iter().filter(|&&touch| touch).count(), 8);
    assert_eq!(kinds.iter().filter(|&&touch| !touch).count(), 3);
    for inch in [false, true] {
        let program = post(&stream, inch);
        let lines: Vec<_> = program.lines().collect();
        let moves: Vec<_> = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.starts_with("G31 "))
            .map(|(i, _)| i)
            .collect();
        assert_eq!(moves.len(), kinds.len());
        assert!(program.contains(if inch { "\nG20\n" } else { "\nG21\n" }));
        for (&index, &touch) in moves.iter().zip(&kinds) {
            if touch {
                // Correct touches at X38/X22 (and Y38/Y22) succeed, even though
                // their recorded positions differ from the overtravel endpoints.
                assert_eq!(contact_alarm(&lines, index, 1), None);
                // No-contact endpoints X43/X17 average to30 and pass the later
                // tolerance check; each must alarm before that can happen.
                assert_eq!(
                    contact_alarm(&lines, index, 0),
                    Some("#3000=2(PROBE NO CONTACT)")
                );
            } else {
                assert_eq!(contact_alarm(&lines, index, 0), None);
                assert_eq!(
                    contact_alarm(&lines, index, 1),
                    Some("#3000=1(PROBE UNEXPECTED CONTACT)")
                );
            }
        }
        assert!(program.contains(if inch {
            "G31 X1.6929 Y1.1811 Z0.5906"
        } else {
            "G31 X43. Y30. Z15."
        }));
    }
}

/// Evaluate a G10 axis expression with a sparse, nonzero source-offset table.
/// Unknown variables fail, including the unrelated Fanuc #52xx range.
fn offset_value(word: &str, axis: char, register: usize, source: f64, deviation: f64) -> f64 {
    let expression = word
        .strip_prefix(&format!("{axis}[#"))
        .expect("offset expression");
    let (variable, tail) = expression.split_once("+#").expect("source plus deviation");
    assert_eq!(
        variable.parse::<usize>().expect("source register"),
        register
    );
    let dev = match axis {
        'X' => 190,
        'Y' => 191,
        'Z' => 192,
        _ => unreachable!(),
    };
    assert_eq!(tail, format!("{dev}]"));
    source + deviation
}

#[test]
fn probe_offsets_use_active_winmax_source_and_independent_target() {
    let original = read("crates/post/tests/recorded/streams/probe.cnc");
    // Source indexes 1..6 are G54..G59;7..99 are auxiliary1..93. Omitted defaults to G54.
    for (source, effective, target) in [
        (Some(1), 1, 1),
        (None, 1, 2),
        (Some(3), 3, 5),
        (Some(7), 7, 0),
        (Some(99), 99, 3),
    ] {
        let stream = original
            .replace(
                "PARAM WorkOffset 1.000000",
                &source
                    .map(|source| format!("PARAM WorkOffset {source}"))
                    .unwrap_or_default(),
            )
            .replace("FOUND SET 1", &format!("FOUND SET {target}"))
            .replace("AXIS Y 30000 2 3", "AXIS Y 30000 2 3 AXIS Z 15000 0");
        for inch in [false, true] {
            let program = post(&stream, inch);
            let g10 = program
                .lines()
                .find(|line| line.starts_with("G10 L2 "))
                .expect("writes target offset");
            let words: Vec<_> = g10.split_whitespace().collect();
            assert_eq!(words[2], format!("P{}", target + 1));
            let scale = if inch { 1.0 / 25.4 } else { 1.0 };
            // The source table contains nonzero offsets. The target's previous
            // value must not participate in these expressions.
            for (word, axis, base, source_value, deviation) in [
                (words[3], 'X', 2500, 125.0, 0.2),
                (words[4], 'Y', 2600, -75.0, -0.3),
                (words[5], 'Z', 2700, 15.0, 0.1),
            ] {
                let got = offset_value(
                    word,
                    axis,
                    base + effective,
                    source_value * scale,
                    deviation * scale,
                );
                assert!((got - (source_value + deviation) * scale).abs() < 1e-10);
            }
        }
    }
}
