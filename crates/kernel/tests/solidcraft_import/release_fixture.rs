//! #2578 release check: a design with fillets, saved by a Solidcraft
//! release, imports with the body of its own STEP export. The fixture
//! `testdata/models/filleted_block.sc` (Solidcraft 0.48.1) is a 40 x 30
//! x 20 block with a 3 mm round on one top edge and a variable round, 2 to
//! 5 mm, on the opposite top edge; the variable round is the procedural
//! blend face. `filleted_block.step` is the same design's STEP export from
//! the same release. The modeller measured 23858.637 mm3.
//! Provenance: our own design, made for this test in Solidcraft.

use camcraft_kernel::machining_mesh::machining_mesh;
use camcraft_kernel::solidcore_impl::SolidcoreKernel;
use camcraft_kernel::solidcore_step::import_step;
use solidcore::kernel::CancelToken;
use solidcore_record::brep::Surface;

use super::import;

/// The modeller's own volume of the design, mm3 (Solidcraft `measure`).
const MODELLER_VOLUME: f64 = 23_858.637_478_691_86;

fn fixture(name: &str) -> Vec<u8> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../testdata/models").join(name);
    match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) => panic!("{}: {e}", path.display()),
    }
}

#[test]
fn a_release_design_with_fillets_imports_the_body_of_its_step_export() {
    let sc = match import(&fixture("filleted_block.sc")) {
        Ok(i) => i,
        Err(e) => panic!("the filleted design is refused: {e}"),
    };
    let mut k = SolidcoreKernel::default();
    let mut step = match import_step(&mut k, &fixture("filleted_block.step"), &CancelToken::new()) {
        Ok(s) => s,
        Err(e) => panic!("the STEP export is refused: {e}"),
    };
    assert_eq!(step.solids.len(), 1, "the export holds one solid");
    let step = step.solids.remove(0);
    let blends = sc.solid.record.faces.iter().filter(|f| matches!(f.surface, Surface::Blend { .. })).count();
    println!(
        "sc: app {} faces {} blends {blends} volume {} box {:?}..{:?}",
        sc.app,
        sc.solid.record.faces.len(),
        sc.solid.measure.volume,
        sc.solid.measure.min,
        sc.solid.measure.max
    );
    println!(
        "step: faces {} volume {} box {:?}..{:?}",
        step.record.faces.len(),
        step.measure.volume,
        step.measure.min,
        step.measure.max
    );
    assert_eq!(sc.design, "filleted_block");
    assert_eq!(sc.units, "mm");
    assert!(blends > 0, "the design keeps its variable round as a blend face");
    let (a, b) = (sc.solid.measure.volume, step.measure.volume);
    assert!((a - MODELLER_VOLUME).abs() < 1e-6 * MODELLER_VOLUME, ".sc volume {a}, modeller {MODELLER_VOLUME}");
    assert!((a - b).abs() < 1e-9 * a, ".sc volume {a}, STEP volume {b}");
    assert_eq!(sc.solid.record.faces.len(), step.record.faces.len(), "the same faces");
    assert_eq!(sc.solid.record.faces.len(), 8, "six block faces and two rounds");
    // The boxes are the block's. The kernel's box can be a little too large
    // on a trimmed curved face (#939), so each side is within 1 micron.
    let want = ([-20.0, -15.0, 0.0], [20.0, 15.0, 20.0]);
    for (what, m) in [(".sc", &sc.solid.measure), ("STEP", &step.measure)] {
        for ax in 0..3 {
            assert!((m.min[ax] - want.0[ax]).abs() < 1e-3, "{what} min {:?}", m.min);
            assert!((m.max[ax] - want.1[ax]).abs() < 1e-3, "{what} max {:?}", m.max);
        }
    }
    // What toolpaths read: the two machining meshes agree.
    let tol = 0.005;
    let mesh = |s, what: &str| match machining_mesh(s, tol, &CancelToken::new()) {
        Ok(m) => m,
        Err(e) => panic!("the {what} body does not mesh: {e}"),
    };
    let (ms, mt) = (mesh(&sc.solid, ".sc"), mesh(&step, "STEP"));
    println!("mesh sc {:?}\nmesh step {:?}", ms.report, mt.report);
    for (what, m) in [(".sc", &ms), ("STEP", &mt)] {
        assert_eq!(m.report.open_sides, 0, "{what}: {:?}", m.report);
        assert!(m.report.deviation.largest() <= tol, "{what}: {:?}", m.report.deviation);
    }
    assert_eq!(ms.report.triangles, mt.report.triangles, "the same machining mesh");
    assert!((ms.report.volume - mt.report.volume).abs() < 1e-9 * a, "mesh volumes {} and {}", ms.report.volume, mt.report.volume);
}
