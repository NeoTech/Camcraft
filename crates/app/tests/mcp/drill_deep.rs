//! #2443: the drill's `deep` cycle word - full-retract pecking (G83) that
//! the user asks for, where Auto would chip-break (G73). The word rides
//! `edit_operation`, generation, the post and the project file.
//! Provenance: our own tests over the MCP drill surface; G83 and G73 are
//! the RS-274/NGC deep-hole and chip-break peck cycles.

use super::{tmp_dir, write_bored_disc_stl, Client};
use serde_json::json;

fn cycle_lines(text: &str, word: &str) -> usize {
    text.lines().filter(|l| l.split_whitespace().any(|w| w == word)).count()
}

#[test]
fn the_deep_cycle_word_posts_full_retract_pecks_and_rides_the_project_file() {
    let dir = tmp_dir("drill-deep");
    let stl = dir.join("disc.stl");
    write_bored_disc_stl(&stl);
    let mut c = Client::new();
    c.request("initialize", json!({ "protocolVersion": "2025-06-18" }));
    c.ok("import_model", json!({ "path": stl.display().to_string() }));
    c.ok("set_stock", json!({ "box": { "dx": 30.0, "dy": 30.0, "dz": 10.0 } }));
    c.ok("add_tool", json!({ "catalog": "DR-0400" }));
    c.ok("add_operation", json!({ "kind": "drill", "tool": "DR-0400", "source": "disc", "name": "D1" }));
    c.ok("detect_holes", json!({ "name": "D1" }));

    // A 6 mm deep blind hole on a 4 mm drill: Auto chip-breaks.
    c.ok("generate", json!({ "name": "D1" }));
    let auto_nc = dir.join("auto.nc");
    c.ok("post", json!({ "output": auto_nc.display().to_string(), "post": "Fanuc" }));
    let auto = std::fs::read_to_string(&auto_nc).expect("auto program");
    assert!(cycle_lines(&auto, "G73") > 0 && cycle_lines(&auto, "G83") == 0, "{auto}");

    c.ok("edit_operation", json!({ "name": "D1", "params": { "cycle": "deep" } }));
    c.ok("generate", json!({ "name": "D1" }));
    let deep_nc = dir.join("deep.nc");
    c.ok("post", json!({ "output": deep_nc.display().to_string(), "post": "Fanuc" }));
    let deep = std::fs::read_to_string(&deep_nc).expect("deep program");
    assert!(cycle_lines(&deep, "G83") > 0 && cycle_lines(&deep, "G73") == 0, "{deep}");

    let file = dir.join("deep.camproj");
    c.ok("save_project", json!({ "path": file.display().to_string() }));
    let mut back = Client::new();
    back.ok("open_project", json!({ "path": file.display().to_string() }));
    let job = back.ok("get_job", json!({}));
    let op = job["operations"].as_array().and_then(|o| o.iter().find(|o| o["name"] == "D1")).cloned().expect("D1");
    assert_eq!(op["params"]["drill"]["cycle"], json!("deep"), "{op}");
    let _ = std::fs::remove_dir_all(&dir);
}
