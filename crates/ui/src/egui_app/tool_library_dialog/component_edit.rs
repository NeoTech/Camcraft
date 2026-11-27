//! #3898 (U7 of #3663): the component editor of the tool library window -
//! one document component (a user holder, an insert, a cutter body) with
//! its name, an insert's ISO 1832 code and a holder's gauge length, its
//! library status and the tools built from it. The values are the
//! window's own draft (`ComponentDraft`); only "Apply component" lands
//! them, as one undoable `EditDocumentComponent` that derives each of
//! those tools again in the same undo step. A holder's gauge length is
//! offered when its built-in shape is known (`holder_entry`): the outline
//! is the holder library's at the new length (`ComponentGeometry::holder`).
//! Provenance: our own editor over our own component commands; no
//! HSMWorks material involved.

use egui::{RichText, Ui};

use camcraft_app::component::{ADAPTIVE_PORT, ComponentKind, CuttingItem, ToolComponent};
use camcraft_app::component_geometry::ComponentGeometry;
use camcraft_app::component_library::SyncStatus;
use camcraft_app::insert_iso1832::InsertSpec;
use camcraft_app::schema::DocumentComponent;
use camcraft_app::tool_assembly as ta;

use crate::state::{ComponentAction, ComponentDraft, ErrorScope, State, draft_of, gauge_of, holder_entry};

/// The button back to the page the editor was opened from.
pub(crate) const BACK: &str = "Back to the list";
pub(crate) const APPLY: &str = "Apply component";
pub(crate) const NAME: &str = "Component name";
pub(crate) const CODE: &str = "Insert code";
pub(crate) const GAUGE: &str = "Holder gauge length (mm)";

/// The component and geometry the draft describes, or why not.
pub(crate) fn build(d: &ComponentDraft, row: &DocumentComponent) -> Result<(ToolComponent, ComponentGeometry), String> {
    let kind = match row.component.kind() {
        ComponentKind::CuttingItem(CuttingItem::Insert(_)) => {
            let spec = InsertSpec::parse(d.code.trim()).map_err(|e| format!("insert code '{}': {e}", d.code.trim()))?;
            ComponentKind::CuttingItem(CuttingItem::Insert(spec))
        }
        k => k.clone(),
    };
    let component = ToolComponent::new(&d.id, d.name.trim(), kind).map_err(|e| e.to_string())?;
    let geometry = match (d.gauge, holder_entry(row)) {
        (Some(g), Some(entry)) if Some(g) != gauge_of(row) => {
            ComponentGeometry::holder(&entry, ADAPTIVE_PORT, g).map_err(|e| format!("gauge length {g} mm: {e}"))?
        }
        _ => row.geometry.clone(),
    };
    Ok((component, geometry))
}

/// Open document component `id` in the editor, on the page shown now.
pub(crate) fn open(state: &mut State, id: &str) {
    let Some(row) = ta::component(&state.world, id) else {
        return;
    };
    if let Some(v) = state.tool_library.as_mut() {
        v.component_edit = Some(draft_of(&row));
    }
}

fn close(state: &mut State) {
    if let Some(v) = state.tool_library.as_mut() {
        v.component_edit = None;
    }
}

pub(super) fn draw(ui: &mut Ui, state: &mut State) {
    let Some(mut d) = state.tool_library.as_ref().and_then(|v| v.component_edit.clone()) else {
        return;
    };
    // The row comes from State's cache: a still frame reads no table.
    let derived = state.component_derived(None);
    let Some((row, status, users)) = derived.rows.iter().find(|(r, _, _)| r.component.id() == d.id).cloned() else {
        // Deleted (or undone) under the editor: back to the list.
        close(state);
        return;
    };
    let mut back = false;
    let mut ask = None;
    ui.horizontal(|ui| {
        back = ui.button(BACK).on_hover_text("close the editor; an unapplied draft is dropped").clicked();
        ui.heading(row.component.name());
        ui.weak(format!("{} · {}", super::components::kind_text(&row.component), d.id));
    });
    ui.separator();
    if let Some(msg) = state.error_for(ErrorScope::Tool).map(str::to_owned) {
        super::super::error_text::error_text(ui, super::super::colorvision::text_inks(ui, &state.world).error, &msg);
    }
    egui::Grid::new("tool_library_component_edit").num_columns(2).show(ui, |ui| {
        let l = ui.label(NAME);
        ui.add(egui::TextEdit::singleline(&mut d.name).desired_width(260.0)).labelled_by(l.id);
        ui.end_row();
        if matches!(row.component.kind(), ComponentKind::CuttingItem(CuttingItem::Insert(_))) {
            let l = ui.label(CODE);
            ui.add(egui::TextEdit::singleline(&mut d.code).desired_width(160.0)).labelled_by(l.id)
                .on_hover_text("the ISO 1832 designation, for example CCMT09T304");
            ui.end_row();
        }
        if let Some(g) = d.gauge.as_mut() {
            let l = ui.label(GAUGE);
            ui.add(egui::DragValue::new(g).speed(0.5).range(1.0..=1000.0)).labelled_by(l.id)
                .on_hover_text("from the gauge line to the nose face; the outline is the built-in holder's at this length");
            ui.end_row();
        }
        let (machine, tool) = super::components::sides(&row.component);
        ui.label("Machine side");
        ui.label(machine);
        ui.end_row();
        ui.label("Tool side");
        ui.label(tool);
        ui.end_row();
        ui.label("Library");
        ui.label(super::components::status_text(&status));
        ui.end_row();
    });
    ui.add_space(4.0);
    ui.label(RichText::new("USED BY").small().weak());
    if users.is_empty() {
        ui.weak("No tool stack uses this component.");
    } else {
        ui.label(format!(
            "Shared by {} tool(s): {}. Apply component changes each of them in the same undo step.",
            users.len(),
            users.join(", ")
        ));
    }
    ui.separator();
    let changed = d != draft_of(&row);
    ui.horizontal_wrapped(|ui| {
        if ui
            .add_enabled(changed, egui::Button::new(APPLY))
            .on_hover_text("write this draft to the component and each tool built from it (one undo step)")
            .clicked()
        {
            ask = Some(build(&d, &row).map(|edit| ComponentAction::Edit(Box::new(edit))));
        }
        let differs = matches!(status, SyncStatus::UpdateAvailable { .. });
        if ui
            .add_enabled(differs, egui::Button::new(super::components::SYNC))
            .on_hover_text("take the library copy into the document; each tool built from it follows (one undo step)")
            .on_disabled_hover_text("the component does not differ from a library copy")
            .clicked()
        {
            ask = Some(Ok(ComponentAction::Sync { id: d.id.clone() }));
        }
        if ui
            .add(egui::Button::new(super::components::SAVE))
            .on_hover_text("write this component to your library file and link it (the file write has no undo)")
            .clicked()
        {
            ask = Some(Ok(ComponentAction::SaveToLibrary { id: d.id.clone() }));
        }
    });
    if back {
        close(state);
        return;
    }
    if let Some(v) = state.tool_library.as_mut() {
        v.component_edit = Some(d);
    }
    match ask {
        Some(Ok(a)) => state.queue_component(a),
        Some(Err(e)) => state.refuse(ErrorScope::Tool, format!("component not changed: {e}")),
        None => {}
    }
}
