//! #3892 (U1 of #3663): the tool library window. It replaces the old
//! "Tool library" dock tab: a non-modal in-app `egui::Window` that can stay
//! open beside the viewport while Properties shows its own subject. Each
//! door (the ribbon's Tools cell, the View menu, the gate route) opens it
//! through `actions::target` with the empty tool name.
//! #3893 (U2): a tool opens in the window's editor. The list folds into a
//! rail on the left; the editor has a Tool tab and a Details tab, and one
//! Apply lands the draft through `ChangeTool` (and the tool number through
//! `SetToolNumber`, as in Properties). The window's draft is its own
//! (`ToolLibraryView::editor`), so Properties' draft never mixes with it.
//! #3894 (U3): the editor shows the tool above its tabs - an orbitable 3D
//! model (the default) or a to-scale side view with dimension lines
//! (`preview`). Display only.
//! #3895 (U4): the Tool tab's assembly group sets the holder, the gauge
//! length and the stick-out from the window's own holder draft; Apply
//! holder is the one undoable `SetToolBody::library`.
//! #3896 (U5): the Recipes tab shows the cutting data per material with
//! the surface speed and the feed per tooth. "Suggest" and "Copy from
//! tool" fill the window's draft only; the same Apply lands it.
//! Provenance: our own window, after the tool shape window pattern
//! (`pages::tool_shape_editor_window`) and the egui `Window` docs; no
//! HSMWorks material involved.

pub(crate) mod assembly_group;
mod details_tab;
mod list;
pub(crate) mod model;
pub(crate) mod preview;
mod rail;
mod recipes_fill;
pub(crate) mod recipes_tab;
mod side_view;
mod tool_tab;

use egui::{Id, Ui, Window};

use crate::actions;
use crate::state::{ErrorScope, State, ToolEditorTab, UiTarget};

/// The window's title, and its AccessKit name.
pub(crate) const TITLE: &str = "Tool library";

/// The rail's width when a tool is open.
const RAIL_W: f32 = 170.0;

/// Draw the window when it is open (`state.tool_library` is `Some`).
/// Its ✕ closes it; closing is view state, nothing is queued.
pub(crate) fn tool_library_window(ctx: &egui::Context, state: &mut State) {
    if state.tool_library.is_none() {
        return;
    }
    let screen = ctx.viewport_rect();
    let mut open = true;
    Window::new(TITLE)
        .id(Id::new("tool-library"))
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        // Over the viewport, left of the Properties pane, so the two can
        // be used at the same time.
        .default_pos([screen.left() + screen.width() * 0.2, screen.top() + 0.15 * screen.height()])
        .default_size([620.0, (screen.height() * 0.7).max(320.0)])
        .min_width(420.0)
        .show(ctx, |ui| body(ui, state));
    if !open {
        state.close_tool_library();
    }
}

/// The full list, or the rail beside the editor when a tool is open.
fn body(ui: &mut Ui, state: &mut State) {
    if state.tool_library.as_ref().and_then(|v| v.open_tool()).is_none() {
        list::draw(ui, state);
        return;
    }
    let pick = ui
        .horizontal_top(|ui| {
            let pick = ui
                .vertical(|ui| {
                    ui.set_width(RAIL_W);
                    rail::draw(ui, state)
                })
                .inner;
            ui.separator();
            ui.vertical(|ui| editor(ui, state));
            pick
        })
        .inner;
    match pick {
        Some(rail::Pick::List) => state.close_tool_in_library(),
        Some(rail::Pick::Tool(name)) => state.open_tool_in_library(&name),
        None => {}
    }
}

/// What the editor asks for after its draft is put back.
enum After {
    None,
    /// The open tool left the document (deleted or renamed).
    Gone,
    /// "Edit tool shape" on this tool.
    Shape(String),
}

/// The heading, the preview, the Tool, Recipes and Details tabs, and the Apply row. The draft
/// is swapped into `tool_ui` only while it is drawn (`with_library_draft`).
fn editor(ui: &mut Ui, state: &mut State) {
    let after = state
        .with_library_draft(|state, face| {
            if !state.tool_ui.exists {
                return After::Gone;
            }
            let name = state.tool_ui.name.clone();
            // #3895: the holder draft follows the tool before the preview
            // draws it (a turning tool has none).
            if crate::state::is_spun_kind(state.tool_ui.tool_kind()) {
                state.sync_tool_profile_ui();
            }
            ui.heading(format!("Tool: {name}"));
            preview::draw(ui, state, face);
            let tab = &mut face.tab;
            let mut shape = false;
            ui.horizontal(|ui| {
                ui.selectable_value(tab, ToolEditorTab::Tool, "Tool")
                    .on_hover_text("type, geometry and the rows the tool's kind reads");
                ui.selectable_value(tab, ToolEditorTab::Recipes, recipes_tab::TAB)
                    .on_hover_text("speeds and feeds for each workpiece material");
                ui.selectable_value(tab, ToolEditorTab::Details, "Details")
                    .on_hover_text("material group, catalog facts and the tool number");
            });
            ui.separator();
            let footer_h = if state.error_for(ErrorScope::Tool).is_some() { 68.0 } else { 40.0 };
            egui::ScrollArea::vertical()
                .id_salt("tool_library_editor")
                .auto_shrink([false, false])
                .max_height((ui.available_height() - footer_h).max(80.0))
                .show(ui, |ui| match *tab {
                    ToolEditorTab::Tool => shape = tool_tab::draw(ui, state),
                    ToolEditorTab::Recipes => recipes_tab::draw(ui, state),
                    ToolEditorTab::Details => details_tab::draw(ui, state),
                });
            footer(ui, state, &name);
            if shape { After::Shape(name) } else { After::None }
        })
        .unwrap_or(After::None);
    match after {
        After::None => {}
        After::Gone => state.close_tool_in_library(),
        After::Shape(name) => open_shape(state, &name),
    }
}

/// Apply, with the Tool pane's refusal above it. One Apply queues the
/// draft (`ChangeTool`, or `ReshapeTool` when a drawn cutter rides it)
/// and, when the number changed, `SetToolNumber` - the same commands as
/// Properties' Apply.
fn footer(ui: &mut Ui, state: &mut State, name: &str) {
    ui.separator();
    if let Some(msg) = state.error_for(ErrorScope::Tool).map(str::to_owned) {
        super::error_text::error_text(ui, super::colorvision::text_inks(ui, &state.world).error, &msg);
    }
    let refusal = (state.tool_ui.tool_kind() == camcraft_app::tools::ToolKind::Waterjet)
        .then(|| state.tool_ui.to_tool().torch_refusal())
        .flatten();
    if ui
        .add_enabled(refusal.is_none(), egui::Button::new("Apply"))
        .on_hover_text("write this draft to the tool (one undo step)")
        .on_disabled_hover_text(refusal.as_deref().unwrap_or_default())
        .clicked()
    {
        let apply = state.apply_tool_numbers(name);
        state.pending.push_back(apply);
        super::tool_settings::push_number_if_changed(state, name);
    }
}

/// "Edit tool shape": the shape window edits through Properties' draft
/// (`tool_ui`), so when Properties shows another tool, it follows this one
/// first. Otherwise the two windows would show two tools in one draft.
fn open_shape(state: &mut State, name: &str) {
    if matches!(&state.ui_target, UiTarget::Tool(n) if !n.is_empty() && n != name) {
        actions::target(state, UiTarget::Tool(name.to_owned()));
    }
    actions::open_tool_shape_editor(state, name);
}
