//! The "Add built-in vise…" dialog body (#788), drawn inside the fixture
//! library's modal: pick a size from the table, edit any figure to match
//! the shop's own vise, then either fit it to the setup's stock (fixed
//! jaw on the stock's +Y side, centred, floor under it, jaws opened to
//! the stock) or type where its frame goes. Place queues one undoable
//! `InsertVise` through the command queue.
//! Provenance: our own layout over our own vise commands.

use egui::{RichText, Ui};

use camcraft_app::units::Unit;
use camcraft_app::vise;

use crate::state::{CommandKind, ViseDraft};

use super::fields::{num, num_plain};

/// What the body's buttons asked for.
pub(crate) enum ViseAct {
    Close,
    Place(Box<CommandKind>),
}

/// The Place button's command for this draft.
#[must_use]
pub(crate) fn place_command(d: &ViseDraft) -> CommandKind {
    CommandKind::InsertBuiltinVise {
        setup: (!d.setup.is_empty()).then(|| d.setup.clone()),
        params: d.params.clone(),
        fit_stock: d.fit_stock,
        pos: d.pos,
        yaw_deg: d.yaw_deg,
    }
}

pub(crate) fn body(ui: &mut Ui, u: Unit, d: &mut ViseDraft) -> Option<ViseAct> {
    let mut act = None;
    ui.heading("Add built-in vise");
    ui.label(RichText::new("A generic vise built from these figures; edit them to match yours.").small());
    ui.add_space(4.0);
    ui.label("Size (jaw width)");
    let mut picked = None;
    ui.horizontal_wrapped(|ui| {
        for (k, row) in vise::SIZES.iter().enumerate() {
            if ui.selectable_label(d.size == k, row.label).clicked() {
                picked = Some(k);
            }
        }
    });
    if let Some(k) = picked {
        d.pick(k);
    }
    let p = &mut d.params;
    egui::Grid::new("builtin-vise-dims").num_columns(2).show(ui, |ui| {
        for (label, v) in [
            ("Jaw width", &mut p.jaw_width),
            ("Jaw height", &mut p.jaw_height),
            ("Jaw thickness", &mut p.jaw_thickness),
            ("Max opening", &mut p.max_opening),
            ("Overall length", &mut p.length),
            ("Overall width", &mut p.width),
            ("Base height", &mut p.base_height),
            ("Housing length", &mut p.housing_length),
            ("Housing height", &mut p.housing_height),
        ] {
            ui.label(label);
            num(ui, u, v);
            ui.end_row();
        }
    });
    ui.checkbox(&mut d.fit_stock, "Fit to the stock (jaws open to it)");
    if !d.fit_stock {
        egui::Grid::new("builtin-vise-place").num_columns(2).show(ui, |ui| {
            ui.label("Jaw opening");
            num(ui, u, &mut d.params.opening);
            ui.end_row();
            for (k, axis) in ["Fixed jaw X", "Fixed jaw Y", "Floor Z"].iter().enumerate() {
                ui.label(*axis);
                num(ui, u, &mut d.pos[k]);
                ui.end_row();
            }
            ui.label("Turn about Z (yaw)");
            num_plain(ui, &mut d.yaw_deg);
            ui.end_row();
        });
    }
    if let Some(err) = &d.error {
        ui.colored_label(ui.visuals().error_fg_color, err);
    }
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if ui.button("Place").clicked() {
            act = Some(ViseAct::Place(Box::new(place_command(d))));
        }
        if ui.button("Cancel").clicked() {
            act = Some(ViseAct::Close);
        }
    });
    act
}
