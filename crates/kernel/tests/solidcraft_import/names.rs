//! The face names of a Solidcraft design (#3915): from model cache format
//! 10, SolidCraft's name of each face of the final body is read beside it,
//! one for each record face, in record order. Names never refuse an import:
//! names that do not fit are dropped with one note. The fixture is the
//! sample design marked as format 10, with its final body replaced by a
//! block made and encoded with the pinned solidcore crates (`solidcore-ops`,
//! `native::encode`), and with names of our own: each face is named by the
//! plane it lies on, so a pin can check that every triangle of a named face
//! lies on that plane. Tiling alone does not prove that: a shifted name list
//! covers every face too.
//! Provenance: our own fixture, built by the test. The names layout is
//! SolidCraft's published crates/doc/src/model_cache.rs (v0.50.0).

use std::path::PathBuf;

use camcraft_kernel::machining_mesh::{MachiningMesh, machining_mesh};
use camcraft_kernel::solidcraft::{NAMES_FORMAT, SolidcraftImport, SourceName};
use serde_json::{Value, json};
use solidcore::kernel::{CancelToken, UpstreamFixes};
use solidcore_record::brep::{Brep as Record, Loop, envelope};
use solidcore_record::native::{self, Runtime, Saved};

use super::{import, last_slice};
use crate::sc_util::{Parts, sample_bytes};

/// The sample's final model body. Its body table entry and its payload
/// slice have the same index, and so does its names record.
pub(super) const FINAL: usize = 13;
/// The format the sample was written in: its names are not read.
const FORMAT_8: &str = "solidcraft-model-cache/8";
/// The chord the pins mesh at (mm).
const CHORD: f64 = 0.005;
/// How far a corner may lie off its named plane (mm).
const ON_PLANE: f64 = 1e-6;

/// The 10 x 20 x 30 block as a record: six faces, each on an axis plane.
pub(super) fn block() -> Record {
    solidcore_ops::session::set_thread_fixes(UpstreamFixes::NAMES.iter().copied())
        .expect("every fix on, as Solidcraft runs");
    let b = solidcore_ops::make_box([0.0; 3], [10.0, 20.0, 30.0]).expect("block");
    let (record, _) = b.to_record().expect("record of the block");
    record
}

/// The axis and the coordinate that every corner of record face `f`
/// shares. Read from the face's loops, not from its surface kind.
fn plane_of(r: &Record, f: usize) -> (usize, f64) {
    let mut corners: Vec<[f64; 3]> = Vec::new();
    for l in &r.faces[f].loops {
        match l {
            Loop::Edges { edges } => {
                for e in edges {
                    for v in r.edges[e.edge as usize].vertices {
                        corners.push(r.vertices[v as usize].point);
                    }
                }
            }
            Loop::Vertex { vertex } => corners.push(r.vertices[*vertex as usize].point),
        }
    }
    let first = corners[0];
    match (0..3).find(|&a| corners.iter().all(|p| p[a] == first[a])) {
        Some(a) => (a, first[a]),
        None => panic!("face {f} of the block lies on no axis plane: {corners:?}"),
    }
}

/// Our own name of each record face, in record order: its plane, `x=10`.
pub(super) fn names_of(r: &Record) -> Vec<String> {
    (0..r.faces.len())
        .map(|f| {
            let (a, c) = plane_of(r, f);
            format!("{}={c}", ["x", "y", "z"][a])
        })
        .collect()
}

/// The plane a name of [`names_of`] gives.
fn plane_named(name: &str) -> (usize, f64) {
    let (axis, at) = name
        .split_once('=')
        .unwrap_or_else(|| panic!("'{name}' is not a plane"));
    let a = ["x", "y", "z"].iter().position(|x| *x == axis);
    match (a, at.parse::<f64>()) {
        (Some(a), Ok(c)) => (a, c),
        _ => panic!("'{name}' is not a plane"),
    }
}

/// One info item as SolidCraft writes it: `[aliases, bounds, verified,
/// synonyms]`, and the former name as a fifth item when there is one.
pub(super) fn info(verified: bool, former: Option<&str>) -> Value {
    match former {
        Some(f) => json!([[], [], verified, [], f]),
        None => json!([[], [], verified, []]),
    }
}

/// A names record `{faces, info}`, with `info(i)` the item of face `i`.
pub(super) fn names_record(names: &[String], info: impl Fn(usize) -> Value) -> Value {
    json!({ "faces": names, "info": (0..names.len()).map(info).collect::<Vec<_>>() })
}

/// The sample with `record` as its final body and `change` made to its
/// manifest, re-keyed.
pub(super) fn design_with(record: Record, change: impl FnOnce(&mut Value)) -> Vec<u8> {
    let saved = Saved {
        solid: record,
        metadata: envelope::Metadata {
            version: envelope::VERSION,
            color: None,
            paint: None,
            boundaries: None,
            face_sources: None,
            history: None,
            preserve_boundaries: false,
        },
        runtime: Runtime {
            size: 0.0,
            coarse_missing: None,
        },
    };
    let body = native::encode(&saved).expect("encode the block");
    let mut p = Parts::sample();
    let (at, _) = last_slice(&p);
    let mut payload = p.get("cache/bodies.bin");
    payload.truncate(at);
    payload.extend_from_slice(&body);
    p.set("cache/bodies.bin", payload);
    p.manifest(|m| {
        m["slices"][FINAL][1] = json!(body.len());
        change(m);
    });
    p.rekey();
    p.build()
}

/// The sample with `record` as its final body, its cache marked `format`
/// and `names` as its final body's names record.
fn design(record: Record, format: &str, names: Value) -> Vec<u8> {
    design_with(record, |m| {
        m["format"] = json!(format);
        m["model"]["model_bodies"][FINAL]["names"] = json!(FINAL);
        m["model"]["names"][FINAL] = names;
    })
}

#[track_caller]
fn ok(bytes: &[u8]) -> SolidcraftImport {
    match import(bytes) {
        Ok(i) => i,
        Err(e) => panic!("the design is refused: {e}"),
    }
}

/// Every triangle of `mesh` lies on the plane that its face's name gives,
/// and every name has a triangle. `Err` names the first that does not.
fn aligned(mesh: &MachiningMesh, names: &[String]) -> Result<(), String> {
    let mut seen = vec![false; names.len()];
    for (t, (tri, f)) in mesh.triangles.iter().zip(&mesh.tri_face).enumerate() {
        let f = *f as usize;
        let name = names
            .get(f)
            .ok_or_else(|| format!("triangle {t}: face {f} has no name"))?;
        let (a, c) = plane_named(name);
        for v in tri {
            let p = mesh.positions[*v as usize];
            if (p[a] - c).abs() > ON_PLANE {
                return Err(format!(
                    "triangle {t} of face {f} ('{name}') has a corner at {p:?}"
                ));
            }
        }
        seen[f] = true;
    }
    match seen.iter().position(|s| !s) {
        Some(f) => Err(format!("face {f} ('{}') has no triangle", names[f])),
        None => Ok(()),
    }
}

#[test]
fn format_10_names_are_read_one_for_each_record_face() {
    let record = block();
    let names = names_of(&record);
    assert_eq!(names.len(), 6, "a block has six faces");
    // Face 0: a Split Face piece with its former name (five items). Face 1:
    // a name worked out from geometry. Face 2: a former of null, which
    // SolidCraft's reader takes as none. The rest: four items, verified.
    let items = |i: usize| match i {
        0 => info(true, Some("F1:side:l1")),
        1 => info(false, None),
        2 => json!([[], ["F1:start"], true, ["F1:other"], null]),
        _ => info(true, None),
    };
    let i = ok(&design(record, NAMES_FORMAT, names_record(&names, items)));
    assert_eq!(i.names_note, None);
    let got = i.names.expect("format-10 names are read");
    assert_eq!(got.len(), i.solid.record.faces.len());
    let want: Vec<SourceName> = names
        .iter()
        .enumerate()
        .map(|(k, n)| SourceName {
            name: n.clone(),
            verified: k != 1,
            former: (k == 0).then(|| "F1:side:l1".to_owned()),
        })
        .collect();
    assert_eq!(got, want);
}

#[test]
fn every_triangle_of_a_named_face_lies_on_that_faces_plane() {
    let record = block();
    let names = names_of(&record);
    let bytes = design(
        record,
        NAMES_FORMAT,
        names_record(&names, |_| info(true, None)),
    );
    let i = ok(&bytes);
    let read: Vec<String> = match &i.names {
        Some(n) => n.iter().map(|n| n.name.clone()).collect(),
        None => panic!("no names: {:?}", i.names_note),
    };
    assert_eq!(read, names, "the names come back in record order");
    let mesh = match machining_mesh(&i.solid, CHORD, &CancelToken::new()) {
        Ok(m) => m,
        Err(e) => panic!("the block does not mesh: {e}"),
    };
    assert_eq!(mesh.tri_face.len(), mesh.triangles.len());
    if let Err(e) = aligned(&mesh, &read) {
        panic!("{e}");
    }
    // The pin can fail: the same names one place on cover every face too.
    let mut shifted = read.clone();
    shifted.rotate_left(1);
    assert!(
        aligned(&mesh, &shifted).is_err(),
        "a shifted name list must not pass"
    );
}

#[test]
fn a_format_8_cache_reads_no_names_and_says_nothing() {
    // The sample as written (format 8) holds names for its final body.
    let m = Parts::sample().json("cache/model.json");
    assert_eq!(m["format"], json!(FORMAT_8));
    let held = m["model"]["names"][FINAL]["faces"]
        .as_array()
        .map_or(0, Vec::len);
    assert!(held > 0, "the sample holds names");
    // They are not read, and nothing is said: it imports as before.
    let i = ok(&sample_bytes());
    assert!(i.names.is_none());
    assert_eq!(i.names_note, None);
    // Names that fit, in a cache marked format 8: still none, no note.
    let record = block();
    let names = names_of(&record);
    let i = ok(&design(
        record,
        FORMAT_8,
        names_record(&names, |_| info(true, None)),
    ));
    assert!(i.names.is_none());
    assert_eq!(i.names_note, None);
}

/// SolidCraft 0.46.2's own format-10 file, which `hole_on_pocket_wall.rs`
/// already reads. Keep it only if the maintainer allows real files as
/// fixtures here (C13 question Q4); the block pins above do not need it.
#[test]
fn a_solidcraft_written_file_names_its_faces_in_record_order() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../testdata/models/hole_on_pocket_wall.sc");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let i = ok(&bytes);
    let names = match &i.names {
        Some(n) => n,
        None => panic!("no names: {:?}", i.names_note),
    };
    assert_eq!(names.len(), i.solid.record.faces.len());
    assert_eq!(names.len(), 13);
    let mesh = match machining_mesh(&i.solid, CHORD, &CancelToken::new()) {
        Ok(m) => m,
        Err(e) => panic!("the file does not mesh: {e}"),
    };
    let corners = |name: &str| -> Vec<[f64; 3]> {
        let f = match names.iter().position(|n| n.name == name) {
            Some(f) => f as u32,
            None => panic!("no face named {name}"),
        };
        mesh.triangles
            .iter()
            .zip(&mesh.tri_face)
            .filter(|(_, g)| **g == f)
            .flat_map(|(t, _)| t.map(|v| mesh.positions[v as usize]))
            .collect()
    };
    // The plate's two caps: each flat on one axis plane, the same axis, and
    // at the two ends of the body on it.
    let (top, start) = (corners("F2:top"), corners("F2:start"));
    let flat = |c: &[[f64; 3]]| {
        (0..3).find(|&a| !c.is_empty() && c.iter().all(|p| (p[a] - c[0][a]).abs() < ON_PLANE))
    };
    let a = match (flat(&top), flat(&start)) {
        (Some(a), Some(b)) if a == b => a,
        other => panic!("the caps are not flat on one axis: {other:?}"),
    };
    let lo = mesh
        .positions
        .iter()
        .map(|p| p[a])
        .fold(f64::INFINITY, f64::min);
    let hi = mesh
        .positions
        .iter()
        .map(|p| p[a])
        .fold(f64::NEG_INFINITY, f64::max);
    let (t, s) = (top[0][a], start[0][a]);
    assert!(
        (t.min(s) - lo).abs() < ON_PLANE && (t.max(s) - hi).abs() < ON_PLANE,
        "caps at {t} and {s}, body from {lo} to {hi}"
    );
}
