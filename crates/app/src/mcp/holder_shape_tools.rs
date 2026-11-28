//! #4391: the MCP door to the holder form (#4338) - edit a user holder of
//! the document's component table by its shape fields (taper, clamp, bore,
//! gauge length, nose and largest diameter). The outline is built with
//! `HolderShape::geometry`, as the form builds it, and the edit lands as
//! the very undoable `EditDocumentComponent` the form queues: one History
//! step, and every tool built from the holder is derived again in it. A
//! value that cannot be steel is refused with the `ShapeError` text; an
//! edit that breaks a tool's fit is refused by the command, which names
//! the tool.
//!
//! Provenance: our own tool surface over this repo's holder shape model
//! (#4338) and component commands (#3644); no HSMWorks material involved.

use serde_json::{Map, Value, json};

use super::setup_tools::{cmd_err, obj};
use super::tools::{Tool, opt_str, req_str};
use crate::commands::EditDocumentComponent;
use crate::script::Env;
use crate::tool_assembly as ta;
use camcraft_tools::component::{ADAPTIVE_PORT, ToolComponent};
use camcraft_tools::holder_library::{ER_SIZES, HolderShape, ShapeClamp, tapers};

pub(super) const TOOLS: &[Tool] = &[Tool {
    name: "edit_user_holder",
    description: "Edit a holder of the document's component table by its shape, as the holder form does: taper, clamp, bore, gauge_len, nose_d, largest_d and name. A field the call does not state keeps the holder's value (an ER or Weldon clamp without largest_d takes nose_d: it has one body diameter). The outline is the holder library's outline of the shape, never smaller than the stated steel. Every tool whose assembly uses the holder is derived again in the same undo step. One undo step. Refused by name: an id the document does not hold, a holder whose outline is not a shape outline (use upsert_tool_component), an unknown taper or clamp, a value that cannot describe steel, an empty name, an edit that changes nothing, and an edit that a tool built from the holder cannot take (the tool is named; nothing changes).",
    schema: s_edit,
    run: edit_user_holder,
}];

fn s_edit() -> Value {
    let tapers: Vec<&str> = tapers().iter().map(|t| t.name).collect();
    let mut clamps: Vec<&str> = ER_SIZES.iter().map(|e| e.name).collect();
    clamps.extend(["weldon", "shrink"]);
    obj(
        json!({
            "id": { "type": "string", "description": "The document component's id (a holder: an adaptive item)." },
            "name": { "type": "string", "description": "The holder's new name." },
            "taper": { "type": "string", "enum": tapers, "description": "The machine side." },
            "clamp": { "type": "string", "enum": clamps, "description": "An ER collet chuck size, a Weldon bore or a shrink-fit bore." },
            "bore": { "type": "number", "description": "The Weldon or shrink-fit bore (mm); an ER chuck reads its range from its size." },
            "gauge_len": { "type": "number", "description": "From the gauge line to the nose face (mm)." },
            "nose_d": { "type": "number", "description": "The steel at the nose face: the ER nut, the Weldon body, the shrink nose (mm)." },
            "largest_d": { "type": "number", "description": "The widest steel of the body below the flange (mm)." },
        }),
        &["id"],
    )
}

fn opt_num(a: &Map<String, Value>, key: &str) -> Result<Option<f64>, String> {
    match a.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => v.as_f64().map(Some).ok_or_else(|| format!("\"{key}\" must be a number, got {v}")),
    }
}

fn clamp_of(word: &str) -> Result<ShapeClamp, String> {
    if word.eq_ignore_ascii_case("weldon") {
        return Ok(ShapeClamp::Weldon);
    }
    if word.eq_ignore_ascii_case("shrink") {
        return Ok(ShapeClamp::Shrink);
    }
    ER_SIZES.iter().find(|e| e.name.eq_ignore_ascii_case(word)).map(|e| ShapeClamp::Er(*e)).ok_or_else(|| {
        let names: Vec<&str> = ER_SIZES.iter().map(|e| e.name).collect();
        format!("unknown clamp '{word}' - one of {}, weldon, shrink", names.join(", "))
    })
}

fn clamp_word(c: ShapeClamp) -> &'static str {
    match c {
        ShapeClamp::Er(er) => er.name,
        ShapeClamp::Weldon => "weldon",
        ShapeClamp::Shrink => "shrink",
    }
}

/// The holder's shape as the call states it, over the shape it has.
fn stated(held: HolderShape, a: &Map<String, Value>) -> Result<HolderShape, String> {
    let mut s = held;
    if let Some(t) = opt_str(a, "taper")? {
        let all = tapers();
        s.taper = all.iter().find(|x| x.name.eq_ignore_ascii_case(t.trim())).copied().ok_or_else(|| {
            let names: Vec<&str> = all.iter().map(|x| x.name).collect();
            format!("unknown taper '{t}' - one of {}", names.join(", "))
        })?;
    }
    if let Some(c) = opt_str(a, "clamp")? {
        s.clamp = clamp_of(c.trim())?;
    }
    s.bore = opt_num(a, "bore")?.unwrap_or(s.bore);
    s.gauge_len = opt_num(a, "gauge_len")?.unwrap_or(s.gauge_len);
    s.nose_d = opt_num(a, "nose_d")?.unwrap_or(s.nose_d);
    s.largest_d = match opt_num(a, "largest_d")? {
        Some(d) => d,
        // An ER or Weldon body has one diameter: it follows the nose.
        None if s.clamp != ShapeClamp::Shrink => s.nose_d,
        None => s.largest_d,
    };
    Ok(s)
}

fn shape_json(s: &HolderShape) -> Value {
    json!({
        "taper": s.taper.name,
        "clamp": clamp_word(s.clamp),
        "bore": s.bore,
        "gauge_len": s.gauge_len,
        "nose_d": s.nose_d,
        "largest_d": s.largest_d,
    })
}

fn edit_user_holder(env: &mut Env, a: &Map<String, Value>) -> Result<Value, String> {
    let id = req_str(a, "id")?.trim();
    let row = ta::component(&env.world, id).ok_or_else(|| format!("the document holds no tool component '{id}'"))?;
    let held = HolderShape::of(row.component.kind(), &row.geometry, ADAPTIVE_PORT).ok_or_else(|| {
        format!("the tool component '{id}' has no holder shape outline - its shape fields cannot be edited (use upsert_tool_component)")
    })?;
    let shape = stated(held, a)?;
    let name = opt_str(a, "name")?.map_or_else(|| row.component.name().to_owned(), |n| n.trim().to_owned());
    if name.is_empty() {
        return Err("holder not changed: the holder needs a name".to_owned());
    }
    // The same build as the holder form's Apply.
    let (kind, geometry) = shape
        .kind()
        .and_then(|kind| Ok((kind, shape.geometry(ADAPTIVE_PORT)?)))
        .map_err(|e| format!("holder not changed: {e}"))?;
    let component = ToolComponent::new(id, &name, kind).map_err(|e| format!("holder not changed: {e}"))?;
    let cmd = EditDocumentComponent::new(&env.world, component, geometry).map_err(cmd_err)?;
    let updated = cmd.updated_tools();
    env.history.execute(&env.world, cmd);
    Ok(json!({ "id": id, "name": name, "shape": shape_json(&shape), "updated_tools": updated }))
}
