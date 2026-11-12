//! The `.mch` file's `edm.codes` (#542, #290 D5): the control's own
//! EDM words as machine data - the block a sinker post writes for a
//! condition select, a burn to depth, an electrode pickup and its
//! reference. A post writes the RS-274 motion words itself; every
//! EDM-specific code comes from here, and a missing one refuses the
//! post by name. No vendor table ships: a real control's codes are
//! filled from its manual, the same law as the M-codes held as data.
//!
//! Provenance: our own grammar (#290 D5); the code roles are general
//! die-sink EDM practice from public machine-tool handbooks. No vendor
//! machine file, manual table or post was copied.

use std::collections::BTreeMap;

/// The sinker code roles the generic sinker post (`sinker-iso.cps`)
/// reads, with the placeholders each template may hold (the post fills
/// them): the unit word for the machine's unit (`metric` or `inch`),
/// the electrode pickup and reference, the condition select, the burn
/// to depth and the program end. Orbit words are `edm.orbits[].word`
/// (`{radius}`).
pub const SINKER_CODES: [(&str, &str); 7] = [
    ("metric", ""),
    ("inch", ""),
    ("pickup", "{tool}"),
    ("reference", "{offset}"),
    ("condition", "{id}"),
    ("depth", "{z} {f}"),
    ("end", ""),
];

/// Read `edm.codes`: an object of role -> template text. A role may be
/// any word (wire codes join later, #533); every value must be a
/// non-empty string - a garbled code refuses the file by its role,
/// never reads as unstated.
///
/// # Errors
/// `codes` not an object, or a value that is not a non-empty string.
pub(super) fn parse_codes(v: Option<&serde_json::Value>) -> Result<BTreeMap<String, String>, String> {
    let map = match v {
        None | Some(serde_json::Value::Null) => return Ok(BTreeMap::new()),
        Some(serde_json::Value::Object(m)) => m,
        Some(x) => return Err(format!("machine file: edm.codes must be an object of role: code, not {x}")),
    };
    let mut out = BTreeMap::new();
    for (role, code) in map {
        match code {
            serde_json::Value::String(s) if !s.trim().is_empty() => {
                out.insert(role.clone(), s.clone());
            }
            x => return Err(format!("machine file: edm code {role:?} must be a non-empty word, not {x}")),
        }
    }
    Ok(out)
}

/// The same law at the project reader's gate.
#[must_use]
pub(super) fn codes_sane(codes: &BTreeMap<String, String>) -> bool {
    codes.values().all(|c| !c.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn codes_read_as_words_and_garbage_refuses_by_role() {
        let ok = parse_codes(Some(&json!({ "condition": "E{id}", "depth": "Z{z}" }))).expect("reads");
        assert_eq!(ok.get("condition").map(String::as_str), Some("E{id}"));
        assert!(parse_codes(None).expect("absent").is_empty());
        let err = parse_codes(Some(&json!({ "depth": 3 }))).expect_err("a number is no code");
        assert!(err.contains("\"depth\""), "{err}");
        let err = parse_codes(Some(&json!({ "pickup": " " }))).expect_err("blank");
        assert!(err.contains("\"pickup\""), "{err}");
        assert!(parse_codes(Some(&json!(["E1"]))).is_err());
    }
}
