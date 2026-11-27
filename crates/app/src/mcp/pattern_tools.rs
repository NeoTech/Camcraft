//! The MCP tools for pattern folders (#3012): create, edit, set the
//! members of, turn one copy on or off, and delete a folder, plus the
//! folder rows `get_job` lists. Each tool runs the undoable command the
//! app issues, so one `undo` reverts it and every refusal the command
//! makes comes back as its text.
//!
//! Provenance: our own tool surface over this repo's command layer.

use flecs_ecs::prelude::{Entity, World};
use serde_json::{json, Map, Value};

use super::edit::find_op;
use super::tools::{opt_str, req_str, Tool};
use crate::commands::{
    CreatePatternFolder, DeletePatternFolder, EditPatternFolder, PatternEdit, SetPatternInstanceEnabled,
    SetPatternMembers,
};
use crate::model;
use crate::pattern_folder::{copy_count, folder_rows, pattern_of};
use crate::schema::{PatternDef, PatternKind, PatternOrder};
use crate::script::Env;

pub(super) const TOOLS: &[Tool] = &[
    Tool { name: "create_pattern_folder", description: "Make a pattern folder: the member operations (one setup, milling only, in no other folder) are posted again at each copy of a linear or grid pattern, all in the setup's WCS. Copy 1 is the members' own place. order by_tool (default) cuts every copy with one tool change per member; by_part cuts all members at one copy, then the next copy. The folder takes the row of its first member, and the members follow it. Answers with the folder. One undo step.", schema: s_create, run: create_pattern_folder },
    Tool { name: "edit_pattern_folder", description: "Change a pattern folder's pattern, its order (by_tool or by_part) or its suppression. A suppressed folder makes no copies: its members post once, as plain operations. A smaller pattern drops the off-switches of copies it no longer has. One undo step.", schema: s_edit, run: edit_pattern_folder },
    Tool { name: "set_pattern_members", description: "Replace a pattern folder's member operations (in folder order; an empty list keeps the folder with no members). The members move to directly after the folder; operations taken out keep their rows. One undo step.", schema: s_members, run: set_pattern_members },
    Tool { name: "set_pattern_instance_enabled", description: "Turn one copy (1-based, grid copies count along the first direction first) of a pattern folder on or off. At least one copy stays on. One undo step.", schema: s_instance, run: set_pattern_instance_enabled },
    Tool { name: "delete_pattern_folder", description: "Delete a pattern folder. Its members stay in the list where they stand, as plain operations. One undo step.", schema: s_name, run: delete_pattern_folder },
];

/// A folder by its key or its display name.
pub(super) fn find_folder(env: &Env, name: &str) -> Result<String, String> {
    folder_rows(&env.world)
        .into_iter()
        .find(|(k, e)| k == name || model::operation_label(&env.world, k, e) == name)
        .map(|(k, _)| k)
        .ok_or_else(|| format!("no pattern folder named '{name}'"))
}

/// An operation or a pattern folder: a row that rename and reorder take.
pub(super) fn find_row(env: &Env, name: &str) -> Result<String, String> {
    find_op(env, name).or_else(|_| find_folder(env, name)).map_err(|_| format!("no operation or pattern folder named '{name}'"))
}

fn order_word(o: PatternOrder) -> &'static str {
    match o {
        PatternOrder::ByTool => "by_tool",
        PatternOrder::ByPart => "by_part",
    }
}

fn pattern_view(kind: &PatternKind) -> Value {
    match kind {
        PatternKind::Linear { dir, count, spacing } => json!({ "type": "linear", "dir": dir, "count": count, "spacing": spacing }),
        PatternKind::Grid { dir1, count1, spacing1, dir2, count2, spacing2 } => json!({
            "type": "grid", "dir1": dir1, "count1": count1, "spacing1": spacing1,
            "dir2": dir2, "count2": count2, "spacing2": spacing2
        }),
    }
}

fn folder_view(world: &World, key: &str, e: &Entity, def: &PatternDef) -> Value {
    json!({
        "key": key,
        "name": model::operation_label(world, key, e),
        "setup": crate::program_entry::effective_setup(world, e),
        "suppressed": def.suppressed,
        "order": order_word(def.order_mode),
        "pattern": pattern_view(&def.kind),
        "copies": copy_count(&def.kind),
        "members": def.members,
        "disabled": def.disabled_instances,
    })
}

/// The `get_job` rows: every folder, by key order.
pub(super) fn folders_view(world: &World) -> Vec<Value> {
    folder_rows(world)
        .into_iter()
        .filter_map(|(k, e)| pattern_of(world, &e).map(|def| folder_view(world, &k, &e, &def)))
        .collect()
}

fn answer(env: &Env, key: &str) -> Result<Value, String> {
    let e = model::find_by_name(&env.world, key).ok_or_else(|| format!("no pattern folder named '{key}'"))?;
    let def = pattern_of(&env.world, &e).ok_or_else(|| format!("no pattern folder named '{key}'"))?;
    Ok(folder_view(&env.world, key, &e, &def))
}

fn num(o: &Map<String, Value>, key: &str) -> Result<f64, String> {
    o.get(key).and_then(Value::as_f64).ok_or_else(|| format!("\"{key}\" must be a number"))
}

fn count(o: &Map<String, Value>, key: &str) -> Result<u32, String> {
    o.get(key)
        .and_then(Value::as_u64)
        .and_then(|n| u32::try_from(n).ok())
        .ok_or_else(|| format!("\"{key}\" must be a whole number"))
}

fn dir(o: &Map<String, Value>, key: &str) -> Result<[f64; 3], String> {
    let v: Vec<f64> = o
        .get(key)
        .and_then(Value::as_array)
        .and_then(|a| a.iter().map(Value::as_f64).collect::<Option<Vec<f64>>>())
        .unwrap_or_default();
    match v[..] {
        [x, y] => Ok([x, y, 0.0]),
        [x, y, z] => Ok([x, y, z]),
        _ => Err(format!("\"{key}\" must be [x, y] or [x, y, z] numbers")),
    }
}

fn pattern(a: &Map<String, Value>) -> Result<Option<PatternKind>, String> {
    let Some(v) = a.get("pattern").filter(|v| !v.is_null()) else {
        return Ok(None);
    };
    let o = v.as_object().ok_or("\"pattern\" must be an object")?;
    match o.get("type").and_then(Value::as_str) {
        Some("linear") => Ok(Some(PatternKind::Linear { dir: dir(o, "dir")?, count: count(o, "count")?, spacing: num(o, "spacing")? })),
        Some("grid") => Ok(Some(PatternKind::Grid {
            dir1: dir(o, "dir1")?,
            count1: count(o, "count1")?,
            spacing1: num(o, "spacing1")?,
            dir2: dir(o, "dir2")?,
            count2: count(o, "count2")?,
            spacing2: num(o, "spacing2")?,
        })),
        _ => Err("\"pattern\".\"type\" must be \"linear\" or \"grid\"".into()),
    }
}

fn order(a: &Map<String, Value>) -> Result<Option<PatternOrder>, String> {
    match opt_str(a, "order")? {
        None => Ok(None),
        Some("by_tool") => Ok(Some(PatternOrder::ByTool)),
        Some("by_part") => Ok(Some(PatternOrder::ByPart)),
        Some(other) => Err(format!("\"order\" must be \"by_tool\" or \"by_part\", got \"{other}\"")),
    }
}

fn members(env: &Env, a: &Map<String, Value>) -> Result<Vec<String>, String> {
    match a.get("members") {
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(Value::Array(list)) => list
            .iter()
            .map(|v| v.as_str().ok_or_else(|| "\"members\" must be operation names".to_owned()).and_then(|n| find_op(env, n)))
            .collect(),
        Some(_) => Err("\"members\" must be an array of operation names".into()),
    }
}

fn create_pattern_folder(env: &mut Env, a: &Map<String, Value>) -> Result<Value, String> {
    let kind = pattern(a)?.ok_or("\"pattern\" is required")?;
    let list = members(env, a)?;
    let cmd = CreatePatternFolder::new(&env.world, &kind, order(a)?.unwrap_or_default(), &list).map_err(|e| e.to_string())?;
    let key = cmd.name().to_owned();
    env.history.execute(&env.world, cmd);
    answer(env, &key)
}

fn edit_pattern_folder(env: &mut Env, a: &Map<String, Value>) -> Result<Value, String> {
    let key = find_folder(env, req_str(a, "name")?)?;
    let suppressed = match a.get("suppressed") {
        None | Some(Value::Null) => None,
        Some(Value::Bool(b)) => Some(*b),
        Some(other) => return Err(format!("\"suppressed\" must be true or false, got {other}")),
    };
    let edit = PatternEdit { kind: pattern(a)?, order_mode: order(a)?, suppressed };
    let cmd = EditPatternFolder::new(&env.world, &key, &edit).map_err(|e| e.to_string())?;
    env.history.execute(&env.world, cmd);
    answer(env, &key)
}

fn set_pattern_members(env: &mut Env, a: &Map<String, Value>) -> Result<Value, String> {
    let key = find_folder(env, req_str(a, "name")?)?;
    if !a.get("members").is_some_and(Value::is_array) {
        return Err("\"members\" is required: an array of operation names".into());
    }
    let list = members(env, a)?;
    let cmd = SetPatternMembers::new(&env.world, &key, &list).map_err(|e| e.to_string())?;
    env.history.execute(&env.world, cmd);
    answer(env, &key)
}

fn set_pattern_instance_enabled(env: &mut Env, a: &Map<String, Value>) -> Result<Value, String> {
    let key = find_folder(env, req_str(a, "name")?)?;
    let instance = count(a, "instance")?;
    let enabled = a.get("enabled").and_then(Value::as_bool).ok_or("\"enabled\" must be true or false")?;
    let cmd = SetPatternInstanceEnabled::new(&env.world, &key, instance, enabled).map_err(|e| e.to_string())?;
    env.history.execute(&env.world, cmd);
    answer(env, &key)
}

fn delete_pattern_folder(env: &mut Env, a: &Map<String, Value>) -> Result<Value, String> {
    let key = find_folder(env, req_str(a, "name")?)?;
    let members = answer(env, &key)?["members"].clone();
    let cmd = DeletePatternFolder::new(&env.world, &key).map_err(|e| e.to_string())?;
    env.history.execute(&env.world, cmd);
    Ok(json!({ "deleted": key, "ungrouped": members }))
}

fn obj(props: Value, required: &[&str]) -> Value {
    json!({ "type": "object", "properties": props, "required": required, "additionalProperties": false })
}

fn s_pattern() -> Value {
    json!({
        "type": "object",
        "description": "The copies, in mm, in the XY plane of the setup's WCS. Linear: {\"type\": \"linear\", \"dir\": [x, y], \"count\", \"spacing\"}. Grid: {\"type\": \"grid\", \"dir1\", \"count1\", \"spacing1\", \"dir2\", \"count2\", \"spacing2\"} (directions not parallel). Counts 1-1000, at most 10000 copies; spacings above 0.",
        "properties": {
            "type": { "type": "string", "enum": ["linear", "grid"] },
            "dir": { "type": "array", "items": { "type": "number" } },
            "count": { "type": "integer", "minimum": 1 },
            "spacing": { "type": "number" },
            "dir1": { "type": "array", "items": { "type": "number" } },
            "count1": { "type": "integer", "minimum": 1 },
            "spacing1": { "type": "number" },
            "dir2": { "type": "array", "items": { "type": "number" } },
            "count2": { "type": "integer", "minimum": 1 },
            "spacing2": { "type": "number" }
        },
        "required": ["type"]
    })
}

fn s_order() -> Value {
    json!({ "type": "string", "enum": ["by_tool", "by_part"], "description": "by_tool (default): one section per member over every copy; by_part: every member at one copy, then the next." })
}

fn s_members_list() -> Value {
    json!({ "type": "array", "items": { "type": "string" }, "description": "Member operations (key or display name), in folder order." })
}

fn s_create() -> Value {
    obj(json!({ "pattern": s_pattern(), "order": s_order(), "members": s_members_list() }), &["pattern"])
}

fn s_edit() -> Value {
    obj(
        json!({
            "name": { "type": "string", "description": "The folder (key or display name)." },
            "pattern": s_pattern(),
            "order": s_order(),
            "suppressed": { "type": "boolean", "description": "Suppressed: no copies; the members post once." }
        }),
        &["name"],
    )
}

fn s_members() -> Value {
    obj(json!({ "name": { "type": "string", "description": "The folder (key or display name)." }, "members": s_members_list() }), &["name", "members"])
}

fn s_instance() -> Value {
    obj(
        json!({
            "name": { "type": "string", "description": "The folder (key or display name)." },
            "instance": { "type": "integer", "minimum": 1, "description": "The copy number, 1-based." },
            "enabled": { "type": "boolean" }
        }),
        &["name", "instance", "enabled"],
    )
}

fn s_name() -> Value {
    obj(json!({ "name": { "type": "string", "description": "The folder (key or display name)." } }), &["name"])
}
