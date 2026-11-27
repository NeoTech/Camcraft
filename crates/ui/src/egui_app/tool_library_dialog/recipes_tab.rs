//! #3896 (U5 of #3663): the Recipes tab of the tool library window's
//! editor - the tool's speeds and feeds for each workpiece material. It
//! draws the same rows as Properties' Cutting data group
//! (`tool_cutting_rows::cutting_rows_with`). Suggest, Copy and Add write
//! the window's draft only: the editor's one Apply lands it as
//! `ChangeTool`, one undo step. No command is new.
//! #3930: the rest of Properties' old Cutting data group (the tool's own
//! spindle rpm and feed, the lathe rows) follows the table
//! (`tool_settings_grid::cutting_base`), so the summary page loses no edit.
//! #4337 (U11): one line says what a recipe is; a star marks the row of
//! the stock material in force (the setup stock's, else the document's),
//! the row operations take; each row's surface speed and chip load stand
//! under the table (`recipes_derived`, computed when their input
//! changes); the buttons (`recipes_actions`) and a units note follow.
//! Provenance: our own tab over the tool draft and the handbook relations
//! in `recipes_fill`; no HSMWorks material involved.

use egui::{RichText, Ui};

use camcraft_app::tool_cutting_rows::{CuttingRow, material_label};
use camcraft_app::tools::{ToolKind, ToolMaterial, WorkMaterial};
use camcraft_app::units::Unit;

use super::super::fields::combo;
use super::super::layout::{prop_row, prop_row_hint};
use super::super::tool_cutting_rows::{Extra, cutting_rows, cutting_rows_with};
use super::recipes_derived::derived_line;
use crate::state::DerivedKey;
use super::recipes_fill::{is_marked, names_material, rotating};
use crate::state::{State, ToolUi};


/// The tab's name, and its AccessKit label.
pub(crate) const TAB: &str = "Recipes";
/// The mark on the row of the stock material in force.
pub(crate) const STAR: &str = "★";

/// The flute counts the combo offers; a stated count outside it is shown
/// as its own entry and never rewritten (the operation page's rule).
const FLUTES: [u32; 7] = [1, 2, 3, 4, 5, 6, 8];

pub(super) fn draw(ui: &mut Ui, state: &mut State) {
    rows(ui, state);
    ui.separator();
    super::super::tool_settings_grid::cutting_base(ui, state);
}

/// The per-material rows, with the derived line and the buttons for a
/// rotating tool.
fn rows(ui: &mut Ui, state: &mut State) {
    let kind = state.tool_ui.tool_kind();
    if kind == ToolKind::Wire {
        note(ui, "An EDM wire has no cutting rows: its cut is the machine's EDM condition.");
        return;
    }
    let u = state.units();
    let torch = matches!(kind, ToolKind::Torch | ToolKind::Waterjet);
    if !rotating(kind) {
        cutting_rows(ui, &mut state.tool_ui, u, torch);
        return;
    }
    let active = camcraft_app::setup::active(&state.world);
    let stock = camcraft_app::stock_material::in_force(&state.world, active.as_deref());
    let named = stock.is_some_and(|m| names_material(&state.tool_ui.cutting, m));
    note(ui, &explanation(stock, named));
    let lead = |ui: &mut Ui, row: &CuttingRow| {
        if is_marked(row, stock, named) {
            ui.label(RichText::new(STAR).color(ui.visuals().text_color()))
                .on_hover_text("the row that operations on the stock material take");
        } else {
            ui.label("");
        }
    };
    cutting_rows_with(ui, &mut state.tool_ui, u, false, Some(&Extra { lead: &lead }));
    for line in derived(state, u) {
        note(ui, &line);
    }
    super::recipes_actions::buttons(ui, state, stock);
    note(ui, &units_note(u));
    edge_rows(ui, &mut state.tool_ui);
}

/// The tab's first line: what a recipe is, and which row operations take.
#[must_use]
pub(crate) fn explanation(stock: Option<WorkMaterial>, named: bool) -> String {
    let head = "One recipe for each work material.";
    match stock {
        Some(m) if named => format!(
            "{head} The {STAR} marks the stock material of this document ({}): operations take that row.",
            m.label()
        ),
        Some(m) => format!(
            "{head} No row names the stock material ({}), so operations take the Unknown row: the {STAR} marks it.",
            m.label()
        ),
        None => format!(
            "{head} No stock material is stated (the Stock page states it), so operations take the Unknown row and no row has the {STAR}."
        ),
    }
}

/// The derived line of each drawn row, top down (the default row first
/// when the draft stores none, as the table draws it), from the cache
/// while the input stays.
fn derived(state: &mut State, unit: Unit) -> Vec<String> {
    let t = &state.tool_ui;
    let lead = (!t.cutting.iter().any(CuttingRow::is_unknown)).then(|| CuttingRow::plain(None, t.rpm, t.feed));
    let keys: Vec<DerivedKey> = lead
        .iter()
        .chain(t.cutting.iter().filter(|r| !r.is_recipe()))
        .map(|r| DerivedKey { material: r.material, diameter: t.diameter, rpm: r.rpm, feed: r.feed, flutes: t.flutes, unit })
        .collect();
    let Some(view) = state.tool_library.as_mut() else {
        return keys.iter().map(derived_line).collect();
    };
    let c = &mut view.recipes;
    match &c.derived {
        Some((k, lines)) if *k == keys => lines.clone(),
        _ => {
            let lines: Vec<String> = keys.iter().map(derived_line).collect();
            c.derived = Some((keys, lines.clone()));
            c.computes = c.computes.saturating_add(1);
            lines
        }
    }
}

/// The units of the table's numbers and of the derived line.
#[must_use]
pub(crate) fn units_note(u: Unit) -> String {
    let (vc, fz) = match u {
        Unit::Millimeters => ("m/min", "mm"),
        Unit::Inches => ("sfm", "in"),
    };
    format!(
        "Units: feed {}, steps {}. Surface speed in {vc}, chip load in {fz} per tooth. A zero step reads \"not stated\": the {} row gives it.",
        u.feed_suffix(),
        u.suffix(),
        material_label(None)
    )
}

/// A dimmed line of the tab (the form's `hint` would turn into the
/// tooltip of the row above).
fn note(ui: &mut Ui, text: &str) {
    ui.add(egui::Label::new(RichText::new(text).weak()).wrap());
}

/// The tool's edge material and flute count: the inputs of the feed per
/// tooth and of "Suggest". Draft fields; Apply lands them.
fn edge_rows(ui: &mut Ui, t: &mut ToolUi) {
    tool_material_row(ui, t);
    flutes_row(ui, t);
}

/// The edge material combo. #4336: the Tool tab's CUTTER group draws it too.
pub(super) fn tool_material_row(ui: &mut Ui, t: &mut ToolUi) {
    let tm_names = ["Not stated", ToolMaterial::Hss.label(), ToolMaterial::Carbide.label()];
    let mut tm = t.tool_material.map_or(0, |m| m as usize + 1);
    prop_row_hint(ui, "Tool material", "the cutting edge's material; the table's speeds depend on it", |ui| {
        combo(ui, "tool_recipes_tool_material", &mut tm, &tm_names)
    });
    t.tool_material = tm.checked_sub(1).and_then(|i| ToolMaterial::ALL.get(i).copied());
}

/// The flute count combo. #4336: the Tool tab's CUTTER group draws it too.
pub(super) fn flutes_row(ui: &mut Ui, t: &mut ToolUi) {
    let odd = t.flutes.filter(|z| !FLUTES.contains(z));
    let names: Vec<String> = std::iter::once("Not stated".to_owned())
        .chain(FLUTES.iter().chain(odd.iter()).map(u32::to_string))
        .collect();
    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let mut z = match (odd, t.flutes) {
        (Some(_), _) => names.len() - 1,
        (None, Some(n)) => FLUTES.iter().position(|f| *f == n).map_or(0, |i| i + 1),
        (None, None) => 0,
    };
    prop_row(ui, "Flutes", |ui| combo(ui, "tool_recipes_flutes", &mut z, &refs));
    t.flutes = names.get(z).and_then(|n| n.parse::<u32>().ok());
}
