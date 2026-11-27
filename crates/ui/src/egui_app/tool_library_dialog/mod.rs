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
//! #3897 (U6): a "Cutting tools | Holders" switch heads the window; the
//! Holders page lists the built-in holders by taper and opens one in a
//! read-only holder view (`holders`, `holder_view`). The editor stands as
//! in the #3663 mock-up: the tabs and their rows on the left, the preview
//! on the right. The window opens wide enough for the Recipes table and
//! the preview side by side, as far as the screen permits ([`width`]).
//! On a narrow window the rail folds and the preview keeps a least width
//! ([`rail_w`], [`columns`]), so the side drawing always has room.
//! #3898 (U7): the tools list has a "My library" source (`components`)
//! with the component library and the document's component table; a
//! document component opens in the component editor (`component_edit`),
//! which stands in for the page; the Holders page lists the user holders
//! of the document; the Tool tab's assembly group has the tool's
//! component stack (`stack`). Each document change is one undoable
//! command.
//! #4334 (U8): one frame around the pages (`frame`): a header with the
//! way back, the name and the Cutting tools | Holders tabs, and one footer
//! with a hint and the page's buttons (Close; Duplicate, Cancel, Apply).
//! Provenance: our own window, after the tool shape window pattern
//! (`pages::tool_shape_editor_window`) and the egui `Window` docs; no
//! HSMWorks material involved.

pub(crate) mod assembly_group;
pub(crate) mod categories;
pub(crate) mod component_edit;
pub(crate) mod components;
mod details_tab;
pub(crate) mod frame;
/// #3908: display only - private to this window, so no law can read it.
mod flutes;
pub(crate) mod holder_view;
pub(crate) mod holders;
pub(crate) mod list;
pub(crate) mod model;
pub(crate) mod preview;
mod rail;
mod recipes_fill;
pub(crate) mod recipes_tab;
pub(crate) mod side_view;
pub(crate) mod stack;
pub(crate) mod thumbs;
mod tool_tab;

use egui::{Id, Ui, Window};

use crate::actions;
use crate::state::{LibraryPage, State, ToolEditorTab, UiTarget};

/// The window's title, and its AccessKit name.
pub(crate) const TITLE: &str = "Tool library";
/// #3897: the labels of the page switch at the head of the window.
pub(crate) const TOOLS_PAGE: &str = "Cutting tools";
pub(crate) const HOLDERS_PAGE: &str = "Holders";

/// The rail's width when a tool is open.
const RAIL_W: f32 = 170.0;
/// #3897: the editor's two columns - the tabs' rows (wide enough for the
/// Recipes table without a horizontal scroll bar) and the preview.
pub(crate) const PARAMS_W: f32 = 780.0;
pub(crate) const PREVIEW_W: f32 = 320.0;
/// The window's width that shows the rail, the rows and the preview side
/// by side (with the separators and the window's margins).
pub(crate) const WIDE_W: f32 = RAIL_W + PARAMS_W + PREVIEW_W + 60.0;
/// #3943 review: on a narrow window the rail folds to this width, and the
/// preview keeps [`PREVIEW_MIN`] while the rows keep [`ROWS_MIN`], so the
/// side drawing always has room (`side_view::MIN_DRAW`).
const RAIL_NARROW: f32 = 120.0;
pub(crate) const PREVIEW_MIN: f32 = 250.0;
const ROWS_MIN: f32 = 280.0;
/// The gap between the rows and the preview (the separator and spacing).
const COL_GAP: f32 = 24.0;

/// The rail's width on a tools page `avail` wide.
#[must_use]
pub(crate) fn rail_w(avail: f32) -> f32 {
    if avail < RAIL_W + ROWS_MIN + PREVIEW_MIN + COL_GAP + 16.0 { RAIL_NARROW } else { RAIL_W }
}

/// The rows' and the preview's widths in an editor `avail` wide: the
/// preview takes 40 % up to [`PREVIEW_W`], and never less than
/// [`PREVIEW_MIN`] while the rows keep [`ROWS_MIN`].
#[must_use]
pub(crate) fn columns(avail: f32) -> (f32, f32) {
    let floor = PREVIEW_MIN.min(avail - ROWS_MIN - COL_GAP).max(160.0);
    let preview = PREVIEW_W.min(avail * 0.4).max(floor);
    ((avail - preview - COL_GAP).max(200.0), preview)
}
/// The gap between the window and the screen's edge.
const MARGIN: f32 = 8.0;
/// The share of the screen's width left of the Properties pane (the dock's
/// default split: `layout::BROWSER_FRAC` and `VIEWPORT_FRAC`).
const BESIDE_PROPS: f32 = 0.77;

/// #3897: the window's width and left edge on a `screen`: [`WIDE_W`] when
/// it fits left of the Properties pane, else as wide as that space
/// permits. The window then covers the browser, never Properties.
#[must_use]
pub(crate) fn width(screen: egui::Rect) -> (f32, f32) {
    let right = screen.left() + screen.width() * BESIDE_PROPS - MARGIN;
    let room = (right - screen.left() - MARGIN).max(320.0);
    let w = WIDE_W.min(room);
    (w, (right - w).max(screen.left() + MARGIN))
}

/// Draw the window when it is open (`state.tool_library` is `Some`).
/// #4334: the window has no egui title bar; the frame's header holds the
/// title and the close button (the AccessKit name stays [`TITLE`]), and
/// the window moves by a drag on its background. Closing is view state,
/// nothing is queued.
pub(crate) fn tool_library_window(ctx: &egui::Context, state: &mut State) {
    if state.tool_library.is_none() {
        return;
    }
    let screen = ctx.viewport_rect();
    let (w, left) = width(screen);
    // Flat, docked look (#4245): the theme's window fill and hairline, a
    // small radius, no shadow; the panels give their own margins.
    let frame = egui::Frame::window(&ctx.global_style()).shadow(egui::Shadow::NONE).inner_margin(0.0);
    Window::new(TITLE)
        .id(Id::new("tool-library"))
        .title_bar(false)
        .frame(frame)
        .collapsible(false)
        .resizable(true)
        // Left of the Properties pane, so the two can be used at the same
        // time; as wide as the rows and the preview need (#3897).
        .default_pos([left, screen.top() + 0.15 * screen.height()])
        .default_size([w, (screen.height() * 0.7).max(320.0)])
        .min_width(w)
        .show(ctx, |ui| body(ui, state));
}

/// #4334: the header, the footer, then the page between them.
fn body(ui: &mut Ui, state: &mut State) {
    // The tool editor's draft follows the document before the header and
    // the footer read it (a cheap swap; the rows read it again later).
    let gone = state.with_library_draft(|state, _| !state.tool_ui.exists).unwrap_or(false);
    if gone {
        state.close_tool_in_library();
    }
    let Some(face) = frame::face(state) else {
        return;
    };
    let component_changed = component_edit::changed(state);
    let band = egui::Frame::NONE.inner_margin(egui::Margin::symmetric(10, 6));
    let head = egui::Panel::top("tool_library_head")
        .frame(band)
        .show(ui, |ui| frame::header(ui, state, &face))
        .inner;
    let foot = egui::Panel::bottom("tool_library_foot")
        .frame(band)
        .show(ui, |ui| frame::footer(ui, state, &face, component_changed))
        .inner;
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.inner_margin(egui::Margin::symmetric(10, 6)))
        .show(ui, |ui| page(ui, state));
    if let Some(ask) = head.or(foot) {
        act(state, &face, ask);
    }
}

/// The page the window shows: the component editor stands in for the
/// page while it is open (#3898).
fn page(ui: &mut Ui, state: &mut State) {
    let Some(page) = state.tool_library.as_ref().map(|v| v.page) else {
        return;
    };
    if state.tool_library.as_ref().is_some_and(|v| v.component_edit.is_some()) {
        component_edit::draw(ui, state);
        return;
    }
    match page {
        LibraryPage::Tools => tools_page(ui, state),
        LibraryPage::Holders => {
            if state.tool_library.as_ref().is_some_and(|v| v.holders.open.is_some()) {
                holder_view::draw(ui, state);
            } else {
                holders::draw(ui, state);
            }
        }
    }
}

/// Do what the frame was asked. Back and Cancel drop an unapplied draft,
/// as the old way back did; Apply queues the same commands as before.
fn act(state: &mut State, face: &frame::Face, ask: frame::Ask) {
    use frame::{Ask, Face};
    match (ask, face) {
        (Ask::Page(p), _) => {
            state.show_library_page(p);
            if let Some(v) = state.tool_library.as_mut() {
                v.component_edit = None;
            }
        }
        (Ask::Close, _) => state.close_tool_library(),
        (Ask::Back | Ask::Cancel, Face::Component(_)) => component_edit::close(state),
        (Ask::Back | Ask::Cancel, Face::Tool { .. }) => state.close_tool_in_library(),
        (Ask::Back | Ask::Cancel, Face::Holder(_)) => state.open_holder_in_library(None),
        (Ask::Back | Ask::Cancel, Face::List(..)) => {}
        (Ask::Apply, Face::Component(_)) => component_edit::apply(state),
        (Ask::Apply, Face::Tool { .. }) => apply_tool(state),
        (Ask::Apply, _) => {}
    }
}

/// One Apply queues the draft (`ChangeTool`, or `ReshapeTool` when a drawn
/// cutter rides it) and, when the number changed, `SetToolNumber` - the
/// same commands as Properties' Apply.
fn apply_tool(state: &mut State) {
    let _ = state.with_library_draft(|state, _| {
        let name = state.tool_ui.name.clone();
        let apply = state.apply_tool_numbers(&name);
        state.pending.push_back(apply);
        super::tool_settings::push_number_if_changed(state, &name);
    });
}

/// The full list, or the rail beside the editor when a tool is open.
fn tools_page(ui: &mut Ui, state: &mut State) {
    if state.tool_library.as_ref().and_then(|v| v.open_tool()).is_none() {
        list::draw(ui, state);
        return;
    }
    let rail = rail_w(ui.available_width());
    let pick = ui
        .horizontal_top(|ui| {
            let pick = ui
                .vertical(|ui| {
                    ui.set_width(rail);
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

/// (#3897) Two columns: the Tool, Recipes and Details tabs with their rows
/// on the left, the preview on the right. The draft is swapped into
/// `tool_ui` only while it is drawn (`with_library_draft`).
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
            // #4334: the name heads the window (the frame's header), and
            // Apply stands in the shared footer, under this panel.
            let footer_h = 8.0;
            // Bounded by the screen, not only by the room left: a column
            // as tall as the room would let the window grow each frame.
            let screen_h = ui.ctx().viewport_rect().height();
            let col_h = (ui.available_height() - footer_h).min(screen_h * 0.7 - 150.0).max(160.0);
            let (rows_w, preview_w) = columns(ui.available_width());
            let mut shape = false;
            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(rows_w);
                    ui.set_max_height(col_h);
                    shape = tabs(ui, state, &mut face.tab, col_h);
                });
                // A plain separator would take the whole height left in
                // the window and push Apply out of it: this one is col_h.
                ui.add_sized([8.0, col_h], egui::Separator::default().vertical());
                ui.vertical(|ui| {
                    ui.set_width(preview_w);
                    preview::draw(ui, state, face, egui::Vec2::new(preview_w, col_h));
                });
            });
            if shape { After::Shape(name) } else { After::None }
        })
        .unwrap_or(After::None);
    match after {
        After::None => {}
        After::Gone => state.close_tool_in_library(),
        After::Shape(name) => open_shape(state, &name),
    }
}

/// The tab row and the chosen tab's rows in a scroll area of `col_h`.
/// Gives whether "Edit tool shape" was clicked.
fn tabs(ui: &mut Ui, state: &mut State, tab: &mut ToolEditorTab, col_h: f32) -> bool {
    let mut shape = false;
    ui.horizontal(|ui| {
        ui.selectable_value(tab, ToolEditorTab::Tool, "Tool").on_hover_text("type, geometry and the rows the tool's kind reads");
        ui.selectable_value(tab, ToolEditorTab::Recipes, recipes_tab::TAB)
            .on_hover_text("speeds and feeds for each workpiece material");
        ui.selectable_value(tab, ToolEditorTab::Details, "Details")
            .on_hover_text("material group, catalog facts and the tool number");
    });
    ui.separator();
    egui::ScrollArea::vertical()
        .id_salt("tool_library_editor")
        .auto_shrink([false, false])
        .max_height((col_h - 44.0).max(80.0))
        .show(ui, |ui| match *tab {
            ToolEditorTab::Tool => shape = tool_tab::draw(ui, state),
            ToolEditorTab::Recipes => recipes_tab::draw(ui, state),
            ToolEditorTab::Details => details_tab::draw(ui, state),
        });
    shape
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
