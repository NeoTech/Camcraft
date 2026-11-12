//! The EDM side of the script's `machine` global (#542, #290 D5): a
//! wire or sinker machine's declared kind and its `edm` section (codes,
//! orbits, pickup, conditions) cross into the script, and the serves
//! law - a post that declares the machine kinds it serves (`serves =
//! "sinker_edm";`) posts for those kinds only, and an EDM machine takes
//! only a post that declares its kind.
//!
//! Provenance: our own design over the umbrella's decisions (#290
//! D1-D10). No vendor post or machine file was read.

use super::PostError;
use crate::machine::{Kind, Machine};
use serde_json::{json, Value};

/// The kinds only a post written for them may serve.
fn is_edm(kind: Kind) -> bool {
    matches!(kind, Kind::WireEdm | Kind::SinkerEdm)
}

/// The mirror's EDM keys for a checker-parsed machine: its kind word
/// and its `edm` section. Nothing for any other kind, so every other
/// mirror is byte-identical to the one before #542.
pub(super) fn mirror_edm(m: &Machine, out: &mut Value) {
    let kind = crate::machine::kind(Some(m));
    if !is_edm(kind) {
        return;
    }
    if let Some(o) = out.as_object_mut() {
        o.insert("kind".to_owned(), json!(kind.word()));
        let edm = m.edm.as_ref().and_then(|e| serde_json::to_value(e).ok()).unwrap_or_else(|| json!({}));
        o.insert("edm".to_owned(), edm);
    }
}

/// The `edm` object the script reads: the mirror's own (already in mm),
/// or - through the raw door - the file read by the same parser the app
/// mounts with, so units scale the same way. `null` when no EDM section
/// is stated or the raw file does not parse (the script then refuses
/// for want of the code it needs, by name).
pub(super) fn edm_prop(m: &Value) -> Value {
    let raw = m.get("kinematics").and_then(|k| k.get("default")).is_some();
    if raw {
        return crate::machine::parse_mch(&m.to_string())
            .ok()
            .and_then(|p| p.edm)
            .and_then(|e| serde_json::to_value(e).ok())
            .unwrap_or(Value::Null);
    }
    m.get("edm").filter(|e| e.is_object()).cloned().unwrap_or(Value::Null)
}

/// The serves law, checked before `onOpen`: `props` is the script's
/// `machine` global, `serves` the script's declared kind words (comma
/// separated; `None` = the script declares none, a milling/turning/
/// plasma dialect as before).
///
/// # Errors
/// An EDM machine under a post that does not declare its kind, or a
/// post that declares kinds the machine is not.
pub(super) fn serves_gate(serves: Option<&str>, props: &Value) -> Result<(), PostError> {
    let word = props.get("kind").and_then(Value::as_str).unwrap_or("");
    let kind = Kind::from_word(word);
    let name = props
        .get("name")
        .and_then(Value::as_str)
        .filter(|n| !n.is_empty())
        .or_else(|| props.get("model").and_then(Value::as_str).filter(|n| !n.is_empty()))
        .unwrap_or("the machine");
    let declared: Vec<Kind> = serves
        .map(|s| s.split(',').filter_map(|w| Kind::from_word(w.trim())).collect())
        .unwrap_or_default();
    match (serves, kind) {
        (None, Some(k)) if is_edm(k) => Err(PostError::Refused(format!(
            "{name} is a {} machine, and this post does not declare that it serves one - \
             only a post written for an EDM control (serves = \"{}\") may post for it",
            k.label(),
            k.word()
        ))),
        (Some(s), _) if declared.is_empty() => {
            Err(PostError::Refused(format!("the post declares serves = {s:?}, which names no machine kind")))
        }
        (Some(s), k) if !k.is_some_and(|k| declared.contains(&k)) => Err(PostError::Refused(format!(
            "this post serves {s} machines only - {name} is {}",
            k.map_or_else(|| "not declared one of them".to_owned(), |k| format!("a {} machine", k.label()))
        ))),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_serves_law_reads_both_ways() {
        let sinker = json!({ "name": "S", "kind": "sinker_edm" });
        let mill = json!({ "name": "M", "kind": "" });
        let err = serves_gate(None, &sinker).expect_err("an undeclared post on a sinker");
        assert!(err.to_string().contains("sinker EDM") && err.to_string().contains("serves"), "{err}");
        assert!(serves_gate(Some("sinker_edm"), &sinker).is_ok());
        assert!(serves_gate(None, &mill).is_ok(), "a mill post on a mill");
        let err = serves_gate(Some("sinker_edm"), &mill).expect_err("a sinker post on a mill");
        assert!(err.to_string().contains("serves sinker_edm machines only"), "{err}");
        let err = serves_gate(Some("wire_edm"), &sinker).expect_err("a wire post on a sinker");
        assert!(err.to_string().contains("sinker EDM machine"), "{err}");
        assert!(serves_gate(Some("nonsense"), &sinker).is_err());
    }
}
