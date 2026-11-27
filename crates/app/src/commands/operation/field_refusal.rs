//! The "not usable" refusal by name (#3545): when an operation's
//! parameters fail `OpParams::is_sane`, the commands say which field
//! broke which rule, in the field names that MCP and the project file
//! use, so a caller can correct the call.
//!
//! Provenance: our own domain model.

use crate::operation::OpParams;
use crate::project;

/// Top-level fields that every kind carrying them asks to be positive
/// (`OpParams::is_sane` and the nested kinds' own doors).
const POSITIVE: [&str; 6] = [
    "stepdown",
    "stepover",
    "feed",
    "plunge_feed",
    "tolerance",
    "helix_radius",
];

/// Top-level fields that every kind carrying them asks to be 0 or more.
const NON_NEGATIVE: [&str; 1] = ["stock_to_leave"];

/// The refusal for parameters that failed `is_sane`: the generic
/// sentence, followed by the first field that breaks its rule when
/// one of the shared fields is the cause.
pub(super) fn unusable(params: &OpParams) -> String {
    let head = format!(
        "operation parameters are not usable for a {} operation",
        params.kind()
    );
    match field_problem(params) {
        Some(why) => format!("{head}: {why}"),
        None => head,
    }
}

/// The first shared field that breaks its rule, by name and value.
/// Nested objects (a wire's 4-axis block, a tilt) are not read: their
/// fields have rules of their own.
fn field_problem(params: &OpParams) -> Option<String> {
    let json = project::params_to_json(params);
    // The parameters are externally tagged: `{ "<kind>": { fields } }`.
    let fields = json.as_object()?.values().next()?.as_object()?;
    let number = |key: &str| fields.get(key).and_then(serde_json::Value::as_f64);
    POSITIVE
        .iter()
        .find_map(|&key| {
            number(key)
                .filter(|v| !(v.is_finite() && *v > 0.0))
                .map(|v| format!("{key} must be greater than 0 (got {v})"))
        })
        .or_else(|| {
            NON_NEGATIVE.iter().find_map(|&key| {
                number(key)
                    .filter(|v| !(v.is_finite() && *v >= 0.0))
                    .map(|v| format!("{key} must be 0 or more (got {v})"))
            })
        })
}
