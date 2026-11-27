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
//! tool as it is. A lathe tool's stack waits for the lathe side of the
//! resolver (#3643). Frame cost: State keeps the offer and the rows
//! (`state_component_views`), keyed on the component table, the stacks,
//! the library read and the draft; a still frame builds nothing.
//! Provenance: our own group over our own component commands; no
//! HSMWorks material involved.

use egui::{RichText, Ui};

use camcraft_app::model;
use camcraft_app::tool_assembly as ta;

#[cfg(test)]
pub(crate) use crate::state::candidates;
use crate::state::{ComponentAction, StackDraft, State, is_spun_kind};

pub(crate) const HEADING: &str = "Component stack";
pub(crate) const ADD: &str = "Add component";
pub(crate) const REMOVE: &str = "Remove last component";
pub(crate) const APPLY: &str = "Apply stack";
pub(crate) const DETACH: &str = "Detach to standalone tool";
const HEADS: [&str; 6] = ["#", "Component", "Port", "Turn (°)", "Stick-out (mm)", "Library"];

pub(super) fn draw(ui: &mut Ui, state: &mut State) {
    ui.add_space(6.0);
    ui.separator();
    ui.strong(HEADING);
    let name = state.tool_ui.name.clone();
    let Some(entity) = model::find_by_name(&state.world, &name) else {
        return;
    };
    if !is_spun_kind(state.tool_ui.tool_kind()) {
        ui.label("A lathe tool's component stack is not resolved yet (#3643).");
        return;
    }
    let stored = ta::assembly_of(&state.world, &entity).map(|a| a.0);
    let doc = state.doc_version;
    let mut draft = match state.tool_library.as_ref().and_then(|v| v.stack.clone()) {
        Some(d) if d.tool == name && d.synced == doc => d,
        _ => StackDraft { tool: name.clone(), synced: doc, links: stored.clone().unwrap_or_default() },
    };
    let derived = state.component_derived(None);
    let view = state.component_stack_view(&draft.links);
    let mut edit = None;
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
                if let Some((r, _, users)) = row
                    && ui
                        .small_button(format!("Edit {}", r.component.name()))
                        .on_hover_text(format!("edit this shared component: {} tool(s) use it", users.len()))
                        .clicked()
                {
                    edit = Some(link.component.clone());
                }
                ui.end_row();
            }
        });
    }
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
    let mut ask = None;
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
