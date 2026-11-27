//! #3898 (U7 of #3663): the "My library" source of the tool library
//! window - the component library (the built-in components and the
//! user's own file) by group, a name filter and the "compatible parts"
//! filter of a picked component (`component_fit::compatible_parents` and
//! `compatible_children`), then the document's component table with the
//! library status of each row. A click on a name picks the row; the
//! buttons act on the picked row. Each document change is one undoable
//! command (`state_components`); the library file writes have no undo.
//! Frame cost: State keeps the compatible set, the library status and the
//! users of each row (`state_component_views`), keyed on the component
//! table, the stacks, the library read and the filter; a still frame
//! builds nothing.
//! Provenance: our own list over our own component records; no HSMWorks
//! material involved.

use egui::{RichText, Ui};

use camcraft_app::component::{ComponentKind, CuttingItem, ToolComponent};
use camcraft_app::component_library::SyncStatus;

use crate::state::{ComponentAction, ComponentDerived as Derived, ComponentGroup, ErrorScope, LoadedLibrary, State, group_of};

/// The source's label in the list's source switch.
pub(crate) const SOURCE: &str = "My library";
/// The row that shows every group.
pub(crate) const ALL: &str = "All components";
/// The buttons of a picked library row.
pub(crate) const ADD: &str = "Add to document";
pub(crate) const COPY: &str = "Copy as user component";
pub(crate) const SHOW_COMPATIBLE: &str = "Show compatible parts";
pub(crate) const SHOW_ALL: &str = "Show all parts";
pub(crate) const DELETE_LIB: &str = "Delete from My library";
/// The buttons of a picked document row.
pub(crate) const EDIT: &str = "Edit component…";
pub(crate) const SYNC: &str = "Sync from library";
pub(crate) const SAVE: &str = "Save to My library";
pub(crate) const DELETE_DOC: &str = "Delete from document";
/// The heading of the document's table.
pub(crate) const DOCUMENT: &str = "DOCUMENT COMPONENTS";
/// The column heads of the library rows.
pub(crate) const HEADS: [&str; 5] = ["Name", "Kind", "Machine side", "Tool side", "Source"];
const GROUP_W: f32 = 170.0;

/// The groups in the order the column shows them.
pub(crate) const GROUPS: [(ComponentGroup, &str); 5] = [
    (ComponentGroup::Holders, "Holders"),
    (ComponentGroup::Adapters, "Adapters and extensions"),
    (ComponentGroup::Bodies, "Cutter bodies"),
    (ComponentGroup::Inserts, "Inserts"),
    (ComponentGroup::SolidTools, "Solid tools"),
];

/// The kind of a component in words.
#[must_use]
pub(crate) fn kind_text(c: &ToolComponent) -> &'static str {
    match group_of(c) {
        ComponentGroup::Holders => "Holder",
        ComponentGroup::Adapters => "Adapter",
        ComponentGroup::Bodies => "Cutter body",
        ComponentGroup::Inserts => "Insert",
        ComponentGroup::SolidTools => "Solid tool",
    }
}

/// The machine side and the tool side of a component in words.
#[must_use]
pub(crate) fn sides(c: &ToolComponent) -> (String, String) {
    match c.kind() {
        ComponentKind::CuttingItem(CuttingItem::Insert(spec)) => ("-".to_owned(), spec.to_string()),
        ComponentKind::CuttingItem(CuttingItem::Solid { tool, mcs }) => (mcs.to_string(), format!("Ø {}", tool.diameter)),
        ComponentKind::ToolItem { mcs, seats } => {
            let ports: Vec<&str> = seats.iter().map(|s| s.port.as_str()).collect();
            (mcs.to_string(), format!("seats {}", ports.join(", ")))
        }
        ComponentKind::AdaptiveItem { mcs, csw } => (mcs.to_string(), csw.to_string()),
    }
}

/// The library status of a document row in words.
#[must_use]
pub(crate) fn status_text(s: &SyncStatus) -> &'static str {
    match s {
        SyncStatus::NotLinked => "not in a library",
        SyncStatus::UpToDate => "up to date",
        SyncStatus::UpdateAvailable { .. } => "differs from the library",
        SyncStatus::MissingFromLibrary => "missing from the library",
    }
}

pub(super) fn draw(ui: &mut Ui, state: &mut State) {
    let lib = state.component_library();
    let Some(view) = state.tool_library.as_ref().map(|v| v.components.clone()) else {
        return;
    };
    let d = state.component_derived(view.compatible.as_deref());
    let mut next = view.clone();
    let mut asks: Vec<ComponentAction> = Vec::new();
    let mut edit = None;
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.set_width(GROUP_W);
            group_column(ui, &lib, &mut next.group);
        });
        ui.separator();
        ui.vertical(|ui| {
            let label = ui.label("Component filter");
            ui.add(egui::TextEdit::singleline(&mut next.filter).desired_width(ui.available_width())).labelled_by(label.id);
            match (&lib.error, &lib.path) {
                (Some(e), _) => ui.weak(e.as_str()),
                (None, Some(p)) => ui.weak(format!("My library: {}", p.display())),
                (None, None) => ui.weak(""),
            };
            if let Some(msg) = state.error_for(ErrorScope::Tool).map(str::to_owned) {
                super::super::error_text::error_text(ui, super::super::colorvision::text_inks(ui, &state.world).error, &msg);
            }
            library_actions(ui, &lib, &d, &mut next, &mut asks);
            egui::ScrollArea::vertical().id_salt("tool_library_components").max_height(260.0).auto_shrink([false, true]).show(ui, |ui| {
                library_rows(ui, &lib, &d, &mut next);
            });
            ui.add_space(6.0);
            ui.label(RichText::new(DOCUMENT).small().weak());
            edit = document_actions(ui, &d, &next, &mut asks);
            egui::ScrollArea::vertical().id_salt("tool_library_document_components").auto_shrink([false, false]).show(ui, |ui| {
                document_rows(ui, &d, &mut next);
            });
        });
    });
    if let Some(v) = state.tool_library.as_mut() {
        v.components = next;
    }
    for a in asks {
        state.queue_component(a);
    }
    if let Some(id) = edit {
        super::component_edit::open(state, &id);
    }
}

fn group_column(ui: &mut Ui, lib: &LoadedLibrary, group: &mut Option<ComponentGroup>) {
    ui.label(RichText::new("COMPONENTS").small().weak());
    let count = |g: Option<ComponentGroup>| lib.all.iter().filter(|c| g.is_none_or(|g| group_of(&c.component) == g)).count();
    let mut item = |ui: &mut Ui, g: Option<ComponentGroup>, text: &str| {
        ui.horizontal(|ui| {
            if ui.selectable_label(*group == g, text).clicked() {
                *group = g;
            }
            ui.weak(count(g).to_string());
        });
    };
    item(ui, None, ALL);
    for (g, text) in GROUPS {
        item(ui, Some(g), text);
    }
}

/// The buttons for the picked library row, and the compatible filter's note.
fn library_actions(
    ui: &mut Ui,
    lib: &LoadedLibrary,
    d: &Derived,
    view: &mut crate::state::ComponentsView,
    asks: &mut Vec<ComponentAction>,
) {
    let picked = view.selected.as_deref().and_then(|id| lib.all.iter().find(|c| c.component.id() == id));
    ui.horizontal_wrapped(|ui| {
        let in_doc = picked.is_some_and(|c| d.rows.iter().any(|(r, _, _)| r.component.id() == c.component.id()));
        if ui
            .add_enabled(picked.is_some() && !in_doc, egui::Button::new(ADD))
            .on_hover_text("put the picked component into this document, linked to its library copy (one undo step)")
            .on_disabled_hover_text("pick a component that the document does not hold")
            .clicked()
            && let Some(c) = picked
        {
            asks.push(ComponentAction::AddFromLibrary { id: c.component.id().to_owned() });
        }
        if ui
            .add_enabled(picked.is_some(), egui::Button::new(COPY))
            .on_hover_text("a new user component in this document, made from the picked one, that you can edit")
            .clicked()
            && let Some(c) = picked
        {
            asks.push(ComponentAction::CopyAsUser { id: c.component.id().to_owned() });
        }
        if ui
            .add_enabled(picked.is_some(), egui::Button::new(SHOW_COMPATIBLE))
            .on_hover_text("show only the parts that fit the picked component: the holders that take it, and what it takes")
            .clicked()
        {
            view.compatible = view.selected.clone();
        }
        let user = picked.is_some_and(|c| !c.is_builtin());
        if ui
            .add_enabled(user, egui::Button::new(DELETE_LIB))
            .on_hover_text("delete the picked component from your library file (no undo: the file is not the document)")
            .on_disabled_hover_text("built-in components are read-only")
            .clicked()
            && let Some(c) = picked
        {
            asks.push(ComponentAction::DeleteFromLibrary { id: c.component.id().to_owned() });
        }
    });
    if let Some(id) = view.compatible.clone() {
        let name = lib.all.iter().find(|c| c.component.id() == id).map_or(id.as_str(), |c| c.component.name());
        ui.horizontal(|ui| {
            ui.label(format!("Compatible with {name}"));
            if ui.button(SHOW_ALL).clicked() {
                view.compatible = None;
            }
        });
    }
}

fn library_rows(ui: &mut Ui, lib: &LoadedLibrary, d: &Derived, view: &mut crate::state::ComponentsView) {
    let f = view.filter.to_lowercase();
    let shown: Vec<_> = lib
        .all
        .iter()
        .filter(|c| view.group.is_none_or(|g| group_of(&c.component) == g))
        .filter(|c| c.component.name().to_lowercase().contains(&f))
        .filter(|c| d.compatible.as_ref().is_none_or(|ids| ids.iter().any(|id| id == c.component.id())))
        .collect();
    if shown.is_empty() {
        ui.label("No component matches the filters.");
        return;
    }
    egui::Grid::new("tool_library_component_rows").striped(true).show(ui, |ui| {
        for h in HEADS {
            ui.label(RichText::new(h).strong());
        }
        ui.end_row();
        for c in shown {
            let id = c.component.id();
            let sel = view.selected.as_deref() == Some(id);
            if ui.selectable_label(sel, c.component.name()).on_hover_text(id).clicked() {
                view.selected = Some(id.to_owned());
            }
            let (machine, tool) = sides(&c.component);
            ui.label(kind_text(&c.component));
            ui.label(machine);
            ui.label(tool);
            ui.label(if c.is_builtin() { "Built-in" } else { SOURCE });
            ui.end_row();
        }
    });
}

/// The buttons for the picked document row; gives the row to edit.
fn document_actions(
    ui: &mut Ui,
    d: &Derived,
    view: &crate::state::ComponentsView,
    asks: &mut Vec<ComponentAction>,
) -> Option<String> {
    let picked = view.doc_selected.as_deref().and_then(|id| d.rows.iter().find(|(r, _, _)| r.component.id() == id));
    let id = picked.map(|(r, _, _)| r.component.id().to_owned());
    let mut edit = None;
    ui.horizontal_wrapped(|ui| {
        if ui.add_enabled(picked.is_some(), egui::Button::new(EDIT)).on_hover_text("edit the picked component; each tool built from it follows (one undo step)").clicked() {
            edit.clone_from(&id);
        }
        let differs = picked.is_some_and(|(_, s, _)| matches!(s, SyncStatus::UpdateAvailable { .. }));
        if ui
            .add_enabled(differs, egui::Button::new(SYNC))
            .on_hover_text("take the library copy into the document; each tool built from it follows (one undo step)")
            .on_disabled_hover_text("pick a component that differs from its library copy")
            .clicked()
            && let Some(id) = &id
        {
            asks.push(ComponentAction::Sync { id: id.clone() });
        }
        if ui
            .add_enabled(picked.is_some(), egui::Button::new(SAVE))
            .on_hover_text("write the picked component to your library file and link it (the file write has no undo)")
            .clicked()
            && let Some(id) = &id
        {
            asks.push(ComponentAction::SaveToLibrary { id: id.clone() });
        }
        let unused = picked.is_some_and(|(_, _, users)| users.is_empty());
        if ui
            .add_enabled(unused, egui::Button::new(DELETE_DOC))
            .on_hover_text("delete the picked component from this document (one undo step)")
            .on_disabled_hover_text("pick a component that no tool stack uses")
            .clicked()
            && let Some(id) = &id
        {
            asks.push(ComponentAction::DeleteFromDocument { id: id.clone() });
        }
    });
    edit
}

fn document_rows(ui: &mut Ui, d: &Derived, view: &mut crate::state::ComponentsView) {
    if d.rows.is_empty() {
        ui.weak("The document holds no tool component. Add one from the list above.");
        return;
    }
    egui::Grid::new("tool_library_document_component_rows").striped(true).show(ui, |ui| {
        for h in ["Name", "Kind", "Library", "Used by"] {
            ui.label(RichText::new(h).strong());
        }
        ui.end_row();
        for (r, status, users) in d.rows.iter() {
            let id = r.component.id();
            let sel = view.doc_selected.as_deref() == Some(id);
            if ui.selectable_label(sel, r.component.name()).on_hover_text(id).clicked() {
                view.doc_selected = Some(id.to_owned());
            }
            ui.label(kind_text(&r.component));
            ui.label(status_text(status));
            ui.label(if users.is_empty() { "-".to_owned() } else { users.join(", ") });
            ui.end_row();
        }
    });
}
