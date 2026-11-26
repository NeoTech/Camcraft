//! Provenance: #4031's cutter hand, using our own starter catalog.
//! Gate map: covers tools::catalog_asset, tools::cutter_hand.

use camcraft_tools::catalog_asset::{CATALOG_JSON, parse_catalog};
use camcraft_tools::cutter_hand::CutterHand;
use serde_json::{Value, json};

fn catalog_with(hand: Value) -> String {
    let mut value: Value = serde_json::from_str(CATALOG_JSON).expect("catalog JSON");
    let row = value["tools"].as_array_mut().expect("tools").iter_mut()
        .find(|r| r["name"] == "EM-0600-4FL").expect("end mill row");
    row["definition"]["cutter_hand"] = hand;
    value.to_string()
}

#[test]
fn catalog_preserves_a_left_hand_cutter_and_defaults_legacy_tools_to_right() {
    let legacy = parse_catalog(CATALOG_JSON).expect("legacy catalog");
    assert!(legacy.iter().all(|(_, t)| t.cutter_hand == CutterHand::Right));
    let left = parse_catalog(&catalog_with(json!("left"))).expect("left-hand cutter catalog");
    let mill = left.iter().find(|(n, _)| n == "EM-0600-4FL").expect("end mill");
    assert_eq!(mill.1.cutter_hand, CutterHand::Left);
    assert!(mill.1.is_sane(), "left-handed cutters are representable");
}

#[test]
fn catalog_refuses_invalid_cutter_hands_by_tool_and_field_name() {
    for bad in [json!("backwards"), json!(7), Value::Null] {
        let err = parse_catalog(&catalog_with(bad.clone())).expect_err("bad hand");
        assert!(err.contains("EM-0600-4FL") && err.contains("cutter hand"), "{bad}: {err}");
    }
}
