//! #4095: the Klartext post writes a section's smoothing ask as cycle 32
//! TOLERANCE (iTNC 530 Klartext manual 533 190-22, section 8.10), from the
//! resolver flags `g62Mode` and `g62TMaxUm` (#4226), and resets the cycle at
//! the section end. The Picomax states no installed options, so its control
//! (profile itnc-49x) gets `CYCL DEF 32.2 HSC-MODE` only when the machine
//! file lists option 2.

use super::*;

/// The contour stream with a smoothing ask before its tool.
fn asked(tol: &str, mode: &str) -> String {
    let out = STREAM.replacen(
        "TOOL 2 FLAT-6",
        &format!("PARAM Smoothing {tol}\nPARAM SmoothingMode {mode}\nTOOL 2 FLAT-6"),
        1,
    );
    assert_ne!(out, STREAM, "the variant must change the stream");
    out
}

/// The Picomax machine file with `installed_options` in its identity.
fn with_options(options: &[&str]) -> String {
    let mut machine: Value = serde_json::from_str(MCH).unwrap();
    machine["controller"]["identity"]["installed_options"] = serde_json::json!(options);
    machine.to_string()
}

/// The blocks of `nc` without their block numbers.
fn blocks(nc: &str) -> Vec<String> {
    nc.lines()
        .map(|l| match l.split_once(' ') {
            Some((n, rest)) if n.parse::<u32>().is_ok() => rest.to_string(),
            _ => l.trim().to_string(),
        })
        .collect()
}

/// A finish ask, then a section with no ask: the golden pins the cycle 32
/// blocks before the first move and the reset before M9 of the first section.
#[test]
fn klartext_smoothing_posts_on_the_picomax_golden() {
    let stream = format!("{}{STREAM}", asked("0.02", "finish"));
    let nc = post(MCH, &stream, &Value::Null).expect("a finish ask posts on the Picomax");
    recorded::compare(SCRIPT, "picomax_smoothing", "nc", &nc);
}

#[test]
fn a_finish_ask_writes_t_only_and_the_section_end_resets_it() {
    let stream = format!("{}{STREAM}", asked("0.02", "finish"));
    let b = blocks(&post(MCH, &stream, &Value::Null).unwrap());
    let on = b.iter().position(|l| l == "CYCL DEF 32.1 T0.02").expect("cycle 32 with T");
    assert_eq!(b[on - 1], "CYCL DEF 32.0 TOLERANCE");
    // Option 2 is not listed: no HSC-MODE block.
    assert!(!b.iter().any(|l| l.contains("HSC-MODE")), "{b:#?}");
    // Before the first move of the section, after the spindle and coolant.
    assert_eq!(b[on - 2], "M9", "{b:#?}");
    assert!(b[on + 1].starts_with("L X+10 Y+0 R0 FMAX"), "{b:#?}");
    // The reset ends the first section, before its M9 and M5.
    let reset = b.iter().position(|l| l == "CYCL DEF 32.1").expect("the reset");
    assert_eq!(b[reset - 1], "CYCL DEF 32.0 TOLERANCE");
    assert_eq!(b[reset + 1..reset + 3], ["M9", "M5"]);
    // One cycle 32 and one reset: the second section has no ask.
    assert_eq!(b.iter().filter(|l| *l == "CYCL DEF 32.0 TOLERANCE").count(), 2, "{b:#?}");
    let second = b.iter().rposition(|l| l.starts_with("TOOL CALL 2 ")).unwrap();
    assert!(second > reset, "{b:#?}");
}

#[test]
fn option_2_writes_hsc_mode_for_finish_and_rough() {
    let mch = with_options(&["2"]);
    for (mode, word) in [("finish", "HSC-MODE:0"), ("rough", "HSC-MODE:1")] {
        let b = blocks(&post(&mch, &asked("0.1", mode), &Value::Null).expect(mode));
        let on = b.iter().position(|l| l == "CYCL DEF 32.1 T0.1").expect("cycle 32 with T");
        assert_eq!(b[on + 1], format!("CYCL DEF 32.2 {word} TA0"), "{mode}: {b:#?}");
        let reset = b.iter().position(|l| l == "CYCL DEF 32.1").expect("the reset");
        assert!(reset > on + 1, "{b:#?}");
    }
    // Other options do not permit HSC-MODE.
    let b = blocks(&post(&with_options(&["1", "19"]), &asked("0.1", "finish"), &Value::Null).unwrap());
    assert!(!b.iter().any(|l| l.contains("HSC-MODE")), "{b:#?}");
}

#[test]
fn each_klartext_smoothing_refusal_names_its_cause() {
    for (mch, stream, diagnostic) in [
        (
            MCH.to_string(),
            asked("0.1", "rough"),
            "smoothing mode rough: controller profile itnc-49x takes CYCL DEF 32.2 HSC-MODE only with software option 2",
        ),
        (
            MCH.to_string(),
            asked("10.001", "finish"),
            "smoothing tolerance T10.001 mm is above the largest T of controller profile itnc-49x (10 mm)",
        ),
        (
            MCH.to_string(),
            asked("0.0004", "finish"),
            "smoothing tolerance 0.0004 mm writes T0 at 0.001 mm",
        ),
        (
            MCH.to_string(),
            STREAM.replacen("TOOL 2 FLAT-6", "PARAM Smoothing -0.1\nTOOL 2 FLAT-6", 1),
            "smoothing tolerance -0.1 is not a positive number of mm",
        ),
        (
            MCH.to_string(),
            asked("0.1", "medium"),
            "smoothing mode \"medium\" is not finish or rough",
        ),
        (
            MCH.to_string(),
            STREAM.replacen("TOOL 2 FLAT-6", "PARAM SmoothingMode finish\nTOOL 2 FLAT-6", 1),
            "smoothing mode finish without a tolerance",
        ),
    ] {
        let why = post(&mch, &stream, &Value::Null).expect_err(diagnostic);
        assert!(why.contains(diagnostic), "{diagnostic}: {why}");
    }
}

/// T is on the 1 um grid of the flag: 10 mm is the largest T of the
/// Picomax profile and posts; a value that rounds to 0.001 mm posts as T0.001.
#[test]
fn the_t_range_bounds_post_on_the_picomax() {
    for (tol, word) in [("10", "CYCL DEF 32.1 T10"), ("0.0006", "CYCL DEF 32.1 T0.001")] {
        let b = blocks(&post(MCH, &asked(tol, "finish"), &Value::Null).expect(tol));
        assert!(b.iter().any(|l| l == word), "{tol}: {b:#?}");
    }
}
