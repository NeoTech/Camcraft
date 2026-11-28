//! #4081: the Laser Profile through headless MCP - a laser .mch mounted,
//! the starter laser head from the catalog, the op added with tabs,
//! edited (the power), undone, redone and generated; a power out of
//! range is refused by name.
//! Provenance: our own MCP tests; the holed plate follows mcp_waterjet.rs.

use super::Client;
use serde_json::{json, Value};
use std::fmt::Write as _;
use std::path::PathBuf;

const LASER: &str = r#"{"general":{"model":"Laser test","kind":"laser"},"kinematics":{"default":{"parts":[{"name":"X","type":"linear","travel":{"min":-100,"max":360}},{"name":"Y","type":"linear","travel":{"min":-150,"max":150}},{"name":"Z","type":"linear","travel":{"min":-20,"max":60}},{"name":"laser head","type":"head"}]}}}"#;

fn tmp_dir() -> PathBuf {
    let path = std::env::temp_dir().join(format!("camcraft-mcp-laser-4081-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("temp directory");
    path
}

/// A 40 x 40 x 3 mm plate with a 10 x 10 mm through hole, as ASCII STL.
fn holed_plate_stl() -> String {
    let ring = |h: f64, z: f64| [[-h, -h, z], [h, -h, z], [h, h, z], [-h, h, z]];
    let mut tris: Vec<[[f64; 3]; 3]> = Vec::new();
    let mut quad = |a: [f64; 3], b: [f64; 3], c: [f64; 3], d: [f64; 3]| {
        tris.push([a, b, c]);
        tris.push([a, c, d]);
    };
    for (z, up) in [(3.0, true), (0.0, false)] {
        let (outer, inner) = (ring(20.0, z), ring(5.0, z));
        for i in 0..4 {
            let j = (i + 1) % 4;
            if up { quad(outer[i], outer[j], inner[j], inner[i]) } else { quad(outer[i], inner[i], inner[j], outer[j]) }
        }
    }
    for (radius, outward) in [(20.0, true), (5.0, false)] {
        let (lo, hi) = (ring(radius, 0.0), ring(radius, 3.0));
        for i in 0..4 {
            let j = (i + 1) % 4;
            if outward { quad(lo[i], lo[j], hi[j], hi[i]) } else { quad(lo[i], hi[i], hi[j], lo[j]) }
        }
    }
    let mut s = String::from("solid plate\n");
    for t in tris {
        s.push_str("facet normal 0 0 0\nouter loop\n");
        for v in t {
            let _ = writeln!(s, "vertex {} {} {}", v[0], v[1], v[2]);
        }
        s.push_str("endloop\nendfacet\n");
    }
    s.push_str("endsolid plate\n");
    s
}

fn params(c: &mut Client) -> Value {
    let job = c.ok("get_job", json!({}));
    job["operations"].as_array().and_then(|ops| ops.iter().find(|o| o["name"] == json!("Laser1"))).expect("Laser1")["params"]
        ["laser_profile"]
        .clone()
}

#[test]
fn laser_profile_is_added_edited_undone_redone_and_generated_over_mcp() {
    let dir = tmp_dir();
    let mch = dir.join("laser.mch");
    std::fs::write(&mch, LASER).expect("write mch");
    let stl = dir.join("plate.stl");
    std::fs::write(&stl, holed_plate_stl()).expect("write stl");
    let mut c = Client::new();
    c.ok("load_machine", json!({ "path": mch }));
    c.ok("add_tool", json!({ "catalog": "LZ-KERF-020", "name": "Beam" }));
    c.ok("import_model", json!({ "path": stl, "name": "plate" }));
    let kinds = c.ok("list_operation_types", json!({}));
    assert!(kinds.to_string().contains("Laser Profile"), "the kind is listed");
    let added = c.ok("add_operation", json!({
        "kind": "Laser Profile", "name": "Laser1", "tool": "Beam", "source": "plate",
        "params": { "clearance_z": 10.0, "top_z": 3.0, "bottom_z": 0.0, "feed": 1200.0,
            "tabs": { "count": 1, "width": 1.0, "at": [[20.0, 0.0]] } }
    }));
    assert_eq!(added["kind"], json!("Laser Profile"), "{added}");
    let before = params(&mut c);
    assert_eq!((before["tabs"]["count"].clone(), before["power"].clone()), (json!(1), json!(50.0)), "{before}");
    let edited = c.ok("edit_operation", json!({ "name": "Laser1", "params": { "power": 85.0 } }));
    assert_eq!(edited["params"]["laser_profile"]["power"], json!(85.0), "{edited}");
    c.ok("undo", json!({}));
    assert_eq!(params(&mut c), before);
    c.ok("redo", json!({}));
    assert_eq!(params(&mut c)["power"], json!(85.0));
    let refused = c.refused("edit_operation", json!({ "name": "Laser1", "params": { "power": 120.0 } }));
    assert!(!refused.is_empty(), "a power above 100 % is refused");
    assert_eq!(params(&mut c)["power"], json!(85.0), "the refusal changes nothing");
    let generated = c.ok("generate", json!({ "name": "Laser1" }));
    assert_eq!(generated["generated"][0]["status"], json!("ok"), "{generated}");
    let _ = std::fs::remove_dir_all(&dir);
}
