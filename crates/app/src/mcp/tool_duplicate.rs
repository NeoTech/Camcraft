//! The MCP tool `duplicate_tool` (#4340): runs the very undoable
//! `DuplicateTool` command the GUI's tool editor queues (one History step)
//! and reports the copy's name and number.
//!
//! Provenance: our own tool surface over this repo's `DuplicateTool`.

use serde_json::{Map, Value, json};

use super::setup_tools::{cmd_err, obj};
use super::tools::{Tool, req_str};
use crate::commands::DuplicateTool;
use crate::script::Env;

pub(super) const TOOLS: &[Tool] = &[Tool {
    name: "duplicate_tool",
    description: "Copy a document tool as a new tool: the same geometry, cutting data, holder, stick-out, drawn cutter and component stack, under a free name (\"<name> (2)\", \"(3)\", ...) and the next free tool number the mounted machine does not reserve. One undo step removes the copy. Refused by name, changing nothing: an unknown name, an entity that is not a tool, no free copy name, or no free tool number.",
    schema,
    run: duplicate_tool,
}];

fn schema() -> Value {
    obj(
        json!({ "name": { "type": "string", "description": "The document tool to copy (get_job's tools[].name)." } }),
        &["name"],
    )
}

fn duplicate_tool(env: &mut Env, a: &Map<String, Value>) -> Result<Value, String> {
    let source = req_str(a, "name")?;
    let cmd = DuplicateTool::new(&env.world, source).map_err(cmd_err)?;
    let (name, number) = (cmd.name().to_string(), cmd.number());
    env.history.execute(&env.world, cmd);
    Ok(json!({ "source": source, "tool": name, "number": number }))
}
