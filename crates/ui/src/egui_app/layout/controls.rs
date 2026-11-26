//! Basic form controls routed through the layout seam (#1122): buttons,
//! check boxes, read-only list rows and notes, radio choices and number
//! fields, so pages do not call raw `ui.button` / `ui.small_button` /
//! `ui.checkbox` / `ui.label` / `ui.add` / `ui.radio_value` directly and a
//! future look or kit can restyle them in one place.
//!
//! Provenance: our own thin wrappers over egui's `Ui` controls (vendored
//! egui 0.36 docs); no vendor material involved.

use egui::{Button, DragValue, Label, Response, RichText, Ui, Vec2, WidgetText};

/// A form action button (e.g. "Detect holes").
pub fn button(ui: &mut Ui, label: impl Into<WidgetText>) -> Response {
    ui.button(label)
}

/// A form action button that is greyed out while `enabled` is false
/// (e.g. "Add skim" at the skim limit).
pub fn button_enabled(ui: &mut Ui, enabled: bool, label: impl Into<WidgetText>) -> Response {
    ui.add_enabled(enabled, Button::new(label))
}

/// A compact inline button that is greyed out while `enabled` is false.
pub fn small_button_enabled(ui: &mut Ui, enabled: bool, label: impl Into<WidgetText>) -> Response {
    ui.add_enabled(enabled, Button::new(label).small())
}

/// A check box with its own caption, for a box that is not a property
/// row (a group header's switch); a row uses `check_row`.
pub fn checkbox(ui: &mut Ui, v: &mut bool, label: impl Into<WidgetText>) -> Response {
    ui.checkbox(v, label)
}

/// A compact inline button inside a list or row (e.g. "Remove 1").
pub fn small_button(ui: &mut Ui, label: impl Into<WidgetText>) -> Response {
    ui.small_button(label)
}

/// One read-only text row in a form list (e.g. a detected hole entry).
pub fn list_row(ui: &mut Ui, text: impl Into<WidgetText>) -> Response {
    ui.label(text)
}

/// [`list_row`] cut to the width it has, with an ellipsis (a picked
/// edge's long name in a list).
pub fn list_row_truncated(ui: &mut Ui, text: impl Into<WidgetText>) -> Response {
    ui.add(Label::new(text).truncate())
}

/// A dimmed line that wraps, standing alone in a group (e.g. why a group
/// is not taken). A line under a row uses `consequence`.
pub fn note(ui: &mut Ui, text: impl Into<String>) -> Response {
    ui.add(Label::new(RichText::new(text).weak()).wrap())
}

/// A configured number field inside a `prop_row`, at its own width: it
/// joins the pane's Tab / Enter ring (#2516) and is named by the row's
/// label (#1119).
pub fn drag_row(ui: &mut Ui, dv: DragValue<'_>) -> Response {
    let r = ui.add(dv);
    seam_field(ui, r)
}

/// [`drag_row`] at the control column's width (`CTRL_W`).
pub fn drag_sized_row(ui: &mut Ui, dv: DragValue<'_>) -> Response {
    let r = ui.add_sized(Vec2::new(super::CTRL_W, ui.spacing().interact_size.y), dv);
    seam_field(ui, r)
}

fn seam_field(ui: &Ui, r: Response) -> Response {
    super::field_ring::register(ui, r.id);
    super::name_by_row(ui, r)
}

/// One radio choice inside a `prop_row` or stacked group.
pub fn radio_value<V: PartialEq>(
    ui: &mut Ui,
    current: &mut V,
    value: V,
    label: impl Into<WidgetText>,
) -> Response {
    ui.radio_value(current, value, label)
}

/// Two mutually exclusive radio choices laid out in order (inside a
/// right-to-left `prop_row`, `first` sits to the right of `second`).
pub fn radio_pair<V: PartialEq>(ui: &mut Ui, current: &mut V, first: (V, &str), second: (V, &str)) {
    radio_value(ui, current, first.0, first.1);
    radio_value(ui, current, second.0, second.1);
}
