//! #1536: a STEP face whose tessellation never converges is refused by
//! name inside a bounded time, never a hang; smooth B-spline faces still
//! import; degenerate B-spline input still ends cleanly. #1568: a finer
//! pass that runs out of evaluations keeps the coarser mesh only on a face
//! proven continuous, and is refused by name on one that may jump.
//!
//! Fixtures (testdata/models/): `bspline_discontinuous.step` is box.step
//! with its top face a degree-1 B-spline whose interior knot is repeated
//! past its degree, so the surface jumps at u = 0.5 and truck's
//! subdivision recurses for ever; `bspline_dome.step` is box.step with a
//! smooth biquadratic dome on top.
//!
//! Gate map (M4.4): covers app::geometry::import::step,
//! app::geometry::import::step_guard.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use camcraft_app::geometry::import::{self, step};
use camcraft_app::geometry::{Mesh, Point3};

fn testdata(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/models").join(name)
}

/// Run `f` on its own thread and fail the test - instead of hanging the
/// suite - when it has not answered within `limit`.
fn within<R: Send + 'static>(limit: Duration, f: impl FnOnce() -> R + Send + 'static) -> R {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    match rx.recv_timeout(limit) {
        Ok(r) => r,
        Err(_) => panic!("import did not finish within {limit:?} - the hang is back"),
    }
}

fn volume(m: &Mesh) -> f64 {
    m.facets
        .iter()
        .map(|t| {
            let (a, b, c) = (t.a, t.b, t.c);
            (a.x * (b.y * c.z - b.z * c.y) - a.y * (b.x * c.z - b.z * c.x) + a.z * (b.x * c.y - b.y * c.x)) / 6.0
        })
        .sum()
}

/// The default cap, as a user's import runs it: bounded, and named. Since
/// #1568 raised the cap to 150 M this spends ~10 s in release and ~27 s
/// in debug (it was 4.5 s debug at 25 M); the lowered-cap pins below cover
/// the same refusal in milliseconds.
#[test]
fn a_face_that_never_converges_is_refused_by_name_within_the_budget() {
    let started = Instant::now();
    let r = within(Duration::from_secs(120), || import::load_model(&testdata("bspline_discontinuous.step")));
    let err = r.expect_err("a surface that never tessellates must be refused").to_string();
    assert!(
        err.contains("face #53 (B-spline surface, face 6 of 6 in shell #17)"),
        "the refusal names the face, its kind and its shell: {err}"
    );
    assert!(err.contains("did not converge"), "{err}");
    assert!(err.contains("import an STL"), "the refusal says what to do: {err}");
    eprintln!("refused in {:?}", started.elapsed());
}

#[test]
fn a_lowered_cap_refuses_the_same_face_at_once() {
    let r = within(Duration::from_secs(30), || {
        step::with_face_eval_cap(100_000, || import::load_model(&testdata("bspline_discontinuous.step")))
    });
    let err = r.expect_err("refused").to_string();
    assert!(err.contains("face #53") && err.contains("within 100000 evaluations"), "{err}");
}

#[test]
fn a_smooth_bspline_face_still_imports_closed_at_its_volume() {
    let m = within(Duration::from_secs(60), || import::load_model(&testdata("bspline_dome.step")))
        .expect("a smooth B-spline face imports");
    let b = m.bounds().expect("non-empty");
    // The dome's crown: 1 + 0.2 * (2 * 0.5 * 0.5)^2 = 1.05 at u = v = 0.5.
    assert!((b.max.z - 1.05).abs() < 5e-3, "dome crown at {}", b.max.z);
    let want = 1.0 + 0.2 / 9.0;
    let got = volume(&m);
    assert!((got - want).abs() < 2e-3, "volume {got}, want {want}");
    assert!(b.min.x.abs() < 1e-9 && (b.max.x - 1.0).abs() < 1e-9, "{b:?}");
}

#[test]
fn degenerate_bspline_variants_end_cleanly_and_bounded() {
    // Knots, multiplicities and points bent into nonsense: each must come
    // back - an import or a refusal - never a hang or a crash.
    let text = std::fs::read_to_string(testdata("bspline_discontinuous.step")).expect("fixture");
    let knots = "(2, 2, 2), (2, 2), (0.0, 0.5, 1.0), (0.0, 1.0)";
    assert!(text.contains(knots), "fixture layout changed");
    let variants = [
        "(2, 2, 2), (2, 2), (0.0, 0.0, 1.0), (0.0, 1.0)",
        "(2, 2, 2), (2, 2), (0.0, 0.5, 0.5), (0.0, 1.0)",
        "(2, 2, 2), (2, 2), (1.0, 0.5, 0.0), (0.0, 1.0)",
        "(2, 2, 2), (2, 2), (0.0, 0.5, 1.0), (1.0, 1.0)",
        "(3, 3, 3), (2, 2), (0.0, 0.5, 1.0), (0.0, 1.0)",
        "(1, 1, 1), (2, 2), (0.0, 0.5, 1.0), (0.0, 1.0)",
        "(2, 2, 2), (2, 2), (0.0, 0.5, 1.0E300), (0.0, 1.0)",
    ];
    for v in variants {
        let data = text.replace(knots, v).into_bytes();
        let started = Instant::now();
        let r = within(Duration::from_secs(60), move || {
            step::with_face_eval_cap(200_000, || import::parse_model(&data, import::Format::Step))
        });
        if let Ok(m) = &r {
            assert!(m.facets.iter().all(|t| [t.a, t.b, t.c].iter().all(|p: &Point3| p.is_finite())), "{v}");
        }
        eprintln!("{v}: {} in {:?}", r.map(|m| format!("{} facets", m.len())).unwrap_or_else(|e| e.to_string()), started.elapsed());
    }
}


/// #1568: a finer pass that runs out of evaluations on a face proven
/// continuous keeps the coarser pass's mesh, at that mesh's own chord,
/// instead of refusing the model. At a 1000-evaluation cap the dome's
/// first pass (chord ~0.002) fits and the pass at its 0.001 target trips.
#[test]
fn a_finer_pass_that_runs_out_on_a_smooth_face_keeps_the_coarser_mesh() {
    let data = std::fs::read(testdata("bspline_dome.step")).expect("fixture");
    let (full, full_chord) = step::parse_with_chord(&data).expect("the dome imports");
    let (m, chord) = within(Duration::from_secs(60), move || {
        step::with_face_eval_cap(1_000, || step::parse_with_chord(&data))
    })
    .expect("a continuous face that only ran out on a finer pass keeps the coarser mesh");
    assert!(chord > full_chord, "the kept mesh reports its own, coarser chord: {chord} vs {full_chord}");
    assert!(m.len() < full.len(), "{} facets kept, {} at full detail", m.len(), full.len());
    let want = 1.0 + 0.2 / 9.0;
    assert!((volume(&m) - want).abs() < 2e-3, "volume {}, want {want}", volume(&m));
}

/// #1568 / #1536: the same fallback is refused for a face that may jump.
/// Its first pass converges (wider than the jump, it smears it into a
/// closed shell of the right volume - which is why closure and volume are
/// not the test), and the finer pass trips: the import is refused by name.
#[test]
fn a_finer_pass_that_runs_out_on_a_face_that_may_jump_is_refused() {
    let r = within(Duration::from_secs(30), || {
        step::with_face_eval_cap(1_000, || import::load_model(&testdata("bspline_discontinuous.step")))
    });
    let err = r.expect_err("a smeared jump is never kept").to_string();
    assert!(err.contains("face #53 (B-spline surface") && err.contains("did not converge"), "{err}");
}
