//! #4094: the G200 parameter set follows the stated control. Q395 exists
//! only from iTNC 530 60642x-04, TNC 640 34059x-04, TNC 620 81760x-01 and
//! TNC 320 77185x-01 (docs/research/heidenhain/differences.md B2/B3,
//! dialects.md). #4224: PECK posts the explicit chain on each level, never
//! a G205.

use super::*;
use camcraft_post::machine::parse_mch;
use camcraft_post::post::machine as post_machine;

/// One peck hole (R on the surface: the explicit chain, #4224) and one
/// G200 hole (drill).
fn two_holes() -> String {
    let mut rs = setup(3000.0);
    let mut n = 0;
    add_hole(&mut rs, &mut n, hole(CycleKind::Peck, 10.0, 0.0, -8.0, 4.0, 0.2, Some(150.0)), None);
    add_hole(&mut rs, &mut n, hole(CycleKind::Drill, 30.0, 0.0, -8.0, 0.0, 0.0, Some(90.0)), None);
    finish(&mut rs);
    stream(&rs)
}

fn with_identity(model: &str, nc: &str) -> Value {
    let mut m = machine(None);
    m["controller"]["identity"] = json!({ "model": model, "nc_software": nc });
    m
}

fn run_on(m: &Value, script: &str) -> engine::PostOutput {
    engine::run(script, m, &two_holes()).unwrap_or_else(|e| panic!("Heidenhain engine run: {e}"))
}

const OLD_G200: &[(u16, &str)] = &[(200,"+0"),(201,"-8"),(206,"+90"),(202,"+8"),(210,"+0"),(203,"+0"),(204,"+0"),(211,"+0")];

/// #4224: the peck hole is the explicit chain on each level.
fn assert_peck_chain(out: &engine::PostOutput, what: &str) {
    assert!(!out.program.contains("G205") && !out.program.contains("Q258"), "{what}: {}", out.program);
    let bs = bodies(&out.program);
    assert!(bs.iter().any(|b| b == "G00 X+10 Y+20 Z-3"), "{what}: the chip-break hop is posted: {bs:?}");
    assert!(bs.iter().any(|b| b == "G01 X+10 Y+20 Z-8 F150"), "{what}: the last infeed is posted: {bs:?}");
}

fn assert_old_level(out: &engine::PostOutput, what: &str) {
    assert!(out.warnings.is_empty(), "{what}: {:?}", out.warnings);
    let bs = bodies(&out.program);
    assert_qs(definition(&bs, "G200"), OLD_G200);
    assert!(!out.program.contains("Q208") && !out.program.contains("Q395"), "{what}: {}", out.program);
    assert_eq!(calls(&bs), 1, "{what}");
    assert_peck_chain(out, what);
}

#[test]
fn picomax_340490_03_writes_g200_without_q395_and_the_peck_chain() {
    let mch = read("assets/machines/fehlmann-picomax55.mch");
    let picomax = post_machine::mirror(&parse_mch(&mch).expect("the Picomax machine file"));
    let out = run_on(&picomax, &read("assets/posts/heidenhain.cps"));
    assert_old_level(&out, "Picomax 55, iTNC 530 340490-03");
}

#[test]
fn controls_before_the_q395_level_write_g200_without_q395() {
    let script = read("assets/posts/heidenhain.cps");
    for (model, nc) in [
        ("iTNC 530", "340490-03"),
        ("iTNC 530", "340494-05"),
        ("iTNC 530", "340490-08"),
        ("iTNC 530", "340422-02"),
        ("iTNC 530", "606420-03"),
        ("TNC 640", "340590-02"),
        ("TNC 620", "340560-02"),
        ("TNC 320", "340551-06"),
    ] {
        assert_old_level(&run_on(&with_identity(model, nc), &script), &format!("{model} {nc}"));
    }
}

#[test]
fn controls_from_the_q395_level_keep_q395() {
    let script = read("assets/posts/heidenhain.cps");
    for (model, nc) in [
        ("iTNC 530", "606425-04"),
        ("iTNC 530", "606425-04 SP8"),
        ("TNC 640", "340595-04"),
        ("TNC 640", "340595-11"),
        ("TNC 620", "817600-01"),
        ("TNC 620", "817600-18"),
        ("TNC 320", "771851-18"),
    ] {
        let out = run_on(&with_identity(model, nc), &script);
        assert!(out.warnings.is_empty(), "{model} {nc}: {:?}", out.warnings);
        let bs = bodies(&out.program);
        assert!(!out.program.contains("Q208"), "{model} {nc}: {}", out.program);
        assert_peck_chain(&out, &format!("{model} {nc}"));
        let mut g200 = OLD_G200.to_vec();
        g200.push((395, "+0"));
        assert_qs(definition(&bs, "G200"), &g200);
    }
}

#[test]
fn an_unknown_control_or_version_is_refused_by_name() {
    let script = read("assets/posts/heidenhain.cps");
    for (model, nc) in [
        ("iTNC 530", "340420-05"),
        ("TNC 640", "340593-03"),
        ("TNC 426", "280476-01"),
        ("TNC7", "817625-17"),
        ("iTNC 530", "unknown"),
    ] {
        assert_refused(&script, model, nc);
    }
}

fn assert_refused(script: &str, model: &str, nc: &str) {
    let out = run_on(&with_identity(model, nc), script);
    let why = format!("the cycle 200/205 parameter set of {model} NC {nc} is not known to this post - refused, not posted");
    assert!(out.warnings.iter().any(|w| w.contains(&why)), "{model} {nc:?}: {:?}", out.warnings);
    assert!(!out.program.contains("G200") && !out.program.contains("G205"), "{model} {nc:?}: {}", out.program);
}

/// #4172: the NC software must be the complete canonical identifier
/// (6 digits, "-", 2 release digits, optional " SP<n>") of a variant and a
/// release that the research names. A prefix match never selects a level.
/// (Outer white space is refused before, when the machine file loads.)
#[test]
fn a_malformed_or_unsupported_nc_identifier_is_refused_by_name() {
    let script = read("assets/posts/heidenhain.cps");
    for (model, nc) in [
        // Malformed: a suffix, a third release digit, one release digit.
        ("iTNC 530", "340490-03junk"),
        ("iTNC 530", "340490-030"),
        ("iTNC 530", "340490-3"),
        ("iTNC 530", "340490-03 sp8"),
        ("iTNC 530", "606425-04 SP"),
        ("iTNC 530", "606425-04 SP0"),
        ("iTNC 530", "606425-04SP8"),
        // Release zero, and releases that the research does not name.
        ("iTNC 530", "340490-00"),
        ("iTNC 530", "340490-09"),
        ("iTNC 530", "606425-05"),
        ("TNC 640", "340595-12"),
        ("TNC 620", "817600-00"),
        // Variants outside the family.
        ("iTNC 530", "340499-03"),
        ("iTNC 530", "340495-03"),
        ("iTNC 530", "606426-04"),
        ("iTNC 530", "340421-02"),
        // A valid identifier under the wrong model.
        ("TNC 640", "340490-03"),
    ] {
        assert_refused(&script, model, nc);
    }
}

/// A depth reference that the old cycle cannot take is refused, never
/// dropped. The CAM asks for Q395=0 today: a script variant asks for 1.
#[test]
fn a_q395_ask_the_340_49x_cycle_cannot_take_is_refused_by_name() {
    let original = read("assets/posts/heidenhain.cps");
    let script = original.replace("var DEPTH_REFERENCE = 0;", "var DEPTH_REFERENCE = 1;");
    assert_ne!(script, original, "the variant must change the asked depth reference");
    let out = run_on(&with_identity("iTNC 530", "340490-03"), &script);
    assert!(
        out.warnings.iter().any(|w| w.contains("G200 Q395=1 cannot be written for iTNC 530 NC 340490-03: its cycle 200 has no Q395")),
        "{:?}",
        out.warnings
    );
    assert!(!out.program.contains("G200"), "{}", out.program);
    let newer = run_on(&with_identity("iTNC 530", "606425-04"), &script);
    assert!(newer.warnings.is_empty(), "{:?}", newer.warnings);
    assert!(definition(&bodies(&newer.program), "G200").ends_with("Q211=+0 Q395=+1"), "{}", newer.program);
}
