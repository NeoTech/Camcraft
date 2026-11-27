//! The Stock page's "Face to size" group (#3196): face a block to height
//! from numbers - the facing tool, an optional box stock size (its top at
//! the top Z), the top Z and the final Z (program Z), the depth per pass,
//! the stepover, the cut direction and the feeds. The button queues
//! `CommandKind::FaceToSize`, which lands ONE undoable `FaceToSize` (the
//! stock and a Face on it). Nothing here writes the world.
//! Provenance: our own page law over plain facing practice (issue #3196);
//! no vendor material consulted.

use egui::{RichText, Ui};

use camcraft_app::commands::FaceToSizeSpec;
use camcraft_app::model;

use crate::state::{CommandKind, State};

use super::fields::{combo_owned, num_feed_named, num_named};
use super::layout::prop_grid;

/// The group title, pinned by the page tests.
pub(crate) const GROUP: &str = "Face to size";
/// The button, pinned by the page tests.
pub(crate) const APPLY: &str = "Add face to size";
/// The tick that sets the stock to the stated box.
pub(crate) const SET_STOCK: &str = "Set the stock to this box";

/// The document's tools, by name.
fn tool_names(state: &State) -> Vec<String> {
    model::named_entities(&state.world)
        .into_iter()
        .filter(|(_, e)| model::is_tool(&state.world, e))
        .map(|(n, _)| n)
        .collect()
}

/// The program Z of the active setup's WCS origin (0 when none is placed).
fn program_zero_z(state: &State) -> f64 {
    let setup = camcraft_app::setup::active(&state.world);
    camcraft_app::setup::wcs_entity(&state.world, setup.as_deref())
        .and_then(|e| model::transform_of(&state.world, &e))
        .map_or(0.0, |l| l.pos.z)
}

/// What a fresh group offers: the setup's stock as it stands (faced 1 mm
/// from its top), else a 100 x 100 x 25 block with its top at Z0.
fn first_draft(state: &State, tools: &[String]) -> FaceToSizeSpec {
    let setup = camcraft_app::setup::active(&state.world);
    let held = model::stock_world_aabb_for(&state.world, setup.as_deref());
    let top_z = held.map_or(0.0, |b| b.max.z - program_zero_z(state));
    let stock = match held {
        Some(b) => [b.max.x - b.min.x, b.max.y - b.min.y, b.max.z - b.min.z],
        None => [100.0, 100.0, 25.0],
    };
    let tool = tools.first().cloned().unwrap_or_default();
    // 70 % of the first tool's diameter: rows that overlap, no ribs.
    let stepover = model::find_by_name(&state.world, &tool)
        .and_then(|e| model::tool_of(&state.world, &e))
        .map_or(3.0, |t| (t.diameter * 0.7 * 100.0).round() / 100.0);
    FaceToSizeSpec {
        name: None,
        tool,
        stock: held.is_none().then_some(stock),
        top_z,
        final_z: top_z - 1.0,
        stepdown: 0.5,
        stepover,
        climb: true,
        feed: 1200.0,
        plunge_feed: 300.0,
    }
}

/// The draft's box when the stock is not set (kept for the tick).
#[derive(Clone)]
struct Draft {
    spec: FaceToSizeSpec,
    size: [f64; 3],
}

pub(super) fn group_body(ui: &mut Ui, state: &mut State) {
    let u = state.units();
    let tools = tool_names(state);
    let id = ui.id().with("stock_face_to_size_draft");
    let mut d: Draft = ui.data_mut(|m| {
        m.get_temp_mut_or_insert_with(id, || {
            let spec = first_draft(state, &tools);
            let size = spec.stock.unwrap_or([100.0, 100.0, 25.0]);
            Draft { spec, size }
        })
        .clone()
    });
    if tools.is_empty() {
        ui.label(RichText::new("Add a facing tool on the Tools page first.").weak());
        return;
    }
    let mut set_stock = d.spec.stock.is_some();
    prop_grid(ui, "stock_face_to_size_grid", |ui| {
        ui.label("Tool");
        let mut idx = tools.iter().position(|t| *t == d.spec.tool).unwrap_or(0);
        combo_owned(ui, "face_to_size_tool", &mut idx, &tools);
        d.spec.tool = tools.get(idx).cloned().unwrap_or_default();
        ui.end_row();
        ui.label("");
        ui.checkbox(&mut set_stock, SET_STOCK);
        ui.end_row();
        if set_stock {
            for (label, v) in ["Block length (X)", "Block width (Y)", "Block height (Z)"].into_iter().zip(d.size.iter_mut()) {
                ui.label(label);
                num_named(ui, u, v, label);
                ui.end_row();
            }
        }
        for (label, v) in [
            ("Top Z", &mut d.spec.top_z),
            ("Final Z", &mut d.spec.final_z),
            ("Depth per pass", &mut d.spec.stepdown),
            ("Stepover", &mut d.spec.stepover),
        ] {
            ui.label(label);
            num_named(ui, u, v, label);
            ui.end_row();
        }
        ui.label("");
        ui.checkbox(&mut d.spec.climb, "Climb cut");
        ui.end_row();
        ui.label("Feed");
        num_feed_named(ui, u, &mut d.spec.feed, "Feed");
        ui.end_row();
        ui.label("Plunge feed");
        num_feed_named(ui, u, &mut d.spec.plunge_feed, "Plunge feed");
        ui.end_row();
    });
    d.spec.stock = set_stock.then_some(d.size);
    let passes = ((d.spec.top_z - d.spec.final_z) / d.spec.stepdown).ceil();
    if passes.is_finite() && passes >= 1.0 {
        ui.label(RichText::new(format!("{passes:.0} pass(es) from the top Z down to the final Z; Z values are program Z")).small().weak());
    }
    let hover = "Adds a Face on the stock (and sets the stock when ticked), as one undo step. \
                 Generate, verify and post it as any other operation.";
    if ui.button(APPLY).on_hover_text(hover).clicked() {
        state.pending.push_back(CommandKind::FaceToSize(d.spec.clone()));
    }
    ui.data_mut(|m| m.insert_temp(id, d));
}
