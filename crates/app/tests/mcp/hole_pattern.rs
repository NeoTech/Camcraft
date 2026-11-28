//! #3195: `add_hole_pattern` - a Drill, Bore or Thread operation whose
//! holes come from numbers (a point list, a line, a grid, a full or
//! partial bolt-hole circle) on a stock-only setup, with no 3D model.
//! The operation and its optional spot drill are one undo step; the
//! holes generate, verify against the stock, post standard canned
//! cycles and round-trip the project file. Bad numbers are refused by
//! name and leave the document as it was.
//!
//! Gate map: covers app::mcp::hole_pattern_tools,
//! camcraft_geometry::hole_pattern.

use super::{tmp_dir, Client};
use serde_json::{json, Value};

fn stock_only() -> Client {
    let mut c = Client::new();
    c.request("initialize", json!({ "protocolVersion": "2025-06-18" }));
    c.ok("set_stock", json!({ "box": { "dx": 100.0, "dy": 80.0, "dz": 20.0 } }));
    c.ok("add_tool", json!({ "catalog": "DR-0600" }));
    c
}

fn ops(c: &mut Client) -> Vec<Value> {
    c.ok("get_job", json!({}))["operations"].as_array().cloned().unwrap_or_default()
}

fn holes_of(op: &Value, kind: &str) -> Vec<Value> {
    op["params"][kind]["holes"].as_array().cloned().unwrap_or_default()
}

fn post(c: &mut Client, tag: &str) -> String {
    let dir = tmp_dir(tag);
    let nc = dir.join("out.nc");
    c.ok("post", json!({ "output": nc.display().to_string(), "post": "Fanuc" }));
    let text = std::fs::read_to_string(&nc).expect("the program is written");
    let _ = std::fs::remove_dir_all(&dir);
    text
}

/// Each X/Y pair that a canned-cycle line or a modal hole line posts.
fn has_xy(program: &str, x: &str, y: &str) -> bool {
    program.lines().any(|l| {
        let w: Vec<&str> = l.split_whitespace().collect();
        w.contains(&format!("X{x}").as_str()) && w.contains(&format!("Y{y}").as_str())
    })
}

#[test]
fn a_bolt_circle_with_a_spot_drill_is_one_undo_step_and_posts_g81() {
    let mut c = stock_only();
    c.ok("add_tool", json!({ "catalog": "DR-0400" }));
    let add = c.ok("add_hole_pattern", json!({
        "kind": "Drill", "tool": "DR-0600", "name": "BHC",
        "pattern": { "circle": { "center": [0.0, 0.0], "diameter": 60.0, "count": 4, "start_angle": 0.0 } },
        "diameter": 6.0, "depth": 10.0,
        "params": { "cycle": "drill" },
        "spot": { "tool": "DR-0400", "depth": 1.5 }
    }));
    assert_eq!(add["operations"], json!(["BHC spot", "BHC"]), "{add}");
    assert_eq!(add["holes"], json!(4), "{add}");
    assert_eq!(add["top"], json!(20.0), "the holes start on the stock's top: {add}");
    assert!(add.get("warning").is_none(), "every hole is on the stock: {add}");

    let list = ops(&mut c);
    assert_eq!(list.len(), 2, "{list:?}");
    let spot = holes_of(&list[0], "drill");
    let main = holes_of(&list[1], "drill");
    assert_eq!(main.len(), 4);
    assert_eq!(main[0], json!({ "xy": [30.0, 0.0], "radius": 3.0, "entry_z": 20.0, "floor_z": 10.0, "through": false, "stepped": false }));
    assert_eq!(main[1]["xy"], json!([0.0, 30.0]));
    assert_eq!(spot.len(), 4);
    assert_eq!(spot[2]["floor_z"], json!(18.5), "the spot drill stops at its own depth");
    assert_eq!(list[0]["params"]["drill"]["cycle"], json!("spot"), "{}", list[0]);

    // One undo removes both operations; one redo puts both back.
    c.ok("undo", json!({}));
    assert!(ops(&mut c).is_empty(), "one undo step");
    c.ok("redo", json!({}));
    assert_eq!(ops(&mut c).len(), 2, "one redo step");

    c.ok("generate", json!({}));
    let v = c.ok("verify", json!({}));
    assert_eq!(v["ok"], json!(true), "the pattern verifies against the stock: {v}");
    let program = post(&mut c, "bhc");
    assert!(program.lines().any(|l| l.split_whitespace().any(|w| w == "G81")), "{program}");
    for (x, y) in [("30.", "0."), ("0.", "30."), ("-30.", "0."), ("0.", "-30.")] {
        assert!(has_xy(&program, x, y), "a hole at X{x} Y{y}:\n{program}");
    }
}

#[test]
fn a_grid_pecks_with_g83_and_round_trips_the_project_file() {
    let mut c = stock_only();
    c.ok("add_tool", json!({ "catalog": "DR-0400" }));
    // A long-series drill: the catalog's 12 mm flute does not reach 18 mm.
    c.ok("edit_tool", json!({ "name": "DR-0400", "patch": { "flute_len": 22.0 } }));
    // 18 mm deep with a 4 mm drill is past 4 x D: the Auto cycle is G83.
    let add = c.ok("add_hole_pattern", json!({
        "kind": "Drill", "tool": "DR-0400", "name": "GRID",
        "pattern": { "grid": { "start": [-30.0, -20.0], "pitch_x": 30.0, "pitch_y": 40.0, "count_x": 3, "count_y": 2 } },
        "diameter": 4.0, "depth": 18.0
    }));
    assert_eq!(add["holes"], json!(6), "{add}");
    c.ok("generate", json!({}));
    assert_eq!(c.ok("verify", json!({}))["ok"], json!(true));
    let program = post(&mut c, "grid");
    assert!(program.lines().any(|l| l.split_whitespace().any(|w| w == "G83")), "{program}");
    assert!(has_xy(&program, "30.", "20."), "{program}");

    let dir = tmp_dir("hole-pattern-file");
    let file = dir.join("grid.camproj");
    c.ok("save_project", json!({ "path": file.display().to_string() }));
    let mut d = Client::new();
    d.ok("open_project", json!({ "path": file.display().to_string() }));
    let before = holes_of(&ops(&mut c)[0], "drill");
    let after = holes_of(&ops(&mut d)[0], "drill");
    assert_eq!(before, after, "the holes load as they were saved");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_line_and_a_point_list_fill_bore_and_thread_holes() {
    let mut c = stock_only();
    c.ok("add_tool", json!({ "catalog": "EM-0600-4FL" }));
    c.ok("add_tool", json!({ "catalog": "TM-0600-P125" }));
    let bore = c.ok("add_hole_pattern", json!({
        "kind": "Bore", "tool": "EM-0600-4FL",
        "pattern": { "line": { "start": [-30.0, 0.0], "pitch": 20.0, "count": 4, "angle": 0.0 } },
        "diameter": 12.0, "depth": 8.0, "top": 20.0
    }));
    assert_eq!(bore["holes"], json!(4), "{bore}");
    let thread = c.ok("add_hole_pattern", json!({
        "kind": "Thread", "tool": "TM-0600-P125",
        "pattern": { "points": [[-30.0, 25.0], [30.0, 25.0]] },
        "diameter": 6.647, "depth": 10.0, "through": true, "params": { "pitch": 1.25 }
    }));
    assert_eq!(thread["holes"], json!(2), "{thread}");
    let list = ops(&mut c);
    assert_eq!(holes_of(&list[0], "bore").len(), 4);
    let t = holes_of(&list[1], "thread");
    assert_eq!(t[1]["xy"], json!([30.0, 25.0]));
    assert_eq!(t[1]["through"], json!(true));
}

#[test]
fn a_tapped_line_after_its_tap_drill_posts_g84() {
    let mut c = stock_only();
    c.ok("add_tool", json!({ "catalog": "DR-0500" }));
    c.ok("add_tool", json!({ "catalog": "TAP-M6" }));
    let line = json!({ "line": { "start": [-20.0, 0.0], "pitch": 20.0, "count": 3 } });
    c.ok("add_hole_pattern", json!({
        "kind": "Drill", "tool": "DR-0500", "name": "TAPDRILL", "pattern": line, "diameter": 5.0, "depth": 15.0,
        "params": { "cycle": "drill" }
    }));
    c.ok("add_hole_pattern", json!({
        "kind": "Drill", "tool": "TAP-M6", "name": "TAP", "pattern": line, "diameter": 5.0, "depth": 12.0,
        "params": { "cycle": "tap" }
    }));
    c.ok("generate", json!({}));
    let v = c.ok("verify", json!({}));
    assert_eq!(v["ok"], json!(true), "{v}");
    let program = post(&mut c, "tap");
    assert!(program.lines().any(|l| l.split_whitespace().any(|w| w == "G84")), "{program}");
    assert!(has_xy(&program, "20.", "0."), "{program}");
}

#[test]
fn bad_numbers_are_refused_by_name_and_change_nothing() {
    let mut c = stock_only();
    let base = json!({ "kind": "Drill", "tool": "DR-0600", "diameter": 6.0, "depth": 5.0 });
    let with = |extra: Value| {
        let mut a = base.clone();
        for (k, v) in extra.as_object().cloned().unwrap_or_default() {
            a[k] = v;
        }
        a
    };
    let cases = [
        (json!({ "pattern": { "circle": { "center": [0.0, 0.0], "diameter": -5.0, "count": 4 } } }), "diameter"),
        (json!({ "pattern": { "line": { "start": [0.0, 0.0], "pitch": 0.0, "count": 3 } } }), "pitch"),
        (json!({ "pattern": { "points": [] } }), "no points"),
        (json!({ "pattern": { "points": [[0.0, 0.0]], "grid": {} } }), "exactly one"),
        (json!({ "pattern": { "points": [[0.0, 0.0]] }, "depth": 0.0 }), "depth"),
        (json!({ "pattern": { "points": [[0.0, 0.0]] }, "diameter": 0.0 }), "diameter"),
        (json!({ "pattern": { "points": [[0.0, 0.0]] }, "params": { "holes": [] } }), "holes"),
        (json!({ "pattern": { "points": [[0.0, 0.0]] }, "kind": "Pocket" }), "Drill, Bore or Thread"),
        (json!({ "pattern": { "points": [[0.0, 0.0]] }, "spot": { "tool": "DR-0600" } }), "spot depth"),
        (json!({ "pattern": { "points": [[0.0, 0.0]] }, "spot": { "tool": "NOPE", "depth": 1.0 } }), "NOPE"),
    ];
    for (extra, word) in cases {
        let why = c.refused("add_hole_pattern", with(extra.clone()));
        assert!(why.contains(word), "{extra}: '{why}' should name '{word}'");
        assert!(ops(&mut c).is_empty(), "a refusal adds nothing: {extra}");
    }
    // A hole off the stock is made, with a warning that names it.
    let off = c.ok("add_hole_pattern", with(json!({ "pattern": { "points": [[0.0, 0.0], [90.0, 0.0]] } })));
    let warning = off["warning"].as_str().unwrap_or_default();
    assert!(warning.contains("(90.000, 0.000)"), "{off}");
}

#[test]
fn no_stock_and_no_top_is_refused_by_name() {
    let mut c = Client::new();
    c.request("initialize", json!({ "protocolVersion": "2025-06-18" }));
    c.ok("add_tool", json!({ "catalog": "DR-0600" }));
    let why = c.refused("add_hole_pattern", json!({
        "kind": "Drill", "tool": "DR-0600", "pattern": { "points": [[0.0, 0.0]] }, "diameter": 6.0, "depth": 5.0
    }));
    assert!(why.contains("top"), "{why}");
}
