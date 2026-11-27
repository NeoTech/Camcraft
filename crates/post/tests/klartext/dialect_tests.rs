//! #4095: the Klartext post reads the shared dialect resolver. It writes
//! L M140 MB MAX, CYCL DEF 247 with Q339 (a preset-table row) and M137, so a
//! profile that does not confirm one of these is refused by name. The manual
//! quill procedure stays verified for iTNC 530 NC 340490-03 only (#4114).

use super::*;

fn with_identity(model: &str, nc: &str) -> String {
    let mut machine: Value = serde_json::from_str(MCH).unwrap();
    machine["controller"]["identity"] = serde_json::json!({ "model": model, "nc_software": nc });
    machine.to_string()
}

#[test]
fn the_dialect_resolver_decides_the_klartext_profile() {
    for (model, nc, diagnostic) in [
        (
            "iTNC 530",
            "340420-03",
            "controller profile itnc-340420 (iTNC 530 NC 340420-03) does not confirm cycle 247 as a preset-table row, which this post writes",
        ),
        (
            "TNC 410",
            "286060-01",
            "controller profile tnc410 (TNC 410 NC 286060-01) does not confirm M140 MB MAX, cycle 247 as a preset-table row, M137",
        ),
        ("TNC 640", "340595-11", "controller profile tnc640-d (TNC 640 NC 340595-11) does not confirm M137"),
        (
            "iTNC 530",
            "340490-04",
            "controller profile itnc-49x (iTNC 530 NC 340490-04) is not verified for the manual quill procedure",
        ),
        ("iTNC 530", "340490-3", "unsupported controller profile: NC software \"340490-3\" is not a canonical HEIDENHAIN identifier"),
        ("Sinumerik 840D", "340490-03", "unsupported controller profile: controller model \"Sinumerik 840D\" is not a HEIDENHAIN control"),
    ] {
        let why = post(&with_identity(model, nc), STREAM, &Value::Null).expect_err(nc);
        assert!(why.contains(diagnostic), "{model} {nc}: {why}");
    }
}

/// The Picomax control resolves to 4 coordinate decimals (0.1 um input,
/// profile itnc-49x): a 0.3 um step keeps its fourth decimal.
#[test]
fn the_picomax_profile_writes_four_decimals() {
    let stream = STREAM.replace(
        "Arc CCW X 100000 Y 35000 Z 9000 F 350000 I 100000 J 25000",
        "Arc CCW X 100000 Y 35000.3 Z 9000 F 350000 I 100000 J 25000",
    );
    assert_ne!(stream, STREAM, "the variant must change the stream");
    let nc = post(MCH, &stream, &Value::Null).expect("the Picomax posts");
    assert!(nc.contains("C X+0 Y+10.0003 DR+"), "{nc}");
}
