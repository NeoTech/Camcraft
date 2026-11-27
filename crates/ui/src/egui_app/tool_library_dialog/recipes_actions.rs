//! #4337 (U11 of #3663): the Recipes tab's button row under the table -
//! "Add material…", "Suggest feeds and speeds" and "Copy recipes from
//! tool…". Each one writes the window's draft only; the editor's one
//! Apply lands it as `ChangeTool`, one undo step. A button that cannot act
//! is disabled and says why on hover.
//! Provenance: our own controls over `recipes_fill`; no HSMWorks
//! material involved.

use egui::Ui;

use camcraft_app::model;
use camcraft_app::tool_cutting_rows::CuttingRow;
use camcraft_app::tools::WorkMaterial;

use super::recipes_fill::{add_material, addable, copy_rows, copyable, suggest, suggest_refusal};
use crate::state::State;

/// The menu button that adds a row for a material.
pub(crate) const ADD: &str = "Add material…";
/// The menu button that fills a material's row from the cutting table.
pub(crate) const SUGGEST: &str = "Suggest feeds and speeds";
/// The menu button that copies the rows of another tool.
pub(crate) const COPY: &str = "Copy recipes from tool…";

/// The three buttons. `stock` is the stock material in force: Suggest
/// offers it first.
pub(super) fn buttons(ui: &mut Ui, state: &mut State, stock: Option<WorkMaterial>) {
    ui.horizontal_wrapped(|ui| {
        add_button(ui, state);
        suggest_button(ui, state, stock);
        copy_button(ui, state);
    });
}

fn add_button(ui: &mut Ui, state: &mut State) {
    let offers: Vec<WorkMaterial> = addable(&state.tool_ui.cutting).collect();
    ui.add_enabled_ui(!offers.is_empty(), |ui| {
        ui.menu_button(ADD, |ui| {
            for m in &offers {
                if ui.button(m.label()).clicked() {
                    if add_material(&mut state.tool_ui, *m) {
                        state.status = format!("{} row added - Apply writes it to the tool", m.label());
                    }
                    ui.close();
                }
            }
        })
        .response
        .on_hover_text("a row for one more work material, from the tool's own rpm and feed")
        .on_disabled_hover_text("each work material has its row");
    });
}

/// The materials Suggest offers: the stock material first, then each one
/// that a plain row of the draft names.
#[must_use]
pub(crate) fn suggest_offers(rows: &[CuttingRow], stock: Option<WorkMaterial>) -> Vec<WorkMaterial> {
    let named = rows.iter().filter(|r| !r.is_recipe()).filter_map(|r| r.material);
    let mut v: Vec<WorkMaterial> = stock.into_iter().chain(named).collect();
    let mut seen = Vec::with_capacity(v.len());
    v.retain(|m| {
        let new = !seen.contains(m);
        seen.push(*m);
        new
    });
    v
}

/// "Suggest feeds and speeds": a menu of the materials; the pick fills
/// that material's row from the cutting table.
fn suggest_button(ui: &mut Ui, state: &mut State, stock: Option<WorkMaterial>) {
    let offers = suggest_offers(&state.tool_ui.cutting, stock);
    let why = suggest_refusal(&state.tool_ui).or_else(|| {
        offers.is_empty().then(|| "add a material row, or state the stock material, first".to_owned())
    });
    let mut pick = None;
    ui.add_enabled_ui(why.is_none(), |ui| {
        ui.menu_button(SUGGEST, |ui| {
            for m in &offers {
                let text = if Some(*m) == stock { format!("{} {}", super::recipes_tab::STAR, m.label()) } else { m.label().to_owned() };
                if ui.button(text).clicked() {
                    pick = Some(*m);
                    ui.close();
                }
            }
        })
        .response
        .on_hover_text("fill a material's row with the cutting table's spindle rpm and feed for this tool")
        .on_disabled_hover_text(why.as_deref().unwrap_or_default());
    });
    let Some(m) = pick else {
        return;
    };
    let machine = camcraft_app::machine::of_setup(&state.world, None).1;
    state.status = match suggest(&mut state.tool_ui, m, machine.as_ref()) {
        Ok(()) => format!("{} row suggested - Apply writes it to the tool", m.label()),
        Err(why) => format!("No suggestion: {why}"),
    };
}

/// "Copy recipes from tool…": a menu of the other document tools. The
/// rows of a tool are read only while the menu is open.
fn copy_button(ui: &mut Ui, state: &mut State) {
    let me = state.tool_ui.name.clone();
    let others: Vec<String> = state.tool_names().into_iter().filter(|n| *n != me).collect();
    ui.add_enabled_ui(!others.is_empty(), |ui| {
        ui.menu_button(COPY, |ui| {
            for n in &others {
                let rows: Vec<CuttingRow> = model::find_by_name(&state.world, n)
                    .and_then(|e| model::tool_of(&state.world, &e))
                    .map(|t| copyable(&t.cutting).copied().collect())
                    .unwrap_or_default();
                let b = ui
                    .add_enabled(!rows.is_empty(), egui::Button::new(n.as_str()))
                    .on_disabled_hover_text(format!("tool {n} has no material rows"));
                if b.clicked() {
                    let count = copy_rows(&mut state.tool_ui, &rows);
                    state.status = format!("{count} rows copied from tool {n} - Apply writes them to the tool");
                    ui.close();
                }
            }
        })
        .response
        .on_hover_text("copy another tool's material rows into this draft")
        .on_disabled_hover_text("no other tool is in the document");
    });
}
