//! The tool editor's holder library picker (#793, #792): choose a
//! built-in holder (BT30 / BT40 / CAT40 or Tormach TTS, ER chuck or
//! Weldon, or a standalone ER chuck), set its gauge length and projection, and Apply
//! holder - which queues the one undoable
//! `SetToolBody::library`. #2018: one Weldon row for each taper, its
//! bore the library's standard bore for the tool's shank. #2016: the tool keeps a reference to the
//! entry; the Profile table above shows its outline, and an edited
//! point makes it a drawn profile of its own. Provenance: our own
//! pane over `app::holder_library`; no vendor material consulted. #3895:
//! the tool library window's assembly group draws the same fields; #3930:
//! the tool shape window no longer does.

use egui::{Color32, Ui};

use camcraft_app::holder_library::{self, Clamp, HolderEntry, PickerRow};
use camcraft_app::units::{self, Unit};

use crate::state::State;

fn note_ink() -> Color32 {
    crate::egui_app::theme::current().tool_note
}

/// #3895: the picker without its Apply holder button - the pick, the
/// gauge length, the projection and what the pick is. The tool library
/// window's assembly group adds its own rows before the button. #3930:
/// that group is the one place that draws the picker. #4336 (U10): the
/// rows read as the #3663 mock-up - Holder, Stick-out (the projection),
/// then the holder's own gauge length; the group shows the total and the
/// clamping-range check (`assembly_group`).
pub(super) fn holder_fields(ui: &mut Ui, state: &mut State, u: Unit) {
    let picked = state.tool_profile_ui.library.clone();
    let mut choose = None;
    super::layout::prop_row(ui, HOLDER, |ui| {
        egui::ComboBox::from_id_salt("tool_library_holder")
            .selected_text(picked.as_deref().unwrap_or("choose a holder"))
            .width(170.0)
            .show_ui(ui, |ui| {
                for r in holder_library::picker_rows() {
                    let n = r.name();
                    if ui.selectable_label(picked.as_deref() == Some(n.as_str()), &n).on_hover_text(describe_row(&r, u)).clicked() {
                        choose = Some(n);
                    }
                }
            });
    });
    if let Some(n) = choose {
        state.pick_library_holder(&n);
    }
    let Some(n) = state.tool_profile_ui.library.clone() else {
        return;
    };
    super::layout::prop_row_hint(ui, STICK_OUT, "from the tool tip to the holder's nose face", |ui| {
        super::fields::num_row(ui, u, &mut state.tool_profile_ui.projection)
    });
    super::layout::prop_row_hint(
        ui,
        HOLDER_GAUGE,
        "the holder's own length: from the gauge line (the spindle face) to its nose face",
        |ui| super::fields::num_row(ui, u, &mut state.tool_profile_ui.gauge),
    );
    // What the pick is: the entry as the tool's shank resolves it (a
    // Weldon row takes its bore from the shank, #2018), else the row.
    let shank = state.tool_ui.shank_d;
    let what = holder_library::resolve(&n, shank).map(|e| describe(&e, u)).ok().or_else(|| {
        holder_library::picker_rows().into_iter().find(|r| r.name() == n).map(|r| describe_row(&r, u))
    });
    if let Some(w) = what {
        ui.colored_label(note_ink(), w);
    }
}

/// #4336: the row labels of the picker.
pub(crate) const HOLDER: &str = "Holder";
pub(crate) const STICK_OUT: &str = "Stick-out";
pub(crate) const HOLDER_GAUGE: &str = "Holder gauge length";

/// "Apply holder": queue the one `SetToolBody::library` of the pick.
pub(super) fn apply_holder_button(ui: &mut Ui, state: &mut State) {
    let pick = state.tool_profile_ui.library.is_some();
    if ui
        .add_enabled(pick, egui::Button::new("Apply holder"))
        .on_hover_text("set the library holder as the tool's holder (one undo step); the tool keeps the holder by name until its profile is edited")
        .clicked()
    {
        state.queue_apply_tool_holder();
    }
}

/// One line on what a picker row is: an entry as [`describe`] says, or
/// a taper's Weldon or shrink-fit row with the bores it takes the tool's
/// shank into.
fn describe_row(r: &PickerRow, u: Unit) -> String {
    match r {
        PickerRow::Entry(e) => describe(e, u),
        PickerRow::Weldon { taper: t, bores } | PickerRow::Shrink { taper: t, bores } => {
            let l = |v: f64| units::len(v, u, 2);
            let b: Vec<String> = bores.iter().map(|b| format!("Ø{}", l(*b))).collect();
            let kind = if matches!(r, PickerRow::Shrink { .. }) { "shrink fit" } else { "Weldon" };
            format!(
                "{} {} Ø{} × {} {}; {kind}, bore for the tool's shank: {}",
                t.name,
                t.ring,
                l(t.flange_d),
                l(t.flange_t),
                u.suffix(),
                b.join(", ")
            )
        }
    }
}

/// One line on what the entry is: flange, then nut or body.
fn describe(e: &HolderEntry, u: Unit) -> String {
    let l = |v: f64| units::len(v, u, 2);
    let nose = match e.clamp {
        Clamp::Er(er) => format!(
            "{} nut Ø{} × {}, collets Ø{}-{}",
            er.name,
            l(er.nut_d),
            l(er.nut_len),
            l(er.min_bore),
            l(er.max_bore)
        ),
        Clamp::Weldon { bore, body_d } => format!("Weldon bore Ø{}, body Ø{}", l(bore), l(body_d)),
        Clamp::Shrink { bore, nose_d, body_d } => {
            format!("shrink-fit bore Ø{}, nose Ø{}, body Ø{}", l(bore), l(nose_d), l(body_d))
        }
    };
    match e.taper {
        Some(t) => format!("{} {} Ø{} × {} {}; {nose}", t.name, t.ring, l(t.flange_d), l(t.flange_t), u.suffix()),
        None => format!("straight-shank chuck {}; {nose}", u.suffix()),
    }
}
