//! #3898 (U7 of #3663): the component stack of the tool open in the tool
//! library window - the spindle-side chain (holder, adapter or cutter
//! body, insert or solid tool) as ISO 13399 links of the document's
//! component table, each with its port, its turn about Z and its
//! stick-out, and its library status. "Add component" offers only the
//! parts that fit the last holding part (`component_fit::compatible_children`),
//! or the holding parts for an empty stack. The links are the window's
//! own draft (`StackDraft`); "Apply stack" lands them as one undoable
//! `SetToolAssembly`, which derives the tool's record and body in the
//! same step; "Detach to standalone tool" clears the stack and keeps the
//! tool as it is. A lathe tool (turning, groove, thread, form) has the
//! same group: its empty stack offers only a lathe holder or a boring
//! bar, then the inserts that fit the holder's seat, and the lathe side
//! of the resolver (#3643, #4278) builds it. Under the rows the group
//! shows, read-only, what the resolver gives for the shown stack: the
//! gauge length, the cutting diameter and the corner radius (for a lathe
//! tool: gauge length 0, the insert's inscribed circle as the diameter,
//! the nose radius and the orientation), or its refusal by name. "Swap" on an
//! applied stack offers, for one row, only the parts with which the whole
//! stack still resolves, and lands the new stack as one undoable
//! `SetToolAssembly`. Frame cost: State keeps the offer, the values, the
//! swap lists and the rows (`state_component_views`), keyed on the
//! component table, the stacks, the library read, the tool's record and
//! the draft; a still frame builds nothing.
//! Provenance: our own group over our own component commands; no
//! HSMWorks material involved.

use egui::{RichText, Ui};

use camcraft_app::model;
use camcraft_app::schema::AssemblyLink;
use camcraft_app::tool_assembly as ta;

#[cfg(test)]
pub(crate) use crate::state::candidates;
use crate::state::{ComponentAction, StackDraft, StackValues, State, SwapOffer};

pub(crate) const HEADING: &str = "Component stack";
pub(crate) const ADD: &str = "Add component";
pub(crate) const REMOVE: &str = "Remove last component";
pub(crate) const APPLY: &str = "Apply stack";
pub(crate) const DETACH: &str = "Detach to standalone tool";
/// The labels of the derived values under the rows.
pub(crate) const GAUGE: &str = "Gauge length";
pub(crate) const DIAMETER: &str = "Cutting diameter";
pub(crate) const CORNER: &str = "Corner radius";
/// The labels of a lathe stack's derived values (#3643).
pub(crate) const INSCRIBED: &str = "Inscribed circle (diameter)";
pub(crate) const NOSE: &str = "Nose radius";
pub(crate) const ORIENTATION: &str = "Orientation";

/// The label of row `name`'s swap menu.
#[must_use]
pub(crate) fn swap_label(name: &str) -> String {
    format!("Swap {name}…")
}
const HEADS: [&str; 6] = ["#", "Component", "Port", "Turn (°)", "Stick-out (mm)", "Library"];

pub(super) fn draw(ui: &mut Ui, state: &mut State) {
    ui.add_space(6.0);
    ui.separator();
    ui.strong(HEADING);
    let name = state.tool_ui.name.clone();
    let Some(entity) = model::find_by_name(&state.world, &name) else {
        return;
    };
    let stored = ta::assembly_of(&state.world, &entity).map(|a| a.0);
    let doc = state.doc_version;
    let mut draft = match state.tool_library.as_ref().and_then(|v| v.stack.clone()) {
        Some(d) if d.tool == name && d.synced == doc => d,
        _ => StackDraft { tool: name.clone(), synced: doc, links: stored.clone().unwrap_or_default() },
    };
    let derived = state.component_derived(None);
    let current = model::tool_of(&state.world, &entity);
    let view = state.component_stack_view(&name, current.as_ref(), &draft.links);
    // A swap lands a whole stack: it is offered on the applied stack only.
    let applied = stored.as_ref() == Some(&draft.links);
    let mut edit = None;
    let mut swap: Option<Vec<AssemblyLink>> = None;
    if draft.links.is_empty() {
        ui.weak("No component stack: the tool is standalone.");
    } else {
        egui::Grid::new("tool_library_stack").striped(true).show(ui, |ui| {
            for h in HEADS {
                ui.label(RichText::new(h).strong());
            }
            ui.label("");
            ui.end_row();
            for (i, link) in draft.links.iter_mut().enumerate() {
                let row = derived.rows.iter().find(|(r, _, _)| r.component.id() == link.component);
                ui.label(i.saturating_add(1).to_string());
                ui.label(row.map_or_else(|| format!("{} (not in the document)", link.component), |(r, _, _)| r.component.name().to_owned()));
                match link.mount.as_mut() {
                    Some(m) => {
                        ui.label(m.port.clone().unwrap_or_else(|| "-".to_owned()));
                        ui.add(egui::DragValue::new(&mut m.z_rot_deg).speed(1.0).range(-360.0..=360.0));
                        let mut p = m.projection_mm.unwrap_or(0.0);
                        ui.add(egui::DragValue::new(&mut p).speed(0.5).range(0.0..=1000.0))
                            .on_hover_text("from the holder's port to the cutting tip; 0: home on the bore stop");
                        m.projection_mm = (p > 0.0).then_some(p);
                    }
                    None => {
                        ui.label("spindle");
                        ui.label("-");
                        ui.label("-");
                    }
                }
                ui.label(row.map_or("-", |(_, s, _)| super::components::status_text(s)));
                ui.horizontal(|ui| {
                    if let Some((r, _, users)) = row
                        && ui
                            .small_button(format!("Edit {}", r.component.name()))
                            .on_hover_text(format!("edit this shared component: {} tool(s) use it", users.len()))
                            .clicked()
                    {
                        edit = Some(link.component.clone());
                    }
                    if let Some((r, _, _)) = row {
                        swap_menu(ui, r.component.name(), applied, view.swaps.get(i).map_or(&[][..], Vec::as_slice), &mut swap);
                    }
                });
                ui.end_row();
            }
        });
    }
    values_row(ui, view.values.as_ref(), applied);
    let offer = &view.offer;
    let mut add = None;
    ui.horizontal_wrapped(|ui| {
        let label = ui.label(ADD);
        egui::ComboBox::from_id_salt("tool_library_stack_add")
            .selected_text(if offer.is_empty() { "nothing fits" } else { "choose a part" })
            .show_ui(ui, |ui| {
                for (i, (text, _)) in offer.iter().enumerate() {
                    if ui.selectable_label(false, text).clicked() {
                        add = Some(i);
                    }
                }
            })
            .response
            .labelled_by(label.id)
            .on_hover_text("the document's components that fit; add parts to the document on the My library page");
        if ui.add_enabled(!draft.links.is_empty(), egui::Button::new(REMOVE)).clicked() {
            draft.links.pop();
        }
    });
    if let Some((_, link)) = add.and_then(|i| offer.get(i)) {
        draft.links.push(link.clone());
    }
    let mut ask = swap.map(|links| ComponentAction::SetStack { tool: name.clone(), links: Some(links) });
    ui.horizontal_wrapped(|ui| {
        let changed = Some(&draft.links) != stored.as_ref() && !draft.links.is_empty();
        if ui
            .add_enabled(changed, egui::Button::new(APPLY))
            .on_hover_text("build the tool from this stack: its record and holder body follow (one undo step)")
            .clicked()
        {
            ask = Some(ComponentAction::SetStack { tool: name.clone(), links: Some(draft.links.clone()) });
        }
        if ui
            .add_enabled(stored.is_some(), egui::Button::new(DETACH))
            .on_hover_text("drop the stack; the tool keeps its record and body as they are (one undo step)")
            .clicked()
        {
            ask = Some(ComponentAction::SetStack { tool: name.clone(), links: None });
        }
    });
    if let Some(v) = state.tool_library.as_mut() {
        v.stack = Some(draft);
    }
    if let Some(a) = ask {
        state.queue_component(a);
    }
    if let Some(id) = edit {
        super::component_edit::open(state, &id);
    }
}

/// Row `name`'s swap menu: the replacements with which the stack still
/// resolves; a pick sets `swap` to the whole new stack.
fn swap_menu(ui: &mut Ui, name: &str, applied: bool, offers: &[SwapOffer], swap: &mut Option<Vec<AssemblyLink>>) {
    let enabled = applied && !offers.is_empty();
    ui.add_enabled_ui(enabled, |ui| {
        ui.menu_button(swap_label(name), |ui| {
            for (text, links) in offers {
                if ui.button(text).clicked() {
                    *swap = Some(links.clone());
                    ui.close();
                }
            }
        })
        .response
        .on_hover_text("put a part that fits in this place; the stack is built again (one undo step)")
        .on_disabled_hover_text(if applied { "no other part in the document fits here" } else { "apply or undo the stack edits first" });
    });
}

/// The resolver's values for the shown stack, read-only.
fn values_row(ui: &mut Ui, values: Option<&Result<StackValues, String>>, applied: bool) {
    match values {
        None => {}
        Some(Ok(v)) => {
            ui.horizontal_wrapped(|ui| {
                ui.label(format!("{GAUGE}: {:.3} mm", v.gauge_len));
                if let Some(l) = v.lathe {
                    ui.label(format!("{INSCRIBED}: {:.3} mm", v.diameter));
                    ui.label(format!("{NOSE}: {:.3} mm", l.nose_radius));
                    ui.label(format!("{ORIENTATION}: {}", l.orientation.map_or_else(|| "-".to_owned(), |q| q.to_string())));
                } else {
                    ui.label(format!("{DIAMETER}: {:.3} mm", v.diameter));
                    ui.label(format!("{CORNER}: {:.3} mm", v.corner_r));
                }
                if !applied {
                    ui.weak("(from the stack edits; not applied)");
                }
            });
        }
        Some(Err(e)) => {
            ui.weak(format!("The stack does not resolve: {e}"));
        }
    }
}
