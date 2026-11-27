//! The MCP document tool component tools (#3647): list the built-in,
//! user-library and document components (with a compatibility query),
//! and add, edit, delete and sync a document component. Each write runs
//! the very undoable command the GUI queues (`UpsertDocumentComponent`,
//! `DeleteDocumentComponent`, `SyncDocumentComponent`): one History
//! step, and every tool built from an edited component is derived again
//! in that step.
//!
//! Provenance: our own tool surface over this repo's component commands
//! (#3644), component library (#3645) and ISO 13399 fit checks (#3643).

use serde_json::{Map, Value, json};

use super::component_library_tools::{
    full_library, kind_word, library_dto, library_path, library_prop, parse_component, row_json, status_json,
    user_library,
};
use super::setup_tools::{cmd_err, obj};
use super::tools::{Tool, opt_str, req_str};
use crate::commands::{DeleteDocumentComponent, SyncDocumentComponent, UpsertDocumentComponent};
use crate::component_library;
use crate::project::component_to_dto;
use crate::schema::DocumentComponent;
use crate::script::Env;
use crate::tool_assembly as ta;
use camcraft_tools::component::ToolComponent;
use camcraft_tools::component_fit::{compatible_children, compatible_parents};

pub(super) const TOOLS: &[Tool] = &[
    Tool {
        name: "list_tool_components",
        description: "List tool components: the built-in ones (read only, builtin: ids), the user's component library and the document's component table. Each row has scope (builtin, user or document), kind (insert, solid, tool_item or adaptive), ports and the component itself (the form upsert_tool_component takes); a document row also has its library sync status. kind and scope filter the rows. compatible_with names a component (looked up in the document, then the user library, then the built-ins) and keeps only the rows that fit with it, each with fits: [{as: parent | child, port}] (as parent: the named component mounts on this row's port; as child: this row mounts on the named component's port). Refused by name: an unknown kind or scope, an unknown compatible_with id, and an unreadable library file.",
        schema: s_list,
        run: list_tool_components,
    },
    Tool {
        name: "upsert_tool_component",
        description: "Add a tool component to the document's component table, or replace the component of the same id. A replace derives every tool whose assembly uses the component again (diameter, corner radius, holder body) in the same undo step, and keeps the row's library link. One undo step. Refused by name: a malformed component, an unknown key, a component that states library_ref (use sync_tool_component), an edit that changes nothing, and an edit that a tool built from it cannot take (the tool is named; nothing changes).",
        schema: s_upsert,
        run: upsert_tool_component,
    },
    Tool {
        name: "delete_tool_component",
        description: "Delete a component from the document's component table. One undo step. Refused by name: an id the document does not hold, and a component that a tool assembly still uses (the tools are named).",
        schema: s_id,
        run: delete_tool_component,
    },
    Tool {
        name: "sync_tool_component",
        description: "Replace a document component with its library copy (the copy it links to, or the library component of the same id) and record the link. Every tool built from it is derived again in the same undo step. One undo step. Refused by name: an id the document does not hold, a copy the library does not hold, a component already up to date, and a copy that a tool built from it cannot take.",
        schema: s_sync,
        run: sync_tool_component,
    },
];

/// Every component, assembly and component library row (#3647).
pub(super) fn all() -> impl Iterator<Item = &'static Tool> {
    TOOLS.iter().chain(super::assembly_tools::TOOLS).chain(super::component_library_tools::TOOLS)
}

const KINDS: &[&str] = &["insert", "solid", "tool_item", "adaptive"];
const SCOPES: &[&str] = &["all", "builtin", "user", "document"];

fn s_list() -> Value {
    obj(
        json!({
            "kind": { "type": "string", "enum": KINDS, "description": "Only components of this kind." },
            "scope": { "type": "string", "enum": SCOPES, "description": "Only components from here (default all)." },
            "compatible_with": { "type": "string", "description": "A component id: only the components that fit with it." },
            "library": library_prop(),
        }),
        &[],
    )
}

fn s_upsert() -> Value {
    obj(
        json!({ "component": { "type": "object", "description": "The component: {id, name, item, geometry?} (the component object of a list_tool_components row)." } }),
        &["component"],
    )
}

fn s_id() -> Value {
    obj(json!({ "id": { "type": "string", "description": "The document component's id." } }), &["id"])
}

fn s_sync() -> Value {
    obj(
        json!({
            "id": { "type": "string", "description": "The document component's id." },
            "library": library_prop(),
        }),
        &["id"],
    )
}

/// One candidate row: its scope, the component, and the listed JSON
/// without the fit (added when `compatible_with` is given).
struct Row {
    scope: &'static str,
    component: ToolComponent,
    json: Value,
}

fn word<'a>(a: &'a Map<String, Value>, key: &str, words: &[&str]) -> Result<Option<&'a str>, String> {
    match opt_str(a, key)? {
        None => Ok(None),
        Some(w) if words.contains(&w) => Ok(Some(w)),
        Some(w) => Err(format!("unknown {key} '{w}' - one of {}", words.join(", "))),
    }
}

fn list_tool_components(env: &mut Env, a: &Map<String, Value>) -> Result<Value, String> {
    let kind = word(a, "kind", KINDS)?;
    let scope = word(a, "scope", SCOPES)?.unwrap_or("all");
    let wants = |s: &str| scope == "all" || scope == s;
    let target_id = opt_str(a, "compatible_with")?.map(str::trim);
    let table = ta::components(&env.world);
    // The user library is read when its rows are listed, or when a
    // compatibility query names a component the document does not hold.
    let in_doc = target_id.is_some_and(|id| table.iter().any(|r| r.component.id() == id));
    let user = if wants("user") || (target_id.is_some() && !in_doc) { Some(user_library(a)?) } else { None };
    let sync_lib = match &user {
        Some(u) => Ok(component_library::with_builtins(u.clone())),
        None if table.iter().any(|r| r.library_ref.is_some()) => full_library(a),
        None => Ok(Vec::new()),
    };

    let mut rows = Vec::new();
    for r in table {
        let mut json = row_json("document", &component_to_dto(&r), &r.component);
        json["sync_status"] = status_json(&r, &sync_lib);
        rows.push(Row { scope: "document", component: r.component, json });
    }
    for e in user.iter().flatten() {
        rows.push(Row { scope: "user", json: row_json("user", &library_dto(e), &e.component), component: e.component.clone() });
    }
    for e in component_library::builtins() {
        rows.push(Row { scope: "builtin", json: row_json("builtin", &library_dto(&e), &e.component), component: e.component });
    }

    let target = match target_id {
        None => None,
        Some(id) => Some(
            rows.iter()
                .find(|r| r.component.id() == id)
                .map(|r| (r.scope, r.component.clone()))
                .ok_or_else(|| format!("no tool component '{id}' in the document or the component library"))?,
        ),
    };

    let mut out = Vec::new();
    for mut r in rows {
        if !wants(r.scope) || kind.is_some_and(|k| kind_word(&r.component) != k) {
            continue;
        }
        if let Some((_, t)) = &target {
            let mut fits: Vec<Value> = compatible_parents(t, [&r.component])
                .into_iter()
                .map(|f| json!({ "as": "parent", "port": f.port }))
                .collect();
            fits.extend(compatible_children(t, [&r.component]).into_iter().map(|f| json!({ "as": "child", "port": f.port })));
            if fits.is_empty() {
                continue;
            }
            r.json["fits"] = Value::Array(fits);
        }
        out.push(r.json);
    }
    let library = library_path(a).ok().map(|p| p.display().to_string());
    Ok(json!({
        "count": out.len(),
        "components": out,
        "compatible_with": target.map(|(s, c)| json!({ "id": c.id(), "scope": s })),
        "library": library,
    }))
}

/// The document row `id` as a listed row, with its sync status.
fn document_row(env: &Env, a: &Map<String, Value>, id: &str) -> Value {
    match ta::component(&env.world, id) {
        Some(r) => {
            let mut json = row_json("document", &component_to_dto(&r), &r.component);
            let library = if r.library_ref.is_some() { full_library(a) } else { Ok(Vec::new()) };
            json["sync_status"] = status_json(&r, &library);
            json
        }
        None => Value::Null,
    }
}

fn upsert_tool_component(env: &mut Env, a: &Map<String, Value>) -> Result<Value, String> {
    let row = parse_component(a, "component")?;
    let id = row.component.id().to_owned();
    let held = ta::component(&env.world, &id);
    let added = held.is_none();
    // #3646 / #3643: the stated cutter facts, running data and lathe body
    // land with the row; a key the call does not state keeps the held
    // row's value, and the library link stays (as a catalog-fact edit
    // keeps them).
    let row = match held {
        None => row,
        Some(h) => DocumentComponent {
            library_ref: h.library_ref,
            milling: row.milling.or(h.milling),
            running: row.running.or(h.running),
            lathe_body: row.lathe_body.or(h.lathe_body),
            ..row
        },
    };
    let cmd = UpsertDocumentComponent::with_row(&env.world, row).map_err(cmd_err)?;
    let updated = cmd.updated_tools();
    env.history.execute(&env.world, cmd);
    Ok(json!({ "component": document_row(env, a, &id), "added": added, "updated_tools": updated }))
}

fn delete_tool_component(env: &mut Env, a: &Map<String, Value>) -> Result<Value, String> {
    let id = req_str(a, "id")?.trim();
    let cmd = DeleteDocumentComponent::new(&env.world, id).map_err(cmd_err)?;
    env.history.execute(&env.world, cmd);
    Ok(json!({ "deleted": id }))
}

fn sync_tool_component(env: &mut Env, a: &Map<String, Value>) -> Result<Value, String> {
    let id = req_str(a, "id")?.trim();
    let library = full_library(a)?;
    let cmd = SyncDocumentComponent::new(&env.world, id, &library).map_err(cmd_err)?;
    let updated = cmd.updated_tools();
    env.history.execute(&env.world, cmd);
    Ok(json!({ "component": document_row(env, a, id), "updated_tools": updated }))
}
