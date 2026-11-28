//! The property-page form law (user review 2026-11-06: "crammed, hard to
//! understand, hard to read"): every page lays its fields out through
//! these helpers, so the whole operation dialog reads one way -
//!
//! - a property grid: label LEFT in a responsive column, control RIGHT
//!   on the same row, right-aligned and compact ([`prop_row`],
//!   [`check_row`], [`prop_grid`]);
//! - units as a suffix on the value, never in the label
//!   ([`value_with_unit`]);
//! - help as a hover tooltip on the label ([`hint`] attaches it to the
//!   row just drawn), never an inline paragraph;
//! - a field whose 0 means "derived" as an explicit Auto toggle
//!   ([`auto_value`]) - Auto on writes the model's 0, so the params and
//!   the commands stay exactly what they were;
//! - concept groups, collapsible, an optional feature's group enabled by
//!   a checkbox in its header ([`group`]);
//! - a one-line coloured status banner that expands on click
//!   (`status_banner`) and a bottom action bar (`action_bar`), in
//!   the sibling module `status`.
//!
//! Every helper is immediate-mode layout only: it edits the `&mut` it is
//! handed (the page's draft), never the world - the pages' commands stay
//! the one way a change lands (AGENTS.md, "Everything Is a Command").
//!
//! Provenance: our own layout code over egui's Grid / CollapsingState /
//! layouts (the vendored egui 0.36 docs); the label-left property grid
//! is the generic pattern of desktop property inspectors. No vendor
//! material, strings or layouts copied.

use egui::{Align, DragValue, Id, Label, Layout, Response, RichText, TextEdit, Ui, Vec2};

use super::row_name::{name_by_row, named_by};

/// The label column's minimum width when the pane has room.
pub const LABEL_W: f32 = 120.0;
/// A compact control's width (combos, numbers).
pub const CTRL_W: f32 = 96.0;
/// Space between two groups.
pub const GROUP_GAP: f32 = 6.0;

/// Where the last row's label response waits for a [`hint`] this frame.
fn last_label_id() -> Id {
    Id::new("camcraft.form.last_label")
}

fn remember(ui: &Ui, r: &Response) {
    let frame = ui.ctx().cumulative_frame_nr();
    ui.ctx().data_mut(|d| d.insert_temp(last_label_id(), (frame, r.clone())));
}

fn label_cell(ui: &mut Ui, label: &str) -> Response {
    let h = ui.spacing().interact_size.y;
    let available = (ui.available_width() - ui.spacing().item_spacing.x).max(0.0);
    // Share spare width with labels while keeping a compact control reachable.
    let width = (available * 0.5).max(LABEL_W).min((available - CTRL_W).max(0.0));
    ui.allocate_ui_with_layout(Vec2::new(width, h), Layout::left_to_right(Align::Center), |ui| {
        ui.set_min_width(width);
        ui.add(Label::new(label).truncate())
    })
    .inner
}

/// The id of the label of the property row drawn last this frame, for a
/// control to be named by it (`Response::labelled_by`, #1119). `None` when
/// no row was drawn this frame.
pub fn row_label_id(ui: &Ui) -> Option<Id> {
    let frame = ui.ctx().cumulative_frame_nr();
    let last: Option<(u64, Response)> = ui.ctx().data(|d| d.get_temp(last_label_id()));
    last.and_then(|(f, r)| (f == frame).then_some(r.id))
}

/// One property row: `label` in the label column, `add`'s controls at
/// the right, on one line. The label's full text rides its hover (a
/// truncated label never hides its name). The seam's controls drawn by
/// `add` take the label as their accessible name ([`name_by_row`]).
pub fn prop_row<R>(ui: &mut Ui, label: &str, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.horizontal(|ui| {
        let lab = label_cell(ui, label);
        remember(ui, &lab);
        named_by(ui, lab.id, |ui| ui.with_layout(Layout::right_to_left(Align::Center), add).inner)
    })
    .inner
}

/// [`prop_row`] with its help as the label's hover tooltip.
pub fn prop_row_hint<R>(ui: &mut Ui, label: &str, help: &str, add: impl FnOnce(&mut Ui) -> R) -> R {
    let out = prop_row(ui, label, add);
    hint(ui, help);
    out
}

/// A property row whose control is a single-line text field, named by the
/// row's label (#1119). `width` sizes the field; the label keeps its cell.
pub fn text_row(ui: &mut Ui, label: &str, text: &mut String, id_salt: &str, width: f32) -> Response {
    prop_row(ui, label, |ui| {
        let mut r = ui.add(TextEdit::singleline(text).id_salt(id_salt).desired_width(width));
        if let Some(l) = row_label_id(ui) {
            r = r.labelled_by(l);
        }
        r
    })
}

/// A checkbox as a property row: the caption is the row's label, the
/// box sits in the control column. Returns the box's response (callers
/// read `.changed()`).
pub fn check_row(ui: &mut Ui, label: &str, v: &mut bool) -> Response {
    ui.horizontal(|ui| {
        // The caption toggles too, as a checkbox's own text would.
        let lab = label_cell(ui, label);
        remember(ui, &lab);
        let hit = ui.interact(lab.rect, lab.id.with("toggle"), egui::Sense::click());
        let mut r = ui.with_layout(Layout::right_to_left(Align::Center), |ui| ui.checkbox(v, "")).inner.labelled_by(lab.id);
        // The blank checkbox's empty name must not hide its associated caption.
        r.ctx.accesskit_node_builder(r.id, |node| node.clear_label());
        if hit.clicked() {
            *v = !*v;
            r.mark_changed();
        }
        r
    })
    .inner
}

/// Attach `help` as the hover tooltip of the row drawn last this frame;
/// with no row drawn yet this frame, it shows as a small dimmed line.
pub fn hint(ui: &mut Ui, help: &str) {
    let frame = ui.ctx().cumulative_frame_nr();
    let last: Option<(u64, Response)> = ui.ctx().data(|d| d.get_temp(last_label_id()));
    match last {
        Some((f, r)) if f == frame => {
            r.on_hover_text(help);
        }
        _ => {
            ui.label(RichText::new(help).small().weak());
        }
    }
}

/// A dimmed caption under a row - the ONE consequence line a safety rule
/// may keep visible (e.g. what stands in when an entry cannot fit).
pub fn consequence(ui: &mut Ui, text: &str) {
    ui.horizontal(|ui| {
        ui.add_space(LABEL_W + ui.spacing().item_spacing.x);
        ui.add(Label::new(RichText::new(text).small().weak()).wrap());
    });
}

/// A two-column property grid (label | control) for pages that draw
/// their rows inside one `egui::Grid`: `add` calls `ui.end_row()` after
/// each row. The label column keeps [`LABEL_W`].
pub fn prop_grid<R>(ui: &mut Ui, id: &str, add: impl FnOnce(&mut Ui) -> R) -> R {
    egui::Grid::new(id)
        .num_columns(2)
        .min_col_width(LABEL_W)
        .spacing(Vec2::new(ui.spacing().item_spacing.x, 4.0))
        .striped(false)
        .show(ui, add)
        .inner
}

/// What a [`group`] call drew: whether its header checkbox flipped this
/// frame, and the body's answer when the body was open (and enabled).
pub struct GroupOut<R> {
    pub toggled: bool,
    pub inner: Option<R>,
}

/// A titled, collapsible concept group (open by default). With
/// `enabled`, the header carries the checkbox that switches the whole
/// optional feature on or off; an off group draws no body (nothing it
/// would hold applies).
pub fn group<R>(
    ui: &mut Ui,
    title: &str,
    enabled: Option<&mut bool>,
    add: impl FnOnce(&mut Ui) -> R,
) -> GroupOut<R> {
    group_open(ui, title, enabled, true, add)
}

/// [`group`] with its first-frame open state (`default_open`).
pub fn group_open<R>(
    ui: &mut Ui,
    title: &str,
    enabled: Option<&mut bool>,
    default_open: bool,
    add: impl FnOnce(&mut Ui) -> R,
) -> GroupOut<R> {
    let id = ui.make_persistent_id(("camcraft.group", title));
    let state = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, default_open);
    let mut toggled = false;
    let mut on = true;
    let mut title_clicked = false;
    let mut header = state.show_header(ui, |ui| {
        if let Some(e) = enabled {
            toggled = ui.checkbox(e, "").on_hover_text(format!("use {title}")).changed();
            on = *e;
        }
        // The title folds the group as the arrow does: a header that names
        // a group reads as its handle.
        title_clicked = ui
            .add(Label::new(RichText::new(title).strong()).sense(egui::Sense::click()))
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked();
    });
    if title_clicked {
        header.toggle();
    }
    let body = header.body(|ui| {
        if on {
            Some(egui::Frame::NONE.inner_margin(egui::Margin { left: 2, right: 0, top: 0, bottom: 2 }).show(ui, |ui| super::field_ring::in_group(ui, id, add)).inner)
        } else {
            None
        }
    });
    ui.add_space(GROUP_GAP);
    GroupOut { toggled, inner: body.2.and_then(|b| b.inner) }
}

/// A number with its unit as a suffix inside the field; returns the
/// response (`changed()` on an edit).
pub fn value_with_unit(ui: &mut Ui, v: &mut f64, unit: &str, speed: f64) -> Response {
    let mut dv = DragValue::new(v).speed(speed);
    if !unit.is_empty() {
        dv = dv.suffix(format!(" {unit}"));
    }
    let r = ui.add_sized(Vec2::new(CTRL_W, ui.spacing().interact_size.y), dv);
    name_by_row(ui, r)
}

/// The toggle for a field whose 0 means "derived" (or "no limit"): the
/// box captioned `toggle` ("Auto", "No limit") on writes 0 - the model's
/// own sentinel - and shows `derived` dimmed in place of the number; off
/// seeds `seed` for the user to edit, drawn by `edit`. Returns whether
/// the value moved this frame.
pub fn auto_value(
    ui: &mut Ui,
    label: &str,
    toggle: &str,
    v: &mut f64,
    derived: &str,
    seed: f64,
    edit: impl FnOnce(&mut Ui, &mut f64),
) -> bool {
    let before = *v;
    prop_row(ui, label, |ui| {
        // The model's 0 is the Auto sentinel, and the keys "0", "0." and
        // "0.5" all read as 0 on the way to a value (#2178). While the
        // user types in the edit field, a 0 therefore stays manual: the
        // hold lasts until the field loses focus.
        let hold = ui.make_persistent_id(("camcraft.auto_value_hold", label));
        let held: bool = ui.data(|d| d.get_temp(hold)).unwrap_or(false);
        let mut auto = *v == 0.0 && !held;
        let toggle_resp = ui.checkbox(&mut auto, toggle);
        if toggle_resp.changed() {
            *v = if auto { 0.0 } else { seed };
            ui.data_mut(|d| d.remove_temp::<bool>(hold));
        }
        if auto {
            ui.add_enabled(false, Label::new(RichText::new(derived).weak()));
        } else {
            edit(ui, v);
            let editing = ui.memory(|m| m.focused()).is_some_and(|f| f != toggle_resp.id);
            if editing {
                ui.data_mut(|d| d.insert_temp(hold, true));
            } else {
                ui.data_mut(|d| d.remove_temp::<bool>(hold));
            }
        }
    });
    *v != before
}

/// [`auto_value`] for a field the model keeps as an `Option`: `None` is
/// the derived figure. Auto on writes `None`; off seeds `Some(seed)` for
/// `edit` to draw. Returns whether the value moved this frame.
pub fn auto_opt(
    ui: &mut Ui,
    label: &str,
    toggle: &str,
    v: &mut Option<f64>,
    derived: &str,
    seed: f64,
    edit: impl FnOnce(&mut Ui, &mut f64),
) -> bool {
    let before = *v;
    prop_row(ui, label, |ui| {
        let mut auto = v.is_none();
        if ui.checkbox(&mut auto, toggle).changed() {
            *v = if auto { None } else { Some(seed) };
        }
        match v {
            Some(x) => edit(ui, x),
            None => {
                ui.add_enabled(false, Label::new(RichText::new(derived).weak()));
            }
        }
    });
    *v != before
}
