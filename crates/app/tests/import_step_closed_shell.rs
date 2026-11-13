//! #1573: a STEP shell the file declares closed must mesh closed, or the
//! import is refused by name; a face the kernel returns no mesh for is
//! refused by name on its own. Never a model with a wall missing.
//!
//! Fixture: `testdata/models/fillet_corner.step` with its corner blend's
//! weights bent (`1E300, 1E-300`), which truck cannot place a boundary on:
//! it returns no polygon for the blend and the rest of the shell imported
//! open, with no refusal, before this law.
//!
//! Gate map (M4.4): covers app::geometry::import::step,
//! app::geometry::import::step_closed.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use camcraft_app::geometry::import::{self, step, Format, ImportError};

fn model(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/models").join(name)
}

fn within<R: Send + 'static>(limit: Duration, f: impl FnOnce() -> R + Send + 'static) -> R {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    match rx.recv_timeout(limit) {
        Ok(r) => r,
        Err(_) => panic!("import did not finish within {limit:?}"),
    }
}

/// The fixture with its corner blend bent so truck drops the blend face.
fn bent() -> String {
    let text = std::fs::read_to_string(model("fillet_corner.step")).expect("fixture");
    let weights = "RATIONAL_B_SPLINE_SURFACE(((1.0,1.0,2.0),";
    assert_eq!(text.matches(weights).count(), 1, "fixture layout changed");
    assert!(text.contains("#1288=CLOSED_SHELL('',("), "fixture layout changed");
    text.replace(weights, "RATIONAL_B_SPLINE_SURFACE(((1.0E300,1.0E-300,2.0),")
}

fn import(text: String) -> Result<camcraft_app::geometry::Mesh, ImportError> {
    within(Duration::from_secs(90), move || {
        step::with_face_eval_cap(200_000, || import::parse_model(text.as_bytes(), Format::Step))
    })
}

#[test]
fn a_closed_shell_that_meshes_open_is_refused_by_name() {
    let err = import(bent()).expect_err("a closed shell with a face missing must not import");
    let msg = err.to_string();
    assert!(msg.contains("closed shell #1288 (10 faces) meshes open"), "{msg}");
    assert!(msg.contains("unmatched edges, the first from ("), "{msg}");
    assert!(msg.contains("got no mesh from the kernel"), "the dropped face is named: {msg}");
}

#[test]
fn a_face_the_kernel_cannot_mesh_is_refused_in_an_open_shell_too() {
    // Declared open, and no solid naming it: the closed-shell law does not
    // apply, but a face left out is still a wall the file has and the mesh
    // does not.
    let text = bent()
        .replace("#1288=CLOSED_SHELL(", "#1288=OPEN_SHELL(")
        .replace("#1289=MANIFOLD_SOLID_BREP('',#1288);", "#1289=SHELL_BASED_SURFACE_MODEL('',(#1288));");
    let err = import(text).expect_err("a dropped face must not import silently");
    let msg = err.to_string();
    assert!(msg.contains("in shell #1288 could not be tessellated"), "{msg}");
    assert!(msg.contains("returned no mesh for it"), "{msg}");
}

#[test]
fn every_solid_step_fixture_still_imports() {
    for name in [
        "box.step",
        "bspline_dome.step",
        "fillet_corner.step",
        "Milling-test.step",
        "Milling-test-inch.step",
        "sample_ap214.step",
        "lathe_holder_offset.step",
        "tool_csys_box_iso13399.step",
    ] {
        let path = model(name);
        let r = within(Duration::from_secs(240), move || import::load_model(&path));
        assert!(r.is_ok(), "{name}: {:?}", r.err());
    }
}
