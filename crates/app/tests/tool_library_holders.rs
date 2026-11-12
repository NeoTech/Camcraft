//! #889: a `.camtools` tool may carry its body (the drawn holder
//! profile, or a placed mesh) and the import takes it or not by the
//! `holders` parameter of the one `ImportTools` command (the Tool
//! library page's "Import tool holders", MCP's `holders`). Pinned: the
//! export -> import -> export round trip is byte-stable with the body;
//! holders off lands every tool bare and says so; a malformed or
//! ill-fitting body refuses the file by name with nothing applied; a
//! file without bodies imports as before; the skip compares the body
//! only when one lands; one undo removes tools and bodies; MCP's flag.
//! Provenance: our own tests over our own format and commands.
//!
//! Gate map (M4.4): covers app::tool_library, app::commands (tool_import),
//! app::mcp (import_tool_library), app::script (importTools).

use camcraft_app::commands::{AddTool, History, ImportReport, ImportTools, SetToolBody, ToolBodySpec};
use camcraft_app::geometry::{Point3, Triangle};
use camcraft_app::mcp::Server;
use camcraft_app::model;
use camcraft_app::project::ToolBodyDto;
use camcraft_app::tool_body::body_of;
use camcraft_app::tool_library::{bodies_in, export_tool_library, parse_tool_library};
use camcraft_app::tools::{Tool, catalog};
use camcraft_app::world::new_world;
use flecs_ecs::prelude::World;
use serde_json::{Value, json};

const DRAWN: &str = "Six";
const MESH: &str = "Four";
const BARE: &str = "Eight";

fn cat(name: &str) -> Tool {
    catalog().into_iter().find(|(n, _)| *n == name).map(|(_, t)| t).expect("catalog tool")
}

fn prism(r: f64, z0: f64, z1: f64, n: usize) -> Vec<Triangle> {
    let p = |x: f64, y: f64, z: f64| Point3::new(x, y, z);
    let q = |k: usize, z: f64| {
        let a = std::f64::consts::TAU * k as f64 / n as f64;
        p(r * a.cos(), r * a.sin(), z)
    };
    let mut v = Vec::new();
    for k in 0..n {
        let (a0, a1, b0, b1) = (q(k, z0), q(k + 1, z0), q(k, z1), q(k + 1, z1));
        v.push(Triangle::new(a0, a1, b1));
        v.push(Triangle::new(a0, b1, b0));
        v.push(Triangle::new(p(0.0, 0.0, z0), a1, a0));
        v.push(Triangle::new(p(0.0, 0.0, z1), b0, b1));
    }
    v
}

/// A document: a 6 mm end mill on a BT30 ER20 (drawn), a 4 mm one under
/// a meshed collar, and an 8 mm one with its holder cylinder only.
fn source() -> (World, History) {
    let w = new_world();
    let mut h = History::default();
    for (n, c) in [(DRAWN, "EM-0600-4FL"), (MESH, "EM-0400-2FL"), (BARE, "EM-0800-4FL")] {
        h.execute(&w, AddTool::new(&w, n, cat(c)).expect("add"));
    }
    h.execute(&w, SetToolBody::library(&w, DRAWN, "BT30 ER20", None, None).expect("holder"));
    let t = cat("EM-0400-2FL");
    let z0 = t.flute_len + t.shank_len;
    let spec = ToolBodySpec { source: "collar.stl".into(), facets: prism(40.0, z0, z0 + 35.0, 32), axis: [0.0, 0.0, 1.0], offset: 0.0 };
    h.execute(&w, SetToolBody::new(&w, MESH, Some(spec)).expect("mesh body"));
    (w, h)
}

fn body_dto(w: &World, n: &str) -> Option<ToolBodyDto> {
    model::find_by_name(w, n).and_then(|e| body_of(w, &e)).map(|b| ToolBodyDto::from_body(&b))
}

fn import(w: &World, h: &mut History, text: &str, holders: bool) -> Result<ImportReport, String> {
    let lib = parse_tool_library(text)?;
    let cmd = ImportTools::with_holders(w, lib, holders).map_err(|e| e.to_string())?;
    let r = cmd.report().clone();
    h.execute(w, cmd);
    Ok(r)
}

#[test]
fn export_import_with_holders_is_byte_stable() {
    let (src, _) = source();
    let text = export_tool_library(&src).expect("export");
    let lib = parse_tool_library(&text).expect("parse");
    assert_eq!(bodies_in(&lib), 2, "the drawn and the meshed body ride the file");
    let (dst, mut h) = (new_world(), History::default());
    let r = import(&dst, &mut h, &text, true).expect("import");
    assert_eq!(r.holders, 2, "{}", r.summary());
    assert!(r.summary().contains("2 with their holder"), "{}", r.summary());
    for n in [DRAWN, MESH] {
        assert_eq!(body_dto(&dst, n), body_dto(&src, n), "{n}: the body arrives bit-identical");
        assert!(body_dto(&dst, n).is_some());
    }
    assert_eq!(body_dto(&dst, BARE), None);
    assert_eq!(export_tool_library(&dst).expect("re-export"), text, "export -> import -> export is byte-stable");
    // The default constructor takes holders.
    let (dst2, mut h2) = (new_world(), History::default());
    let cmd = ImportTools::new(&dst2, parse_tool_library(&text).expect("parse")).expect("plan");
    h2.execute(&dst2, cmd);
    assert_eq!(body_dto(&dst2, DRAWN), body_dto(&src, DRAWN));
}

#[test]
fn holders_off_lands_every_tool_bare_and_undo_restores() {
    let (src, _) = source();
    let text = export_tool_library(&src).expect("export");
    let (dst, mut h) = (new_world(), History::default());
    let r = import(&dst, &mut h, &text, false).expect("import");
    assert_eq!((r.holders, r.holders_left_out, r.added.len()), (0, 2, 3), "{}", r.summary());
    assert!(r.summary().contains("2 holders in the file not imported"), "{}", r.summary());
    for n in [DRAWN, MESH, BARE] {
        assert!(model::find_by_name(&dst, n).is_some(), "{n} arrives");
        assert_eq!(body_dto(&dst, n), None, "{n}: holders off lands the holder cylinder only");
    }
    assert!(h.undo(&dst), "one undo step");
    assert!([DRAWN, MESH, BARE].iter().all(|n| model::find_by_name(&dst, n).is_none()));
    let r = import(&dst, &mut h, &text, true).expect("import with holders");
    assert_eq!(r.holders, 2);
    assert!(h.undo(&dst));
    assert!(model::find_by_name(&dst, DRAWN).is_none(), "undo removes the tool and its body");
    assert!(h.redo(&dst));
    assert_eq!(body_dto(&dst, DRAWN), body_dto(&src, DRAWN), "redo brings the body back");
}

/// The file's `definition` of tool `n`, for corrupting.
fn with_def(text: &str, n: &str, f: impl Fn(&mut Value)) -> String {
    let mut doc: Value = serde_json::from_str(text).expect("json");
    if let Some(rows) = doc["tools"].as_array_mut() {
        for row in rows.iter_mut().filter(|r| r["name"] == json!(n)) {
            f(&mut row["definition"]);
        }
    }
    doc.to_string()
}

#[test]
fn a_malformed_or_ill_fitting_body_refuses_the_file_by_name() {
    let (src, _) = source();
    let text = export_tool_library(&src).expect("export");
    let cases = [
        // Not a body at all.
        with_def(&text, DRAWN, |d| d["body"] = json!({ "nonsense": 1 })),
        // A drawn outline that is no outline.
        with_def(&text, DRAWN, |d| d["body"]["spun"] = json!([[0.0, 1.0, 0.0]])),
        // A mesh with no placement.
        with_def(&text, MESH, |d| {
            if let Some(o) = d["body"].as_object_mut() {
                o.remove("axis");
            }
        }),
        // Flutes that now run up into the holder: the body no longer fits.
        with_def(&text, DRAWN, |d| d["flute_len"] = json!(200.0)),
    ];
    for bad in cases {
        let dst = new_world();
        let mut h = History::default();
        let err = import(&dst, &mut h, &bad, true).expect_err("refused");
        assert!(err.contains(&format!("'{DRAWN}'")) || err.contains(&format!("'{MESH}'")), "named: {err}");
        assert!(model::named_entities(&dst).iter().all(|(_, e)| model::tool_of(&dst, e).is_none()), "nothing applied");
        assert!(!h.undo(&dst), "no history entry");
    }
}

#[test]
fn a_file_without_bodies_imports_as_before_and_the_skip_reads_bodies_only_when_one_lands() {
    let (src, _) = source();
    let text = export_tool_library(&src).expect("export");
    // A pre-#889 file: no body keys at all.
    let old = with_def(&with_def(&text, DRAWN, |d| { d.as_object_mut().map(|o| o.remove("body")); }), MESH, |d| {
        d.as_object_mut().map(|o| o.remove("body"));
    });
    assert!(!old.contains("\"body\""));
    let lib = parse_tool_library(&old).expect("an old file reads");
    assert_eq!(bodies_in(&lib), 0);
    // Into the source (bodied tools): a bare library tool matches on its
    // record - nothing to import, the user's fitted holders stay.
    let mut h = History::default();
    let err = import(&src, &mut h, &old, true).expect_err("all skipped");
    assert!(err.contains("nothing to import"), "{err}");
    // A bodied library onto the same tools held bare: the body differs,
    // so the tool arrives renamed with its holder - the bare one stays.
    let bare = new_world();
    let mut h = History::default();
    for (n, c) in [(DRAWN, "EM-0600-4FL"), (MESH, "EM-0400-2FL"), (BARE, "EM-0800-4FL")] {
        h.execute(&bare, AddTool::new(&bare, n, cat(c)).expect("add"));
    }
    let r = import(&bare, &mut h, &text, true).expect("import");
    assert_eq!(r.skipped, vec![BARE.to_owned()], "{}", r.summary());
    assert_eq!(r.renamed.len(), 2, "{}", r.summary());
    assert_eq!(body_dto(&bare, DRAWN), None, "the document's own tool is never overwritten");
    assert_eq!(body_dto(&bare, &format!("{DRAWN} (2)")), body_dto(&src, DRAWN));
    // Holders off, the same import skips all three.
    assert!(h.undo(&bare));
    let err = import(&bare, &mut h, &text, false).expect_err("all skipped");
    assert!(err.contains("nothing to import"), "{err}");
}

fn call(s: &mut Server, id: i64, name: &str, args: Value) -> (bool, Value) {
    let msg = json!({ "jsonrpc": "2.0", "id": id, "method": "tools/call", "params": { "name": name, "arguments": args } });
    let reply = s.handle(&msg).expect("a reply");
    let r = &reply["result"];
    let err = r["isError"] == json!(true);
    (err, if err { r["content"][0]["text"].clone() } else { r["structuredContent"].clone() })
}

#[test]
fn mcp_import_tool_library_takes_the_holders_flag() {
    let dir = std::env::temp_dir().join(format!("camcraft-toollib-holders-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("dir");
    let lib = dir.join("shop.camtools");
    let (src, _) = source();
    let text = export_tool_library(&src).expect("export");
    std::fs::write(&lib, &text).expect("write");
    let path = lib.display().to_string();

    let mut s = Server::new();
    s.handle(&json!({ "jsonrpc": "2.0", "id": 0, "method": "initialize", "params": { "protocolVersion": "2025-06-18" } }));
    let list = s.handle(&json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" })).expect("list");
    let tool = list["result"]["tools"].as_array().and_then(|t| t.iter().find(|t| t["name"] == json!("import_tool_library")).cloned());
    assert_eq!(tool.expect("listed")["inputSchema"]["properties"]["holders"]["type"], json!("boolean"));

    let (err, off) = call(&mut s, 2, "import_tool_library", json!({ "path": path, "holders": false }));
    assert!(!err, "{off}");
    assert_eq!((off["holders"].clone(), off["holders_left_out"].clone()), (json!(0), json!(2)), "{off}");
    let out = dir.join("off.camtools");
    let (err, _) = call(&mut s, 3, "export_tool_library", json!({ "path": out.display().to_string() }));
    assert!(!err);
    assert!(!std::fs::read_to_string(&out).expect("read").contains("\"body\""), "holders off: no bodies");
    let (err, _) = call(&mut s, 4, "undo", json!({}));
    assert!(!err);

    let (err, on) = call(&mut s, 5, "import_tool_library", json!({ "path": path }));
    assert!(!err, "{on}");
    assert_eq!(on["holders"], json!(2), "holders default to true: {on}");
    let (err, _) = call(&mut s, 6, "export_tool_library", json!({ "path": out.display().to_string() }));
    assert!(!err);
    assert_eq!(std::fs::read_to_string(&out).expect("read"), text, "the bodies round-trip over MCP");

    let (err, msg) = call(&mut s, 7, "import_tool_library", json!({ "path": path, "holders": "yes" }));
    assert!(err && msg.as_str().is_some_and(|m| m.contains("holders")), "{msg}");
    let _ = std::fs::remove_dir_all(&dir);
}
