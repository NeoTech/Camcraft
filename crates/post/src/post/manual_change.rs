//! The manual tool change law (#4085): a machine whose installation states a
//! manual tool change (`tooling.tool_change` = `manual` or
//! `manual-retract-top`) is posted only by a script that declares that it
//! writes the change for that mode: `manualToolChange = ["manual-retract-top"]`.
//! A script that does not declare the mode would write a tool call with no
//! stop, and the program would run on with the old tool in the spindle and the
//! new tool's length active. Such a pair is refused by name, with the post and
//! the machine, before any hook runs.
//!
//! Provenance: this engine's manifest-flag refusal pattern (`units.rs`,
//! `coolantModes`, `afterModes`) and the machine-file grammar of #3181. No
//! vendor material.

use super::js::Script;
use super::PostError;
use serde_json::Value;

/// The manual tool change modes of the machine-file grammar (#3181).
const MANUAL_MODES: [&str; 2] = ["manual", "manual-retract-top"];

/// The refusal for a script that does not declare the machine's manual tool
/// change mode, or `None` when the script may run. `mch` is the checked
/// machine (static form, or a file form without installation facts).
pub fn refusal(s: &mut Script, mch: &Value) -> Result<Option<String>, PostError> {
    let Some(mode) = mch
        .pointer("/tooling/tool_change")
        .and_then(Value::as_str)
        .filter(|m| MANUAL_MODES.contains(m))
    else {
        return Ok(None);
    };
    let declared = s.words_global("manualToolChange")?.unwrap_or_default();
    if declared.iter().any(|w| w == mode) {
        return Ok(None);
    }
    let who = super::units::post_name(s)?;
    let machine = machine_name(mch);
    let what = if declared.is_empty() {
        "does not declare manualToolChange (it writes no stop for a manual tool change)".to_owned()
    } else {
        format!("handles only the manual tool change modes {}", declared.join(", "))
    };
    Ok(Some(format!(
        "{machine} changes tools by hand (tooling.tool_change = \"{mode}\"), but {who} {what} - the program would run on with the old tool and the new tool's length; pick a post that writes the manual tool change for this machine"
    )))
}

/// The machine by the name its file gives, or "this machine".
fn machine_name(mch: &Value) -> String {
    mch.get("name")
        .and_then(Value::as_str)
        .or_else(|| mch.pointer("/general/model").and_then(Value::as_str))
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .map_or_else(|| "this machine".to_owned(), |n| format!("the machine {n}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn manual(mode: &str) -> Value {
        json!({ "name": "Hand Mill", "tooling": { "tool_change": mode } })
    }

    #[test]
    fn a_manual_machine_needs_the_declared_mode() {
        let mut none = Script::load("description = \"Plain ISO (3-axis mill)\";").expect("loads");
        for mode in MANUAL_MODES {
            let why = refusal(&mut none, &manual(mode)).expect("reads").expect("refused");
            assert!(why.contains("the machine Hand Mill changes tools by hand"), "{why}");
            assert!(why.contains("the Plain ISO post does not declare manualToolChange"), "{why}");
            assert!(why.contains(mode), "{why}");
        }
        let mut top = Script::load("manualToolChange = [\"manual-retract-top\"];").expect("loads");
        assert_eq!(refusal(&mut top, &manual("manual-retract-top")).expect("reads"), None);
        let why = refusal(&mut top, &manual("manual")).expect("reads").expect("refused");
        assert!(why.contains("handles only the manual tool change modes manual-retract-top"), "{why}");
    }

    #[test]
    fn an_automatic_or_unstated_change_never_asks_the_manifest() {
        let mut none = Script::load("function onOpen() {}").expect("loads");
        assert_eq!(refusal(&mut none, &manual("changer")).expect("reads"), None);
        assert_eq!(refusal(&mut none, &json!({ "name": "VMC" })).expect("reads"), None);
        assert_eq!(refusal(&mut none, &Value::Null).expect("reads"), None);
    }
}
