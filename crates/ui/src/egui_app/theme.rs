//! The theme seam of the shell: the token table, its clamps and its
//! visuals live in the `camcraft-ui-kit` crate (#4394) and are re-exported
//! here, so every `egui_app::theme::...` path keeps working. This module
//! keeps what belongs to the running shell: the process-wide live table
//! (`current`, `set_current`) and `apply`, which installs it.
//! Provenance: our own module (the M6.1 A token law, split for the UI kit
//! crate of #4245); no HSMWorks material.

pub use camcraft_ui_kit::theme::*;

mod live;
pub use live::{current, set_current};

/// Install the live theme. Called per Context (windowed); the switcher
/// re-calls it on change (M6.1 C).
pub fn apply(ctx: &egui::Context) {
    apply_to(ctx, &current());
}

#[cfg(test)]
mod live_tests;
