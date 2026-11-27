//! #3893 (U2 of #3663): the Details tab of the tool library window's
//! editor - the workpiece material group and the catalog facts of #898
//! (part number, vendor, coating, notes). It calls the same rows as
//! Properties' Catalog group (`tool_settings_grid::catalog_facts`, which
//! draws `tool_catalog_rows::catalog_rows`), so the two cannot drift.
//! #4336 (U10): the tool number and the helix angle moved to the Tool tab.
//! Provenance: our own tab over the tool draft; no HSMWorks material
//! involved.

use egui::Ui;

use crate::state::State;

pub(super) fn draw(ui: &mut Ui, state: &mut State) {
    super::super::tool_settings_grid::catalog_facts(ui, state, false);
}
