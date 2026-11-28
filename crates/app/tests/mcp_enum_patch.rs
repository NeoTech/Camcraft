//! #2876, #3476: merge-patch refusals explain retained shapes and kind wrappers.
//! Gate map (M4.4): covers app::mcp::edit, app::mcp, app::project::json.
//! Provenance: our own MCP contract and RFC 7386 merge-patch semantics.

use camcraft_app::mcp::Server;
use serde_json::{Value, json};

struct Client(Server);

impl Client {
    fn new() -> Self {
        let mut client = Self(Server::new());
        client.ok(
            "set_stock",
            json!({ "box": { "dx": 60.0, "dy": 60.0, "dz": 20.0 } }),
        );
        client.ok("add_tool", json!({ "catalog": "EM-0600-4FL" }));
        client
    }

    fn call(&mut self, name: &str, arguments: Value) -> Value {
        let reply = self
            .0
            .handle(&json!({
                "jsonrpc": "2.0", "id": 1, "method": "tools/call",
                "params": { "name": name, "arguments": arguments }
            }))
            .expect("a request gets a response");
        assert!(reply["result"].is_object(), "tool response: {reply}");
        reply["result"].clone()
    }

    fn ok(&mut self, name: &str, arguments: Value) -> Value {
        let result = self.call(name, arguments);
        assert_ne!(result["isError"], json!(true), "{name}: {result}");
        result["structuredContent"].clone()
    }

    fn refused(&mut self, name: &str, arguments: Value, path: &str) {
        let result = self.call(name, arguments);
        assert_eq!(
            result["isError"],
            json!(true),
            "{name} must refuse: {result}"
        );
        let text = result["content"][0]["text"].as_str().expect("refusal text");
        for word in [path, "arc", "linear", "null"] {
            assert!(text.contains(word), "refusal must name {word}: {text}");
        }
    }

    fn contour(&mut self, params: Value) {
        self.ok(
            "add_operation",
            json!({
                "kind": "2D Contour", "tool": "EM-0600-4FL", "name": "Contour", "params": params
            }),
        );
    }

    fn params(&mut self) -> Value {
        self.ok("get_job", json!({}))["operations"][0]["params"].clone()
    }
}

#[test]
fn nested_job_params_refusal_supplies_a_usable_flat_call() {
    let mut client = Client::new();
    client.ok(
        "add_operation",
        json!({ "kind": "2D Pocket", "tool": "EM-0600-4FL", "name": "Pocket" }),
    );
    let name = "Pocket \"finish\" \\ pass";
    client.ok(
        "rename_operation",
        json!({ "name": "Pocket", "new_name": name }),
    );
    // Keep a redo entry. A refused edit must not replace it or add an undo step.
    client.ok(
        "edit_operation",
        json!({ "name": name, "params": { "stepdown": 1.25 } }),
    );
    client.ok("undo", json!({}));
    let before = client.ok("get_job", json!({}));
    assert_eq!(before["can_redo"], json!(true));
    let mut nested = before["operations"][0]["params"].clone();
    assert!(nested["pocket"].is_object(), "get_job kind wrapper: {nested}");
    assert_ne!(nested["pocket"]["stepdown"], json!(0.75));
    nested["pocket"]["stepdown"] = json!(0.75);
    let result = client.call(
        "edit_operation",
        json!({ "name": name, "params": nested }),
    );
    assert_eq!(result["isError"], json!(true), "nested edit: {result}");
    assert_eq!(client.ok("get_job", json!({})), before);
    let text = result["content"][0]["text"].as_str().expect("refusal text");
    assert!(
        text.contains("Corrected call: edit_operation "),
        "nested kind refusal must include the corrected edit_operation call: {text}"
    );
    let (_, call) = text.split_once("Corrected call: edit_operation ").expect("corrected call");
    let arguments: Value = serde_json::from_str(call).expect("corrected arguments are JSON");
    assert_eq!(arguments, json!({ "name": name, "params": nested["pocket"] }));
    // The old redo still works. Then retry the actual call from the refusal.
    client.ok("redo", json!({}));
    assert_eq!(client.params()["pocket"]["stepdown"], json!(1.25));
    client.ok("undo", json!({}));
    assert_eq!(client.ok("get_job", json!({})), before);
    client.ok("edit_operation", arguments);
    assert_eq!(client.params(), nested);
    client.ok("undo", json!({}));
    assert_eq!(client.params(), before["operations"][0]["params"]);
    client.ok("redo", json!({}));
    assert_eq!(client.params(), nested);
}

#[test]
fn invalid_kind_wrappers_and_unknown_fields_remain_refused() {
    let mut client = Client::new();
    client.contour(json!({}));
    let before = client.ok("get_job", json!({}));
    for patch in [
        json!({ "pocket": { "stepdown": 0.75 } }),
        json!({ "contour": { "unknown_field": 0.75 } }),
        json!({ "contour": { "stepdown": 0.75 }, "unknown_field": 1.0 }),
        json!({ "contour": { "stepdown": 0.75 }, "stepover": 0.25 }),
        json!({ "contour": { "stepdown": 0.75 }, "pocket": {} }),
        json!({ "contour": { "stepdown": -0.75 } }),
        json!({ "contour": { "stepdown": "not a number" } }),
        json!({ "contour": { "clearance_z": { "ofset": 1.0 } } }),
        json!({ "unknown_field": 0.75 }),
        json!({ "clearance_z": { "ofset": 1.0 } }),
        json!({ "contour": 0.75 }),
        json!([]),
    ] {
        let result = client.call(
            "edit_operation",
            json!({ "name": "Contour", "params": patch }),
        );
        assert_eq!(result["isError"], json!(true), "invalid patch: {patch}: {result}");
        let text = result["content"][0]["text"].as_str().expect("refusal text");
        assert!(!text.contains("Corrected call:"), "do not suggest a lossy call: {text}");
        assert_eq!(client.ok("get_job", json!({})), before);
    }
}

#[test]
fn corrected_contour_call_preserves_nested_fields_and_null_removal() {
    let mut client = Client::new();
    client.contour(json!({}));
    let before = client.ok("get_job", json!({}));
    let fields = json!({
        "stepdown": 0.75,
        "lead": { "linear": null, "arc": { "radius": 3.0 } }
    });
    let result = client.call(
        "edit_operation",
        json!({ "name": "Contour", "params": { "contour": fields } }),
    );
    assert_eq!(result["isError"], json!(true), "nested edit: {result}");
    assert_eq!(client.ok("get_job", json!({})), before);
    let text = result["content"][0]["text"].as_str().expect("refusal text");
    let (_, call) = text.split_once("Corrected call: edit_operation ").expect("corrected call");
    let arguments: Value = serde_json::from_str(call).expect("corrected arguments are JSON");
    assert_eq!(arguments, json!({ "name": "Contour", "params": fields }));
    client.ok("edit_operation", arguments);
    assert_eq!(client.params()["contour"]["stepdown"], json!(0.75));
    assert_eq!(client.params()["contour"]["lead"], json!({ "arc": { "radius": 3.0 } }));
    client.ok("undo", json!({}));
    assert_eq!(client.params(), before["operations"][0]["params"]);
}

#[test]
fn add_contour_lead_shape_refusal_names_retained_shapes() {
    let mut client = Client::new();
    client.refused(
        "add_operation",
        json!({
            "kind": "2D Contour", "tool": "EM-0600-4FL", "name": "Contour",
            "params": { "lead": { "arc": { "radius": 3.0 } } }
        }),
        "lead",
    );
    assert_eq!(client.ok("get_job", json!({}))["operations"], json!([]));
    client.contour(json!({ "lead": { "linear": null, "arc": { "radius": 3.0 } } }));
    assert_eq!(
        client.params()["contour"]["lead"],
        json!({ "arc": { "radius": 3.0 } })
    );
}

#[test]
fn edit_contour_lead_shape_refusal_preserves_parameters_and_undo() {
    let mut client = Client::new();
    client.contour(json!({}));
    let before = client.params();
    client.refused(
        "edit_operation",
        json!({
            "name": "Contour", "params": { "lead": { "arc": { "radius": 3.0 } } }
        }),
        "lead",
    );
    assert_eq!(client.params(), before);
    client.ok("undo", json!({}));
    assert_eq!(client.ok("get_job", json!({}))["operations"], json!([]));
    client.ok("redo", json!({}));
    client.ok(
        "edit_operation",
        json!({
            "name": "Contour", "params": { "lead": { "linear": null, "arc": { "radius": 3.0 } } }
        }),
    );
    assert_eq!(
        client.params()["contour"]["lead"],
        json!({ "arc": { "radius": 3.0 } })
    );
    client.ok(
        "edit_operation",
        json!({
            "name": "Contour", "params": { "lead": { "arc": { "radius": 4.0 } } }
        }),
    );
    assert_eq!(
        client.params()["contour"]["lead"],
        json!({ "arc": { "radius": 4.0 } })
    );
}

#[test]
fn nested_lead_out_shape_refusal_names_full_parameter_path() {
    let mut client = Client::new();
    client.contour(
        json!({ "links": { "approach": { "lead_out": { "linear": { "length": 8.0 } } } } }),
    );
    let before = client.params();
    client.refused("edit_operation", json!({
        "name": "Contour", "params": { "links": { "approach": { "lead_out": { "arc": { "radius": 3.0 } } } } }
    }), "links.approach.lead_out");
    assert_eq!(client.params(), before);
    client.ok(
        "edit_operation",
        json!({
            "name": "Contour", "params": { "links": { "approach": { "lead_out": {
                "linear": null, "arc": { "radius": 3.0 }
            } } } }
        }),
    );
    assert_eq!(
        client.params()["contour"]["links"]["approach"]["lead_out"],
        json!({ "arc": { "radius": 3.0 } })
    );
}
