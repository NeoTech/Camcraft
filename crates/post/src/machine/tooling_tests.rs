//! #828: the tooling keys' reader and refusals, by name.

use super::*;
use serde_json::json;

fn keys(v: &Value) -> Vec<String> {
    tooling_refusals(v).into_iter().map(|(k, _)| k).collect()
}

#[test]
fn absent_keys_read_unstated() {
    assert_eq!(parse_tooling(&json!({ "tooling": { "number_of_tools": 10 } })), Ok((None, None)));
    assert_eq!(parse_tooling(&json!({})), Ok((None, None)));
    assert_eq!(parse_tooling(&json!({ "tooling": { "spindle_mount": null, "default_holder": null } })), Ok((None, None)));
}

#[test]
fn stated_keys_read_canonical() {
    let v = json!({ "tooling": { "spindle_mount": "bt30", "default_holder": "bt30  er20" } });
    assert_eq!(parse_tooling(&v), Ok((Some("BT30".into()), Some("BT30 ER20".into()))));
    let v = json!({ "tooling": { "default_holder": "TTS ER20" } });
    assert_eq!(parse_tooling(&v), Ok((None, Some("TTS ER20".into()))));
}

#[test]
fn an_unknown_holder_is_refused_by_name() {
    let v = json!({ "tooling": { "default_holder": "BT30 ER99" } });
    let e = parse_tooling(&v).expect_err("refused");
    assert!(e.starts_with("tooling.default_holder:") && e.contains("BT30 ER99"), "{e}");
}

#[test]
fn a_holder_off_the_mount_is_refused_by_name() {
    let v = json!({ "tooling": { "spindle_mount": "TTS", "default_holder": "BT30 ER20" } });
    let e = parse_tooling(&v).expect_err("refused");
    assert!(e.contains("tooling.default_holder") && e.contains("BT30") && e.contains("TTS"), "{e}");
    let v = json!({ "tooling": { "spindle_mount": "BT40", "default_holder": "ER20 chuck" } });
    assert_eq!(keys(&v), vec!["tooling.default_holder".to_owned()]);
}

#[test]
fn malformed_words_are_refused_by_name() {
    let v = json!({ "tooling": { "spindle_mount": "R8", "default_holder": 3 } });
    assert_eq!(keys(&v), vec!["tooling.spindle_mount".to_owned(), "tooling.default_holder".to_owned()]);
    let v = json!({ "tooling": { "default_holder": "" } });
    assert_eq!(keys(&v), vec!["tooling.default_holder".to_owned()]);
}

#[test]
fn stored_values_re_judge_the_same() {
    assert_eq!(mount_check(Some("BT30"), Some("BT30 ER20")), Ok(()));
    assert_eq!(mount_check(None, None), Ok(()));
    assert!(mount_check(Some("TTS"), Some("BT30 ER20")).is_err());
    assert!(mount_check(None, Some("no such holder")).is_err());
}
