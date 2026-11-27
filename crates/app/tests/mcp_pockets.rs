//! `detect_pockets` over MCP (#35): an imported STL with a rounded
//! pocket and one with a through slot list their features in world
//! coordinates; `create` adds the Pocket (or Slot) that machines one as
//! a single undo step, the pocket generates and verifies clean; and every
//! bad argument is a tool error, never a panic. #2440: holes through a
//! slot's floor keep the slot, a hole tangent to its wall too; a boss in
//! the channel does not.
//!
//! Gate map (#35): covers app::mcp (pockets, tools), app::recognize,
//! app::commands (CreateOperation).

mod pocket_support;

use camcraft_app::mcp::Server;
use pocket_support::{block_with_pocket, height_field, rounded_rect, write_stl};

use serde_json::{json, Value};
use std::path::PathBuf;

fn tmp_dir(tag: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("camcraft-mcp-pockets-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).expect("temp dir");
    p
}

struct Client {
    server: Server,
    next: i64,
}

impl Client {
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

/// A document holding `mesh` as the model "part", a stock box with
/// margin round the 60 x 40 x 20 block, and a 6 mm end mill.
fn job(tag: &str, mesh: &camcraft_app::geometry::Mesh) -> (Client, PathBuf) {
    let dir = tmp_dir(tag);
    let stl = dir.join("part.stl");
    write_stl(mesh, &stl);
    let mut c = Client { server: Server::new(), next: 1 };
    let imp = c.ok("import_model", json!({ "path": stl.display().to_string() }));
    assert_eq!(imp["model"], json!("part"));
    c.ok("set_stock", json!({ "box": { "dx": 80.0, "dy": 60.0, "dz": 20.0 } }));
    c.ok("add_tool", json!({ "catalog": "EM-0600-4FL" }));
    (c, dir)
}

fn ops(c: &mut Client) -> usize {
    c.ok("get_job", json!({}))["operations"].as_array().map_or(0, Vec::len)
}

#[test]
fn detect_pockets_lists_a_rounded_pocket_and_creates_its_pocket() {
    let mesh = block_with_pocket(30.0, 20.0, &rounded_rect(15.0, 8.0, 4.0, 6), 8.0, 20.0);
    let (mut c, dir) = job("rounded", &mesh);
    let got = c.ok("detect_pockets", json!({ "model": "part" }));
    assert_eq!(got["count"], json!(1), "{got}");
    let f = &got["features"][0];
    assert_eq!(f["kind"], json!("closed_pocket"), "{f}");
    assert_eq!((f["floor_z"].clone(), f["top_z"].clone(), f["depth"].clone()), (json!(8.0), json!(20.0), json!(12.0)), "{f}");
    assert!(f["min_corner_radius"].as_f64().is_some_and(|r| (r - 4.0).abs() < 1e-3), "{f}");
    assert_eq!(f["operation_kind"], json!("2D Pocket"));
    assert!(f["floor"].as_str().is_some_and(|s| s.starts_with("face:0x")), "{f}");
    // The floor id is the one list_features prints for the same face.
    let faces = c.ok("list_features", json!({ "model": "part", "kinds": ["faces"] }));
    assert!(faces["faces"].as_array().is_some_and(|l| l.iter().any(|x| x["id"] == f["floor"])), "{faces}");
    assert_eq!(ops(&mut c), 0, "listing creates nothing");

    let made = c.ok("detect_pockets", json!({ "model": "part", "create": 0, "tool": "EM-0600-4FL", "name": "P1" }));
    assert_eq!(made["created"]["operation"], json!("P1"), "{made}");
    assert_eq!(made["created"]["kind"], json!("2D Pocket"));
    let g = c.ok("get_operation_geometry", json!({ "operation": "P1" }));
    assert_eq!(g["source"]["picks"][0]["id"], f["floor"], "{g}");
    let generated = c.ok("generate", json!({ "name": "P1" }));
    assert_eq!(generated["generated"][0]["status"], json!("ok"), "{generated}");
    let ver = c.ok("verify", json!({}));
    assert_eq!(ver["ok"], json!(true), "{ver}");

    // Undo the generation, then the creation: nothing is left.
    c.ok("undo", json!({}));
    c.ok("undo", json!({}));
    assert_eq!(ops(&mut c), 0);
    c.ok("redo", json!({}));
    assert_eq!(ops(&mut c), 1);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn detect_pockets_creates_a_slot_for_a_through_slot() {
    let mesh = height_field(&[-30.0, 30.0], &[-20.0, -5.0, 5.0, 20.0], |_, j| if j == 1 { 10.0 } else { 20.0 });
    let (mut c, dir) = job("slot", &mesh);
    let got = c.ok("detect_pockets", json!({ "model": "part" }));
    let f = &got["features"][0];
    assert_eq!(f["kind"], json!("slot"), "{got}");
    assert_eq!(f["slot"]["ends"], json!(["open", "open"]));
    assert_eq!(f["slot"]["width"], json!(10.0));
    assert_eq!(f["open_spans"].as_array().map(Vec::len), Some(2));
    let made = c.ok("detect_pockets", json!({ "model": "part", "create": 0, "tool": "EM-0600-4FL" }));
    assert_eq!(made["created"]["operation"], json!("Slot1"), "{made}");
    let job = c.ok("get_job", json!({}));
    let slot = &job["operations"][0]["params"]["slot"];
    assert_eq!(slot["width"], json!(10.0), "{job}");
    c.ok("undo", json!({}));
    assert_eq!(ops(&mut c), 0);
    let _ = std::fs::remove_dir_all(&dir);
}

/// #2440: a through slot along X (y -5..5, floor z 10) in a 20 mm block,
/// with `cell` standing in two 3 x 4 mm cells of its floor. 2 mm is a
/// hole below the floor; 15 mm is a boss standing in it.
fn slot_with_cells(cell: f64) -> camcraft_app::geometry::Mesh {
    let xs = [-30.0, -15.0, -12.0, 12.0, 15.0, 30.0];
    let ys = [-20.0, -5.0, -2.0, 2.0, 5.0, 20.0];
    height_field(&xs, &ys, |i, j| match (i, j) {
        (1 | 3, 2) => cell,
        (_, 1..=3) => 10.0,
        _ => 20.0,
    })
}

/// #2440: holes through a slot's floor do not hide the slot - the slot
/// reads open at both ends at its full width, and `create` makes a Slot
/// that generates. The holes are inner rings of the floor (`islands`).
#[test]
fn a_slot_with_holes_through_its_floor_is_still_a_slot() {
    let (mut c, dir) = job("slot-holes", &slot_with_cells(2.0));
    let got = c.ok("detect_pockets", json!({ "model": "part" }));
    let features = got["features"].as_array().cloned().unwrap_or_default();
    let (i, f) = features.iter().enumerate().find(|(_, f)| f["floor_z"] == json!(10.0)).expect("the slot floor is listed");
    assert_eq!(f["kind"], json!("slot"), "{got}");
    assert_eq!(f["islands"], json!(2), "{f}");
    assert_eq!(f["slot"]["ends"], json!(["open", "open"]), "{f}");
    assert_eq!(f["slot"]["width"], json!(10.0), "{f}");
    assert_eq!((f["top_z"].clone(), f["depth"].clone()), (json!(20.0), json!(10.0)), "{f}");
    let made = c.ok("detect_pockets", json!({ "model": "part", "create": i, "tool": "EM-0600-4FL" }));
    assert_eq!(made["created"]["kind"], json!("Slot"), "{made}");
    let generated = c.ok("generate", json!({ "name": made["created"]["operation"] }));
    assert_eq!(generated["generated"][0]["status"], json!("ok"), "{generated}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// #2440: a boss standing in the channel still breaks the slot read (the
/// slot cycle would cut through it); the floor lists as an open pocket.
#[test]
fn a_boss_standing_in_the_channel_is_not_a_slot() {
    let (mut c, dir) = job("slot-boss", &slot_with_cells(15.0));
    let got = c.ok("detect_pockets", json!({ "model": "part" }));
    let features = got["features"].as_array().cloned().unwrap_or_default();
    let f = features.iter().find(|f| f["floor_z"] == json!(10.0)).expect("the channel floor is listed");
    assert_eq!(f["kind"], json!("open_pocket"), "{got}");
    assert_eq!(f["islands"], json!(2), "{f}");
    assert!(f["slot"].is_null(), "{f}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// #2440, the reported part's case: a slot along Y (x 5..15, floor z 10)
/// in a 40 x 40 x 20 block, with a blind diamond hole (centre (12, 0),
/// half diagonal 3, floor z 4) whose corner touches the slot wall at
/// (15, 0) - a hole tangent to the wall. The floor's hole ring and its
/// outer ring share that one vertex. Built facet by facet, watertight,
/// with no T-junction.
fn slot_with_a_tangent_hole() -> camcraft_app::geometry::Mesh {
    use camcraft_app::geometry::{Mesh, Point3, Triangle};
    type P = (f64, f64, f64);
    let mut out: Vec<Triangle> = Vec::new();
    let mut tri = |a: P, b: P, c: P, n: [f64; 3]| {
        let (a, b, c) = (Point3::new(a.0, a.1, a.2), Point3::new(b.0, b.1, b.2), Point3::new(c.0, c.1, c.2));
        let (u, v) = (b - a, c - a);
        let k = [u.y * v.z - u.z * v.y, u.z * v.x - u.x * v.z, u.x * v.y - u.y * v.x];
        out.push(if k[0] * n[0] + k[1] * n[1] + k[2] * n[2] > 0.0 { Triangle::new(a, b, c) } else { Triangle::new(a, c, b) });
    };
    let (x0, x1, y0, y1) = (-15.0, 25.0, -20.0, 20.0);
    let (sl, sr, fz, tz, hz) = (5.0, 15.0, 10.0, 20.0, 4.0);
    // Bottom, the two outer sides and the two top lands.
    for (q, n) in [
        ([(x0, y0, 0.0), (x1, y0, 0.0), (x1, y1, 0.0), (x0, y1, 0.0)], [0.0, 0.0, -1.0]),
        ([(x0, y0, 0.0), (x0, y1, 0.0), (x0, y1, tz), (x0, y0, tz)], [-1.0, 0.0, 0.0]),
        ([(x1, y0, 0.0), (x1, y1, 0.0), (x1, y1, tz), (x1, y0, tz)], [1.0, 0.0, 0.0]),
        ([(x0, y0, tz), (sl, y0, tz), (sl, y1, tz), (x0, y1, tz)], [0.0, 0.0, 1.0]),
        ([(sr, y0, tz), (x1, y0, tz), (x1, y1, tz), (sr, y1, tz)], [0.0, 0.0, 1.0]),
    ] {
        tri(q[0], q[1], q[2], n);
        tri(q[0], q[2], q[3], n);
    }
    // The slot walls, each with the floor's vertex at y 0 on its foot.
    for (x, n) in [(sl, [1.0, 0.0, 0.0]), (sr, [-1.0, 0.0, 0.0])] {
        tri((x, y0, fz), (x, 0.0, fz), (x, y0, tz), n);
        tri((x, 0.0, fz), (x, y1, fz), (x, y1, tz), n);
        tri((x, 0.0, fz), (x, y1, tz), (x, y0, tz), n);
    }
    // The two ends: the block's outline less the slot's notch, (x, z).
    let end = [
        [(x0, 0.0), (x1, 0.0), (sr, fz)],
        [(x1, 0.0), (x1, tz), (sr, fz)],
        [(x1, tz), (sr, tz), (sr, fz)],
        [(x0, 0.0), (sr, fz), (sl, fz)],
        [(x0, 0.0), (sl, fz), (x0, tz)],
        [(x0, tz), (sl, fz), (sl, tz)],
    ];
    for (y, n) in [(y0, [0.0, -1.0, 0.0]), (y1, [0.0, 1.0, 0.0])] {
        for t in end {
            tri((t[0].0, y, t[0].1), (t[1].0, y, t[1].1), (t[2].0, y, t[2].1), n);
        }
    }
    // The floor less the diamond: two fans, each star-shaped from its apex.
    let (r, t, l, b) = ((15.0, 0.0), (12.0, 3.0), (9.0, 0.0), (12.0, -3.0));
    let up = [0.0, 0.0, 1.0];
    for (apex, ring) in [((sl, y0), [(sr, y0), r, b, l, (sl, 0.0)]), ((sl, y1), [(sl, 0.0), l, t, r, (sr, y1)])] {
        for w in ring.windows(2) {
            tri((apex.0, apex.1, fz), (w[0].0, w[0].1, fz), (w[1].0, w[1].1, fz), up);
        }
    }
    // The hole: four walls facing its centre, and its floor.
    let c = (12.0, 0.0);
    for (p, q) in [(r, b), (b, l), (l, t), (t, r)] {
        let n = [c.0 - 0.5 * (p.0 + q.0), c.1 - 0.5 * (p.1 + q.1), 0.0];
        tri((p.0, p.1, hz), (q.0, q.1, hz), (q.0, q.1, fz), n);
        tri((p.0, p.1, hz), (q.0, q.1, fz), (p.0, p.1, fz), n);
    }
    tri((r.0, r.1, hz), (t.0, t.1, hz), (l.0, l.1, hz), up);
    tri((r.0, r.1, hz), (l.0, l.1, hz), (b.0, b.1, hz), up);
    Mesh::from_facets(out)
}

/// #2440: a hole tangent to the slot wall shares one vertex with the
/// floor's outline. The slot still reads as a slot (it read as nothing),
/// open at both ends, and `create` makes a Slot that generates.
#[test]
fn a_slot_with_a_hole_tangent_to_its_wall_is_still_a_slot() {
    let (mut c, dir) = job("slot-tangent", &slot_with_a_tangent_hole());
    let got = c.ok("detect_pockets", json!({ "model": "part" }));
    let features = got["features"].as_array().cloned().unwrap_or_default();
    let (i, f) = features.iter().enumerate().find(|(_, f)| f["floor_z"] == json!(10.0)).expect("the slot floor is listed");
    assert_eq!(f["kind"], json!("slot"), "{got}");
    assert_eq!(f["islands"], json!(1), "{f}");
    assert_eq!(f["slot"]["ends"], json!(["open", "open"]), "{f}");
    assert_eq!(f["slot"]["width"], json!(10.0), "{f}");
    assert_eq!(f["slot"]["length"], json!(40.0), "{f}");
    assert_eq!((f["top_z"].clone(), f["depth"].clone()), (json!(20.0), json!(10.0)), "{f}");
    assert!(f["area"].as_f64().is_some_and(|a| (a - 382.0).abs() < 1e-6), "the floor less the diamond: {f}");
    let made = c.ok("detect_pockets", json!({ "model": "part", "create": i, "tool": "EM-0600-4FL" }));
    assert_eq!(made["created"]["kind"], json!("Slot"), "{made}");
    let generated = c.ok("generate", json!({ "name": made["created"]["operation"] }));
    assert_eq!(generated["generated"][0]["status"], json!("ok"), "{generated}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn detect_pockets_refuses_bad_arguments_by_name() {
    let mesh = height_field(&[-20.0, 20.0], &[-20.0, 20.0], |_, _| 20.0);
    let (mut c, dir) = job("refuse", &mesh);
    let none = c.ok("detect_pockets", json!({ "model": "part" }));
    assert_eq!((none["count"].clone(), none["features"].clone()), (json!(0), json!([])), "a plain block has none");
    let msg = c.refused("detect_pockets", json!({ "model": "part", "create": 0, "tool": "EM-0600-4FL" }));
    assert!(msg.contains("0 recognised"), "{msg}");
    let msg = c.refused("detect_pockets", json!({ "model": "ghost" }));
    assert!(msg.contains("no model named"), "{msg}");
    let msg = c.refused("detect_pockets", json!({ "model": "EM-0600-4FL" }));
    assert!(msg.contains("is not a model"), "{msg}");
    let msg = c.refused("detect_pockets", json!({ "model": "part", "tool": "EM-0600-4FL" }));
    assert!(msg.contains("go with \"create\""), "{msg}");
    let msg = c.refused("detect_pockets", json!({ "model": "part", "create": "first" }));
    assert!(msg.contains("feature index"), "{msg}");
    let msg = c.refused("detect_pockets", json!({ "model": "part", "creat": 0 }));
    assert!(msg.contains("unknown argument"), "{msg}");

    let pocket = block_with_pocket(30.0, 20.0, &rounded_rect(15.0, 8.0, 0.0, 1), 8.0, 20.0);
    let (mut c, dir2) = job("refuse2", &pocket);
    let msg = c.refused("detect_pockets", json!({ "model": "part", "create": 0 }));
    assert!(msg.contains("\"tool\" is required"), "{msg}");
    let msg = c.refused("detect_pockets", json!({ "model": "part", "create": 0, "tool": "no-such-tool" }));
    assert!(!msg.is_empty(), "a missing tool is the create command's refusal: {msg}");
    assert_eq!(ops(&mut c), 0);
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&dir2);
}

/// #3545: a parameter that fails the door is named with its rule and its
/// value, so the caller can correct the call; the operation is unchanged.
#[test]
fn a_bad_pocket_parameter_is_refused_by_field_name() {
    let mesh = block_with_pocket(30.0, 20.0, &rounded_rect(15.0, 8.0, 4.0, 6), 8.0, 20.0);
    let (mut c, dir) = job("field-name", &mesh);
    c.ok("add_operation", json!({ "kind": "2D Pocket", "tool": "EM-0600-4FL", "name": "P1", "source": "part" }));
    let before = c.ok("get_job", json!({}))["operations"][0]["params"].clone();
    let msg = c.refused("edit_operation", json!({ "name": "P1", "params": { "stepdown": -3.0 } }));
    assert!(msg.contains("not usable for a 2D Pocket operation"), "{msg}");
    assert!(msg.contains("stepdown must be greater than 0 (got -3)"), "{msg}");
    assert_eq!(c.ok("get_job", json!({}))["operations"][0]["params"], before, "a refused edit changes nothing");
    let msg = c.refused("edit_operation", json!({ "name": "P1", "params": { "stock_to_leave": -0.5 } }));
    assert!(msg.contains("stock_to_leave must be 0 or more (got -0.5)"), "{msg}");
    let msg = c.refused("add_operation", json!({ "kind": "2D Pocket", "tool": "EM-0600-4FL", "name": "P2", "source": "part",
        "params": { "feed": 0.0 } }));
    assert!(msg.contains("feed must be greater than 0 (got 0)"), "{msg}");
    assert_eq!(ops(&mut c), 1, "the refused add creates nothing");
    let _ = std::fs::remove_dir_all(&dir);
}
