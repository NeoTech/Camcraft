//! #3915, for #3814: `import_model` with a name that a model already has
//! answers with `changes`: for the faces, the welded faces (patches) and the
//! edge chains, how many ids the update carried, added and lost, and the
//! window's one-line phrase. A first import has no `changes`.
//! Provenance: our own tests over our own tools; the boxes are written by
//! the test.
//!
//! Gate map: covers app::mcp (reimport_changes, tools import_model),
//! camcraft_geometry::geometry::revision.

use super::{Client, tmp_dir};
use serde_json::{Value, json};
use std::path::Path;

/// A 40 x 40 box `dz` high as ASCII STL, z from 0, wound outward.
fn write_box(path: &Path, dz: f64) -> String {
    let v = |i: usize| -> [f64; 3] {
        [
            if i & 1 == 0 { -20.0 } else { 20.0 },
            if i & 2 == 0 { -20.0 } else { 20.0 },
            if i & 4 == 0 { 0.0 } else { dz },
        ]
    };
    let quads = [
        [0, 2, 3, 1],
        [4, 5, 7, 6],
        [0, 1, 5, 4],
        [2, 6, 7, 3],
        [0, 4, 6, 2],
        [1, 3, 7, 5],
    ];
    let mut s = String::from("solid box\n");
    for q in quads {
        for tri in [[q[0], q[1], q[2]], [q[0], q[2], q[3]]] {
            s.push_str(" facet normal 0 0 0\n  outer loop\n");
            for i in tri {
                let p = v(i);
                s.push_str(&format!("   vertex {} {} {}\n", p[0], p[1], p[2]));
            }
            s.push_str("  endloop\n endfacet\n");
        }
    }
    s.push_str("endsolid box\n");
    std::fs::write(path, s).expect("write stl");
    path.display().to_string()
}

/// The id totals `list_features` gives for `part`.
fn totals(c: &mut Client) -> (u64, u64) {
    let f = c.ok(
        "list_features",
        json!({ "model": "part", "kinds": ["faces", "patches"] }),
    );
    (
        f["faces_total"].as_u64().expect("faces_total"),
        f["patches_total"].as_u64().expect("patches_total"),
    )
}

fn count(changes: &Value, domain: &str, key: &str) -> u64 {
    changes[domain][key]
        .as_u64()
        .unwrap_or_else(|| panic!("changes.{domain}.{key} is a count: {changes}"))
}

#[test]
fn a_first_import_answers_with_no_changes() {
    let dir = tmp_dir("reimport-first");
    let mut c = Client::new();
    let out = c.ok(
        "import_model",
        json!({ "path": write_box(&dir.join("part.stl"), 15.0) }),
    );
    assert_eq!(out["model"], json!("part"), "{out}");
    assert!(
        out.get("changes").is_none(),
        "a new model has nothing to compare: {out}"
    );
    // A second model under another name is a first import too.
    let other = c.ok(
        "import_model",
        json!({ "path": write_box(&dir.join("other.stl"), 25.0) }),
    );
    assert!(other.get("changes").is_none(), "{other}");
}

#[test]
fn an_update_counts_what_moved_in_each_domain() {
    let dir = tmp_dir("reimport-moved");
    let mut c = Client::new();
    c.ok(
        "import_model",
        json!({ "path": write_box(&dir.join("part.stl"), 15.0), "name": "part" }),
    );
    let (old_faces, old_patches) = totals(&mut c);
    // A taller box under the same name: the top and the four sides move.
    let out = c.ok(
        "import_model",
        json!({ "path": write_box(&dir.join("tall.stl"), 25.0), "name": "part" }),
    );
    let ch = &out["changes"];
    assert!(ch.is_object(), "an update answers with changes: {out}");
    let (new_faces, new_patches) = totals(&mut c);
    // The counts come from the two id sets: carried + missing is the old
    // set, carried + added is the new one.
    assert_eq!(
        count(ch, "faces", "carried") + count(ch, "faces", "missing"),
        old_faces,
        "{ch}"
    );
    assert_eq!(
        count(ch, "faces", "carried") + count(ch, "faces", "added"),
        new_faces,
        "{ch}"
    );
    assert_eq!(
        count(ch, "patches", "carried") + count(ch, "patches", "missing"),
        old_patches,
        "{ch}"
    );
    assert_eq!(
        count(ch, "patches", "carried") + count(ch, "patches", "added"),
        new_patches,
        "{ch}"
    );
    assert!(
        count(ch, "faces", "added") > 0 && count(ch, "faces", "missing") > 0,
        "the top face moved: {ch}"
    );
    for key in ["carried", "added", "missing"] {
        count(ch, "chains", key);
    }
    let summary = ch["summary"].as_str().unwrap_or_default();
    assert!(
        summary.ends_with("changed") && summary.contains("face"),
        "the window's phrase: {ch}"
    );
    // The rest of the answer is as before.
    assert_eq!(out["model"], json!("part"), "{out}");
    assert!(out["facets"].as_u64().is_some_and(|n| n > 0), "{out}");
}

#[test]
fn the_same_file_again_carries_every_id() {
    let dir = tmp_dir("reimport-same");
    let mut c = Client::new();
    let path = write_box(&dir.join("part.stl"), 15.0);
    c.ok("import_model", json!({ "path": path }));
    let (faces, _) = totals(&mut c);
    // No name: the file stem names the same model, so this is an update.
    let out = c.ok("import_model", json!({ "path": path }));
    let ch = &out["changes"];
    assert_eq!(count(ch, "faces", "carried"), faces, "{ch}");
    for domain in ["faces", "patches", "chains"] {
        assert_eq!(count(ch, domain, "added"), 0, "{domain}: {ch}");
        assert_eq!(count(ch, domain, "missing"), 0, "{domain}: {ch}");
    }
    assert_eq!(ch["summary"], json!("geometry unchanged"), "{ch}");
}

#[test]
fn a_refused_update_answers_with_no_changes() {
    let dir = tmp_dir("reimport-refused");
    let mut c = Client::new();
    c.ok(
        "import_model",
        json!({ "path": write_box(&dir.join("part.stl"), 15.0) }),
    );
    let bad = dir.join("broken.stl");
    std::fs::write(&bad, "solid nothing\nendsolid nothing\n").expect("write stl");
    // The refusal is the importer's, by name; the model keeps its geometry.
    let why = c.refused(
        "import_model",
        json!({ "path": bad.display().to_string(), "name": "part" }),
    );
    assert!(!why.is_empty());
    let (faces, _) = totals(&mut c);
    assert!(faces > 0, "the model is as it was");
}
