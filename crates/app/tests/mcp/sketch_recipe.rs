//! #3194: quick ops - a rectangle, circle, obround or polygon stated by
//! its dimensions becomes a profile on the stock, and a 2D Contour on it,
//! in one undo step - through the undoable `AddQuickOp`,
//! `CreateSketchRecipe` and `EditSketchRecipe` and through the MCP tools
//! `add_quick_op`, `add_sketch_recipe` and `edit_sketch_recipe`. The
//! generate, verify and post with no 3D model, the edit in place, the
//! `.camproj` round trip and the refusals by name are pinned here.
//!
//! Provenance: our own tests over our own commands and tools.
//!
//! Gate map (M4.4): covers app::commands (sketch_recipe), app::mcp
//! (sketch_recipe_tools), app::project (sketch_dto, recipe_dto),
//! app::script (dxf profile_view).

use serde_json::{json, Value};

use camcraft_app::mcp::Server;

struct Client {
    server: Server,
    next: i64,
}

impl Client {
    fn new() -> Self {
        Client { server: Server::new(), next: 1 }
    }

    fn call(&mut self, name: &str, args: Value) -> (bool, Value) {
        let id = self.next;
        self.next += 1;
        let msg = json!({ "jsonrpc": "2.0", "id": id, "method": "tools/call", "params": { "name": name, "arguments": args } });
        let reply = self.server.handle(&msg).expect("a request gets a response");
        let result = &reply["result"];
        let is_err = result["isError"] == json!(true);
        let body = if is_err { result["content"][0]["text"].clone() } else { result["structuredContent"].clone() };
        (is_err, body)
    }

    fn ok(&mut self, name: &str, args: Value) -> Value {
        let (err, body) = self.call(name, args);
        assert!(!err, "{name} refused: {body}");
        body
    }

    fn refused(&mut self, name: &str, args: Value) -> String {
        let (err, body) = self.call(name, args);
        assert!(err, "{name} should have refused, got {body}");
        body.as_str().unwrap_or_default().to_owned()
    }

    fn names(&mut self, list: &str, key: &str) -> Vec<String> {
        let job = self.ok("get_job", json!({}));
        job[list].as_array().map_or_else(Vec::new, |rows| rows.iter().filter_map(|p| p[key].as_str().map(str::to_owned)).collect())
    }

    fn profiles(&mut self) -> Vec<String> {
        self.names("profiles", "profile")
    }

    fn operations(&mut self) -> Vec<String> {
        self.names("operations", "name")
    }

    /// A 200 x 150 x 10 plate and a 4 mm end mill.
    fn plate() -> Self {
        let mut c = Client::new();
        c.ok("set_stock", json!({ "box": { "dx": 200.0, "dy": 150.0, "dz": 10.0 } }));
        c.ok("add_tool", json!({ "catalog": "EM-0400-2FL" }));
        c
    }
}

fn tmp(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("camcraft-3194-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

#[test]
fn each_quick_op_shape_generates_verifies_and_posts_with_no_model() {
    let shapes = [
        json!({ "shape": "rectangle", "width": 80.0, "height": 50.0, "corner_radius": 6.0 }),
        json!({ "shape": "circle", "diameter": 40.0 }),
        json!({ "shape": "obround", "length": 70.0, "width": 20.0 }),
        json!({ "shape": "polygon", "sides": 6, "diameter": 46.0, "across": "flats" }),
    ];
    let dir = tmp("shapes");
    for (i, recipe) in shapes.into_iter().enumerate() {
        let mut c = Client::plate();
        let r = c.ok("add_quick_op", json!({ "recipe": recipe, "tool": "EM-0400-2FL", "depth": 3.0, "position": [100.0, 75.0] }));
        assert_eq!(r["operation"], json!("Contour1"));
        assert_eq!(r["profile"]["recipe"], recipe, "the profile states its recipe");
        assert_eq!(r["profile"]["chains"], json!({ "closed": 1, "open": 0, "rejected": 0 }));
        c.ok("generate", json!({}));
        let verdict = c.ok("verify", json!({}));
        assert_eq!(verdict["ok"], json!(true), "{recipe}: {verdict}");
        let out = dir.join(format!("shape{i}.nc"));
        c.ok("post", json!({ "output": out.display().to_string(), "post": "Fanuc" }));
        let nc = std::fs::read_to_string(&out).expect("the program is written");
        // The stock is 10 mm high on its origin, so 3 mm deep is Z7 - and never lower.
        let feed_z: Vec<f64> = nc
            .lines()
            .filter(|l| ["G01", "G02", "G03", "G1 ", "G2 ", "G3 "].iter().any(|g| l.contains(g)))
            .filter_map(|l| l.split_whitespace().find_map(|w| w.strip_prefix('Z')).and_then(|z| z.parse::<f64>().ok()))
            .collect();
        let lowest = feed_z.iter().copied().fold(f64::INFINITY, f64::min);
        assert!((lowest - 7.0).abs() < 1e-6, "{recipe}: cuts 3 mm deep (Z7), lowest feed Z {lowest}:\n{nc}");
        if i < 3 {
            let arcs = nc.lines().any(|l| l.contains("G2") || l.contains("G3") || l.contains("G02") || l.contains("G03"));
            assert!(arcs, "{recipe}: exact arcs post as G2/G3:\n{nc}");
        }
    }
}

#[test]
fn a_quick_op_undoes_and_redoes_in_one_step() {
    let mut c = Client::plate();
    let before = c.ok("get_job", json!({}));
    c.ok("add_quick_op", json!({ "recipe": { "shape": "circle", "diameter": 30.0 }, "tool": "EM-0400-2FL", "depth": 2.0 }));
    assert_eq!(c.profiles(), ["Circle"]);
    assert_eq!(c.operations(), ["Contour1"]);
    c.ok("undo", json!({}));
    assert!(c.profiles().is_empty() && c.operations().is_empty(), "one undo removes the profile and the contour");
    assert_eq!(c.ok("get_job", json!({}))["tools"], before["tools"], "the tool stays");
    c.ok("redo", json!({}));
    assert_eq!(c.profiles(), ["Circle"]);
    assert_eq!(c.operations(), ["Contour1"]);
}

#[test]
fn a_refused_quick_op_places_nothing_and_leaves_no_history() {
    let mut c = Client::plate();
    let why = c.refused("add_quick_op", json!({ "recipe": { "shape": "rectangle", "width": 10.0, "height": 4.0, "corner_radius": 3.0 }, "tool": "EM-0400-2FL" }));
    assert!(why.contains("larger than half the smaller side"), "{why}");
    let why = c.refused("add_quick_op", json!({ "recipe": { "shape": "circle", "diameter": 30.0 }, "tool": "no such tool" }));
    assert!(why.contains("no such tool"), "the operation's own gate refuses: {why}");
    let why = c.refused("add_quick_op", json!({ "recipe": { "shape": "circle", "diameter": 30.0, "radius": 4.0 }, "tool": "EM-0400-2FL" }));
    assert!(why.contains("radius"), "an unknown key is refused by name: {why}");
    let why = c.refused("add_quick_op", json!({ "recipe": { "shape": "circle", "diameter": 30.0 }, "tool": "EM-0400-2FL", "depth": -1.0 }));
    assert!(why.contains("\"depth\" must be a positive number"), "{why}");
    assert!(c.profiles().is_empty() && c.operations().is_empty(), "nothing is placed");
    c.ok("undo", json!({}));
    assert!(c.ok("get_job", json!({}))["tools"].as_array().is_none_or(Vec::is_empty), "the only undo step left is the tool");
}

#[test]
fn editing_a_recipe_regenerates_the_outline_and_stales_its_operation() {
    let mut c = Client::plate();
    c.ok("add_quick_op", json!({ "recipe": { "shape": "rectangle", "width": 60.0, "height": 40.0 }, "tool": "EM-0400-2FL", "depth": 2.0, "name": "Plate" }));
    c.ok("generate", json!({}));
    assert_eq!(c.ok("get_job", json!({}))["operations"][0]["status"], json!("ok"));

    let r = c.ok("edit_sketch_recipe", json!({ "name": "Plate", "recipe": { "shape": "obround", "length": 90.0, "width": 30.0 } }));
    assert_eq!(r["bounds"], json!({ "min": [-45.0, -15.0], "max": [45.0, 15.0] }));
    assert_eq!(r["recipe"]["shape"], json!("obround"));
    let job = c.ok("get_job", json!({}));
    assert_eq!(job["operations"][0]["status"], json!("stale"), "the edit stales the contour on the profile");

    c.ok("undo", json!({}));
    let job = c.ok("get_job", json!({}));
    assert_eq!(job["profiles"][0]["bounds"], json!({ "min": [-30.0, -20.0], "max": [30.0, 20.0] }), "undo restores the rectangle");
    assert_eq!(job["profiles"][0]["recipe"]["shape"], json!("rectangle"));

    let why = c.refused("edit_sketch_recipe", json!({ "name": "Plate", "recipe": { "shape": "circle", "diameter": 0.0 } }));
    assert!(why.contains("diameter must be at least 0.1 mm"), "{why}");
    let why = c.refused("edit_sketch_recipe", json!({ "name": "Nope", "recipe": { "shape": "circle", "diameter": 5.0 } }));
    assert!(why.contains("Nope"), "{why}");
    c.ok("insert_text", json!({ "text": "A", "height": 5.0 }));
    let why = c.refused("edit_sketch_recipe", json!({ "name": "Text A", "recipe": { "shape": "circle", "diameter": 5.0 } }));
    assert!(why.contains("has no recipe to edit"), "{why}");
}

#[test]
fn a_recipe_profile_round_trips_the_project_file_and_edits_after_load() {
    let dir = tmp("roundtrip");
    let file = dir.join("recipe.camproj");
    let mut c = Client::plate();
    let recipe = json!({ "shape": "polygon", "sides": 5, "diameter": 40.0, "across": "corners" });
    c.ok("add_sketch_recipe", json!({ "recipe": recipe, "position": [50.0, 40.0], "angle": 30.0, "name": "Star" }));
    c.ok("save_project", json!({ "path": file.display().to_string() }));

    let text = std::fs::read_to_string(&file).expect("saved");
    assert!(text.contains("\"version\": 85") || text.contains("\"version\":85"), "v85 is written");

    let mut d = Client::new();
    d.ok("open_project", json!({ "path": file.display().to_string() }));
    let job = d.ok("get_job", json!({}));
    let star = &job["profiles"][0];
    assert_eq!(star["recipe"], recipe);
    assert_eq!(star["placement"], json!({ "x": 50.0, "y": 40.0, "angle": 30.0 }));
    d.ok("edit_sketch_recipe", json!({ "name": "Star", "recipe": { "shape": "circle", "diameter": 12.0 } }));

    // A stored recipe that does not give the stored outline is a corrupt file.
    let mut doc: Value = serde_json::from_str(&text).expect("json");
    let row = doc["entities"].as_array_mut().expect("entities").iter_mut().find(|e| e["name"] == json!("Star")).expect("the profile row");
    row["sketch"]["recipe"]["diameter"] = json!(41.0);
    let bad = dir.join("bad.camproj");
    std::fs::write(&bad, serde_json::to_string(&doc).expect("json")).expect("write");
    let why = Client::new().refused("open_project", json!({ "path": bad.display().to_string() }));
    assert!(why.contains("'Star'") && why.contains("is not the outline of its polygon recipe"), "{why}");
}
