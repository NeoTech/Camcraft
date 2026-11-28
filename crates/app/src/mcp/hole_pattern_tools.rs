//! `add_hole_pattern` (#3195): a Drill, Bore or Thread operation on the
//! stock whose holes come from numbers - a point list, a line, a grid or
//! a full or partial bolt-hole circle (`camcraft_geometry::hole_pattern`),
//! with no 3D model, and an optional spot drill ahead of it. Both
//! operations are made by `CreateOperation` in one undo group.
//!
//! Provenance: our own tool surface over this repo's commands; hole
//! coordinates by plain bolt-circle trigonometry (Smid, *CNC Programming
//! Handbook*).

use super::edit::{kind_index, patched, seed};
use super::tools::{opt_str, req_str, Tool};
use crate::commands::CreateOperation;
use crate::geo2d::Point2;
use crate::model;
use crate::operation::{self, KindInfo, OpParams, SrcRef};
use crate::project;
use crate::script::Env;
use camcraft_geometry::hole_pattern::{CircleSpacing, HolePattern};

use serde_json::{json, Map, Value};

pub(super) const TOOLS: &[Tool] = &[Tool {
    name: "add_hole_pattern",
    description: "Add a Drill, Bore or Thread operation on the stock whose holes come from numbers, with no 3D model. pattern is exactly one of: points [[x, y], ...]; line {start [x, y], pitch, count, angle}; grid {start [x, y], pitch_x, pitch_y, count_x, count_y, angle} (holes along X first, the grid turned about start); circle {center [x, y], diameter, count, start_angle, and for a partial circle angular_pitch or span} (a bolt-hole circle; angles in degrees from +X, counter-clockwise). Each hole has diameter and depth (mm) and starts at top (default: the active setup's stock top); through marks through holes. params patches the operation's other fields (cycle, pitch, ...) but not holes. spot {tool, depth, name} adds a spot-drill Drill operation on the same holes ahead of it. One undo step for all; bad numbers are refused by name and change nothing. A hole off the stock is made, with a warning.",
    schema: s_add,
    run: add_hole_pattern,
}];

fn num(o: &Map<String, Value>, key: &str, ctx: &str) -> Result<Option<f64>, String> {
    match o.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => v.as_f64().filter(|f| f.is_finite()).map(Some).ok_or_else(|| format!("{ctx} \"{key}\" must be a finite number, got {v}")),
    }
}

fn req_num(o: &Map<String, Value>, key: &str, ctx: &str) -> Result<f64, String> {
    num(o, key, ctx)?.ok_or_else(|| format!("{ctx} \"{key}\" is required"))
}

fn req_count(o: &Map<String, Value>, key: &str, ctx: &str) -> Result<u32, String> {
    let v = o.get(key).ok_or_else(|| format!("{ctx} \"{key}\" is required"))?;
    v.as_u64().and_then(|n| u32::try_from(n).ok()).ok_or_else(|| format!("{ctx} \"{key}\" must be a whole number of holes, got {v}"))
}

fn xy(v: Option<&Value>, what: &str) -> Result<Point2, String> {
    let pair = v.and_then(Value::as_array).filter(|a| a.len() == 2);
    let n = |i: usize| pair.and_then(|a| a.get(i)).and_then(Value::as_f64);
    match (n(0), n(1)) {
        (Some(x), Some(y)) => Ok(Point2::new(x, y)),
        _ => Err(format!("the {what} must be [x, y] in mm")),
    }
}

fn obj<'a>(v: &'a Value, what: &str) -> Result<&'a Map<String, Value>, String> {
    v.as_object().ok_or_else(|| format!("the {what} must be an object"))
}

/// The pattern object: exactly one of points, line, grid or circle.
fn pattern(v: &Value) -> Result<HolePattern, String> {
    let p = obj(v, "\"pattern\"")?;
    let keys: Vec<&String> = p.keys().collect();
    let [key] = keys.as_slice() else {
        return Err("\"pattern\" must hold exactly one of points, line, grid or circle".into());
    };
    let body = &p[key.as_str()];
    match key.as_str() {
        "points" => {
            let list = body.as_array().ok_or("pattern \"points\" must be a list of [x, y]")?;
            let pts = list.iter().enumerate().map(|(i, v)| xy(Some(v), &format!("point {}", i + 1))).collect::<Result<_, _>>()?;
            Ok(HolePattern::Points(pts))
        }
        "line" => {
            let o = obj(body, "line")?;
            Ok(HolePattern::Line {
                start: xy(o.get("start"), "line start")?,
                pitch: num(o, "pitch", "line")?.unwrap_or(0.0),
                count: req_count(o, "count", "line")?,
                angle_deg: num(o, "angle", "line")?.unwrap_or(0.0),
            })
        }
        "grid" => {
            let o = obj(body, "grid")?;
            Ok(HolePattern::Grid {
                start: xy(o.get("start"), "grid start")?,
                pitch_x: num(o, "pitch_x", "grid")?.unwrap_or(0.0),
                pitch_y: num(o, "pitch_y", "grid")?.unwrap_or(0.0),
                count_x: req_count(o, "count_x", "grid")?,
                count_y: req_count(o, "count_y", "grid")?,
                angle_deg: num(o, "angle", "grid")?.unwrap_or(0.0),
            })
        }
        "circle" => {
            let o = obj(body, "circle")?;
            let spacing = match (num(o, "angular_pitch", "circle")?, num(o, "span", "circle")?) {
                (None, None) => CircleSpacing::Full,
                (Some(p), None) => CircleSpacing::Pitch(p),
                (None, Some(s)) => CircleSpacing::Span(s),
                (Some(_), Some(_)) => return Err("a circle takes angular_pitch or span, not both".into()),
            };
            Ok(HolePattern::BoltCircle {
                center: xy(o.get("center"), "circle center")?,
                diameter: req_num(o, "diameter", "circle")?,
                count: req_count(o, "count", "circle")?,
                start_deg: num(o, "start_angle", "circle")?.unwrap_or(0.0),
                spacing,
            })
        }
        other => Err(format!("unknown pattern '{other}' - one of points, line, grid or circle")),
    }
}

fn positive(a: &Map<String, Value>, key: &str, ctx: &str) -> Result<f64, String> {
    let v = req_num(a, key, ctx)?;
    if v > 0.0 { Ok(v) } else { Err(format!("{ctx} \"{key}\" must be more than 0 mm (it is {v})")) }
}

fn holes(pts: &[Point2], radius: f64, top: f64, depth: f64, through: bool) -> Value {
    Value::Array(pts.iter().map(|p| json!({ "xy": [p.x, p.y], "radius": radius, "entry_z": top, "floor_z": top - depth, "through": through })).collect())
}

/// The seed's heights that measure from the model, measured from the
/// stock instead: a pattern stands on the stock, and a stock-only setup
/// has no model top to resolve. A stated height in params still wins.
fn stock_heights(params: &OpParams) -> Map<String, Value> {
    let Value::Object(tagged) = project::params_to_json(params) else {
        return Map::new();
    };
    let Some((_, Value::Object(fields))) = tagged.into_iter().next() else {
        return Map::new();
    };
    fields
        .into_iter()
        .filter_map(|(k, mut v)| {
            let to = match v.get("reference")?.as_str()? {
                "top_of_model" => "top_of_stock",
                "bottom_of_model" => "bottom_of_stock",
                _ => return None,
            };
            v["reference"] = json!(to);
            Some((k, v))
        })
        .collect()
}

/// The operation's create command: the kind's seed, the tool's helix,
/// slot and feed seeds (as `add_operation` does), the heights on the
/// stock, then the patch.
fn build(env: &Env, kind: usize, tool: &str, name: &str, patch: Map<String, Value>, holes: Value) -> Result<CreateOperation, String> {
    let mut params = seed(kind);
    if let Some(t) = model::find_by_name(&env.world, tool).and_then(|e| model::tool_of(&env.world, &e)) {
        operation::seed_helix_for_tool(&mut params, t.diameter);
        operation::seed_slot_for_tool(&mut params, t.diameter);
        crate::cutting_rows::seed_feed_for_tool(&env.world, &mut params, &t);
    }
    let named_entry = patch.contains_key("entry");
    let mut full = stock_heights(&params);
    full.extend(patch);
    full.insert("holes".into(), holes);
    let params = patched(&params, &Value::Object(full))?;
    CreateOperation::with_named_entry(&env.world, name, tool, SrcRef::Stock, params, named_entry).map_err(|e| e.to_string())
}

fn free_name(env: &Env, label: &str) -> String {
    (1..).map(|n| format!("{label}{n}")).find(|c| model::find_by_name(&env.world, c).is_none()).unwrap_or_else(|| label.to_owned())
}

fn add_hole_pattern(env: &mut Env, a: &Map<String, Value>) -> Result<Value, String> {
    let kind_word = req_str(a, "kind")?;
    let kind = kind_index(kind_word)?;
    if !matches!(seed(kind), OpParams::Drill { .. } | OpParams::Bore { .. } | OpParams::Thread { .. }) {
        return Err(format!("a hole pattern makes a Drill, Bore or Thread operation, not '{kind_word}'"));
    }
    let tool = req_str(a, "tool")?;
    let pts = pattern(a.get("pattern").ok_or("\"pattern\" is required")?)?.points()?;
    let diameter = positive(a, "diameter", "the hole")?;
    let depth = positive(a, "depth", "the hole")?;
    let through = match a.get("through") {
        None | Some(Value::Null) => false,
        Some(v) => v.as_bool().ok_or_else(|| format!("\"through\" must be true or false, got {v}"))?,
    };
    let setup = crate::setup::active(&env.world);
    let stock = model::stock_world_aabb_for(&env.world, setup.as_deref());
    let top = match (num(a, "top", "the hole")?, &stock) {
        (Some(t), _) => t,
        (None, Some(b)) if b.max.z.is_finite() => b.max.z,
        _ => return Err("the setup has no stock to start the holes on - give \"top\" (the Z where the holes start)".into()),
    };
    let patch = match a.get("params") {
        None | Some(Value::Null) => Map::new(),
        Some(p) => obj(p, "\"params\"")?.clone(),
    };
    if patch.contains_key("holes") {
        return Err("the pattern makes the holes - leave \"holes\" out of params".into());
    }
    let label = KindInfo::ALL.get(kind).map_or("Operation", |k| k.label);
    let name = opt_str(a, "name")?.map_or_else(|| free_name(env, label), str::to_owned);
    let radius = diameter / 2.0;

    let spot = match a.get("spot") {
        None | Some(Value::Null) => None,
        Some(s) => {
            let s = obj(s, "\"spot\"")?;
            let spot_tool = req_str(s, "tool").map_err(|e| format!("spot {e}"))?;
            let spot_depth = match num(s, "depth", "spot")? {
                Some(d) if d > 0.0 => d,
                _ => return Err("the spot depth must be stated and more than 0 mm (\"spot\": {\"depth\": ...})".into()),
            };
            let spot_name = opt_str(s, "name")?.map_or_else(|| format!("{name} spot"), str::to_owned);
            if spot_name == name {
                return Err(format!("the spot drill and the operation cannot both be named '{name}'"));
            }
            let drill = kind_index("Drill")?;
            let spot_patch = Map::from_iter([("cycle".to_owned(), json!("spot"))]);
            let cmd = build(env, drill, spot_tool, &spot_name, spot_patch, holes(&pts, radius, top, spot_depth, false))?;
            Some((spot_name, cmd))
        }
    };
    let main_holes = holes(&pts, radius, top, depth, through);
    // Both commands are built (and so checked) before either runs: a
    // refusal leaves the document and its history as they were.
    let main = build(env, kind, tool, &name, patch.clone(), main_holes.clone())?;

    env.history.begin_group(format!("Hole pattern: {name}"));
    let mut made = Vec::new();
    let main = match spot {
        None => main,
        Some((spot_name, cmd)) => {
            env.history.execute(&env.world, cmd);
            made.push(spot_name);
            // A create command takes its program position when it is
            // built: build the operation again so it runs after the spot.
            match build(env, kind, tool, &name, patch, main_holes) {
                Ok(again) => again,
                Err(e) => {
                    env.history.cancel_group();
                    env.history.undo(&env.world);
                    return Err(e);
                }
            }
        }
    };
    env.history.execute(&env.world, main);
    made.push(name.clone());
    env.history.commit_group();

    let mut out = json!({ "operations": made, "holes": pts.len(), "top": top, "status": "never" });
    if let Some(b) = stock {
        let off: Vec<String> = pts
            .iter()
            .filter(|p| p.x < b.min.x || p.x > b.max.x || p.y < b.min.y || p.y > b.max.y)
            .map(|p| format!("({:.3}, {:.3})", p.x, p.y))
            .collect();
        if !off.is_empty() {
            out["warning"] = json!(format!("{} hole(s) off the stock: {}", off.len(), off.join(", ")));
        }
    }
    super::edit::clearance_warning(env, &name, &mut out);
    Ok(out)
}

fn s_add() -> Value {
    let pt = json!({ "type": "array", "items": { "type": "number" }, "minItems": 2, "maxItems": 2 });
    json!({
        "type": "object",
        "properties": {
            "kind": { "type": "string", "enum": ["Drill", "Bore", "Thread"] },
            "tool": { "type": "string", "description": "A document tool by name." },
            "name": { "type": "string", "description": "The operation's name (default: the kind's next free name)." },
            "pattern": {
                "type": "object",
                "description": "Exactly one of points, line, grid or circle.",
                "properties": {
                    "points": { "type": "array", "items": pt },
                    "line": { "type": "object", "properties": {
                        "start": pt, "pitch": { "type": "number" }, "count": { "type": "integer", "minimum": 1 }, "angle": { "type": "number" } },
                        "required": ["start", "count"] },
                    "grid": { "type": "object", "properties": {
                        "start": pt, "pitch_x": { "type": "number" }, "pitch_y": { "type": "number" },
                        "count_x": { "type": "integer", "minimum": 1 }, "count_y": { "type": "integer", "minimum": 1 }, "angle": { "type": "number" } },
                        "required": ["start", "count_x", "count_y"] },
                    "circle": { "type": "object", "properties": {
                        "center": pt, "diameter": { "type": "number" }, "count": { "type": "integer", "minimum": 1 },
                        "start_angle": { "type": "number" }, "angular_pitch": { "type": "number" }, "span": { "type": "number" } },
                        "required": ["center", "diameter", "count"] }
                }
            },
            "diameter": { "type": "number", "description": "Hole diameter, mm." },
            "depth": { "type": "number", "description": "Hole depth below top, mm." },
            "top": { "type": "number", "description": "Z where the holes start (default: the stock top)." },
            "through": { "type": "boolean" },
            "params": { "type": "object", "description": "A patch of the operation's other fields (not holes)." },
            "spot": { "type": "object", "properties": {
                "tool": { "type": "string" }, "depth": { "type": "number" }, "name": { "type": "string" } },
                "required": ["tool", "depth"] }
        },
        "required": ["kind", "tool", "pattern", "diameter", "depth"]
    })
}
