//! #4339: the "Convert to components" preview of the tool library window.
//! It stands in for the tool list while it is open. One row for each
//! standalone document tool: a tick, the tool, the proposed stack (holder,
//! collet, the tool as its cutting item) and the result - "same" (the
//! stack derives the same tool record and gauge length), "different" with
//! each difference, or "no match" with the reason. The "same" rows start
//! ticked; a "different" row converts only when the user ticks it; a "no
//! match" row cannot be ticked. "Convert selected" queues the ticked rows
//! as one undoable `ConvertToComponents`. The rows are the window's own
//! view state (`ConvertView`), computed on open and on Refresh only.
//! Provenance: our own page over our own command; no HSMWorks material
//! involved.

use egui::{RichText, Ui};

use camcraft_app::tool_convert::Verdict;

use crate::state::{ComponentAction, State};

/// The list header's button that opens the preview, and the page heading.
pub(crate) const OPEN: &str = "Convert to components…";
pub(crate) const HEADING: &str = "Convert to components";
/// The page's buttons.
pub(crate) const APPLY: &str = "Convert selected";
pub(crate) const SAME: &str = "Select the same rows";
pub(crate) const EVERY: &str = "Select every row that converts";
pub(crate) const REFRESH: &str = "Refresh";
pub(crate) const CLOSE: &str = "Close preview";
/// What a converted tool leaves behind: the library holder link.
pub(crate) const LINK_NOTE: &str = "A converted tool no longer uses its library holder: it leaves the holder's \"used by\" list, and a later change of the built-in holder reaches it only when you sync the holder component from the library.";
/// The tick of row `tool` (its AccessKit name).
#[must_use]
pub(crate) fn tick_label(tool: &str) -> String {
    format!("Convert {tool}")
}
const HEADS: [&str; 4] = ["", "Tool", "Proposed stack", "Result"];

/// What the page asks for after it is drawn.
enum Ask {
    Convert(Vec<String>),
    Same,
    Every,
    Refresh,
    Close,
}

pub(super) fn draw(ui: &mut Ui, state: &mut State) {
    let done = state.history.done_len();
    let Some(view) = state.tool_library.as_mut().and_then(|v| v.convert.as_mut()) else {
        return;
    };
    ui.strong(HEADING);
    ui.weak("Each standalone tool of the document with the component stack that gives it. The ticked rows convert in one undo step; a tool record never changes without your tick.");
    ui.weak(LINK_NOTE);
    if let Some(e) = &view.error {
        ui.weak(format!("No preview: {e}"));
    }
    if view.at != done {
        ui.weak("The document changed after this preview: Refresh before you convert.");
    }
    let mut ask = None;
    if view.rows.is_empty() && view.error.is_none() {
        ui.weak("The document has no standalone tool: each tool has a component stack.");
    } else {
        egui::ScrollArea::vertical().max_height((ui.available_height() - 40.0).max(120.0)).show(ui, |ui| {
            egui::Grid::new("tool_library_convert").striped(true).show(ui, |ui| {
                for h in HEADS {
                    ui.label(RichText::new(h).strong());
                }
                ui.end_row();
                for (row, picked) in view.rows.iter().zip(view.picked.iter_mut()) {
                    let can = !matches!(row.verdict, Verdict::NoMatch(_));
                    let tick = ui.add_enabled(can, egui::Checkbox::without_text(picked));
                    let name = tick_label(&row.tool);
                    tick.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, can, *picked, &name));
                    ui.label(&row.tool);
                    ui.label(if row.names.is_empty() { "-".to_owned() } else { row.names.join(" > ") });
                    ui.label(result_text(&row.verdict));
                    ui.end_row();
                }
            });
        });
    }
    let selection = view.selection();
    ui.horizontal_wrapped(|ui| {
        let fresh = view.at == done;
        if ui
            .add_enabled(fresh && !selection.is_empty(), egui::Button::new(APPLY))
            .on_hover_text("give each ticked tool its component stack: one undo step")
            .clicked()
        {
            ask = Some(Ask::Convert(selection.clone()));
        }
        if ui.button(SAME).on_hover_text("tick only the rows whose stack gives the same tool").clicked() {
            ask = Some(Ask::Same);
        }
        if ui.button(EVERY).on_hover_text("tick the different rows too: read their differences first").clicked() {
            ask = Some(Ask::Every);
        }
        if ui.button(REFRESH).on_hover_text("compute the preview again from the document").clicked() {
            ask = Some(Ask::Refresh);
        }
        if ui.button(CLOSE).on_hover_text("back to the tool list; nothing changes").clicked() {
            ask = Some(Ask::Close);
        }
    });
    match ask {
        Some(Ask::Convert(tools)) => state.queue_component(ComponentAction::Convert { tools }),
        Some(Ask::Same) => {
            for (row, picked) in view.rows.iter().zip(view.picked.iter_mut()) {
                *picked = row.verdict == Verdict::Same;
            }
        }
        Some(Ask::Every) => {
            for (row, picked) in view.rows.iter().zip(view.picked.iter_mut()) {
                *picked = !matches!(row.verdict, Verdict::NoMatch(_));
            }
        }
        Some(Ask::Refresh) => state.open_convert(),
        Some(Ask::Close) => state.close_convert(),
        None => {}
    }
}

/// The result column: the verdict and its differences or reason.
#[must_use]
pub(crate) fn result_text(v: &Verdict) -> String {
    match v {
        Verdict::Same => "same".to_owned(),
        Verdict::Different(d) => format!("different: {}", d.join("; ")),
        Verdict::NoMatch(why) => format!("no match: {why}"),
    }
}
