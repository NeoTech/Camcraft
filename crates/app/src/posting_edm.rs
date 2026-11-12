//! The sinker burn's stream PARAMs (#542): the op's condition id and the
//! electrode's pickup offset, said before the section opens so the
//! sinker post can select the condition and reference the electrode in
//! its preamble. Every other op's stream is unchanged.
//!
//! Provenance: our own stream vocabulary (docs/CNC-INTERMEDIATE.md) over
//! the umbrella's decisions (#290 D4, D5). No vendor material.

use crate::operation::OpParams;
use crate::post::cnc::Record;
use crate::tools::{Tool, ToolKind};
use serde_json::json;

/// Insert `PARAM EdmCondition` and `PARAM ElectrodePickup` ahead of the
/// stream's first TOOL / SECTION record, for a Sinker Burn on an
/// electrode. A blank condition is not said - the post refuses the
/// section by name for want of it.
pub(super) fn insert(params: &OpParams, tool: &Tool, records: &mut Vec<Record>) {
    let OpParams::SinkerBurn(sb) = params else { return };
    if tool.kind != ToolKind::Electrode {
        return;
    }
    let mut add = Vec::new();
    let id = sb.condition.trim();
    if !id.is_empty() {
        add.push(Record::Param { name: "EdmCondition".into(), value: json!(id) });
    }
    if tool.shank_len.is_finite() && tool.shank_len >= 0.0 {
        add.push(Record::Param { name: "ElectrodePickup".into(), value: json!(tool.shank_len) });
    }
    let at = records
        .iter()
        .position(|r| matches!(r, Record::Tool { .. } | Record::Section { .. }))
        .unwrap_or(records.len());
    records.splice(at..at, add);
}
