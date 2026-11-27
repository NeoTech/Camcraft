//! The MCP sketch-recipe tools (#3194): `add_sketch_recipe` places a
//! shape stated by its dimensions as a profile through the undoable
//! `CreateSketchRecipe`, `edit_sketch_recipe` changes those dimensions
//! through `EditSketchRecipe`, and `add_quick_op` places a recipe profile
//! and a 2D Contour on it through `AddQuickOp`. Each is one History step,
//! and every refusal is the command's own text, returned as a tool error.
//!
//! Provenance: our own tool surface over this repo's recipe commands;
//! plain machining terms only, no vendor material consulted.

use super::edit::{patched, seed};
use super::setup_tools::{cmd_err, obj, opt_nums};
use super::tools::{opt_str, req_str, Tool};
use crate::commands::{AddQuickOp, CreateSketchRecipe, EditSketchRecipe, RecipePlacement};
use crate::model;
use crate::operation::{Height, HeightReference, OpParams, OpType};
use crate::project::RecipeDto;
use crate::script::{dxf, Env};
use crate::sketch_recipe::SketchRecipe;

use serde_json::{json, Map, Value};

const RECIPE_HELP: &str = "The shape, in mm, centred on position: {\"shape\": \"rectangle\", \"width\", \"height\", \"corner_radius\" (default 0)}, {\"shape\": \"circle\", \"diameter\"}, {\"shape\": \"obround\", \"length\", \"width\"} (round ends, length along X) or {\"shape\": \"polygon\", \"sides\" (3 to 1000), \"diameter\", \"across\": \"flats\" or \"corners\"} (one flat at the bottom). Each size is 0.1 to 100000 mm.";

pub(super) const TOOLS: &[Tool] = &[
    Tool {
        name: "add_sketch_recipe",
        description: "Place a rectangle, circle, obround or regular polygon stated by its dimensions as a profile on the top face of a setup's stock - the active setup by default. The outline is exact lines and arcs on the layer \"outline\", centred on position. The result is a profile like one from import_dxf: move_dxf_profile moves it, edit_sketch_recipe changes its dimensions, and get_job lists it under \"profiles\" with its \"recipe\". One undo step. Refused by name, never clamped: a size that is not finite or is outside the limits, a corner radius larger than half the smaller side, an obround wider than long, fewer than 3 sides, a setup with no stock, an unknown setup, a name in use.",
        schema: s_add_sketch_recipe,
        run: add_sketch_recipe,
    },
    Tool {
        name: "edit_sketch_recipe",
        description: "Change the dimensions (or the shape) of a profile made by add_sketch_recipe or add_quick_op. The profile keeps its name and placement; its outline is made again, and every operation on it goes stale. One undo step. Refused by name: an unknown name, a profile from a drawing or a text (it has no recipe), a bad dimension.",
        schema: s_edit_sketch_recipe,
        run: edit_sketch_recipe,
    },
    Tool {
        name: "add_quick_op",
        description: "Place a recipe profile (as add_sketch_recipe does, on the active setup's stock) and add a 2D Contour on it with the given tool, as one undo step. depth (mm below the top of the stock) sets the contour's top at the stock top and its bottom at that depth; params is a patch over the 2D Contour defaults (the fields list_operation_types shows), applied after depth. Not generated until generate runs. Refused by name, and nothing is placed: any refusal of add_sketch_recipe or add_operation.",
        schema: s_add_quick_op,
        run: add_quick_op,
    },
];

fn recipe(a: &Map<String, Value>) -> Result<SketchRecipe, String> {
    let v = a.get("recipe").ok_or("\"recipe\" is required")?;
    let dto: RecipeDto = serde_json::from_value(v.clone()).map_err(|e| format!("\"recipe\" is not a usable shape: {e}"))?;
    Ok(dto.into())
}

fn placement(a: &Map<String, Value>) -> Result<RecipePlacement, String> {
    let [x, y] = opt_nums::<2>(a, "position")?.unwrap_or_default();
    let angle_deg = match a.get("angle") {
        None | Some(Value::Null) => 0.0,
        Some(v) => v.as_f64().ok_or_else(|| format!("\"angle\" must be a number in degrees, got {v}"))?,
    };
    Ok(RecipePlacement { x, y, angle_deg, name: opt_str(a, "name")?.map(str::to_owned) })
}

fn add_sketch_recipe(env: &mut Env, a: &Map<String, Value>) -> Result<Value, String> {
    let cmd = CreateSketchRecipe::new(&env.world, opt_str(a, "setup")?, recipe(a)?, &placement(a)?).map_err(cmd_err)?;
    let name = cmd.name().to_owned();
    env.history.execute(&env.world, cmd);
    dxf::profile_view(&env.world, &name).ok_or_else(|| format!("'{name}' is not a profile"))
}

fn edit_sketch_recipe(env: &mut Env, a: &Map<String, Value>) -> Result<Value, String> {
    let name = req_str(a, "name")?;
    let cmd = EditSketchRecipe::new(&env.world, name, recipe(a)?).map_err(cmd_err)?;
    env.history.execute(&env.world, cmd);
    dxf::profile_view(&env.world, name).ok_or_else(|| format!("'{name}' is not a profile"))
}

fn add_quick_op(env: &mut Env, a: &Map<String, Value>) -> Result<Value, String> {
    let tool = req_str(a, "tool")?;
    let operation = match opt_str(a, "operation")? {
        Some(n) => n.to_owned(),
        None => (1..)
            .map(|n| format!("Contour{n}"))
            .find(|c| model::find_by_name(&env.world, c).is_none())
            .unwrap_or_else(|| "Contour".to_owned()),
    };
    let mut params = seed(OpType::Contour.ordinal());
    if let Some(t) = model::find_by_name(&env.world, tool).and_then(|e| model::tool_of(&env.world, &e)) {
        crate::cutting_rows::seed_feed_for_tool(&env.world, &mut params, &t);
    }
    if let Some(v) = a.get("depth").filter(|v| !v.is_null()) {
        let depth = v.as_f64().filter(|d| d.is_finite() && *d > 0.0).ok_or_else(|| format!("\"depth\" must be a positive number of mm, got {v}"))?;
        if let OpParams::Contour { top_z, bottom_z, .. } = &mut params {
            *top_z = Height::of(HeightReference::TopOfStock, 0.0);
            *bottom_z = Height::of(HeightReference::TopOfStock, -depth);
        }
    }
    if let Some(patch) = a.get("params") {
        params = patched(&params, patch)?;
    }
    let cmd = AddQuickOp::new(&env.world, recipe(a)?, &placement(a)?, &operation, tool, params).map_err(cmd_err)?;
    let profile = cmd.profile().to_owned();
    env.history.execute(&env.world, cmd);
    let mut out = json!({ "operation": operation, "kind": "2D Contour", "status": "never" });
    out["profile"] = dxf::profile_view(&env.world, &profile).unwrap_or(Value::Null);
    super::edit::clearance_warning(env, &operation, &mut out);
    Ok(out)
}

fn recipe_props() -> Value {
    json!({
        "recipe": { "type": "object", "description": RECIPE_HELP },
        "position": { "type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2,
            "description": "[x, y] of the shape's centre on the stock's top face, in mm from the stock origin (default [0, 0])." },
        "angle": { "type": "number", "description": "Turn of the shape about the face normal in degrees, counterclockwise seen from above (default 0)." },
        "name": { "type": "string", "description": "The profile's name; omitted: the shape's name (\"Rectangle\"), made unique." }
    })
}

fn s_add_sketch_recipe() -> Value {
    let mut props = recipe_props();
    props["setup"] = json!({ "type": "string", "description": "The setup whose stock takes the profile; omitted: the active setup." });
    obj(props, &["recipe"])
}

fn s_edit_sketch_recipe() -> Value {
    obj(
        json!({
            "name": { "type": "string", "description": "A recipe profile's name (get_job lists it under \"profiles\" with its \"recipe\")." },
            "recipe": { "type": "object", "description": RECIPE_HELP }
        }),
        &["name", "recipe"],
    )
}

fn s_add_quick_op() -> Value {
    let mut props = recipe_props();
    props["tool"] = json!({ "type": "string", "description": "A document tool's name (add_tool first)." });
    props["operation"] = json!({ "type": "string", "description": "The operation's name; omitted: \"Contour<n>\", made unique." });
    props["depth"] = json!({ "type": "number", "description": "The cut depth in mm below the top of the stock." });
    props["params"] = json!({ "type": "object", "description": "A JSON merge patch over the 2D Contour defaults (e.g. side, stepdown)." });
    obj(props, &["recipe", "tool"])
}
