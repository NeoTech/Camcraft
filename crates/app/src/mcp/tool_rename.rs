//! The MCP tool `rename_tool` (#4370): runs the very undoable
//! `RenameTool` command the GUI's tool editor queues (one History step)
//! and reports the operations that follow the tool to its new name.
//!
//! Provenance: our own tool surface over this repo's `RenameTool`.

use serde_json::{Map, Value, json};

use super::setup_tools::{cmd_err, obj};
use super::tools::{Tool, req_str};
use crate::commands::RenameTool;
use crate::script::Env;

pub(super) const TOOLS: &[Tool] = &[Tool {
    name: "rename_tool",
    description: "Give a document tool a new name. Each operation that uses the tool follows it to the new name in the same step; the tool keeps its number, holder, stick-out, drawn cutter and component stack, and a generated operation that was up to date stays up to date. One undo step restores the old name everywhere. Refused by name, changing nothing: an empty name, an unknown name, an entity that is not a tool, the same name, a name another entity has or a setup reserves, a name an operation still names, or a name the machine reserves.",
    schema,
    run: rename_tool,
}];

fn schema() -> Value {
    obj(
        json!({
            "name": { "type": "string", "description": "The document tool to rename (get_job's tools[].name)." },
            "new_name": { "type": "string", "description": "The tool's new name (leading and trailing spaces are dropped)." }
        }),
        &["name", "new_name"],
    )
}

fn rename_tool(env: &mut Env, a: &Map<String, Value>) -> Result<Value, String> {
    let from = req_str(a, "name")?;
    let to = req_str(a, "new_name")?;
    let cmd = RenameTool::new(&env.world, from, to).map_err(cmd_err)?;
    let reply = json!({ "from": cmd.from(), "tool": cmd.to(), "operations": cmd.operations() });
    env.history.execute(&env.world, cmd);
    Ok(reply)
}
