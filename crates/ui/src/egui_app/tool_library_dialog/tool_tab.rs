//! #3893 (U2 of #3663): the Tool tab of the tool library window's editor -
//! the type, the geometry and the rows that the tool's kind reads. It
//! calls the same rows as Properties (`tool_settings_grid::geometry`, which
//! draws `kind_rows`), so a new kind row lands in one place. The preview
//! stands above the tabs (U3, `preview`); the holder and the recipes come
//! to the window later (U4, U5) and Properties keeps them until then.
//! Provenance: our own tab over the tool draft; no HSMWorks material
//! involved.

use egui::Ui;

use crate::state::State;

/// The rows, then "Edit tool shape". Returns whether that button was
/// clicked; the caller opens the shape window after the window's draft is
/// put back (`State::with_library_draft`).
pub(super) fn draw(ui: &mut Ui, state: &mut State) -> bool {
    super::super::tool_settings_grid::geometry(ui, state);
    ui.add_space(6.0);
    ui.button("Edit tool shape")
        .on_hover_text("open the body, profile and holder editor in its own window")
        .clicked()
}
