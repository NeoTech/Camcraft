//! #542 (part of #290): the generic ISO-style sinker post,
//! `assets/posts/sinker-iso.cps`, posted from a RECORDED stream (the
//! #926 direction): the Sinker Burn's CL golden that app's
//! `sinker_burn_cavity_matches_golden` regenerates, wrapped in the
//! PARAMs and section the in-app dump adds. Every EDM code comes from the
//! fixture dialect's `edm.codes` (#290 D5); a missing one refuses by
//! name, and the serves law keeps a mill post off a sinker and the
//! sinker post off a mill.
//!
//! Regenerate the NC golden after an intentional change:
//! `WRITE_GOLDEN=1 cargo test -p camcraft-post --test sinker_post`.
//!
//! Gate map (#542): covers post::post (engine serves gate, machine_edm,
//! machine mirror/script_props edm), post::machine (edm codes).
//! Provenance: our own SYNTHETIC fixture dialect (invented words, no
//! machine's manual). Not exercised on a real EDM control.

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
        Record::Param { name: "EdmCondition".into(), value: json!("S1") },
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
    // No codes at all (the library template's empty section).
    let mut m = dialect();
    m["edm"] = json!({});
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
    let stream = stream_with(&[]).replace("PARAM EdmCondition", "PARAM Unrelated");
    let out = post(&dialect(), &stream);
    assert!(out.warnings.iter().any(|w| w.contains("names no EDM condition")), "{:?}", out.warnings);
}

#[test]
fn a_stated_orbit_posts_the_machines_word_and_an_unlisted_one_refuses() {
    let out = post(&dialect(), &stream_with(&[("EdmOrbit", json!("circular")), ("EdmOrbitRadius", json!(0.2))]));
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    assert!(out.program.lines().any(|l| l == "M150 R0.2"), "{}", out.program);
    let out = post(&dialect(), &stream_with(&[("EdmOrbit", json!("planetary")), ("EdmOrbitRadius", json!(0.2))]));
    assert!(out.warnings.iter().any(|w| w.contains("lists no planetary orbit")), "{:?}", out.warnings);
}

#[test]
fn a_mill_post_on_a_sinker_and_the_sinker_post_on_a_mill_are_refused() {
    let stream = stream_with(&[]);
    for stem in ["fanuc", "haas", "tormach-plasma", "fanuc-lathe"] {
        let cps = read(&format!("assets/posts/{stem}.cps"));
        let err = engine::run(&cps, &dialect(), &stream).expect_err("a non-EDM post on a sinker");
        assert!(matches!(err, PostError::Refused(_)), "{stem}: {err:?}");
        assert!(err.to_string().contains("sinker EDM machine"), "{stem}: {err}");
    }
    let vmc: Value = serde_json::from_str(&read("assets/machines/vmc.mch")).expect("vmc");
    let err = engine::run(&sinker_cps(), &vmc, &stream).expect_err("the sinker post on a mill");
    assert!(err.to_string().contains("serves sinker_edm machines only"), "{err}");
    // No machine at all is no sinker either.
    let err = engine::run(&sinker_cps(), &Value::Null, &stream).expect_err("no machine");
    assert!(matches!(err, PostError::Refused(_)), "{err:?}");
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
