//! #3237 MCP pins for 2D Chamfer lead-in/lead-out.

use super::*;

#[test]
fn chamfer_leads_are_listed_patched_saved_and_reset() {
    let mut client = Client::new();
    client.request(
        "initialize",
        json!({ "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": { "name": "test", "version": "0" } }),
    );
    client.ok(
        "set_stock",
        json!({ "box": { "dx": 60.0, "dy": 60.0, "dz": 20.0 } }),
    );
    client.ok("add_tool", json!({ "catalog": "CM-0600-90" }));
    let kinds = client.ok("list_operation_types", json!({}));
    let chamfer = kinds["kinds"]
        .as_array()
        .expect("kinds")
        .iter()
        .find(|kind| kind["key"] == json!("chamfer2_d"))
        .expect("Chamfer");
    let fields = chamfer["fields"].as_array().expect("fields");
    assert!(fields.iter().any(|field| {
        field["name"] == json!("lead")
            && field["default"] == json!({ "perpendicular": { "length": 3.0 } })
    }));
    assert!(fields.iter().any(|field| {
        field["name"] == json!("lead_out") && field["default"].is_null()
    }));

    let added = client.ok(
        "add_operation",
        json!({
            "kind": "2D Chamfer",
            "tool": "CM-0600-90",
            "params": {
                "lead": {
                    "perpendicular": null,
                    "arc": { "radius": 2.5 }
                },
                "lead_out": "none"
            }
        }),
    );
    let name = added["operation"].as_str().expect("operation");
    let job = client.ok("get_job", json!({}));
    assert_eq!(
        job["operations"][0]["params"]["chamfer2_d"]["lead"],
        json!({ "arc": { "radius": 2.5 } })
    );
    assert_eq!(
        job["operations"][0]["params"]["chamfer2_d"]["lead_out"],
        json!("none")
    );

    let dir =
        std::env::temp_dir().join(format!("camcraft-3237-mcp-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let project = dir.join("leads.camproj");
    client.ok(
        "save_project",
        json!({ "path": project.display().to_string() }),
    );
    client.ok(
        "open_project",
        json!({ "path": project.display().to_string() }),
    );
    let loaded = client.ok("get_job", json!({}));
    assert_eq!(
        loaded["operations"][0]["params"]["chamfer2_d"]["lead"],
        json!({ "arc": { "radius": 2.5 } })
    );
    assert_eq!(
        loaded["operations"][0]["params"]["chamfer2_d"]["lead_out"],
        json!("none")
    );

    client.ok(
        "edit_operation",
        json!({
            "name": name,
            "params": {
                "lead": "none",
                "lead_out": null
            }
        }),
    );
    let reset = client.ok("get_job", json!({}));
    let params = &reset["operations"][0]["params"]["chamfer2_d"];
    assert_eq!(params["lead"], json!("none"));
    assert!(params.get("lead_out").is_none());
    let _ = std::fs::remove_dir_all(dir);
}
