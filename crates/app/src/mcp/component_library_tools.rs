//! The MCP user component library tools (#3647): save a tool component
//! to the user's component library, delete one from it, and the shared
//! readers the component and assembly tools use (the library file, the
//! sync status, a row's JSON). A save and a delete are file edits, not
//! document commands (`component_library::store`): no undo step, and a
//! `builtin:` id is refused by name.
//!
//! Provenance: our own tool surface over this repo's component library
//! store (#3645).

use std::path::PathBuf;

use serde_json::{Map, Value, json};

use super::setup_tools::obj;
use super::tools::{Tool, opt_str, req_str};
use crate::component_library::{self, LibraryComponent, SyncStatus, store};
use crate::project::{ComponentDto, component_from_dto, component_to_dto};
use crate::schema::DocumentComponent;
use crate::script::Env;
use crate::tool_assembly as ta;
use camcraft_tools::component::{ComponentKind, CuttingItem, ToolComponent};

pub(super) const TOOLS: &[Tool] = &[
    Tool {
        name: "save_library_component",
        description: "Save a tool component to the user's component library: a document component by id, or a full component (the component object of a list_tool_components row). An entry with the same id is replaced. A file edit, not a document change: no undo step. Refused by name: both or neither of id and component, an id the document does not hold, a builtin: id (built-in components are read only), a component that states library_ref, and every component check.",
        schema: s_save,
        run: save_library_component,
    },
    Tool {
        name: "delete_library_component",
        description: "Delete a tool component from the user's component library. A file edit, not a document change: no undo step. Document components linked to it then show the sync status missing_from_library. Refused by name: a builtin: id (built-in components are read only) and an id the library does not hold.",
        schema: s_delete,
        run: delete_library_component,
    },
];

/// The schema row of the optional `library` argument.
pub(super) fn library_prop() -> Value {
    json!({ "type": "string", "description": "A component library file (.camcomponents). Default: the user's component library (CAMCRAFT_COMPONENTS, else the platform data folder)." })
}

fn s_save() -> Value {
    obj(
        json!({
            "id": { "type": "string", "description": "A document component's id: save that component." },
            "component": { "type": "object", "description": "A full component: {id, name, item, geometry?} as list_tool_components shows it, without library_ref." },
            "library": library_prop(),
        }),
        &[],
    )
}

fn s_delete() -> Value {
    obj(
        json!({
            "id": { "type": "string", "description": "The library component's id." },
            "library": library_prop(),
        }),
        &["id"],
    )
}

/// The library file: `library` when given, else the user's own.
pub(super) fn library_path(a: &Map<String, Value>) -> Result<PathBuf, String> {
    match opt_str(a, "library")? {
        Some(p) if !p.trim().is_empty() => Ok(PathBuf::from(p)),
        _ => store::user_dir()
            .map(|d| store::library_path(&d))
            .ok_or_else(|| format!("no user folder for the component library - set {}", store::ENV_VAR)),
    }
}

/// The user's components (not the built-ins). A refused file is an
/// error, never "empty".
pub(super) fn user_library(a: &Map<String, Value>) -> Result<Vec<LibraryComponent>, String> {
    store::load(&library_path(a)?)
}

/// The library a document syncs from: the built-ins, then the user's.
pub(super) fn full_library(a: &Map<String, Value>) -> Result<Vec<LibraryComponent>, String> {
    user_library(a).map(component_library::with_builtins)
}

/// The kind word of a component, as `list_tool_components` filters it.
pub(super) fn kind_word(c: &ToolComponent) -> &'static str {
    match c.kind() {
        ComponentKind::CuttingItem(CuttingItem::Insert(_)) => "insert",
        ComponentKind::CuttingItem(CuttingItem::Solid { .. }) => "solid",
        ComponentKind::ToolItem { .. } => "tool_item",
        ComponentKind::AdaptiveItem { .. } => "adaptive",
    }
}

/// The sync status of a document row. A row with no link needs no
/// library; an unreadable library is said, never read as "missing".
pub(super) fn status_json(row: &DocumentComponent, library: &Result<Vec<LibraryComponent>, String>) -> Value {
    let Some(link) = &row.library_ref else {
        return json!({ "status": "not_linked" });
    };
    let lib = match library {
        Ok(l) => l,
        Err(e) => return json!({ "status": "unreadable", "library_id": link.id, "reason": e }),
    };
    match component_library::sync_status(row, lib) {
        SyncStatus::NotLinked => json!({ "status": "not_linked" }),
        SyncStatus::UpToDate => json!({ "status": "up_to_date", "library_id": link.id }),
        SyncStatus::UpdateAvailable { library_hash } => {
            json!({ "status": "update_available", "library_id": link.id, "library_hash": library_hash })
        }
        SyncStatus::MissingFromLibrary => json!({ "status": "missing_from_library", "library_id": link.id }),
    }
}

/// One listed row: where it lives, its kind and ports, and the wire
/// form under `component` (the form `upsert_tool_component` takes).
pub(super) fn row_json(scope: &str, dto: &ComponentDto, c: &ToolComponent) -> Value {
    json!({ "scope": scope, "kind": kind_word(c), "ports": c.ports(), "component": dto })
}

/// The wire form of a library entry.
pub(super) fn library_dto(e: &LibraryComponent) -> ComponentDto {
    component_to_dto(&DocumentComponent::new(e.component.clone(), e.geometry.clone()))
}

/// The checked component under `key`. A stated `library_ref` is
/// refused: only `sync_tool_component` writes a library link.
pub(super) fn parse_component(a: &Map<String, Value>, key: &str) -> Result<DocumentComponent, String> {
    let v = match a.get(key) {
        Some(v @ Value::Object(_)) => v.clone(),
        None | Some(Value::Null) => return Err(format!("\"{key}\" is required")),
        Some(other) => return Err(format!("\"{key}\" must be an object, got {other}")),
    };
    let dto: ComponentDto = serde_json::from_value(v).map_err(|e| format!("bad \"{key}\": {e}"))?;
    if dto.library_ref.is_some() {
        return Err(format!(
            "\"{key}\" states library_ref - only sync_tool_component writes a library link"
        ));
    }
    component_from_dto(&dto)
}

fn save_library_component(env: &mut Env, a: &Map<String, Value>) -> Result<Value, String> {
    let row = match (opt_str(a, "id")?, a.get("component").filter(|v| !v.is_null())) {
        (Some(_), Some(_)) => return Err("give id or component, not both".to_owned()),
        (None, None) => return Err("give id (a document component) or component (a full component)".to_owned()),
        (Some(id), None) => ta::component(&env.world, id)
            .ok_or_else(|| format!("the document holds no tool component '{}'", id.trim()))?,
        (None, Some(_)) => parse_component(a, "component")?,
    };
    let id = row.component.id().to_owned();
    if component_library::is_builtin_id(&id) {
        return Err(format!("tool component '{id}' is built in - it is not saved to the user library"));
    }
    let entry = LibraryComponent::from_row(row)?;
    let content_hash = entry.content_hash();
    let path = library_path(a)?;
    store::put(&path, entry)?;
    Ok(json!({ "saved": id, "content_hash": content_hash, "library": path.display().to_string() }))
}

fn delete_library_component(_env: &mut Env, a: &Map<String, Value>) -> Result<Value, String> {
    let id = req_str(a, "id")?.trim();
    let path = library_path(a)?;
    store::delete(&path, id)?;
    Ok(json!({ "deleted": id, "library": path.display().to_string() }))
}
