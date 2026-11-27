//! The MCP face-to-size tool (#3196): face a block to height from
//! numbers in one call - one `FaceToSize` run through the session
//! History, so one `undo` takes back the stock it set and the Face.
//! Every refusal is the command's own text, returned as a tool error.
//!
//! Provenance: our own tool surface over this repo's facing command.

use super::setup_tools::{cmd_err, obj};
use super::tools::Tool;
use crate::commands::{FaceToSize, FaceToSizeSpec};
use crate::script::Env;

use serde_json::{json, Map, Value};

/// The feeds a face takes when the caller states none (the seed's).
const FEED: f64 = 1200.0;
const PLUNGE_FEED: f64 = 300.0;

pub(super) const TOOLS: &[Tool] = &[Tool {
    name: "face_to_size",
    description: "Face a block to height from numbers, as one undo step: optionally set the active setup's stock to a box (stock {dx, dy, dz}, its top at top_z), then add a Face on the stock that steps from top_z down to final_z in passes of stepdown, rows stepover apart. Z values are program Z (from the setup's WCS, else the world origin), sizes in mm, feeds in mm/min. Refused by name: final_z not below top_z, stepdown or stepover not above 0, a missing tool or one narrower than the stepover, no stock and no size, top_z under the stock top, final_z at or under the stock bottom. Not generated until generate runs.",
    schema: s_face_to_size,
    run: face_to_size,
}];

fn num(a: &Map<String, Value>, key: &str) -> Result<Option<f64>, String> {
    match a.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(n)) => n.as_f64().map(Some).ok_or_else(|| format!("\"{key}\" {n} is not a number")),
        Some(other) => Err(format!("\"{key}\" must be a number, got {other}")),
    }
}

fn need(a: &Map<String, Value>, key: &str) -> Result<f64, String> {
    num(a, key)?.ok_or_else(|| format!("\"{key}\" is required"))
}

fn stock(a: &Map<String, Value>) -> Result<Option<[f64; 3]>, String> {
    match a.get("stock") {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Object(s)) => Ok(Some([need(s, "dx")?, need(s, "dy")?, need(s, "dz")?])),
        Some(other) => Err(format!("\"stock\" must be {{\"dx\", \"dy\", \"dz\"}} in mm, got {other}")),
    }
}

fn word(a: &Map<String, Value>, key: &str) -> Result<Option<String>, String> {
    match a.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.clone())),
        Some(other) => Err(format!("\"{key}\" must be a string, got {other}")),
    }
}

fn face_to_size(env: &mut Env, a: &Map<String, Value>) -> Result<Value, String> {
    let climb = match a.get("climb") {
        None | Some(Value::Null) => true,
        Some(Value::Bool(b)) => *b,
        Some(other) => return Err(format!("\"climb\" must be true or false, got {other}")),
    };
    let spec = FaceToSizeSpec {
        name: word(a, "name")?,
        tool: word(a, "tool")?.unwrap_or_default(),
        stock: stock(a)?,
        top_z: need(a, "top_z")?,
        final_z: need(a, "final_z")?,
        stepdown: need(a, "stepdown")?,
        stepover: need(a, "stepover")?,
        climb,
        feed: num(a, "feed")?.unwrap_or(FEED),
        plunge_feed: num(a, "plunge_feed")?.unwrap_or(PLUNGE_FEED),
    };
    let cmd = FaceToSize::new(&env.world, &spec).map_err(cmd_err)?;
    let (operation, stock_set) = (cmd.operation().to_owned(), cmd.stock_set());
    env.history.execute(&env.world, cmd);
    Ok(json!({ "operation": operation, "stock_set": stock_set }))
}

fn s_face_to_size() -> Value {
    obj(
        json!({
            "tool": { "type": "string", "description": "The facing tool, by its document name (add_tool first)." },
            "stock": {
                "type": ["object", "null"],
                "description": "A box stock to set, mm: length X, width Y, height Z; its top is at top_z. Omit to face the setup's stock as it stands.",
                "properties": { "dx": { "type": "number" }, "dy": { "type": "number" }, "dz": { "type": "number" } }
            },
            "top_z": { "type": "number", "description": "The Z of the first pass's top, program Z, mm." },
            "final_z": { "type": "number", "description": "The finished Z, program Z, mm; below top_z." },
            "stepdown": { "type": "number", "description": "Depth per pass, mm." },
            "stepover": { "type": "number", "description": "Radial stepover between rows, mm; not wider than the tool." },
            "climb": { "type": "boolean", "description": "Climb cut (default true); false cuts conventional." },
            "feed": { "type": "number", "description": "Cutting feed, mm/min (default 1200)." },
            "plunge_feed": { "type": "number", "description": "Plunge feed, mm/min (default 300)." },
            "name": { "type": "string", "description": "The operation's name (default: Face to size, then Face to size 2, ...)." }
        }),
        &["tool", "top_z", "final_z", "stepdown", "stepover"],
    )
}
