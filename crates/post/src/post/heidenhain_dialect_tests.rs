//! #4095: the resolver table. Each version boundary that a wired flag
//! crosses is pinned on both sides; the sources are in
//! `docs/decisions/4095-heidenhain-dialect-resolver.md`.

use super::*;
use serde_json::json;

fn id(model: &str, nc: &str) -> ControllerIdentity {
    ControllerIdentity { model: model.into(), nc_software: nc.into(), installed_options: None }
}

fn flags(model: &str, nc: &str) -> Flags {
    resolve(&id(model, nc)).unwrap_or_else(|e| panic!("{model} {nc}: {e}")).flags
}

fn profile(model: &str, nc: &str) -> &'static str {
    resolve(&id(model, nc)).unwrap_or_else(|e| panic!("{model} {nc}: {e}")).profile
}

fn refused(model: &str, nc: &str) -> String {
    match resolve(&id(model, nc)) {
        Ok(d) => panic!("{model} {nc}: resolved to {}", d.profile),
        Err(why) => why,
    }
}

#[test]
fn the_picomax_control_is_the_itnc_49x_profile() {
    let d = resolve(&id("iTNC 530", "340490-03")).expect("the Picomax control");
    assert_eq!(d.profile, "itnc-49x");
    assert_eq!(d.nc, NcSoftware { id: 340490, release: 3, service_pack: None });
    assert_eq!(d.options, None, "no option list stated: unknown");
    assert_eq!(
        d.flags,
        Flags {
            language_iso: Tri::Yes,
            decimals: 4,
            tool_call_by_name: Tri::Yes,
            tool_name_max: 16,
            m140: Tri::Yes,
            m136_m137: Tri::Yes,
            drill_c200: C200::Q211,
            cycle247: Cycle247::PresetTable,
            dwell: Dwell::Cycle9,
        }
    );
}

#[test]
fn language_iso_boundaries() {
    for (model, nc, want) in [
        ("TNC 310", "286040-01", Tri::No),
        ("TNC 410", "286060-05", Tri::Yes),
        ("TNC 426", "280470-01", Tri::Unknown),
        ("TNC 426", "280472-01", Tri::Yes),
        ("TNC 320", "340551-03", Tri::Unknown),
        ("TNC 320", "340551-04", Tri::Yes),
        ("TNC 620", "340560-01", Tri::Unknown),
        ("TNC 620", "340560-02", Tri::Yes),
        ("TNC7", "817625-16", Tri::No),
        ("TNC7", "817625-17", Tri::Yes),
        ("TNC7 basic", "817620-20", Tri::Yes),
    ] {
        assert_eq!(flags(model, nc).language_iso, want, "{model} {nc}");
    }
}

#[test]
fn decimals_boundaries() {
    for (model, nc, want) in [
        ("TNC 310", "286140-02", 3),
        ("TNC 410", "286080-01", 3),
        ("TNC 426", "280472-01", 4),
        ("iTNC 530", "340420-01", 4),
        ("TNC 640", "340595-18", 4),
    ] {
        assert_eq!(flags(model, nc).decimals, want, "{model} {nc}");
    }
}

#[test]
fn cycle_200_level_boundaries() {
    for (model, nc, want) in [
        ("TNC 426", "280472-01", C200::Base),
        ("TNC 426", "280474-01", C200::Q211),
        ("iTNC 530", "340422-01", C200::Q211),
        ("iTNC 530", "340494-08", C200::Q211),
        ("iTNC 530", "606420-03", C200::Q211),
        ("iTNC 530", "606425-04", C200::Q395),
        ("iTNC 530", "606425-04 SP8", C200::Q395),
        ("TNC 640", "340590-02", C200::Q211),
        ("TNC 640", "340590-04", C200::Q395),
        ("TNC 620", "734980-02", C200::Q211),
        ("TNC 620", "817600-01", C200::Q395),
        ("TNC 320", "340551-06", C200::Q211),
        ("TNC 320", "771851-01", C200::Q395),
    ] {
        assert_eq!(flags(model, nc).drill_c200, want, "{model} {nc}");
    }
}

#[test]
fn cycle_247_boundaries() {
    for (model, nc, want) in [
        ("TNC 410", "286060-01", Cycle247::None),
        ("TNC 426", "280474-01", Cycle247::Unknown),
        ("TNC 430", "280476-01", Cycle247::Datum),
        ("iTNC 530", "340420-05", Cycle247::Datum),
        ("iTNC 530", "340422-01", Cycle247::PresetTable),
        ("TNC 320", "340551-02", Cycle247::Unknown),
        ("TNC 320", "340551-03", Cycle247::PresetTable),
        ("TNC 640", "340590-01", Cycle247::PresetTable),
    ] {
        assert_eq!(flags(model, nc).cycle247, want, "{model} {nc}");
    }
}

#[test]
fn dwell_boundaries() {
    for (model, nc, want) in [
        ("TNC 310", "286040-01", Dwell::Unknown),
        ("iTNC 530", "606425-04", Dwell::Cycle9),
        ("TNC 640", "340595-06", Dwell::Cycle9),
        ("TNC 640", "340595-07", Dwell::FunctionDwell),
        ("TNC 620", "817600-03", Dwell::Cycle9),
        ("TNC 620", "817600-04", Dwell::FunctionDwell),
        ("TNC 320", "771851-02", Dwell::Cycle9),
        ("TNC 320", "771851-04", Dwell::FunctionDwell),
        ("TNC7", "817625-17", Dwell::FunctionDwell),
    ] {
        assert_eq!(flags(model, nc).dwell, want, "{model} {nc}");
    }
}

#[test]
fn tool_call_and_m140_boundaries() {
    for (model, nc, by_name, max) in [
        ("TNC 410", "286060-01", Tri::No, 0),
        ("TNC 426", "280462-01", Tri::Yes, 16),
        ("TNC 640", "340590-01", Tri::Yes, 16),
        ("TNC 640", "340590-02", Tri::Yes, 32),
    ] {
        let f = flags(model, nc);
        assert_eq!((f.tool_call_by_name, f.tool_name_max), (by_name, max), "{model} {nc}");
    }
    assert_eq!(flags("TNC 426", "280474-01").m140, Tri::No);
    assert_eq!(flags("TNC 426", "280476-01").m140, Tri::Yes);
    assert_eq!(flags("TNC 410", "286060-01").m136_m137, Tri::No);
    assert_eq!(flags("iTNC 530", "340420-01").m136_m137, Tri::Yes);
}

#[test]
fn profiles_follow_the_research_names() {
    for (model, nc, want) in [
        ("iTNC 530", "340420-02", "itnc-340420"),
        ("iTNC 530", "340422-02", "itnc-340422"),
        ("iTNC 530", "340490-08", "itnc-49x"),
        ("iTNC 530", "606425-03", "itnc-49x"),
        ("iTNC 530", "606425-04", "itnc-60642x-04"),
        ("TNC 640", "340595-09", "tnc640-c"),
        ("TNC 640", "340595-10", "tnc640-d"),
        ("TNC 620", "817601-18", "tnc620-81760x"),
        ("TNC 320", "771851-18", "tnc320-77185x"),
    ] {
        assert_eq!(profile(model, nc), want, "{model} {nc}");
    }
}

#[test]
fn an_unknown_or_paraxial_model_is_refused_by_name() {
    let why = refused("Sinumerik 840D", "340490-03");
    assert!(why.contains("\"Sinumerik 840D\" is not a HEIDENHAIN control"), "{why}");
    assert!(refused("iTNC530", "340490-03").contains("\"iTNC530\""));
    let why = refused("TNC 128", "771841-05");
    assert!(why.contains("TNC 128 is a paraxial positioning control"), "{why}");
}

#[test]
fn an_nc_identifier_outside_the_table_is_refused_by_name() {
    for (model, nc) in [
        ("iTNC 530", "340490-09"),
        ("iTNC 530", "606425-05"),
        ("iTNC 530", "340495-03"),
        ("iTNC 530", "340421-02"),
        ("TNC 640", "340595-03"),
        ("TNC 640", "340595-12"),
        ("TNC 640", "340596-11"),
        ("TNC 640", "340490-03"),
        ("TNC 426", "280460-01"),
        ("TNC7", "817625-21"),
    ] {
        let why = refused(model, nc);
        assert!(why.contains(&format!("{model} NC {nc} is not in the HEIDENHAIN dialect table")), "{why}");
    }
}

#[test]
fn a_malformed_nc_identifier_is_refused_by_name() {
    for nc in [
        "340490-03junk",
        "340490-030",
        "340490-3",
        "340 490-03",
        "340490-03 sp8",
        "606425-04 SP",
        "606425-04 SP0",
        "606425-04 SP1234",
        "606425-04SP8",
        "34049-03",
        "unknown",
    ] {
        let why = refused("iTNC 530", nc);
        assert!(why.contains("is not a canonical HEIDENHAIN identifier"), "{nc}: {why}");
    }
    assert!(refused("iTNC 530", "340490-00").contains("release 00 does not exist"));
}

#[test]
fn installed_options_are_canonical_numbers() {
    let mut with = id("iTNC 530", "340490-03");
    with.installed_options = Some(vec!["1".into(), "19".into(), "0".into()]);
    assert_eq!(resolve(&with).expect("options").options, Some(vec![1, 19, 0]));
    with.installed_options = Some(vec![]);
    assert_eq!(resolve(&with).expect("no options").options, Some(vec![]));
    for bad in ["08", "a", "1 ", "+8", "1000"] {
        with.installed_options = Some(vec![bad.into()]);
        let why = resolve(&with).expect_err(bad);
        assert!(why.contains(&format!("installed option \"{bad}\"")), "{why}");
    }
}

#[test]
fn the_script_value_names_the_flags_or_the_refusal() {
    let v = script_value(&json!({ "model": "iTNC 530", "nc_software": "606425-04 SP8", "installed_options": ["1"] }));
    assert_eq!(v["profile"], "itnc-60642x-04");
    assert_eq!(v["nc"], json!({ "id": 606425, "release": 4, "servicePack": 8 }));
    assert_eq!(v["options"], json!([1]));
    assert_eq!(v["languageIso"], "yes");
    assert_eq!(v["decimals"], 4);
    assert_eq!(v["toolCallByName"], "yes");
    assert_eq!(v["toolNameMax"], 16);
    assert_eq!(v["m140"], "yes");
    assert_eq!(v["m136m137"], "yes");
    assert_eq!(v["drillC200"], "q395");
    assert_eq!(v["cycle247"], "presetTable");
    assert_eq!(v["dwell"], "cycle9");
    assert!(v.get("error").is_none());
    let e = script_value(&json!({ "model": "TNC 640", "nc_software": "340595-12" }));
    assert!(e["error"].as_str().is_some_and(|s| s.contains("TNC 640 NC 340595-12")), "{e}");
    assert!(e.get("profile").is_none());
}
