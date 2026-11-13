//! The `sinker-iso.cps` script suite (#542, part of #290; the #926 suite
//! pattern): the generic ISO-style sinker post.
//!
//! - The recorded mill cases (`recorded/streams/*.cnc`) through the post
//!   binary, machine-less as every suite posts them: this post serves
//!   sinker_edm machines only, so each case's golden under
//!   `recorded/golden/sinker-iso/` is its named refusal - never a program.
//! - The sinker's own recorded stream through the binary on the fixture
//!   dialect (`sinker_burn_cavity`, same golden dir): the Sinker Burn's CL
//!   golden that app's `sinker_burn_cavity_matches_golden` regenerates,
//!   wrapped in the PARAMs and section the in-app dump adds.
//! - The same stream through the engine: every EDM code comes from the
//!   fixture dialect's `edm.codes` (#290 D5); a missing one refuses by
//!   name, as do a condition the machine does not list, an orbit (#544)
//!   and a Sinker Side Burn (#545); the engine's `SinkerCondition` door
//!   keeps a mill post off a sinker burn and the script's machine check
//!   keeps the sinker post off a mill (the app's serves door, #1232,
//!   refuses both first - `posting_serves`).
//!
//! Regenerate after an intentional change, then read the diff:
//! `WRITE_GOLDEN=1 cargo test -p camcraft-post --test script_sinker_iso`.
//!
//! Gate map (M4.4, #542): covers assets/posts/sinker-iso.cps, the post
//! binary, post::post (engine SinkerCondition door, machine
//! mirror/script_props edm), post::machine (edm codes).
//! Provenance: our own SYNTHETIC fixture dialect (invented words, no
//! machine's manual). Not exercised on a real EDM control.

#[macro_use]
mod recorded;

post_suite!("sinker-iso.cps");

use camcraft_post::post::cnc::{self, Record};
use camcraft_post::post::{engine, PostError};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn repo(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join(rel)
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

fn sinker_cps() -> String {
    read("assets/posts/sinker-iso.cps")
}

/// The synthetic fixture dialect (raw `.mch` JSON - the CLI's door).
fn dialect() -> Value {
    serde_json::from_str(&read("crates/app/tests/edm/sinker-iso-dialect.mch")).expect("fixture json")
}

/// The recorded burn wrapped as the in-app dump wraps it: commentary,
/// the condition and pickup PARAMs, the electrode, the section.
fn stream_with(extra: &[(&str, Value)]) -> String {
    let cl = cnc::parse(&read("crates/app/tests/golden/sinker_burn_cavity.cl")).expect("the CL golden parses");
    let mut recs = vec![
        Record::Param { name: "OperationName".into(), value: json!("Burn1") },
        Record::Param { name: "OperationKind".into(), value: json!("Sinker Burn") },
        Record::Param { name: "ToolKind".into(), value: json!("sinker electrode") },
        Record::Param { name: "SpindleSpeed".into(), value: json!(0.0) },
        Record::Param { name: "WorkOffset".into(), value: json!("G54") },
        Record::Param { name: "SinkerCondition".into(), value: json!("S1") },
        Record::Param { name: "ElectrodePickup".into(), value: json!(15.0) },
    ];
    for (n, v) in extra {
        recs.push(Record::Param { name: (*n).into(), value: v.clone() });
    }
    recs.push(Record::Tool { number: 1, name: "E-10x6".into() });
    recs.push(Record::Section { begin: true });
    recs.extend(cl);
    recs.push(Record::Section { begin: false });
    cnc::serialize(&recs)
}

fn post(mch: &Value, stream: &str) -> engine::PostOutput {
    engine::run(&sinker_cps(), mch, stream).expect("the sinker post runs")
}

#[test]
fn the_sinker_burn_posts_to_its_golden_against_the_fixture_dialect() {
    let out = post(&dialect(), &stream_with(&[]));
    assert!(out.warnings.is_empty(), "a clean burn posts without a refusal: {:?}", out.warnings);
    let rel = "crates/app/tests/golden/nc/sinker-iso/sinker_burn_cavity.nc";
    if std::env::var_os("WRITE_GOLDEN").is_some() {
        let p = repo(rel);
        std::fs::create_dir_all(p.parent().expect("dir")).expect("mkdir");
        std::fs::write(&p, &out.program).expect("write golden");
    }
    assert_eq!(out.program, read(rel), "the sinker post moved a byte");
    // The words the fixture's codes state, filled: the condition select,
    // the pickup and its offset, the burn to the tip's Z (one 0.05 gap
    // above the 14 mm floor), the end.
    for line in ["G21", "T1 M06", "M100 H15.", "ES1", "M101 Z14.05", "M02"] {
        assert!(out.program.lines().any(|l| l == line), "{line} in\n{}", out.program);
    }
}

#[test]
fn every_missing_sinker_code_refuses_the_post_by_name() {
    for role in ["metric", "pickup", "reference", "condition", "depth", "end"] {
        let mut m = dialect();
        m["edm"]["codes"].as_object_mut().expect("codes").remove(role);
        let out = post(&m, &stream_with(&[]));
        let want = format!("states no edm.codes.{role}");
        assert!(out.warnings.iter().any(|w| w.contains(&want)), "{role}: {:?}", out.warnings);
    }
    // An inch machine needs the inch word, not the metric one.
    let mut m = dialect();
    m["kinematics"]["default"]["units"]["length"] = json!("inch");
    m["edm"]["codes"].as_object_mut().expect("codes").remove("inch");
    let out = post(&m, &stream_with(&[]));
    assert!(out.warnings.iter().any(|w| w.contains("states no edm.codes.inch")), "{:?}", out.warnings);
    // No codes at all (the library template's section, before its
    // codes are filled from the manual).
    let mut m = dialect();
    m["edm"].as_object_mut().expect("edm").remove("codes");
    let out = post(&m, &stream_with(&[]));
    assert!(!out.warnings.is_empty() && out.warnings.iter().all(|w| w.contains("states no edm.codes.")), "{:?}", out.warnings);
}

#[test]
fn a_code_naming_a_placeholder_the_post_does_not_fill_refuses_by_name() {
    let mut m = dialect();
    m["edm"]["codes"]["depth"] = json!("M101 Z{z} Q{power}");
    let out = post(&m, &stream_with(&[]));
    assert!(out.warnings.iter().any(|w| w.contains("edm.codes.depth holds {power}")), "{:?}", out.warnings);
    // A stream with no condition named: refused, never a bare select.
    let stream = stream_with(&[]).replace("PARAM SinkerCondition", "PARAM Unrelated");
    let out = post(&dialect(), &stream);
    assert!(out.warnings.iter().any(|w| w.contains("names no EDM condition")), "{:?}", out.warnings);
}

/// #544: an orbit is expanded into explicit moves the stock sim judged;
/// this post writes a straight burn only and never swaps those moves for
/// the control's orbit word - even one the machine lists - so it refuses
/// the orbit by name, before any preamble.
#[test]
fn an_orbit_is_refused_by_name_even_one_the_machine_lists() {
    for kind in ["circular", "planetary"] {
        let out = post(&dialect(), &stream_with(&[("EdmOrbit", json!(kind)), ("EdmOrbitRadius", json!(0.2))]));
        assert!(out.warnings.iter().any(|w| w.contains(&format!("a {kind} orbit"))), "{kind}: {:?}", out.warnings);
        assert!(!out.program.contains("M150") && !out.program.contains("M101"), "{kind}: nothing burns\n{}", out.program);
    }
}

/// #545: a Sinker Side Burn moves the electrode along X or Y - not a
/// straight burn - and is refused by name, not as a stray feed.
#[test]
fn a_sinker_side_burn_is_refused_by_name() {
    let stream = stream_with(&[]).replace("PARAM OperationKind Sinker Burn", "PARAM OperationKind Sinker Side Burn");
    assert!(stream.contains("Sinker Side Burn"), "the stream names the side burn");
    let out = post(&dialect(), &stream);
    assert!(out.warnings.iter().any(|w| w.contains("a Sinker Side Burn moves the electrode sideways")), "{:?}", out.warnings);
    assert!(!out.program.contains("M101"), "nothing burns\n{}", out.program);
}

/// #531: the condition select names only a condition the machine's
/// table lists, exactly as the table writes it.
#[test]
fn a_condition_the_machine_does_not_list_is_refused_by_name() {
    for id in ["S2", "s1"] {
        let stream = stream_with(&[]).replace("PARAM SinkerCondition S1", &format!("PARAM SinkerCondition {id}"));
        assert!(stream.contains(&format!("PARAM SinkerCondition {id}")), "the stream names {id}");
        let out = post(&dialect(), &stream);
        assert!(out.warnings.iter().any(|w| w.contains(&format!("condition {id} is not in"))), "{id}: {:?}", out.warnings);
        assert!(!out.program.contains("ES") && !out.program.contains("M101"), "{id}: nothing selected\n{}", out.program);
    }
    let mut m = dialect();
    m["edm"].as_object_mut().expect("edm").remove("conditions");
    let out = post(&m, &stream_with(&[]));
    assert!(out.warnings.iter().any(|w| w.contains("condition S1 is not in")), "{:?}", out.warnings);
}

#[test]
fn a_mill_post_on_a_sinker_and_the_sinker_post_on_a_mill_are_refused() {
    let stream = stream_with(&[]);
    for stem in ["fanuc", "haas", "tormach-plasma", "fanuc-lathe", "wire-iso"] {
        let cps = read(&format!("assets/posts/{stem}.cps"));
        let refused = match engine::run(&cps, &dialect(), &stream) {
            Ok(out) => out.warnings.iter().any(|w| w.contains("is sinker EDM work")),
            Err(e) => matches!(e, PostError::Refused(_)),
        };
        assert!(refused, "{stem}: a non-sinker post is refused the sinker burn by name");
    }
    let vmc: Value = serde_json::from_str(&read("assets/machines/vmc.mch")).expect("vmc");
    let out = engine::run(&sinker_cps(), &vmc, &stream).expect("the engine runs, the script refuses");
    assert!(out.warnings.iter().any(|w| w.contains("is not declared a sinker EDM")), "{:?}", out.warnings);
    // No machine at all is no sinker either.
    let out = engine::run(&sinker_cps(), &Value::Null, &stream).expect("the engine runs, the script refuses");
    assert!(out.warnings.iter().any(|w| w.contains("is not declared a sinker EDM")), "{:?}", out.warnings);
}

#[test]
fn a_feed_with_xy_travel_or_an_rpm_is_refused_by_name() {
    let stream = stream_with(&[]).replace("Feed X 12500 Y -7500", "Feed X 13500 Y -7500");
    let out = post(&dialect(), &stream);
    assert!(out.warnings.iter().any(|w| w.contains("X/Y travel")), "{:?}", out.warnings);
    let stream = stream_with(&[]).replace("PARAM SpindleSpeed 0.000000", "PARAM SpindleSpeed 1000");
    let out = post(&dialect(), &stream);
    assert!(out.warnings.iter().any(|w| w.contains("turns no spindle")), "{:?}", out.warnings);
}

#[test]
fn sinker_burn_cavity() {
    // The binary's door: the stream as a file, the dialect as the CLI
    // reads it (the vendor tree mirrored by the binary itself).
    let input = std::env::temp_dir().join(format!("camcraft_recorded_sinker_burn_{}.cnc", std::process::id()));
    std::fs::write(&input, stream_with(&[])).expect("write the sinker stream");
    let mch = repo("crates/app/tests/edm/sinker-iso-dialect.mch");
    let (ext, text) = recorded::verdict_on("sinker-iso.cps", "sinker_burn_cavity", &input, Some(&mch));
    let _ = std::fs::remove_file(&input);
    recorded::compare("sinker-iso.cps", "sinker_burn_cavity", ext, &text);
}
