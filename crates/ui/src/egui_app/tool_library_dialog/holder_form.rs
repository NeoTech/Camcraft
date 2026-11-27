//! #4338 (U12 of #3663): the holder form of the tool library window - a
//! user holder of the document with its name, its machine side (the
//! spindle taper), its tool side (the clamp and the shank range it
//! clamps) and its shape (gauge length, body and nose diameter) as
//! inputs, the holder's largest diameter as the drawing states it (#4455), the drawing to scale with the spindle nose and the shank range,
//! and the tools that use it. The values are the window's own draft
//! (`ComponentDraft::shape`); only the footer's Apply lands them, as one
//! undoable `EditDocumentComponent` that derives each of those tools again
//! in the same undo step - or refuses by name, and changes nothing, when a
//! tool no longer fits. The outline is the holder library's outline of the
//! values (`HolderShape::component`): never smaller than the steel, and
//! refused when it would read back as other values (#4455).
//! Frame cost (`crates/ui/AGENTS.md`): the check of the draft is kept for
//! its shape (`checks`); the drawing is `holder_drawing`'s.
//! Provenance: our own form over our own holder shape and component
//! commands; no HSMWorks material involved.

use std::cell::{Cell, RefCell};

use egui::{RichText, Ui, Vec2};

use camcraft_app::component::{ADAPTIVE_PORT, ToolComponent};
use camcraft_app::component_geometry::ComponentGeometry;
use camcraft_app::holder_library::{ER_SIZES, HolderEntry, HolderShape, ShapeClamp, tapers};

use super::component_edit::{close, current};
use crate::state::{ComponentAction, ComponentDraft, ErrorScope, State};

/// The field labels (their AccessKit names).
pub(crate) const NAME: &str = "Name";
pub(crate) const TAPER: &str = "Spindle taper";
pub(crate) const CLAMP: &str = "Clamp";
pub(crate) const FROM: &str = "Clamps from";
pub(crate) const TO: &str = "Clamps to";
pub(crate) const GAUGE: &str = "Gauge length";
/// The holder's largest diameter, flange included: read-only, the
/// drawing's value (#4455).
pub(crate) const LARGEST: &str = "Largest diameter";
/// The widest steel of the body below the flange.
pub(crate) const BODY: &str = "Body diameter";
pub(crate) const NOSE: &str = "Nose diameter";
pub(crate) const EDIT_OUTLINE: &str = "Edit outline…";
pub(crate) const IMPORT_STEP: &str = "Import model (STEP)…";
/// Why the outline buttons are not offered yet.
pub(crate) const NO_OUTLINE: &str = "an outline edit and a STEP import follow in #4390";
const FIELDS_W: f32 = 380.0;
const WELDON: &str = "Weldon side lock";
const SHRINK: &str = "shrink fit";

thread_local! {
    static KEPT: RefCell<Option<(HolderShape, Checked)>> = const { RefCell::new(None) };
    static CHECKS: Cell<u64> = const { Cell::new(0) };
}

/// The shape checks run on this thread - a still frame adds none.
#[cfg(test)]
pub(crate) fn checks() -> u64 {
    CHECKS.with(Cell::get)
}

/// The library entry and the geometry the shape draws, or why it is
/// refused.
type Checked = Result<(HolderEntry, ComponentGeometry), String>;

/// The library entry and the geometry the shape draws, or why the shape is
/// refused: the values and the outline, as Apply will check them. Kept for
/// the shape.
fn check(shape: &HolderShape) -> Checked {
    KEPT.with(|k| {
        let mut k = k.borrow_mut();
        if let Some((s, out)) = k.as_ref()
            && s == shape
        {
            return out.clone();
        }
        CHECKS.with(|c| c.set(c.get().saturating_add(1)));
        let out = shape.component(ADAPTIVE_PORT).and_then(|(_, g)| Ok((shape.entry()?, g))).map_err(|e| e.to_string());
        *k = Some((*shape, out.clone()));
        out
    })
}

/// The clamp as words.
fn clamp_text(c: ShapeClamp) -> String {
    match c {
        ShapeClamp::Er(er) => format!("{} collet", er.name),
        ShapeClamp::Weldon => WELDON.to_owned(),
        ShapeClamp::Shrink => SHRINK.to_owned(),
    }
}

/// The open holder draft, when the window shows the holder form.
fn draft(state: &State) -> Option<(&ComponentDraft, HolderShape)> {
    let d = state.tool_library.as_ref()?.component_edit.as_ref()?;
    Some((d, d.shape?))
}

/// Why the holder draft cannot be applied, or `None` (also when no holder
/// form is open, or the draft is the holder as it is).
pub(crate) fn refusal(state: &mut State) -> Option<String> {
    let (d, row, _, _) = current(state)?;
    let shape = d.shape?;
    if d == crate::state::draft_of(&row) {
        return None;
    }
    if d.name.trim().is_empty() {
        return Some("the holder needs a name".to_owned());
    }
    match check(&shape) {
        Err(why) => Some(why),
        // A body under the flange's envelope draws no other outline.
        Ok((_, g)) if g == row.geometry && shape.kind().ok().as_ref() == Some(row.component.kind()) && d.name.trim() == row.component.name() => {
            Some("the values draw the outline the holder has already (the flange envelope covers the body)".to_owned())
        }
        Ok(_) => None,
    }
}

/// The largest diameter (mm) of the holder that `shape` draws, flange
/// included: the value the drawing states, `None` while the values are
/// refused (#4455).
pub(crate) fn largest(shape: &HolderShape) -> Option<f64> {
    check(shape).ok().map(|(e, _)| super::holder_drawing::largest_d(&e))
}

/// The tools whose component stack uses the open holder.
pub(crate) fn users(state: &mut State) -> Vec<String> {
    current(state).map(|(_, _, _, users)| users).unwrap_or_default()
}

/// The footer's Apply: one undoable `EditDocumentComponent`, or the
/// refusal by name.
pub(super) fn apply(state: &mut State) {
    let Some((d, row, _, _)) = current(state) else {
        return;
    };
    let Some(shape) = d.shape else {
        return;
    };
    if d == crate::state::draft_of(&row) {
        return;
    }
    let built = shape
        .component(ADAPTIVE_PORT)
        .map_err(|e| e.to_string())
        .and_then(|(kind, geometry)| Ok((ToolComponent::new(&d.id, d.name.trim(), kind).map_err(|e| e.to_string())?, geometry)));
    match built {
        Ok(edit) => state.queue_component(ComponentAction::Edit(Box::new(edit))),
        Err(e) => state.refuse(ErrorScope::Tool, format!("holder not changed: {e}")),
    }
}

/// The footer's Duplicate: a new user holder, copied from the applied one.
pub(super) fn duplicate(state: &mut State) {
    if let Some((d, _)) = draft(state) {
        let id = d.id.clone();
        state.queue_component(ComponentAction::CopyAsUser { id });
    }
}

/// A value row: the label, the input (`mm`) and the unit.
fn number(ui: &mut Ui, label: &str, v: &mut f64, enabled: bool, why: &str) -> bool {
    let l = ui.label(label);
    let r = ui
        .add_enabled(enabled, egui::DragValue::new(v).speed(0.1).range(0.0..=1000.0).max_decimals(3))
        .labelled_by(l.id)
        .on_disabled_hover_text(why);
    ui.weak("mm");
    ui.end_row();
    r.changed()
}

pub(super) fn draw(ui: &mut Ui, state: &mut State) {
    let Some((mut d, _row, _, users)) = current(state) else {
        // Deleted (or undone) under the form: back to the list.
        close(state);
        return;
    };
    let Some(mut s) = d.shape else {
        close(state);
        return;
    };
    let mut open_tool = None;
    if let Some(msg) = state.error_for(ErrorScope::Tool).map(str::to_owned) {
        super::super::error_text::error_text(ui, super::super::colorvision::text_inks(ui, &state.world).error, &msg);
    }
    let h = ui.available_height().min(ui.ctx().viewport_rect().height() * 0.7 - 140.0).max(200.0);
    let checked = check(&s);
    let largest_d = largest(&s);
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.set_width(FIELDS_W);
            egui::ScrollArea::vertical().id_salt("tool_library_holder_form").max_height(h).show(ui, |ui| {
                fields(ui, &mut d.name, &mut s, largest_d);
                if let Err(why) = &checked {
                    super::super::error_text::error_text(ui, super::super::colorvision::text_inks(ui, &state.world).error, why);
                }
                ui.horizontal(|ui| {
                    let _ = ui.add_enabled(false, egui::Button::new(EDIT_OUTLINE)).on_disabled_hover_text(NO_OUTLINE);
                    let _ = ui.add_enabled(false, egui::Button::new(IMPORT_STEP)).on_disabled_hover_text(NO_OUTLINE);
                });
                ui.add_space(6.0);
                ui.label(RichText::new(format!("USED BY {}", super::holder_view::count(users.len()).to_uppercase())).small().weak());
                if users.is_empty() {
                    ui.weak("No tool stack uses this holder.");
                }
                for name in &users {
                    if ui.link(name.as_str()).on_hover_text("edit this tool in this window").clicked() {
                        open_tool = Some(name.clone());
                    }
                }
            });
        });
        ui.separator();
        ui.vertical(|ui| {
            ui.weak("to scale · mm");
            let size = Vec2::new(ui.available_width().max(200.0), (h - 24.0).max(180.0));
            match &checked {
                Ok((entry, _)) => super::holder_drawing::outline(ui, entry, s.clamps(), size, camcraft_app::units::Unit::Millimeters, true),
                Err(_) => {
                    ui.weak("The drawing shows when the values describe a holder.");
                }
            }
        });
    });
    d.shape = Some(s);
    if let Some(v) = state.tool_library.as_mut() {
        v.component_edit = Some(d);
    }
    if let Some(name) = open_tool {
        close(state);
        state.open_tool_in_library(&name);
    }
}

/// The name and the three sections; a change lands in the draft only.
fn fields(ui: &mut Ui, name: &mut String, s: &mut HolderShape, largest_d: Option<f64>) {
    egui::Grid::new("tool_library_holder_name").num_columns(2).show(ui, |ui| {
        let l = ui.label(NAME);
        ui.add(egui::TextEdit::singleline(name).desired_width(220.0)).labelled_by(l.id);
        ui.end_row();
    });
    section(ui, "MACHINE SIDE");
    egui::Grid::new("tool_library_holder_machine").num_columns(2).show(ui, |ui| {
        let l = ui.label(TAPER);
        egui::ComboBox::from_id_salt("holder_taper").selected_text(s.taper.name).show_ui(ui, |ui| {
            for t in tapers() {
                ui.selectable_value(&mut s.taper, t, t.name);
            }
        })
        .response
        .labelled_by(l.id);
        ui.end_row();
    });
    section(ui, "TOOL SIDE");
    egui::Grid::new("tool_library_holder_tool").num_columns(3).show(ui, |ui| {
        let l = ui.label(CLAMP);
        let mut clamp = s.clamp;
        egui::ComboBox::from_id_salt("holder_clamp").selected_text(clamp_text(clamp)).show_ui(ui, |ui| {
            for c in ER_SIZES.iter().map(|e| ShapeClamp::Er(*e)).chain([ShapeClamp::Weldon, ShapeClamp::Shrink]) {
                ui.selectable_value(&mut clamp, c, clamp_text(c));
            }
        })
        .response
        .labelled_by(l.id);
        ui.end_row();
        if clamp != s.clamp {
            set_clamp(s, clamp);
        }
        let bore = !matches!(s.clamp, ShapeClamp::Er(_));
        let (mut from, mut to) = s.clamps();
        let why = "an ER chuck clamps the collet range of its size";
        // A bore clamps one size: either field sets it.
        if number(ui, FROM, &mut from, bore, why) {
            s.bore = from;
        }
        if number(ui, TO, &mut to, bore, why) {
            s.bore = to;
        }
    });
    section(ui, "SHAPE");
    egui::Grid::new("tool_library_holder_shape").num_columns(3).show(ui, |ui| {
        number(ui, GAUGE, &mut s.gauge_len, true, "");
        let shrink = s.clamp == ShapeClamp::Shrink;
        number(ui, BODY, &mut s.largest_d, shrink, "an ER or Weldon body is as wide as its nose: edit the nose diameter");
        if number(ui, NOSE, &mut s.nose_d, true, "") && !shrink {
            s.largest_d = s.nose_d;
        }
        // The outline's widest section, as the drawing states it.
        ui.label(LARGEST);
        ui.label(largest_d.map_or_else(|| "-".to_owned(), largest_text)).on_hover_text("the flange or the body, whichever is wider");
        ui.weak("mm");
        ui.end_row();
    });
}

/// The largest diameter as the form shows it.
pub(crate) fn largest_text(d: f64) -> String {
    format!("{d:.2}")
}

/// A new clamp keeps what still applies: a bore keeps its size (a new one
/// takes the largest collet), a one-body clamp its nose.
fn set_clamp(s: &mut HolderShape, clamp: ShapeClamp) {
    if let ShapeClamp::Er(er) = s.clamp
        && !matches!(clamp, ShapeClamp::Er(_))
    {
        s.bore = er.max_bore;
    }
    s.clamp = clamp;
    match clamp {
        ShapeClamp::Shrink => s.largest_d = s.largest_d.max(s.nose_d),
        ShapeClamp::Er(_) | ShapeClamp::Weldon => s.largest_d = s.nose_d,
    }
}

fn section(ui: &mut Ui, head: &str) {
    ui.add_space(4.0);
    ui.label(RichText::new(head).small().weak());
}
