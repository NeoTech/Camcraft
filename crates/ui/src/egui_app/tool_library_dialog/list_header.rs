//! #4335 (U9 of #3663): the one header row of the tool list - the source
//! tabs (Document, My library, Catalog), the one search box, then New
//! tool…, Import… and Export… at the right. Import… opens a small menu
//! with the import mode (#899), the "Import tool holders" choice (#889)
//! and the file button, so the row stays one row. Esc in the box clears
//! it. All of it is view state; the import and New tool are the commands.
//! Provenance: our own header row after the #3663 mock-up and the egui
//! menu docs; no HSMWorks material involved.

use egui::containers::menu::{MenuButton, MenuConfig};
use egui::{Align, Key, Layout, PopupCloseBehavior, RichText, TextEdit, Ui, WidgetInfo};

use crate::browser_files::{self, NEEDS_PICKER, NO_EXPORT};
use crate::state::{ErrorScope, State, ToolSource};
use crate::{actions, egui_app::colorvision, egui_app::error_text};
use camcraft_app::commands::ImportMode;

/// The search box's AccessKit name and its placeholder text.
pub(crate) const SEARCH: &str = "Search tools";
pub(crate) const PLACEHOLDER: &str = "Filter by name, diameter or part number";
/// The buttons of the row.
pub(crate) const NEW: &str = "New tool…";
pub(crate) const IMPORT: &str = "Import…";
pub(crate) const EXPORT: &str = "Export…";
/// The file button and the holders choice in the Import… menu.
pub(crate) const CHOOSE_FILE: &str = "Choose a library file…";
pub(crate) const HOLDERS: &str = "Import tool holders";
/// The source tabs, in the order the row shows them.
pub(crate) const DOCUMENT: &str = "Document";
pub(crate) const CATALOG: &str = "Catalog";
/// The width the three buttons need at the row's right.
const BUTTONS_W: f32 = 230.0;

/// The header row, then the Tool refusal (if any).
pub(super) fn draw(ui: &mut Ui, state: &mut State) {
    let Some(source) = state.tool_library.as_ref().map(|v| v.source) else {
        return;
    };
    let mut picked = source;
    let (mut new, mut export, mut import) = (false, false, false);
    ui.horizontal(|ui| {
        ui.selectable_value(&mut picked, ToolSource::Document, DOCUMENT).on_hover_text("the tools of this document");
        ui.selectable_value(&mut picked, ToolSource::Library, super::components::SOURCE)
            .on_hover_text("the tool components: the built-in ones, your own library file and the document's");
        ui.selectable_value(&mut picked, ToolSource::Catalog, CATALOG)
            .on_hover_text("the built-in catalog: Add puts a tool into the document");
        ui.add_space(8.0);
        let w = (ui.available_width() - BUTTONS_W).max(120.0);
        search_box(ui, &mut state.tool_search, w);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            export = browser_files::button(
                ui,
                state.in_browser(NO_EXPORT),
                EXPORT,
                "write the document's tools to a CamCraft (.camtools), PathPilot (.csv) or Hurco (.TCM) file",
            );
            import = import_menu(ui, state);
            new = ui.button(NEW).on_hover_text("make a new tool in this document").clicked();
        });
    });
    if picked != source
        && let Some(view) = state.tool_library.as_mut()
    {
        view.source = picked;
    }
    if new {
        actions::begin_new_tool(state);
    }
    if export {
        state.export_tool_library_dialog();
    }
    if import {
        state.import_tool_library_dialog();
    }
    if let Some(msg) = state.error_for(ErrorScope::Tool).map(str::to_owned) {
        error_text::error_text(ui, colorvision::text_inks(ui, &state.world).error, &msg);
    }
}

/// The search box: the placeholder while it is empty; Esc clears it.
fn search_box(ui: &mut Ui, text: &mut String, w: f32) {
    let r = ui.add(TextEdit::singleline(text).hint_text(PLACEHOLDER).desired_width(w));
    let shown = text.clone();
    r.widget_info(|| {
        let mut info = WidgetInfo::text_edit(true, &shown, &shown, PLACEHOLDER);
        info.label = Some(SEARCH.to_owned());
        info
    });
    let r = r.on_hover_text("words and numbers, each must match: \"6 ball\" finds the 6 mm ball nose mills; Esc clears");
    if (r.has_focus() || r.lost_focus()) && ui.input(|i| i.key_pressed(Key::Escape)) {
        text.clear();
    }
}

/// Import…: a menu with the mode, the holders choice and the file
/// button. Gives whether the file button was clicked.
fn import_menu(ui: &mut Ui, state: &mut State) -> bool {
    let mut choose = false;
    // A click on a mode or the check box keeps the menu open.
    let cfg = MenuConfig::new().close_behavior(PopupCloseBehavior::CloseOnClickOutside);
    let (r, _) = MenuButton::new(IMPORT).config(cfg).ui(ui, |ui| {
        ui.set_min_width(220.0);
        ui.label(RichText::new("A NAME THE DOCUMENT HAS").small().weak());
        let mode = &mut state.tool_import_mode;
        ui.selectable_value(mode, ImportMode::Add, "Add (rename duplicates)")
            .on_hover_text("an identical tool is skipped, a different tool of a taken name comes in as \"Name (2)\"");
        ui.selectable_value(mode, ImportMode::Update, "Update same-named")
            .on_hover_text("a same-named tool takes the file's definition in place (number kept; operations using it go Stale); new names are added");
        ui.selectable_value(mode, ImportMode::Replace, "Replace library")
            .on_hover_text("update, and tools the file lacks are removed - refused if an operation uses one");
        ui.separator();
        let mut holders = state.import_tool_holders;
        if ui
            .checkbox(&mut holders, HOLDERS)
            .on_hover_text("take each tool's holder body from the library file; unticked, tools arrive with their holder cylinder only")
            .changed()
        {
            state.set_import_tool_holders(holders);
        }
        ui.separator();
        if browser_files::button(
            ui,
            state.in_browser(NEEDS_PICKER),
            CHOOSE_FILE,
            "import tools from a CamCraft (.camtools/.json), PathPilot (.csv) or Hurco (.TCM) file (one undo step)",
        ) {
            choose = true;
            ui.close();
        }
    });
    r.on_hover_text("import tools from a library file: pick what happens to a taken name, then the file");
    choose
}
