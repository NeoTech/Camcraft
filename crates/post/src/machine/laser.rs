//! Laser cutter machine doors (#4081): an X/Y gantry, an optional Z that
//! sets the focus height, no spindle and no rotary axis.
//! Provenance: our own rules, on the waterjet doors (#1881) and general
//! laser-cutting practice (the beam cuts in X/Y; nothing turns).

use serde_json::Value;
use super::{kind, Kind, Machine};
use super::no_spindle::{xy_gantry_refusals, NoSpindle};

/// File-form refusals, shared by the parser and the editor validator.
#[must_use]
pub fn laser_refusals(v: &Value) -> Vec<(String, String)> {
    let rules = NoSpindle {
        spindle: "a laser has no spindle; remove the spindle key",
        live_tool: "a laser has no live tool",
        coolant: "a laser states no milling coolant keys",
    };
    xy_gantry_refusals(v, "laser", &rules)
}

impl Machine {
    /// The same rules for machines loaded from a saved project.
    #[must_use]
    pub fn laser_sane(&self) -> bool {
        if kind(Some(self)) != Kind::Laser { return true; }
        self.xy_gantry_sane()
    }
}

#[cfg(test)]
#[path = "laser_tests.rs"]
mod tests;
