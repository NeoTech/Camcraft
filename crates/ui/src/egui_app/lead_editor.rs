//! Shared 2D lead-in/lead-out selectors. The vocabulary and numeric
//! progressive disclosure are one UI seam for Contour and Chamfer.
//!
//! Provenance: direct controls over this codebase's five machining
//! `Lead` shapes; no vendor UI or text was consulted.

use camcraft_app::machining2d::Lead;
use camcraft_app::units::Unit;
use egui::Ui;

use super::fields::{combo_changed, num};
use super::layout::{prop_row, prop_row_hint};

const LEADS: [&str; 5] = [
    "None",
    "Linear",
    "Arc",
    "Tangential",
    "Perpendicular",
];
const LEAD_OUTS: [&str; 6] = [
    "Same as lead-in",
    "None",
    "Linear",
    "Arc",
    "Tangential",
    "Perpendicular",
];

/// Edit one required lead shape. Returns true when the user selects it;
/// the borrowed numeric field is written directly into the draft.
pub(super) fn lead_editor(
    ui: &mut Ui,
    u: Unit,
    id: &str,
    label: &str,
    lead: &mut Lead,
) -> bool {
    let mut index = lead_index(*lead);
    let mut changed = false;
    prop_row(ui, label, |ui| {
        changed = combo_changed(ui, id, &mut index, &LEADS);
    });
    if changed {
        *lead = lead_at(index);
    }
    lead_size_row(ui, u, lead, false);
    changed
}

/// Edit the optional, mirror-aware lead-out shape. `None` is "same as
/// lead-in"; `Some(Lead::None)` is a distinct explicit off state.
pub(super) fn lead_out_editor(
    ui: &mut Ui,
    u: Unit,
    id: &str,
    label: &str,
    lead: &mut Option<Lead>,
) -> bool {
    let mut index = match *lead {
        None => 0,
        Some(value) => lead_index(value) + 1,
    };
    let mut changed = false;
    prop_row_hint(
        ui,
        label,
        "a requested lead is halved where it is blocked, then refused by name when it still cannot stay in air",
        |ui| {
            changed = combo_changed(ui, id, &mut index, &LEAD_OUTS);
        },
    );
    if changed {
        *lead = (index != 0).then(|| lead_at(index - 1));
    }
    if let Some(value) = lead {
        lead_size_row(ui, u, value, true);
    }
    changed
}

fn lead_size_row(ui: &mut Ui, u: Unit, lead: &mut Lead, lead_out: bool) {
    let (length, radius) = if lead_out {
        ("Lead-out length", "Lead-out arc radius")
    } else {
        ("Lead length", "Lead arc radius")
    };
    match lead {
        Lead::Linear { length: value } | Lead::Perpendicular { length: value } => {
            prop_row(ui, length, |ui| num(ui, u, value));
        }
        Lead::Arc { radius: value } | Lead::Tangential { radius: value } => {
            prop_row(ui, radius, |ui| num(ui, u, value));
        }
        Lead::None => {}
    }
}

fn lead_index(lead: Lead) -> usize {
    match lead {
        Lead::None => 0,
        Lead::Linear { .. } => 1,
        Lead::Arc { .. } => 2,
        Lead::Tangential { .. } => 3,
        Lead::Perpendicular { .. } => 4,
    }
}

fn lead_at(index: usize) -> Lead {
    match index {
        1 => Lead::Linear { length: 8.0 },
        2 => Lead::Arc { radius: 5.0 },
        3 => Lead::Tangential { radius: 5.0 },
        4 => Lead::Perpendicular { length: 5.0 },
        _ => Lead::None,
    }
}
