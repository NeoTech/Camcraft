//! #3792: `add_tool` says at once when the new tool's speed is outside the
//! mounted machine's spindle range. The tool keeps its stated speed. The
//! machine check at verify still holds the post, and its problem line
//! names the same fix.
//! Provenance: our own command and its MCP contract.
//!
//! Gate map: covers app::mcp (add_tool), app::commands::tool (AddTool),
//! app::posting (the machine check's problem line).

use super::Client;
use serde_json::{json, Value};

const MACHINE: &str = "Tormach 770MX (ML)";
const FIX: &str = "set the tool's speed (edit_tool) or apply_cutting_data";

/// The catalog's 3 mm end mill, at `rpm`.
fn end_mill(rpm: f64) -> Value {
    json!({
        "kind": "end_mill", "diameter": 3.0, "corner_r": 0.0, "tip_angle": 0.0, "pitch": 0.0,
        "nose_radius": 0.0, "width": 0.0, "flute_len": 9.0, "shank_d": 4.0, "shank_len": 40.0,
        "holder_d": 44.0, "holder_len": 35.0, "material_group": "aluminum", "rpm": rpm,
        "feed": 900.0, "tool_material": "carbide", "flutes": 2
    })
}

#[test]
fn add_tool_warns_of_a_speed_above_the_mounted_machine() {
    let mut c = Client::new();
    c.ok("load_machine", json!({ "builtin": MACHINE }));
    let added = c.ok("add_tool", json!({ "catalog": "EM-0300-2FL" }));
    assert_eq!(
        added["warning"],
        json!(format!("EM-0300-2FL runs 15000 rpm, above the {MACHINE}'s 10000 - the post will be held; {FIX}")),
        "{added}"
    );
    // The holder note stays its own line.
    assert!(added["note"].as_str().is_some_and(|n| n.contains("BT30 ER20")), "{added}");
    // The add does not change the stated speed.
    let job = c.ok("get_job", json!({})).to_string();
    assert!(job.contains("15000"), "the tool keeps its 15000 rpm: {job}");
}

#[test]
fn add_tool_warns_of_a_speed_below_the_mounted_machine() {
    let mut c = Client::new();
    c.ok("load_machine", json!({ "builtin": MACHINE }));
    let added = c.ok("add_tool", json!({ "name": "Slow", "definition": end_mill(100.0) }));
    assert_eq!(
        added["warning"],
        json!(format!("Slow runs 100 rpm, below the {MACHINE}'s 250 - the post will be held; {FIX}")),
        "{added}"
    );
}

#[test]
fn add_tool_does_not_warn_inside_the_range_or_without_a_machine() {
    // No machine: no range to read.
    let mut bare = Client::new();
    let added = bare.ok("add_tool", json!({ "catalog": "EM-0300-2FL" }));
    assert_eq!(added["warning"], Value::Null, "{added}");
    // Inside the range, and on each limit: no warning.
    let mut c = Client::new();
    c.ok("load_machine", json!({ "builtin": MACHINE }));
    let added = c.ok("add_tool", json!({ "catalog": "EM-1000-4FL" }));
    assert_eq!(added["warning"], Value::Null, "{added}");
    for (name, rpm) in [("AtMax", 10000.0), ("AtMin", 250.0)] {
        let added = c.ok("add_tool", json!({ "name": name, "definition": end_mill(rpm) }));
        assert_eq!(added["warning"], Value::Null, "{added}");
    }
}

#[test]
fn verify_still_holds_the_post_and_names_the_fix() {
    let mut c = Client::new();
    c.ok("load_machine", json!({ "builtin": MACHINE }));
    c.ok("set_stock", json!({ "box": { "dx": 40.0, "dy": 40.0, "dz": 10.0 } }));
    c.ok("add_tool", json!({ "catalog": "EM-0400-2FL" }));
    c.ok("add_operation", json!({ "kind": "Face", "tool": "EM-0400-2FL", "name": "Face1" }));
    let generated = c.ok("generate", json!({}));
    assert_eq!(generated["generated"][0]["status"], json!("ok"), "{generated}");
    let ver = c.ok("verify", json!({}));
    assert_eq!(ver["gate"]["ok"], json!(false), "the machine check holds the program: {ver}");
    let text = ver.to_string();
    assert!(
        text.contains(&format!("Face1: tool EM-0400-2FL runs 12000 rpm, above the machine's 10000 - {FIX}")),
        "{text}"
    );
}
