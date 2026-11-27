//! The file editor's "Axis chain" group (issue #2756): one row per axis
//! part in chain order, with move up, move down and remove, a linear's
//! reversed direction (#373), a rotary's axis with a note when its
//! letter is not the RS-274 one, and a form that adds a linear or
//! rotary part on the table or under the head.
//!
//! Provenance: our own layout, specced from issue #2756; the edits live
//! in the app layer (`camcraft_app::machine::{chain, draft}`): this
//! file draws the rows and calls the draft on a click.

use egui::{ComboBox, DragValue, Id, Label, Response, RichText, TextEdit, Ui};

use camcraft_app::machine::chain::{suggested_letter, AxisRow, Place, Step};
use camcraft_app::machine::chain_view::ChainView;
use camcraft_app::machine::draft::MachineDraft;
use camcraft_app::machine::validate::Refusal;

use super::layout::{button, group_open, list_row, prop_grid, small_button, GROUP_GAP};
use super::theme::Tokens;

/// The directions a new rotary can turn about.
pub const DIRECTIONS: [(&str, [f64; 3]); 6] = [
    ("+X", [1.0, 0.0, 0.0]),
    ("-X", [-1.0, 0.0, 0.0]),
    ("+Y", [0.0, 1.0, 0.0]),
    ("-Y", [0.0, -1.0, 0.0]),
    ("+Z", [0.0, 0.0, 1.0]),
    ("-Z", [0.0, 0.0, -1.0]),
];

/// The "add axis" form's typed state.
pub struct AxisForm {
    /// The new axis's letter being typed.
    pub name: String,
    pub place: Place,
    /// An index into `DIRECTIONS`; a rotary only.
    pub direction: usize,
}

impl Default for AxisForm {
    fn default() -> Self {
        Self { name: String::new(), place: Place::OnTable, direction: 0 }
    }
}

/// What a click in the group asks of the draft.
pub(crate) enum Edit {
    Move(String, Step),
    Remove(String),
    /// A linear's `direction`: reversed (`-1`) or standard.
    Reverse(String, bool),
    /// A rotary's new `axis`.
    TurnAbout(String, [f64; 3]),
    Add,
}

const TRADE_ABOVE: &str = "Trade places with the axis above";
const TRADE_BELOW: &str = "Trade places with the axis below";
const TRADE_KIND: &str = "An axis trades places only with one of its kind that moves the same thing; to put it elsewhere, remove it and add it again";
/// The width of a field, as in the editor's other groups.
const FIELD_W: f32 = 140.0;

/// The row's label: its letter and kind.
fn row_words(r: &AxisRow) -> String {
    format!("{} - {}", r.name, if r.rotary { "rotary" } else { "linear" })
}

/// A short button keeps its full name for assistive tech and tests
/// ("Move A up" where the button shows "Up").
fn named(r: Response, name: String) -> Response {
    r.ctx.accesskit_node_builder(r.id, |n| n.set_label(name));
    r
}

/// A row's label in the grid's first column, with its help on hover.
/// Returns the label's id, to name the control in the second column.
fn cell(ui: &mut Ui, words: &str, help: &str) -> Id {
    ui.label(words).on_hover_text(help).id
}

/// Draw the group from the draft's view of the chain, built once per
/// edit (`MachineDraft::chain_view`; the frame-cost rule, #2238). Returns
/// the edit a click asks for; the caller applies it after the frame's
/// other fields (`apply`).
pub(crate) fn draw(
    ui: &mut Ui,
    view: &ChainView,
    form: &mut AxisForm,
    refusals: &[Refusal],
    pal: &Tokens,
    error: egui::Color32,
) -> Option<Edit> {
    let rows = &view.rows;
    let mut edit: Option<Edit> = None;
    group_open(ui, "Axis chain", None, true, |ui| {
        if rows.is_empty() {
            list_row(ui, RichText::new("no axis stated").color(pal.text_dim));
        }
        // One grid, as the editor's other groups: the label column and
        // the controls stay together at any width of the pane.
        prop_grid(ui, "machine-axis-chain", |ui| {
            for i in 0..rows.len() {
                if let Some(e) = axis_rows(ui, view, i, refusals, pal, error) {
                    edit = Some(e);
                }
                ui.allocate_space(egui::vec2(0.0, GROUP_GAP));
                ui.end_row();
            }
            new_axis_form(ui, form);
        });
        if button(ui, "Add axis").on_hover_text("Add the axis to the chain").clicked() {
            edit = Some(Edit::Add);
        }
    });
    edit
}

/// The grid rows of axis `i`: its letter and kind with Up, Down and
/// remove, what it moves, its own fields, then its notes and refusals.
fn axis_rows(
    ui: &mut Ui,
    view: &ChainView,
    i: usize,
    refusals: &[Refusal],
    pal: &Tokens,
    error: egui::Color32,
) -> Option<Edit> {
    let rows = &view.rows;
    let r = &rows[i];
    let mut edit = None;
    // A move trades only with a neighbour of the same kind that moves
    // the same thing (the work or the tool).
    let same = |j: Option<usize>| {
        j.and_then(|j| rows.get(j)).is_some_and(|n| n.rotary == r.rotary && n.carries_head == r.carries_head)
    };
    cell(ui, &row_words(r), &format!("Part: {}", r.pointer));
    ui.horizontal(|ui| {
        let moves = [(Step::Up, "Up", same(i.checked_sub(1)), TRADE_ABOVE), (Step::Down, "Down", same(Some(i + 1)), TRADE_BELOW)];
        for (step, words, enabled, tip) in moves {
            let b = ui.add_enabled_ui(enabled, |ui| small_button(ui, words)).inner;
            let b = named(b, format!("Move {} {}", r.name, words.to_lowercase()));
            if b.on_hover_text(tip).on_disabled_hover_text(TRADE_KIND).clicked() {
                edit = Some(Edit::Move(r.pointer.clone(), step));
            }
        }
        let remove = named(small_button(ui, "\u{2716}"), format!("Remove {}", r.name));
        if remove.on_hover_text("Remove this axis; the parts it carries stay").clicked() {
            edit = Some(Edit::Remove(r.pointer.clone()));
        }
    });
    ui.end_row();
    let l = cell(ui, &format!("{} moves", r.name), "An axis with the head below it moves the tool; any other moves the work");
    ui.label(if r.carries_head { "the tool" } else { "the work" }).labelled_by(l);
    ui.end_row();
    if let Some(e) = row_fields(ui, r) {
        edit = Some(e);
    }
    for note in &view.notes[i] {
        ui.label("");
        ui.add(Label::new(RichText::new(format!("Note: {note}")).color(pal.text_dim)).wrap());
        ui.end_row();
    }
    let keys = &view.keys[i];
    for refusal in refusals.iter().filter(|x| keys.contains(&x.key)) {
        ui.label("");
        super::error_text::error_text(ui, error, &refusal.to_string());
        ui.end_row();
    }
    edit
}

/// The "add axis" form: one setting per grid row.
fn new_axis_form(ui: &mut Ui, form: &mut AxisForm) {
    let letter = DIRECTIONS.get(form.direction).and_then(|d| suggested_letter(d.1)).unwrap_or("A");
    let l = cell(ui, "New axis", "X, Y, Z, U, V or W for a linear; A, B or C for a rotary");
    let field = TextEdit::singleline(&mut form.name).id_salt("machine-axis-new").desired_width(FIELD_W).hint_text(letter);
    ui.add(field).labelled_by(l);
    ui.end_row();
    let l = cell(ui, "Place", "On the table, a rotary turns the work; carrying the head, it tilts the tool");
    let words = |p: Place| if p == Place::OnTable { "On the table" } else { "Carries the head" };
    let c = ComboBox::from_id_salt("machine-axis-new-place").width(FIELD_W).selected_text(words(form.place)).show_ui(ui, |ui| {
        for p in [Place::OnTable, Place::CarriesHead] {
            ui.selectable_value(&mut form.place, p, words(p));
        }
    });
    c.response.labelled_by(l);
    ui.end_row();
    let before = form.direction;
    let l = cell(ui, "Rotary turns about", "The direction a new rotary turns about; it suggests the RS-274 letter");
    let words = DIRECTIONS.get(form.direction).map_or("+X", |d| d.0);
    let c = ComboBox::from_id_salt("machine-axis-new-about").width(FIELD_W).selected_text(words).show_ui(ui, |ui| {
        for (i, (words, _)) in DIRECTIONS.iter().enumerate() {
            ui.selectable_value(&mut form.direction, i, *words);
        }
    });
    c.response.labelled_by(l);
    ui.end_row();
    if form.direction != before
        && let Some(d) = DIRECTIONS.get(form.direction)
    {
        suggest(form, d.1);
    }
}

/// A row's own fields: a linear's reversed box, a rotary's axis as
/// ±X/±Y/±Z or three numbers.
fn row_fields(ui: &mut Ui, r: &AxisRow) -> Option<Edit> {
    let mut edit = None;
    if !r.rotary {
        let l = cell(ui, &format!("{} reversed", r.name), "direction -1: this slide's + moves the other way");
        let mut reversed = r.reversed;
        let b = ui.checkbox(&mut reversed, "").labelled_by(l);
        // The blank checkbox's empty name must not hide the row's label.
        b.ctx.accesskit_node_builder(b.id, |n| n.clear_label());
        if b.changed() {
            edit = Some(Edit::Reverse(r.pointer.clone(), reversed));
        }
        ui.end_row();
        return edit;
    }
    let now = r.axis.and_then(unit).and_then(|u| DIRECTIONS.iter().find(|d| d.1 == u)).map_or("other", |d| d.0);
    let help = "The direction the rotary turns about; \"other\" is a tilted axis, set by its three numbers";
    let l = cell(ui, &format!("{} turns about", r.name), help);
    let c = ComboBox::from_id_salt(("machine-axis-about", &r.pointer)).width(FIELD_W).selected_text(now).show_ui(ui, |ui| {
        for (words, axis) in DIRECTIONS {
            if ui.selectable_label(now == words, words).clicked() && now != words {
                edit = Some(Edit::TurnAbout(r.pointer.clone(), axis));
            }
        }
    });
    c.response.labelled_by(l);
    ui.end_row();
    let mut a = r.axis.unwrap_or([0.0; 3]);
    let mut changed = false;
    cell(ui, &format!("{} axis", r.name), "The axis direction as three numbers: x, y, z");
    ui.horizontal(|ui| {
        // Keyed by the part and component, so a note or refusal row that
        // comes or goes above does not take the focus mid-edit.
        for (c, word) in a.iter_mut().zip(["x", "y", "z"]) {
            let id = ("machine-axis-dir", &r.pointer, word);
            let d = ui.push_id(id, |ui| ui.add(DragValue::new(c).speed(0.01).max_decimals(4))).inner;
            changed |= d.on_hover_text(format!("{} axis {word}: any direction, not only ±X, ±Y, ±Z", r.name)).changed();
        }
    });
    ui.end_row();
    if changed {
        edit = Some(Edit::TurnAbout(r.pointer.clone(), a));
    }
    edit
}

/// `axis` scaled to length one, if it has a length.
fn unit(axis: [f64; 3]) -> Option<[f64; 3]> {
    let len = axis.iter().map(|x| x * x).sum::<f64>().sqrt();
    (len.is_finite() && len > 0.0).then(|| axis.map(|x| x / len))
}

/// Picking the new rotary's direction suggests its RS-274 letter, unless
/// the user typed a linear letter.
fn suggest(form: &mut AxisForm, axis: [f64; 3]) {
    let typed = form.name.trim().to_ascii_uppercase();
    if let Some(l) = suggested_letter(axis)
        && matches!(typed.as_str(), "" | "A" | "B" | "C")
    {
        l.clone_into(&mut form.name);
    }
}

/// Apply one edit to the draft. A refusal changes nothing and comes
/// back by name for the status line.
pub(crate) fn apply(draft: &mut MachineDraft, form: &mut AxisForm, edit: Edit) -> Result<(), String> {
    match edit {
        Edit::Move(at, step) => draft.move_axis(&at, step).map(|_| ()),
        Edit::Remove(at) => draft.remove_axis(&at),
        Edit::Reverse(at, reversed) => draft.set_axis_reversed(&at, reversed),
        Edit::TurnAbout(at, axis) => draft.set_rotary_axis(&at, axis),
        Edit::Add => {
            let name = form.name.trim().to_ascii_uppercase();
            let r = if matches!(name.as_str(), "A" | "B" | "C") {
                let axis = DIRECTIONS.get(form.direction).map_or([1.0, 0.0, 0.0], |d| d.1);
                draft.add_rotary_axis(form.place, &name, axis)
            } else {
                draft.add_linear_axis(form.place, &name)
            };
            r.map(|_| form.name.clear())
        }
    }
}
