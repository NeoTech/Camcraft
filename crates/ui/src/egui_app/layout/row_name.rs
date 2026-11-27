//! #1119 slice 3: a property row names the controls drawn in it. While
//! `prop_row` (or a page's own label, through [`named_by`]) draws a row's
//! controls, the row is open, and the seam's fields take its label as
//! their accessible name ([`name_by_row`]). Outside an open row they take
//! no name from the seam.
//!
//! Provenance: our own helpers over egui's `Response::labelled_by` and
//! temp memory (vendored egui 0.36 docs); no vendor material.

use egui::{Id, Response, Ui};

/// Where `prop_row` keeps the label id of the row whose controls it
/// draws now (#1119).
fn open_row_key() -> Id {
    Id::new("camcraft.form.open_row")
}

fn set_open_row(ui: &Ui, row: Option<Id>) {
    ui.ctx().data_mut(|d| d.insert_temp(open_row_key(), row));
}

/// The label id of the property row whose controls are drawn now: `Some`
/// only inside a `prop_row`'s control closure (#1119). Unlike
/// `row_label_id`, a control drawn after a row, outside it, gets `None`.
pub fn open_row_label(ui: &Ui) -> Option<Id> {
    ui.ctx().data(|d| d.get_temp::<Option<Id>>(open_row_key())).flatten()
}

/// Draw `add` as the controls of the row whose label is `label`: the
/// seam's controls in it take that label as their accessible name
/// ([`name_by_row`]). `prop_row` opens its rows this way; a page that
/// lays its own label (a `Grid` row) calls it directly (#1119).
pub fn named_by<R>(ui: &mut Ui, label: Id, add: impl FnOnce(&mut Ui) -> R) -> R {
    let outer = open_row_label(ui);
    set_open_row(ui, Some(label));
    let out = add(ui);
    set_open_row(ui, outer);
    out
}

/// Name the control `r` by the open property row, if there is one: the
/// row's visible label is the control's accessible name (#1119). Nothing
/// visible changes.
pub fn name_by_row(ui: &Ui, r: Response) -> Response {
    match open_row_label(ui) {
        Some(l) => r.labelled_by(l),
        None => r,
    }
}
