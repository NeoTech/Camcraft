//! Face to size from numbers (#3196), driven in-process over the MCP
//! server's JSON-RPC handler: on an empty project one `face_to_size`
//! makes the stock box and a Face on the stock; the op generates,
//! verifies and posts with its Z words at the stated depths; one undo
//! takes both back. Bad numbers are refused by name and change nothing.
//!
//! Provenance: our own fixture over plain facing practice (top Z, final
//! Z, depth per pass); no vendor material consulted.
//!
//! Gate map: covers app::commands::face_to_size, app::mcp::face_to_size_tools.

use camcraft_app::mcp::Server;

use serde_json::{json, Value};
use std::path::PathBuf;

const CUTTER: &str = "EM-1000-4FL";

struct Client {
    server: Server,
    next: i64,
}

impl Client {
    fn new() -> Self {
        let mut c = Self { server: Server::new(), next: 1 };
        c.request("initialize", json!({ "protocolVersion": "2025-06-18" }));
        c
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next;
        self.next += 1;
        let msg = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
        self.server.handle(&msg).expect("a request gets a response")
    }

    fn call(&mut self, name: &str, args: Value) -> (bool, Value) {
        let reply = self.request("tools/call", json!({ "name": name, "arguments": args }));
        let result = &reply["result"];
        assert!(result.is_object(), "tools/call answers with a result: {reply}");
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

    fn ops(&mut self) -> Vec<Value> {
        self.ok("get_job", json!({}))["operations"].as_array().cloned().unwrap_or_default()
    }

    fn has_stock(&mut self) -> bool {
        !self.ok("get_job", json!({}))["setups"][0]["stock"].is_null()
    }
}

fn tmp_dir(tag: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("camcraft-mcp-face-to-size-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).expect("temp dir");
    p
}

/// A block 80 x 60 x 20 with its top at Z0, faced to Z-1.5 in 0.5 mm passes.
fn job() -> Value {
    json!({
        "stock": { "dx": 80.0, "dy": 60.0, "dz": 20.0 },
        "top_z": 0.0, "final_z": -1.5, "stepdown": 0.5, "stepover": 6.0,
        "tool": CUTTER, "feed": 900.0, "plunge_feed": 250.0,
    })
}

/// The cutting Z levels of a posted program: each distinct Z word below
/// the top of the stock.
fn cut_levels(nc: &str) -> Vec<f64> {
    let mut zs: Vec<f64> = nc
        .split_whitespace()
        .filter_map(|w| w.strip_prefix('Z').and_then(|v| v.parse::<f64>().ok()))
        .filter(|z| *z < -1e-6)
        .collect();
    zs.sort_by(|a, b| b.total_cmp(a));
    zs.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
    zs
}

#[test]
fn face_to_size_on_an_empty_project_makes_stock_and_a_face_and_posts_the_stated_depths() {
    let dir = tmp_dir("post");
    let mut c = Client::new();
    c.ok("add_tool", json!({ "catalog": CUTTER }));
    assert!(!c.has_stock(), "an empty project has no stock");
    let made = c.ok("face_to_size", job());
    let op = made["operation"].as_str().unwrap_or_default().to_owned();
    assert!(!op.is_empty(), "{made}");
    let ops = c.ops();
    assert_eq!(ops.len(), 1, "{ops:?}");
    assert_eq!(ops[0]["kind"], json!("Face"), "{ops:?}");
    assert_eq!(ops[0]["source"], json!("stock"), "{ops:?}");
    assert!(c.has_stock(), "the stock is made");

    let generated = c.ok("generate", json!({}));
    assert_eq!(generated["generated"][0]["status"], json!("ok"), "{generated}");
    let ver = c.ok("verify", json!({}));
    assert_eq!(ver["ok"], json!(true), "{ver}");
    let out = dir.join("face.nc");
    let posted = c.ok("post", json!({ "output": out.display().to_string(), "post": "Fanuc" }));
    assert!(posted["lines"].as_u64().is_some_and(|n| n > 0), "{posted}");
    let nc = std::fs::read_to_string(&out).expect("posted file");
    assert_eq!(cut_levels(&nc), vec![-0.5, -1.0, -1.5], "three passes to the final Z:\n{nc}");

    // One undo takes back the generate; the next takes the op AND the stock.
    c.ok("undo", json!({}));
    assert_eq!(c.ops().len(), 1, "the first undo is the generate");
    c.ok("undo", json!({}));
    assert!(c.ops().is_empty(), "one undo removes the face");
    assert!(!c.has_stock(), "the same undo removes the stock it made");
    c.ok("redo", json!({}));
    assert_eq!(c.ops().len(), 1, "redo brings the face back");
    assert!(c.has_stock(), "and its stock");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn face_to_size_keeps_the_existing_stock_when_no_size_is_given() {
    let mut c = Client::new();
    c.ok("add_tool", json!({ "catalog": CUTTER }));
    c.ok("set_stock", json!({ "box": { "dx": 50.0, "dy": 40.0, "dz": 10.0 } }));
    // The stock stands on Z0, so its top is Z10.
    let made = c.ok("face_to_size", json!({
        "top_z": 10.0, "final_z": 9.0, "stepdown": 0.5, "stepover": 6.0, "tool": CUTTER,
    }));
    assert_eq!(made["stock_set"], json!(false), "{made}");
    let generated = c.ok("generate", json!({}));
    assert_eq!(generated["generated"][0]["status"], json!("ok"), "{generated}");
    c.ok("undo", json!({}));
    c.ok("undo", json!({}));
    assert!(c.ops().is_empty());
    assert!(c.has_stock(), "the undo keeps the stock it did not make");
}

#[test]
fn face_to_size_refuses_bad_numbers_by_name_and_changes_nothing() {
    let mut c = Client::new();
    c.ok("add_tool", json!({ "catalog": CUTTER }));
    let with = |k: &str, v: Value| {
        let mut j = job();
        j[k] = v;
        j
    };
    let cases = [
        (with("final_z", json!(0.0)), "below the top Z"),
        (with("final_z", json!(2.0)), "below the top Z"),
        (with("stepdown", json!(0.0)), "depth per pass"),
        (with("stepdown", json!(-1.0)), "depth per pass"),
        (with("stepover", json!(0.0)), "stepover"),
        (with("stepover", json!(12.0)), "wider than the tool"),
        (with("tool", json!("")), "tool"),
        (with("tool", json!("NO-SUCH-TOOL")), "NO-SUCH-TOOL"),
        (with("final_z", json!(-20.0)), "stock bottom"),
    ];
    for (args, words) in cases {
        let why = c.refused("face_to_size", args.clone());
        assert!(why.contains(words), "{args}: '{why}' should name '{words}'");
        assert!(c.ops().is_empty(), "{args}: no op enters");
        assert!(!c.has_stock(), "{args}: no stock enters");
    }
    // Without a size, the project's stock is needed.
    let mut no_size = job();
    no_size.as_object_mut().map(|o| o.remove("stock"));
    let why = c.refused("face_to_size", no_size);
    assert!(why.contains("stock"), "{why}");
    // A top Z below the stock top would cut more than one pass deep.
    c.ok("set_stock", json!({ "box": { "dx": 50.0, "dy": 40.0, "dz": 10.0 } }));
    let why = c.refused("face_to_size", json!({
        "top_z": 8.0, "final_z": 7.0, "stepdown": 0.5, "stepover": 6.0, "tool": CUTTER,
    }));
    assert!(why.contains("below the stock top"), "{why}");
    assert!(c.ops().is_empty());
}
