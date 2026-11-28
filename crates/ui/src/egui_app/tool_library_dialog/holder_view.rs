//! #3897 (U6 of #3663): the read-only holder view of the tool library
//! window, as the holder picture of the #3663 mock-up: the machine side
//! (the spindle taper, marked when it is the mounted machine's), the tool
//! side (the clamp and its clamping range), the shape (gauge length,
//! largest and nose diameter), the document tools that use the holder
//! (a click opens the tool in the editor), and the outline to scale with
//! its dimension lines. Nothing here is editable: a built-in holder is
//! fixed. #3898 (U7): "Copy as user holder" makes an editable copy.
//! #4338 (U12): the drawing is `holder_drawing` (shared with the holder
//! form), with the spindle nose and the shank range.
//! Provenance: our own view over `holder_library::HolderEntry`; no
//! HSMWorks material involved.

use egui::{RichText, Ui, Vec2};

use camcraft_app::holder_library::{Clamp, HolderEntry};
use camcraft_app::tool_body;
use camcraft_app::units::{self, Unit};

use super::holders::{self, READ_ONLY};
use crate::state::State;

#[cfg(test)]
pub(crate) use super::holder_drawing::builds;

/// The way back to the holder list: since #4334 the frame's header button.
#[cfg(test)]
pub(crate) const BACK: &str = super::frame::BACK;
/// The heading of the tools that use the holder.
pub(crate) const USED_BY: &str = "Used by";
/// The drawing's dimension toggle.
pub(crate) const DIMS: &str = "Dimensions";
const FACTS_W: f32 = 380.0;

/// What the user asked for this frame.
enum Ask {
    Tool(String),
    /// #3898: copy the built-in holder as a user holder of the document.
    Copy(String),
}

pub(super) fn draw(ui: &mut Ui, state: &mut State) {
    let Some(view) = state.tool_library.as_ref().map(|v| v.holders.clone()) else {
        return;
    };
    let rows = holders::rows();
    let Some(row) = view.open.as_deref().and_then(|n| rows.iter().find(|r| r.name == n)) else {
        // A name the library does not hold: back to the list.
        state.open_holder_in_library(None);
        return;
    };
    let e = row.entry;
    let u = state.units();
    let machine = holders::machine_taper(state);
    let users = tool_body::tools_using_holder(&state.world, &row.name);
    let mut ask = None;
    let mut hide = view.hide_dims;
    ui.horizontal(|ui| {
        // #4334: the way back and the name head the window (the frame).
        ui.weak(format!("holder · used by {}", count(users.len())));
        // #3898: only a holder with a taper is a built-in component.
        let id = format!("{}holder:{}", camcraft_app::component_library::BUILTIN_PREFIX, row.name);
        if ui
            .add_enabled(e.taper.is_some(), egui::Button::new(holders::COPY))
            .on_hover_text("a user holder in this document, made from this one, that you can edit (one undo step)")
            .on_disabled_hover_text("a straight-shank chuck states no shank size, so it is not a component")
            .clicked()
        {
            ask = Some(Ask::Copy(id));
        }
    });
    ui.weak(READ_ONLY);
    ui.separator();
    // Bounded by the screen: a column as tall as the room left would let
    // the window grow each frame.
    let h = ui.available_height().min(ui.ctx().viewport_rect().height() * 0.7 - 140.0).max(160.0);
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.set_width(FACTS_W);
            egui::ScrollArea::vertical().id_salt("tool_library_holder_facts").max_height(h).show(ui, |ui| {
                facts(ui, &e, machine.as_deref(), u);
                if let Some(t) = used_by(ui, state, &users) {
                    ask = Some(Ask::Tool(t));
                }
            });
        });
        ui.separator();
        ui.vertical(|ui| {
            let mut show = !hide;
            ui.horizontal(|ui| {
                ui.checkbox(&mut show, DIMS).on_hover_text("show the dimension lines of the drawing");
                ui.weak(format!("to scale · {}", u.suffix()));
            });
            hide = !show;
            let size = Vec2::new(ui.available_width().max(160.0), (h - 30.0).max(160.0));
            super::holder_drawing::outline(ui, &e, clamps(&e), size, u, show);
        });
    });
    if let Some(v) = state.tool_library.as_mut() {
        v.holders.hide_dims = hide;
    }
    match ask {
        Some(Ask::Tool(name)) => state.open_tool_in_library(&name),
        Some(Ask::Copy(id)) => state.queue_component(crate::state::ComponentAction::CopyAsUser { id }),
        None => {}
    }
}

/// The shank diameters the holder clamps, from and to.
pub(crate) fn clamps(e: &HolderEntry) -> (f64, f64) {
    match e.clamp {
        Clamp::Er(er) => (er.min_bore, er.max_bore),
        Clamp::Weldon { bore, .. } | Clamp::Shrink { bore, .. } => (bore, bore),
    }
}

pub(crate) fn count(n: usize) -> String {
    if n == 1 { "1 tool".to_owned() } else { format!("{n} tools") }
}

/// The machine side, the tool side and the shape, as label rows.
fn facts(ui: &mut Ui, e: &HolderEntry, machine: Option<&str>, u: Unit) {
    let l = |v: f64| format!("{} {}", units::len(v, u, 2), u.suffix());
    let taper = e.taper.map_or(holders::STRAIGHT, |t| t.name);
    let taper_note = match (e.taper, machine) {
        (Some(t), Some(m)) if m.eq_ignore_ascii_case(t.name) => "the mounted machine's taper".to_owned(),
        (_, Some(m)) => format!("the mounted machine takes {m}"),
        (_, None) => "the machine states no taper".to_owned(),
    };
    let nose = e.body_r() * 2.0;
    let sections: [(&str, Vec<(&str, String)>); 3] = [
        ("MACHINE SIDE", vec![("Spindle taper", format!("{taper} ({taper_note})"))]),
        ("TOOL SIDE", vec![("Clamp", holders::clamp_text(e)), ("Clamping range", format!("Ø {} {}", holders::range_text(e, u), u.suffix()))]),
        (
            "SHAPE",
            vec![("Gauge length", l(e.gauge_len)), ("Largest diameter", l(super::holder_drawing::largest_d(e))), ("Nose diameter", l(nose))],
        ),
    ];
    for (head, rows) in sections {
        ui.add_space(4.0);
        ui.label(RichText::new(head).small().weak());
        egui::Grid::new(("holder_facts", head)).num_columns(2).show(ui, |ui| {
            for (k, v) in rows {
                ui.label(k);
                ui.label(v);
                ui.end_row();
            }
        });
    }
}

/// The tools that use the holder: a link for each, with its stick-out and
/// gauge length. Gives the tool the user opened.
fn used_by(ui: &mut Ui, state: &State, users: &[String]) -> Option<String> {
    ui.add_space(6.0);
    ui.label(RichText::new(format!("{USED_BY} {}", count(users.len()))).small().weak());
    if users.is_empty() {
        ui.weak("No tool of this document uses this holder.");
        return None;
    }
    let u = state.units();
    let refs = tool_body::holder_users(&state.world);
    let mut out = None;
    for name in users {
        ui.horizontal(|ui| {
            if ui.link(name.as_str()).on_hover_text("edit this tool in this window").clicked() {
                out = Some(name.clone());
            }
            if let Some((_, r)) = refs.iter().find(|(n, _)| n == name) {
                let l = |v: f64| units::len(v, u, 2);
                ui.weak(format!("stick-out {}, gauge {} {}", l(r.projection), l(r.gauge_len + r.projection), u.suffix()));
            }
        });
    }
    out
}
