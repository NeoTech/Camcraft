//! #3643 review blockers (PR #4323): a spindle body grows by its pad
//! across an internal shoulder, and a stated but malformed cutting point
//! is refused by name, never read as an absent one. Fixtures are made
//! here or are our own STEP test model.

use super::{spun_body, MeshBody, MeshBodyError, MeshFormat};
use crate::frame::Frame;
use crate::mesh_body_tests::{prism, step_holder};
use crate::mesh_file::Tri;

/// The largest drawn radius at height `h` above the body's base.
fn drawn_r(body: &crate::component_geometry::ComponentBody, z: f64) -> f64 {
    let h = z - body.base();
    body.profile().bands().iter().filter(|d| d.above <= h && h <= d.above + d.len).map(|d| d.r).fold(0.0, f64::max)
}

/// A narrow piece from Z -20 to 0 and a wider piece from Z -10 to 0: the
/// wider piece's shoulder at Z -10 is internal. Every point within the
/// pad of the steel lies inside the drawn body, also just below that
/// shoulder.
#[test]
fn a_spindle_body_grows_by_the_pad_across_an_internal_shoulder() {
    let wide = 10.0 * std::f64::consts::SQRT_2;
    let mut tris: Vec<Tri> = prism(8, 1.0, -20.0, 0.0, &Frame::IDENTITY);
    tris.extend(prism(4, wide, -10.0, 0.0, &Frame::IDENTITY));
    for pad in [0.01, 0.3] {
        let body = spun_body(&tris, pad).unwrap_or_else(|e| panic!("pad {pad}: {e}"));
        assert!((body.base() - (-20.0 - pad)).abs() < 1e-9, "base {}", body.base());
        // The shoulder vertex [wide, 0, -10] grown by the pad along -Z.
        for z in [-10.0 - 0.5 * pad, -10.0 - 0.99 * pad, -10.0, 0.0, 0.99 * pad] {
            let r = drawn_r(&body, z);
            assert!(r >= wide + pad - 1e-9, "pad {pad}: at Z {z} the body is {r}, the grown steel {}", wide + pad);
        }
        // The narrow piece grows by the pad over its whole length too.
        for z in [-20.0 - 0.99 * pad, -15.0, -10.0 - 1.01 * pad] {
            let r = drawn_r(&body, z);
            assert!(r >= 1.0 + pad - 1e-9, "pad {pad}: at Z {z} the body is {r}");
        }
    }
}

/// Our STEP box with its CIP location pointed at a wrong entity: the
/// reader refuses that CIP, and the body refuses the file by name - it
/// never falls back to the default cutting point.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn a_stated_but_malformed_cip_is_refused_by_name() {
    let good = step_holder("MCS");
    let at = "#220 = AXIS2_PLACEMENT_3D('CIP', #221,";
    assert!(good.contains(at), "the fixture has its CIP");
    let bad = good.replacen(at, "#220 = AXIS2_PLACEMENT_3D('CIP', #220,", 1);
    let body = MeshBody::read(MeshFormat::Step, bad.as_bytes()).unwrap_or_else(|e| panic!("{e}"));
    let refusal = |r: Result<(), MeshBodyError>| match r {
        Err(MeshBodyError::RefusedFrame { name, why }) => {
            assert_eq!(name, "CIP");
            assert!(why.contains("#220"), "{why}");
        }
        other => panic!("not refused by name: {other:?}"),
    };
    refusal(body.parts().map(|_| ()));
    refusal(body.spindle_geometry().map(|_| ()));
    refusal(body.lathe_geometry().map(|_| ()));
    let e = body.parts().err().map(|e| e.to_string()).unwrap_or_default();
    assert!(e.contains("'CIP'") && e.contains("default"), "{e}");

    // A stated malformed port is refused the same way.
    let at = "#230 = AXIS2_PLACEMENT_3D('CSW', #231,";
    assert!(good.contains(at), "the fixture has its CSW");
    let bad = good.replacen(at, "#230 = AXIS2_PLACEMENT_3D('CSW', #230,", 1);
    let body = MeshBody::read(MeshFormat::Step, bad.as_bytes()).unwrap_or_else(|e| panic!("{e}"));
    assert!(matches!(body.parts(), Err(MeshBodyError::RefusedFrame { ref name, .. }) if name == "CSW"));

    // The good file still states its CIP.
    let ok = MeshBody::read(MeshFormat::Step, good.as_bytes()).and_then(|b| b.parts()).unwrap_or_else(|e| panic!("{e}"));
    assert!(ok.cip.is_some());
}
