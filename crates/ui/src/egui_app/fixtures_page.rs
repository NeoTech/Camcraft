//! The Fixtures page (a setup's workholding), split out of `pages.rs` and
//! laid out like the WCS page (#1440, after #739): two titled groups -
//! the workholding the collision checks see (each fixture's row, the
//! machine table) and the ways to add more - with the adds as equal
//! buttons filling the pane, two to a row. The page is layout only: the
//! rows keep `fixture_rows`, every edit still lands through its command
//! (`MoveFixture`, `SetFixtureFixed`, `UntagFixture`, `DeleteFixture`,
//! `SetVise`, ...), and the adds only queue the dialog arms and
//! `TagFixture`. A fixture is a plain model wearing the marker; the
//! machine table is derived prose from the machine file, never an entity.
//!
//! Provenance: our own layout over the `layout` form helpers and our own
//! commands; no vendor material.

use egui::{RichText, Ui, Vec2};

use camcraft_app::{fixture, model, units};

use crate::state::{CommandKind, FixtureUi, State};

use super::layout::group;

/// A button row's height (the WCS page's).
const BUTTON_H: f32 = 24.0;

/// The workholding section of the Setups and Fixtures pages (M3.2 stage
/// B): the solids the collision law sees in this context. `ctx` is the
/// setup, `None` for the document's implicit one.
pub(super) fn fixtures_section(ui: &mut Ui, state: &mut State, ctx: Option<&str>) {
    let fixtures = fixture::tagged(&state.world, ctx);
    let table = fixture::table_box(&state.world, ctx);
    // A draft whose row vanished (deleted or untagged, here or via
    // undo) retires with it - the pane never edits a ghost.
    if !state.fixture_ui.name.is_empty() && !fixtures.iter().any(|(n, _)| n == &state.fixture_ui.name) {
        state.fixture_ui = FixtureUi::default();
    }
    state.retire_vise_opening_drafts();
    state.retire_plate_place_drafts();
    state.retire_mod_vise_drafts();
    state.retire_accessory_size_drafts();
    group(ui, "Workholding", None, |ui| holding(ui, state, fixtures, table));
    group(ui, "Add workholding", None, |ui| adds(ui, state, ctx));
}

/// What stands in the setup now: the fixtures' rows and the machine table.
fn holding(
    ui: &mut Ui,
    state: &mut State,
    fixtures: Vec<(String, flecs_ecs::prelude::Entity)>,
    table: Option<fixture::FixtureBox>,
) {
    let u = state.units();
    ui.label(RichText::new(if fixtures.is_empty() && table.is_none() {
        "No workholding modeled: the collision checks see stock and part only."
    } else {
        "Workholding in this setup's collision checks:"
    })
    .small()
    .weak());
    for (name, entity) in fixtures {
        let loc = model::transform_of(&state.world, &entity).unwrap_or_default();
        let fixed = fixture::def_of(&state.world, &entity).is_some_and(|d| d.fixed);
        super::fixture_rows::row(ui, state, &name, &loc, fixed);
        ui.separator();
    }
    if let Some(t) = table {
        ui.label(format!(
            "Machine/Table  span {} x {}  top Z {}  (from the machine file)",
            units::len(t.max.x - t.min.x, u, 1),
            units::len(t.max.y - t.min.y, u, 1),
            units::len(t.max.z, u, 2),
        ));
    }
}

/// The adds as equal buttons, two to a row. The mark button joins only
/// with exactly one model selected, on its own row, so the rows above it
/// never move.
fn adds(ui: &mut Ui, state: &mut State, ctx: Option<&str>) {
    let setup = ctx.unwrap_or_default().to_owned();
    let size = Vec2::new(cell_w(ui, 2), BUTTON_H);
    match super::wcs_page::even_row(ui, &["Import fixture…", "Add built-in vise…"], size) {
        Some(0) => state.pending.push_back(CommandKind::ImportFixtureDialog),
        // #788: a generic parametric vise, placed in one undoable step.
        Some(_) => state.pending.push_back(CommandKind::BuiltinViseDialog(setup.clone())),
        None => {}
    }
    match super::wcs_page::even_row(ui, &["Add accessory…", "Add fixture plate…"], size) {
        // #789: parallels, blocks, strap clamps, soft jaws.
        Some(0) => state.pending.push_back(CommandKind::AccessoryDialog(setup.clone())),
        // #819: a parametric grid plate, fixtures placed on its holes.
        Some(_) => state.pending.push_back(CommandKind::FixturePlateDialog(setup)),
        None => {}
    }
    // The other way in: a model already imported - select its row in the
    // browser and mark it. Offered only for exactly one selected model,
    // so the button never guesses its target.
    let sel: Vec<String> = model::selected_names(&state.world)
        .into_iter()
        .filter(|n| model::find_by_name(&state.world, n).is_some_and(|e| model::is_model(&state.world, &e)))
        .collect();
    if let [name] = sel.as_slice() {
        let label = format!("Mark {name} as fixture");
        let size = Vec2::new(cell_w(ui, 1), BUTTON_H);
        if super::wcs_page::even_row(ui, &[label.as_str()], size).is_some() {
            state.pending.push_back(CommandKind::TagFixture { name: name.clone(), setup: ctx.map(str::to_owned) });
        }
    }
}

/// The width each of `n` equal buttons takes to fill the pane.
fn cell_w(ui: &Ui, n: usize) -> f32 {
    let gaps = ui.spacing().item_spacing.x * n.saturating_sub(1) as f32;
    ((ui.available_width() - gaps) / n.max(1) as f32).max(24.0)
}
