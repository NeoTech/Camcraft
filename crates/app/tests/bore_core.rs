//! #1556: a bore far wider than its tool is never posted as a finished
//! hole with a free core. Detect holes leaves it out of a Bore op and
//! names it; a Bore op handed it by hand refuses at generate with the
//! numbers; a hole the ride clears is bored as before.
//!
//! Gate map: covers app::operation::bore_reach, app::operation::material
//! (detect_holes), app::mcp::edit (detect_holes) and the Bore arm of
//! app::operation::generate, over the motion kernel's bore door.
//!
//! Provenance: the issue's own MCP reproduction on a synthetic plate
//! (a Ø74.3 through bore beside a Ø6 hole, a 4 mm end mill); no vendor
//! material consulted.

use std::path::PathBuf;

use camcraft_app::mcp::Server;
use serde_json::{json, Value};

struct Client {
    server: Server,
    next: i64,
}

impl Client {
    fn new() -> Self {
        Self { server: Server::new(), next: 1 }
    }

    fn call(&mut self, name: &str, args: Value) -> (bool, Value) {
        let id = self.next;
        self.next += 1;
        let msg = json!({ "jsonrpc": "2.0", "id": id, "method": "tools/call",
            "params": { "name": name, "arguments": args } });
        let reply = self.server.handle(&msg).expect("a request gets a response");
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
}

const BIG_R: f64 = 37.15;
const SMALL_R: f64 = 3.0;
const H: f64 = 50.0;
const TOP: f64 = 10.0;
const N: usize = 64;

type P = [f64; 3];

/// A plate of 100 x 100 cells along X, 10 mm thick, each cell bored
/// through at its centre with the given radius. Each bore's vertices are
/// pushed radially onto its cell's square (N a multiple of 8, so the
/// corners are hit), so neighbouring cells share their edge vertices and
/// the mesh closes. Facets are wound by their wanted normal.
fn plate(radii: &[f64]) -> Vec<[P; 3]> {
    let mut tris: Vec<[P; 3]> = Vec::new();
    let mut quad = |a: P, b: P, c: P, d: P, want: P| {
        for t in [[a, b, c], [a, c, d]] {
            let u = [t[1][0] - t[0][0], t[1][1] - t[0][1], t[1][2] - t[0][2]];
            let v = [t[2][0] - t[0][0], t[2][1] - t[0][1], t[2][2] - t[0][2]];
            let n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
            let dot = n[0] * want[0] + n[1] * want[1] + n[2] * want[2];
            tris.push(if dot >= 0.0 { t } else { [t[0], t[2], t[1]] });
        }
    };
    let cells = radii.len();
    for (k, &r) in radii.iter().enumerate() {
        let cx = (2.0 * k as f64 + 1.0) * H - cells as f64 * H;
        let ang = |i: usize| std::f64::consts::TAU * (i % N) as f64 / N as f64;
        let bp = |i: usize, z: f64| [cx + r * ang(i).cos(), r * ang(i).sin(), z];
        let sq = |i: usize, z: f64| {
            let (x, y) = (ang(i).cos(), ang(i).sin());
            let s = x.abs().max(y.abs());
            [cx + x * H / s, y * H / s, z]
        };
        for i in 0..N {
            let j = i + 1;
            quad(bp(i, TOP), bp(j, TOP), sq(j, TOP), sq(i, TOP), [0.0, 0.0, 1.0]);
            quad(bp(i, 0.0), bp(j, 0.0), sq(j, 0.0), sq(i, 0.0), [0.0, 0.0, -1.0]);
            let mid = ang(i) + std::f64::consts::PI / N as f64;
            quad(bp(i, TOP), bp(j, TOP), bp(j, 0.0), bp(i, 0.0), [-mid.cos(), -mid.sin(), 0.0]);
            // The outer wall, except where this cell meets its neighbour.
            let (a, b) = (sq(i, TOP), sq(j, TOP));
            let shared_left = k > 0 && (a[0] - (cx - H)).abs() < 1e-9 && (b[0] - (cx - H)).abs() < 1e-9;
            let shared_right = k + 1 < cells && (a[0] - (cx + H)).abs() < 1e-9 && (b[0] - (cx + H)).abs() < 1e-9;
            if !(shared_left || shared_right) {
                let out = [(a[0] + b[0]) * 0.5 - cx, (a[1] + b[1]) * 0.5, 0.0];
                quad(a, b, sq(j, 0.0), sq(i, 0.0), out);
            }
        }
    }
    tris
}

fn write_stl(tag: &str, radii: &[f64]) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!("camcraft-bore-core-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("plate.stl");
    let mut s = String::from("solid plate\n");
    for t in plate(radii) {
        s.push_str(" facet normal 0 0 0\n  outer loop\n");
        for p in t {
            s.push_str(&format!("   vertex {} {} {}\n", p[0], p[1], p[2]));
        }
        s.push_str("  endloop\n endfacet\n");
    }
    s.push_str("endsolid plate\n");
    std::fs::write(&path, s).expect("write stl");
    path
}

/// The plate imported, its own stock, a 4 mm end mill and a Bore op on it.
fn setup(tag: &str, radii: &[f64]) -> Client {
    let stl = write_stl(tag, radii);
    let mut c = Client::new();
    c.ok("import_model", json!({ "path": stl.display().to_string(), "name": "plate" }));
    c.ok("set_stock", json!({ "from_model": "plate" }));
    c.ok("add_tool", json!({ "catalog": "EM-0400-2FL", "name": "EM4" }));
    c.ok("add_operation", json!({ "kind": "bore", "tool": "EM4", "source": "plate", "name": "B", "params": { "stepdown": 2.0 } }));
    c
}

fn holes(c: &mut Client) -> Vec<Value> {
    let job = c.ok("get_job", json!({}));
    job["operations"][0]["params"]["bore"]["holes"].as_array().cloned().unwrap_or_default()
}

#[test]
fn detect_holes_leaves_the_wide_bore_out_of_a_bore_op_and_names_it() {
    let mut c = setup("detect", &[BIG_R, SMALL_R]);
    let got = c.ok("detect_holes", json!({ "name": "B" }));
    assert_eq!(got["holes"], json!(1), "only the hole a 4 mm ring clears is stored: {got}");
    let skipped = got["skipped"].as_array().cloned().unwrap_or_default();
    assert_eq!(skipped.len(), 1, "the wide bore is named: {got}");
    let note = skipped[0].as_str().unwrap_or_default();
    for want in ["74.300 mm hole at (-50.000, 0.000)", "skipped", "12.000 mm", "4.000 mm tool", "pocket"] {
        assert!(note.contains(want), "the note names `{want}`: {note}");
    }
    let stored = holes(&mut c);
    assert_eq!(stored.len(), 1, "{stored:?}");
    let r = stored[0]["radius"].as_f64().unwrap_or(f64::NAN);
    assert!((r - SMALL_R).abs() < 1e-3, "the Ø6 hole is the one kept: {stored:?}");

    // The kept hole bores and verifies as before.
    c.ok("generate", json!({ "name": "B" }));
    let v = c.ok("verify", json!({}));
    assert_eq!(v["ok"], json!(true), "the Ø6 bore verifies: {v}");
}

#[test]
fn detect_holes_on_a_bore_op_with_only_too_wide_holes_is_refused_by_name() {
    let mut c = setup("only", &[BIG_R]);
    let e = c.refused("detect_holes", json!({ "name": "B" }));
    assert!(e.contains("no hole this Bore op can make") && e.contains("74.300 mm hole"), "{e}");
    assert!(holes(&mut c).is_empty(), "a refused detection stores nothing");

    // A Drill op on the same plate is not filtered: the rule is the Bore ride's.
    c.ok("add_tool", json!({ "catalog": "DR-0400", "name": "DR4" }));
    c.ok("add_operation", json!({ "kind": "drill", "tool": "DR4", "source": "plate", "name": "D" }));
    let got = c.ok("detect_holes", json!({ "name": "D" }));
    assert_eq!(got["holes"], json!(1), "{got}");
    assert_eq!(got["skipped"], json!([]), "{got}");
}

#[test]
fn a_bore_op_handed_the_wide_bore_by_hand_refuses_at_generate() {
    let mut c = setup("byhand", &[BIG_R, SMALL_R]);
    c.ok("detect_holes", json!({ "name": "B" }));
    let mut list = holes(&mut c);
    let mut wide = list[0].clone();
    wide["radius"] = json!(BIG_R);
    wide["xy"] = json!([-50.0, 0.0]);
    list.push(wide);
    c.ok("edit_operation", json!({ "name": "B", "params": { "holes": list } }));
    assert_eq!(holes(&mut c).len(), 2, "the edit stored the wide bore");
    let e = c.refused("generate", json!({ "name": "B" }));
    for want in ["74.300 mm hole at (-50.000, 0.000)", "too wide to bore", "4.000 mm cutter", "core from 4.000 to 66.300 mm", "up to 12.000 mm"] {
        assert!(e.contains(want), "the refusal names `{want}`: {e}");
    }
    // Nothing generated is nothing to post.
    let v = c.ok("verify", json!({}));
    assert_ne!(v["ok"], json!(true), "a refused bore never verifies ok: {v}");
}
