//! The MCP door of #4339: preview (dry run) and apply the conversion of
//! standalone document tools to component stacks, through the very
//! undoable `ConvertToComponents` the tool library window queues (one
//! History step for all the converted tools).
//!
//! Provenance: our own tool surface over this repo's conversion plan
//! (`tool_convert`) and assembly commands (#3644).

use serde_json::{Map, Value, json};

use super::setup_tools::{cmd_err, obj};
use super::tools::Tool;
use crate::commands::ConvertToComponents;
use crate::project::assembly_to_dto;
use crate::schema::ToolAssembly;
use crate::script::Env;
use crate::tool_convert::{plan, ConvertRow, Verdict};

pub(super) const TOOLS: &[Tool] = &[Tool {
    name: "convert_tools_to_components",
    description: "Convert standalone document tools (tools with no component assembly) to component assemblies: the tool's library holder as the built-in holder component, a collet that takes the shank where the holder needs one, and the tool itself as a solid cutting item. dry_run true returns the preview and changes nothing: one row for each tool (the named tools, or every standalone tool) with result same (the assembly derives the same tool record and gauge length), different (with differences) or no match (with reason), the component names and links (the form set_tool_assembly takes). Without dry_run the named tools convert in one undo step; with no tools, every 'same' row converts. Refused by name: an unknown key, a name that is not a document tool, a name twice, a 'no match' tool (its reason), and no 'same' row when tools is omitted.",
    schema: s_convert,
    run: convert_tools_to_components,
}];

fn s_convert() -> Value {
    obj(
        json!({
            "tools": { "type": "array", "items": { "type": "string" }, "description": "The document tools to convert or preview; omitted: every standalone tool (apply: only the 'same' rows)." },
            "dry_run": { "type": "boolean", "description": "true: return the preview and change nothing (default false)." },
        }),
        &[],
    )
}

/// The `tools` argument: `None` when omitted.
fn names(a: &Map<String, Value>) -> Result<Option<Vec<String>>, String> {
    match a.get("tools") {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Array(items)) => items
            .iter()
            .map(|v| v.as_str().map(str::to_owned).ok_or_else(|| format!("\"tools\" holds a non-string: {v}")))
            .collect::<Result<Vec<_>, _>>()
            .map(Some),
        Some(other) => Err(format!("\"tools\" must be an array of tool names, got {other}")),
    }
}

fn row_json(r: &ConvertRow) -> Value {
    let (differences, reason) = match &r.verdict {
        Verdict::Same => (Vec::new(), Value::Null),
        Verdict::Different(d) => (d.clone(), Value::Null),
        Verdict::NoMatch(why) => (Vec::new(), Value::String(why.clone())),
    };
    json!({
        "tool": r.tool,
        "result": r.verdict.word(),
        "differences": differences,
        "reason": reason,
        "components": r.names,
        "links": assembly_to_dto(&ToolAssembly(r.links.clone())),
    })
}

fn convert_tools_to_components(env: &mut Env, a: &Map<String, Value>) -> Result<Value, String> {
    let dry_run = match a.get("dry_run") {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(other) => return Err(format!("\"dry_run\" must be true or false, got {other}")),
    };
    let names = names(a)?;
    if dry_run {
        let rows = plan(&env.world, names.as_deref().unwrap_or(&[]))?;
        return Ok(json!({ "dry_run": true, "rows": rows.iter().map(row_json).collect::<Vec<_>>() }));
    }
    let cmd = match &names {
        Some(n) => ConvertToComponents::new(&env.world, n),
        None => ConvertToComponents::same_rows(&env.world),
    }
    .map_err(cmd_err)?;
    let converted = cmd.tools();
    env.history.execute(&env.world, cmd);
    Ok(json!({ "dry_run": false, "converted": converted }))
}
