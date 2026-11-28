//! The MCP part-in-spindle tools (#3406): state, change or clear a mill
//! setup's part-in-spindle mode, and put or remove a table tool station.
//! Each call is one `SetPartInSpindle` / `SetTableStation` run through the
//! session History, so `undo` walks it; every refusal is the command's
//! own text, returned as a tool error.
//!
//! Provenance: our own tool surface over this repo's setup commands.

use super::setup_tools::{cmd_err, obj, setup_or_active};
use super::tools::Tool;
use crate::commands::{SetPartInSpindle, SetTableStation};
use crate::part_in_spindle::{ApproachSide, Holding, SpindleDir, TableStation};
use crate::script::Env;

use serde_json::{json, Map, Value};

pub(super) const TOOLS: &[Tool] = &[
    Tool {
        name: "set_part_in_spindle",
        description: "Hold the part in a mill's spindle (a collet) with lathe tools on the table: state the stock's length offset register (G43 H), the collet stick-out and outer diameter, mm. enabled false clears the mode. The setup then admits turning operations and an on-centre Drill and refuses milling; its workholding max rpm is required. One undo step.",
        schema: s_mode,
        run: set_part_in_spindle,
    },
    Tool {
        name: "set_table_station",
        description: "Put a table tool station in a part-in-spindle setup: the tool, its own 1-based work offset, its approach side (-X or +X) and its spindle direction (M3 or M4). remove true takes the tool's station off. Two tools never share one work offset. One undo step.",
        schema: s_station,
        run: set_table_station,
    },
];

/// The record as `get_job` and the tools show it (`null`: no mode).
pub(super) fn view(world: &flecs_ecs::prelude::World, setup: Option<&str>) -> Value {
    crate::part_in_spindle::of(world, setup).and_then(|p| serde_json::to_value(p).ok()).unwrap_or(Value::Null)
}

fn num(a: &Map<String, Value>, key: &str) -> Result<f64, String> {
    a.get(key).and_then(Value::as_f64).ok_or_else(|| format!("\"{key}\" is required (a number)"))
}

fn set_part_in_spindle(env: &mut Env, a: &Map<String, Value>) -> Result<Value, String> {
    let name = setup_or_active(env, a)?.ok_or("no setup is declared - add_setup first (the mode belongs to a setup)")?;
    let holding = if a.get("enabled") == Some(&Value::Bool(false)) {
        None
    } else {
        let h = a.get("h_register").and_then(Value::as_u64).ok_or("\"h_register\" is required (a whole number)")?;
        Some(Holding {
            h_register: u32::try_from(h).map_err(|_| format!("\"h_register\" {h} is too large"))?,
            stick_out: num(a, "stick_out")?,
            collet_od: num(a, "collet_od")?,
        })
    };
    let cmd = SetPartInSpindle::new(&env.world, &name, holding).map_err(cmd_err)?;
    env.history.execute(&env.world, cmd);
    Ok(json!({ "setup": name, "part_in_spindle": view(&env.world, Some(&name)) }))
}

fn set_table_station(env: &mut Env, a: &Map<String, Value>) -> Result<Value, String> {
    let name = setup_or_active(env, a)?.ok_or("no setup is declared - add_setup first")?;
    let tool = a.get("tool").and_then(Value::as_str).ok_or("\"tool\" is required (a tool name)")?;
    let cmd = if a.get("remove") == Some(&Value::Bool(true)) {
        SetTableStation::remove(&env.world, &name, tool)
    } else {
        let off = a.get("work_offset").and_then(Value::as_u64).ok_or("\"work_offset\" is required (1-based)")?;
        let word = |k: &str| a.get(k).and_then(Value::as_str).unwrap_or_default();
        let side = ApproachSide::from_word(word("side"))
            .ok_or_else(|| format!("\"side\" must be -X or +X, got {:?}", a.get("side")))?;
        let spindle = SpindleDir::from_word(word("spindle"))
            .ok_or_else(|| format!("\"spindle\" must be M3 or M4, got {:?}", a.get("spindle")))?;
        let work_offset = usize::try_from(off).map_err(|_| format!("\"work_offset\" {off} is too large"))?;
        SetTableStation::put(&env.world, &name, TableStation { tool: tool.to_owned(), work_offset, side, spindle })
    }
    .map_err(cmd_err)?;
    env.history.execute(&env.world, cmd);
    Ok(json!({ "setup": name, "part_in_spindle": view(&env.world, Some(&name)) }))
}

fn s_mode() -> Value {
    obj(
        json!({
            "h_register": { "type": "integer", "description": "The stock's length offset register, for G43 H (1-9999)." },
            "stick_out": { "type": "number", "description": "How far the stock sticks out of the collet nose, mm." },
            "collet_od": { "type": "number", "description": "The collet nose's outer diameter, mm." },
            "enabled": { "type": "boolean", "description": "false clears the mode (no operation may stay in the setup)." },
            "setup": { "type": "string", "description": "A declared setup; defaults to the active one." }
        }),
        &[],
    )
}

fn s_station() -> Value {
    obj(
        json!({
            "tool": { "type": "string", "description": "The table tool." },
            "work_offset": { "type": "integer", "description": "The tool's own 1-based work offset (1 is G54)." },
            "side": { "type": "string", "enum": ["-X", "+X"], "description": "The side the tool approaches the part from." },
            "spindle": { "type": "string", "enum": ["M3", "M4"], "description": "The spindle direction the tool cuts with." },
            "remove": { "type": "boolean", "description": "true takes the tool's station off the table." },
            "setup": { "type": "string", "description": "A declared setup; defaults to the active one." }
        }),
        &["tool"],
    )
}
