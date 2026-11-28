//! #3406: the MCP tools of part-in-spindle mode - `set_part_in_spindle`
//! and `set_table_station` run the undoable commands, `get_job` shows the
//! record, a milling operation in the setup is refused by name, and bad
//! input is a tool error in the command's own words.
//! Provenance: our own tool surface over this repo's setup commands.
//!
//! Gate map: covers app::mcp::part_in_spindle_tools, app::mcp::job.

use super::Client;
use serde_json::json;

#[test]
fn part_in_spindle_tools_configure_the_mode_and_refuse_milling() {
    let mut c = Client::new();
    c.ok("set_stock", json!({ "box": { "dx": 20.0, "dy": 20.0, "dz": 30.0 } }));
    c.ok("add_tool", json!({ "catalog": "TN-1600-0R8" }));
    c.ok("add_tool", json!({ "catalog": "EM-0400-2FL" }));
    c.ok("add_setup", json!({}));
    let names: Vec<String> = c.ok("get_job", json!({}))["setups"].as_array().into_iter().flatten()
        .filter_map(|s| s["name"].as_str().map(str::to_owned)).collect();
    assert_eq!(names, ["Setup1"]);

    assert!(c.refused("set_part_in_spindle", json!({ "h_register": 0, "stick_out": 40.0, "collet_od": 30.0 })).contains("register"));
    let on = c.ok("set_part_in_spindle", json!({ "h_register": 255, "stick_out": 40.0, "collet_od": 30.0 }));
    assert_eq!(on["setup"], json!("Setup1"));
    let st = c.ok("set_table_station", json!({ "tool": "TN-1600-0R8", "work_offset": 2, "side": "-X", "spindle": "M3" }));
    assert_eq!(st["part_in_spindle"]["stations"][0]["work_offset"], json!(2));
    let why = c.refused("set_table_station", json!({ "tool": "EM-0400-2FL", "work_offset": 2, "side": "+X", "spindle": "M4" }));
    assert!(why.contains("work offset 2"), "{why}");
    assert!(c.refused("set_table_station", json!({ "tool": "EM-0400-2FL", "work_offset": 3, "side": "Y", "spindle": "M4" })).contains("side"));

    let job = c.ok("get_job", json!({}));
    let pis = &job["setups"][0]["part_in_spindle"];
    assert_eq!(pis["holding"]["h_register"], json!(255), "{job}");
    assert_eq!(pis["stations"][0]["side"], json!("-X"));
    assert_eq!(pis["stations"][0]["spindle"], json!("M3"));

    let why = c.refused("add_operation", json!({ "kind": "Pocket", "tool": "EM-0400-2FL", "name": "P1", "params": {
        "top_z": { "reference": "fixed", "offset": 30.0 }, "bottom_z": { "reference": "fixed", "offset": 26.0 }
    } }));
    assert!(why.contains("part-in-spindle") && why.contains("milling"), "milling is refused by name: {why}");

    c.ok("undo", json!({}));
    assert!(c.ok("get_job", json!({}))["setups"][0]["part_in_spindle"]["stations"].as_array().is_some_and(Vec::is_empty));
    c.ok("undo", json!({}));
    assert!(c.ok("get_job", json!({}))["setups"][0]["part_in_spindle"].is_null());
    assert!(c.refused("set_part_in_spindle", json!({ "enabled": false })).contains("states no"));
}
