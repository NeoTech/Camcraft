//! The tool editor's holder library picker (#793): choose a built-in
//! steep-taper holder (BT30 / BT40 / CAT40, ER chuck or Weldon), set its
//! gauge length, and Apply holder - which queues the one undoable
//! `SetToolBody::library`. The holder lands as a drawn profile, so the
//! Profile table above edits it like any other. Provenance: our own
//! pane over `app::holder_library`; no vendor material consulted.

use egui::{Color32, Ui};

use camcraft_app::holder_library::{self, Clamp, HolderEntry};
use camcraft_app::units::{self, Unit};

use crate::state::State;

const NOTE: Color32 = Color32::from_rgb(0x9E, 0xA8, 0xB8);
const BAD: Color32 = Color32::from_rgb(230, 90, 80);

/// The picker under the Profile section.
pub(super) fn holder_picker(ui: &mut Ui, state: &mut State, u: Unit) {
    ui.separator();
    ui.strong("Holder library");
    let picked = state.tool_profile_ui.library.clone();
    let mut choose = None;
    egui::ComboBox::from_label("Library holder")
        .selected_text(picked.as_deref().unwrap_or("choose a holder"))
        .width(170.0)
        .show_ui(ui, |ui| {
            for e in holder_library::library() {
                let n = e.name();
                if ui.selectable_label(picked.as_deref() == Some(n.as_str()), &n).on_hover_text(describe(&e, u)).clicked() {
                    choose = Some(n);
                }
            }
        });
    if let Some(n) = choose {
        state.pick_library_holder(&n);
    }
    let entry = state.tool_profile_ui.library.as_deref().and_then(|n| holder_library::find(n).ok());
    if let Some(e) = &entry {
        ui.horizontal(|ui| {
            ui.label(format!("gauge length {}", u.suffix()));
            super::tool_profile_page::len_cell(ui, u, &mut state.tool_profile_ui.gauge);
        })
        .response
        .on_hover_text("from the gauge line (the spindle face) to the holder's nose face");
        ui.colored_label(NOTE, describe(e, u));
        let shank = state.tool_ui.to_tool().shank_d;
        if let Err(err) = e.fits_shank(shank) {
            ui.colored_label(BAD, err.to_string());
        }
    }
    if ui
        .add_enabled(entry.is_some(), egui::Button::new("Apply holder"))
        .on_hover_text("drop the library holder into the tool's holder (one undo step); it then edits as a Custom profile")
        .clicked()
    {
        state.queue_apply_tool_holder();
    }
}

/// One line on what the entry is: flange, then nut or body.
fn describe(e: &HolderEntry, u: Unit) -> String {
    let l = |v: f64| units::len(v, u, 2);
    let t = e.taper;
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
    };
    format!("{} flange Ø{} × {} {}; {nose}", t.name, l(t.flange_d), l(t.flange_t), u.suffix())
}
