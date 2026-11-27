//! #867: one shared widget for every error, refusal and diagnostic text
//! the UI shows, so the user can select it, right-click "Copy message",
//! and - for a long or multi-line message - press a small "Copy" button,
//! then paste it into an issue report. Clipboard copy is ephemeral UI
//! (no document state changes), so it needs no command.
//! Provenance: our own UI law (AGENTS.md: refusals are visible); egui's
//! public `Label::selectable` and `Context::copy_text` APIs.

use egui::{Color32, Label, Response, RichText, Ui};

/// A message longer than this (or with a line break) gets the visible
/// "Copy" button; shorter ones still copy through the context menu.
const COPY_BUTTON_MIN_CHARS: usize = 80;

/// The label of the visible copy control (the structural tests find it).
pub(crate) const COPY_LABEL: &str = "Copy";
/// The context-menu entry, the same wording the status strip uses.
pub(crate) const COPY_MENU_LABEL: &str = "Copy message";

/// Whether `msg` earns the visible Copy button.
pub(crate) fn wants_button(msg: &str) -> bool {
    msg.contains('\n') || msg.chars().count() > COPY_BUTTON_MIN_CHARS
}

/// Show `msg` in `ink` as selectable, wrapped text with copy affordances.
/// Returns the text's response.
pub(crate) fn error_text(ui: &mut Ui, ink: Color32, msg: &str) -> Response {
    error_rich(ui, RichText::new(msg).color(ink), msg, false)
}

/// As [`error_text`] for pre-styled text (e.g. the underlined refusal
/// link); `sense_click` keeps the label clickable for such a link.
pub(crate) fn error_rich(ui: &mut Ui, text: RichText, plain: &str, sense_click: bool) -> Response {
    let mut label = Label::new(text).wrap().selectable(true);
    if sense_click {
        label = label.sense(egui::Sense::click());
    }
    let resp = ui.add(label);
    resp.context_menu(|ui| {
        if ui.button(COPY_MENU_LABEL).clicked() {
            ui.ctx().copy_text(plain.to_owned());
            ui.close();
        }
    });
    if wants_button(plain) && ui.small_button(COPY_LABEL).clicked() {
        ui.ctx().copy_text(plain.to_owned());
    }
    resp
}

#[cfg(test)]
#[path = "error_text_tests.rs"]
mod tests;
