//! The MCP tool assembly tools (#3647): read a document tool's
//! component stack as the resolver judges it, and set or clear the
//! stack through the very undoable `SetToolAssembly` the GUI queues
//! (one History step that lands the derived tool record and holder
//! body).
//!
//! Provenance: our own tool surface over this repo's assembly commands
//! (#3644) and the ISO 13399 stack resolver (#3643).

use serde_json::{Map, Value, json};

use super::component_library_tools::{full_library, kind_word, library_prop, status_json};
use super::setup_tools::{cmd_err, obj};
use super::tools::{Tool, req_str};
use crate::commands::SetToolAssembly;
use crate::model;
use crate::project::{FrameDto, LinkDto, assembly_to_dto};
use crate::schema::AssemblyLink;
use crate::script::Env;
use crate::tool_assembly as ta;
use camcraft_tools::tool_assembly::{AssemblyNode, Mount, resolve};

pub(super) const TOOLS: &[Tool] = &[
    Tool {
        name: "get_tool_assembly",
        description: "Read a document tool's component assembly: each node (component id, name, kind, ports, its mount on the parent: parent index, port, z_rot_deg and the stickout projection_mm, its MCS frame in the tool frame, and its library sync status), the derived cutting diameter and corner radius, the gauge length from the cutting tip to the spindle gauge line, and links (the form set_tool_assembly takes). Refused by name: an unknown or non-tool name, a tool with no assembly, and an assembly that no longer resolves.",
        schema: s_get,
        run: get_tool_assembly,
    },
    Tool {
        name: "set_tool_assembly",
        description: "Set a document tool's component assembly, root first: links is [{component, mount?: {parent, port?, z_rot_deg?, projection_mm?}}] (no mount: the root, on the spindle; parent is a node index). The derived tool record (diameter, corner radius, or a solid tool's own record) and the holder body land in one undo step, and the tool's operations go stale. links null clears the assembly and keeps the tool's records. Refused by name: an unknown or non-tool name, an unknown key, an empty links, a component the document does not hold, a component that does not fit its parent (the property is named), a bad port, frame or stickout, a lathe-held tool, a body in the flutes, and a change that an operation of the tool cannot take.",
        schema: s_set,
        run: set_tool_assembly,
    },
];

fn s_get() -> Value {
    obj(
        json!({
            "tool": { "type": "string", "description": "A document tool's name." },
            "library": library_prop(),
        }),
        &["tool"],
    )
}

fn s_set() -> Value {
    obj(
        json!({
            "tool": { "type": "string", "description": "A document tool's name." },
            "links": { "type": ["array", "null"], "items": { "type": "object" }, "description": "The stack, root first; null clears it." },
        }),
        &["tool", "links"],
    )
}

/// The document tool `name`, or the refusal by name.
fn tool_entity(env: &Env, name: &str) -> Result<flecs_ecs::prelude::Entity, String> {
    let e = model::find_by_name(&env.world, name).ok_or_else(|| format!("no tool named '{name}'"))?;
    if !model::is_tool(&env.world, &e) {
        return Err(format!("'{name}' is not a tool"));
    }
    Ok(e)
}

/// The assembly view of tool `name` (see `get_tool_assembly`).
fn view(env: &Env, a: &Map<String, Value>, name: &str) -> Result<Value, String> {
    let entity = tool_entity(env, name)?;
    let Some(stack) = ta::assembly_of(&env.world, &entity) else {
        return Err(format!("tool '{name}' has no tool assembly - set_tool_assembly builds one"));
    };
    let table = ta::components(&env.world);
    let mut rows = Vec::with_capacity(stack.0.len());
    let mut nodes = Vec::with_capacity(stack.0.len());
    for link in &stack.0 {
        let Some(row) = table.iter().find(|r| r.component.id() == link.component) else {
            return Err(format!("tool '{name}': the assembly names the component '{}', which the document does not hold", link.component));
        };
        nodes.push(AssemblyNode { component: row.component.clone(), geometry: row.geometry.clone(), mount: link.mount.clone() });
        rows.push(row);
    }
    let r = resolve(&nodes).map_err(|e| format!("tool '{name}': assembly: {e}"))?;
    // The library is read only when a node links to it.
    let library = if rows.iter().any(|r| r.library_ref.is_some()) { full_library(a) } else { Ok(Vec::new()) };
    let out: Vec<Value> = rows
        .iter()
        .zip(&stack.0)
        .enumerate()
        .map(|(i, (row, link))| {
            json!({
                "index": i,
                "component": row.component.id(),
                "name": row.component.name(),
                "kind": kind_word(&row.component),
                "ports": row.component.ports(),
                "mount": link.mount.as_ref().map(|m| json!({
                    "parent": m.parent, "port": m.port, "z_rot_deg": m.z_rot_deg, "projection_mm": m.projection_mm,
                })),
                "frame": r.frames.get(i).map(|f| FrameDto { origin: f.origin(), z: f.z(), x: f.x() }),
                "sync_status": status_json(row, &library),
            })
        })
        .collect();
    Ok(json!({
        "tool": name,
        "nodes": out,
        "cutting": { "diameter": r.cutting.diameter, "corner_r": r.cutting.corner_r },
        "gauge_len": r.gauge_len,
        "solid_tool": r.tool.is_some(),
        "holder_body": r.body.is_some(),
        "links": assembly_to_dto(&stack),
    }))
}

fn get_tool_assembly(env: &mut Env, a: &Map<String, Value>) -> Result<Value, String> {
    view(env, a, req_str(a, "tool")?)
}

fn set_tool_assembly(env: &mut Env, a: &Map<String, Value>) -> Result<Value, String> {
    let name = req_str(a, "tool")?;
    tool_entity(env, name)?;
    let links = match a.get("links") {
        None => return Err("\"links\" is required: the stack, root first, or null to clear it".to_owned()),
        Some(Value::Null) => {
            let cmd = SetToolAssembly::clear(&env.world, name).map_err(cmd_err)?;
            env.history.execute(&env.world, cmd);
            return Ok(json!({ "tool": name, "assembly": Value::Null }));
        }
        Some(v @ Value::Array(_)) => {
            serde_json::from_value::<Vec<LinkDto>>(v.clone()).map_err(|e| format!("bad \"links\": {e}"))?
        }
        Some(other) => return Err(format!("\"links\" must be an array or null, got {other}")),
    };
    if links.is_empty() {
        return Err("\"links\" is empty - give null to clear the assembly".to_owned());
    }
    let links: Vec<AssemblyLink> = links
        .into_iter()
        .map(|l| AssemblyLink {
            component: l.component,
            mount: l.mount.map(|m| Mount { parent: m.parent, port: m.port, z_rot_deg: m.z_rot_deg, projection_mm: m.projection_mm }),
        })
        .collect();
    let cmd = SetToolAssembly::new(&env.world, name, links).map_err(cmd_err)?;
    env.history.execute(&env.world, cmd);
    view(env, a, name)
}
