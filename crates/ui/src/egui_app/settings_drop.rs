//! Top-bar Settings button (#2428 Part B). Since #4493 it opens and closes
//! the Settings window (`settings_window.rs`), which holds the per-user
//! preferences; the one-row dropdown is gone.
//!
//! Provenance: our own top-band button following `home.rs`; no vendor
//! material consulted.

use egui::{Button, Rect, RichText, Ui};

use super::theme::Tokens;
use crate::state::State;

/// Accessible button label in the ribbon's top band.
pub(super) const SETTINGS: &str = "Settings";

/// Draw the `"Settings"` button right-aligned to `right` inside `band`.
/// Returns the button's left edge, so items to its left align.
pub(super) fn button(ui: &mut Ui, state: &mut State, pal: &Tokens, band: Rect, right: f32) -> f32 {
    let label = RichText::new(SETTINGS).size(pal.font_label).color(pal.text);
    let text_w = ui
        .painter()
        .layout_no_wrap(SETTINGS.into(), egui::FontId::proportional(pal.font_label), pal.text)
        .size()
        .x;
    let width = text_w + 2.0 * pal.button_padding_x + 8.0;
    let rect = Rect::from_min_max(
        egui::pos2(right - width, band.top() + 2.0),
        egui::pos2(right, band.bottom() - 2.0),
    );
    let resp = ui
        .put(rect, Button::new(label).selected(state.settings_open))
        .on_hover_text("Open the Settings window: appearance, graphics, graphics device and updates");
    if resp.clicked() {
        if state.settings_open {
            state.settings_open = false;
        } else {
            super::settings_window::open(state);
        }
    }
    rect.left()
}
