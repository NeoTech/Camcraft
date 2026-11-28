//! #3893 (U2 of #3663): the Tool tab of the tool library window's editor -
//! the type, the geometry and the rows that the tool's kind reads. It
//! calls the same row pieces as Properties (`tool_settings_grid`), so a
//! new kind row lands in one place. #3895 (U4): the holder, the gauge
//! length and the stick-out follow the rows (`assembly_group`), and
//! #3930 its folded Body section (the stand-in holder, the body). The
//! cutting rows are on the Recipes tab (#3896, U5: `recipes_tab`).
//! #4336 (U10): the tab reads as the #3663 mock-up - Name, Type and Tool
//! number; a CUTTER group (diameter, flute length, corner radius, flutes,
//! helix angle, tool material, centre cutting, then the kind's own rows);
//! a SHANK group; then the assembly group. Each value is a draft field
//! with its unit as a suffix (the form seam); the footer's Apply lands
//! them through `ChangeTool` (and `SetToolNumber`), one undo step.
//! Provenance: our own tab over the tool draft; no HSMWorks material
//! involved.

use egui::{RichText, Ui};

use super::super::layout::prop_row_hint;
use super::super::tool_settings_grid as grid;
use crate::state::State;

/// The group headings (small capitals over a hairline, #4245).
pub(crate) const CUTTER: &str = "CUTTER";
pub(crate) const SHANK: &str = "SHANK";
/// The name row's label.
pub(crate) const NAME: &str = "Name";
/// Why the name is not typed here (no rename command yet).
pub(crate) const NAME_HINT: &str =
    "the tool's name; operations and scripts address the tool by it, so it is not changed here";

/// The rows, then "Edit tool shape". Returns whether that button was
/// clicked; the caller opens the shape window after the window's draft is
/// put back (`State::with_library_draft`).
pub(super) fn draw(ui: &mut Ui, state: &mut State) -> bool {
    let name = state.tool_ui.name.clone();
    prop_row_hint(ui, NAME, NAME_HINT, |ui| ui.label(name));
    grid::type_row(ui, state);
    grid::number_row(ui, state);
    section(ui, CUTTER);
    cutter(ui, state);
    section(ui, SHANK);
    grid::shank_rows(ui, state);
    super::assembly_group::draw(ui, state);
    ui.add_space(6.0);
    ui.button("Edit tool shape")
        .on_hover_text("open the drawn holder profile, the cutter and the body in their own window")
        .clicked()
}

/// The cutter's rows in the mock-up's order, then the rows that only the
/// tool's kind reads (point angle, pitch, hand, ...).
fn cutter(ui: &mut Ui, state: &mut State) {
    grid::diameter_row(ui, state);
    grid::flute_row(ui, state);
    grid::corner_row(ui, state);
    if super::recipes_fill::rotating(state.tool_ui.tool_kind()) {
        super::recipes_tab::flutes_row(ui, &mut state.tool_ui);
        super::super::tool_catalog_rows::helix_row(ui, &mut state.tool_ui);
        super::recipes_tab::tool_material_row(ui, &mut state.tool_ui);
    }
    grid::centre_cutting_row(ui, state);
    grid::kind_specific(ui, state);
}

/// A group heading: small capitals in the dimmed ink over a hairline.
fn section(ui: &mut Ui, title: &str) {
    ui.add_space(8.0);
    ui.label(RichText::new(title).small().weak());
    ui.separator();
}
