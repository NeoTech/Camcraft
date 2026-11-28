//! Waterjet machine doors (#1881).
//! Provenance: our own rules from the issue's XY gantry and optional Z specification.

use serde_json::Value;
use super::{kind, Kind, Machine};
use super::no_spindle::{xy_gantry_refusals, NoSpindle};

/// File-form refusals, shared by the parser and editor validator.
#[must_use]
pub fn waterjet_refusals(v: &Value) -> Vec<(String, String)> {
    let rules = NoSpindle {
        spindle: "a waterjet has no spindle; remove the spindle key",
        live_tool: "a waterjet has no live tool",
        coolant: "a waterjet states no milling coolant keys",
    };
    xy_gantry_refusals(v, "waterjet", &rules)
}

impl Machine {
    /// The same rules for machines loaded from a saved project.
    #[must_use]
    pub fn waterjet_sane(&self) -> bool {
        if kind(Some(self)) != Kind::Waterjet { return true; }
        self.xy_gantry_sane()
    }
}
