//! The Cobra Cut D 0.25 dispatch of app::posting::agie (#2986): a saved Wire Contour
//! condition selects CCD030 or CCD070 after a project reload, for one 40 mm polygon punch only.
//! Provenance: CAMCraft's original synthetic examples in examples/agie.
use super::quality::{Case, run};
use std::fs;

const FINE: &str = include_str!("../../../../examples/agie/fine-punch.camscript");
const PUNCH_OBJ: &str = include_str!("../../../../examples/agie/punch.obj");

/// The fine punch example with `condition` on `height` mm stock, posted to `job`.
fn punch(condition: &str, height: u32, job: &str) -> (String, String) {
    let obj = PUNCH_OBJ
        .replace(" 20\r\n", &format!(" {height}\r\n"))
        .replace(" 20\n", &format!(" {height}\n"));
    let script = FINE
        .replace("AGIE_BERCO_RA035", condition)
        .replace("dz: 20", &format!("dz: {height}"))
        .replace("origin: [0, 0, 20]", &format!("origin: [0, 0, {height}]"))
        .replace("top_z: 20", &format!("top_z: {height}"))
        .replace("to: \"FINE20\"", &format!("to: \"{job}\""))
        .replace(
            "cam.generate();",
            "cam.save(\"cobra.camproj\");\ncam.load(\"cobra.camproj\");\ncam.generate();",
        );
    assert!(script.contains(condition) && script.contains(job), "the fine punch example changed");
    (script, obj)
}

#[test]
fn a_cobra_condition_requests_its_quality_and_wire_on_a_40_mm_punch() {
    for (condition, line) in [
        (
            "AGIE_COBRA_RA030",
            "JE_CreateQuality(je_piece,\"CCD030\",\"0.30,1.50,1.00,F,F,F\",je_q1)",
        ),
        (
            "AGIE_COBRA_RA070",
            "JE_CreateQuality(je_piece,\"CCD070\",\"0.70,1.50,1.00,F,F,F\",je_q1)",
        ),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let (script, obj) = punch(condition, 40, "COBRA40");
        let output = run(temp.path(), "cobra.camscript", &script, &[("punch.obj", &obj)]);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "{condition}: {stderr}");
        let dir = temp.path().join("COBRA40");
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 3, "{condition}");
        let sbl = fs::read_to_string(dir.join("COBRA40.SBL")).unwrap();
        assert!(sbl.contains(line), "{condition}: {sbl}");
        assert_eq!(sbl.matches("JE_CreateQuality").count(), 1);
        // The wire is set on the piece and on the work.
        assert!(sbl.contains("je_piece,NULL,NULL,NULL,ID_THREAD,\"Cobra Cut D 0.25\""));
        assert!(sbl.contains("\"CUT1\",NULL,ID_THREAD,\"Cobra Cut D 0.25\""));
        assert!(sbl.contains("ID_HEIGHT,\"40.000\"") && sbl.contains("ID_SEPCUT,\"1.0\""));
        assert!(!sbl.contains("Berco"), "{condition}: {sbl}");
    }
}

#[test]
fn a_cobra_condition_refuses_other_heights_without_a_directory() {
    for condition in ["AGIE_COBRA_RA030", "AGIE_COBRA_RA070"] {
        for height in [15, 20, 80] {
            let temp = tempfile::tempdir().unwrap();
            let (script, obj) = punch(condition, height, "COBRA");
            let output = run(temp.path(), "cobra.camscript", &script, &[("punch.obj", &obj)]);
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                !output.status.success()
                    && stderr.contains("quality has no measured offset")
                    && stderr.contains("Supported thicknesses in mm are 40"),
                "{condition} {height}: {stderr}"
            );
            assert!(!temp.path().join("COBRA").exists());
        }
    }
}

#[test]
fn a_cobra_condition_refuses_a_group_an_arc_punch_a_die_and_a_cutoff() {
    let cases: [Case; 4] = [
        (
            "punch-pair.camscript",
            include_str!("../../../../examples/agie/punch-pair.camscript"),
            &[
                ("pair-left.obj", include_str!("../../../../examples/agie/pair-left.obj")),
                ("pair-right.obj", include_str!("../../../../examples/agie/pair-right.obj")),
            ],
            "PAIR20",
            "{c} is qualified for one closed polygon punch on a 40 mm plate only",
        ),
        (
            "circle-punch.camscript",
            include_str!("../../../../examples/agie/circle-punch.camscript"),
            &[("circle-punch.dxf", include_str!("../../../../examples/agie/circle-punch.dxf"))],
            "CIRCLE20",
            "{c} is qualified for one closed polygon punch on a 40 mm plate only",
        ),
        (
            "die-plate.camscript",
            include_str!("../../../../examples/agie/die-plate.camscript"),
            &[("die-plate.obj", include_str!("../../../../examples/agie/die-plate.obj"))],
            "DIE20",
            "qualified for one closed punch only",
        ),
        (
            "cutoff-lower.camscript",
            include_str!("../../../../examples/agie/cutoff-lower.camscript"),
            &[("cutoff-lower.obj", include_str!("../../../../examples/agie/cutoff-lower.obj"))],
            "LOWER20",
            "qualified for one closed punch only",
        ),
    ];
    for (name, script, assets, job, why) in cases {
        for condition in ["AGIE_COBRA_RA030", "AGIE_COBRA_RA070"] {
            let temp = tempfile::tempdir().unwrap();
            let script = script.replace("AGIE_QN2", condition);
            let why = why.replace("{c}", condition);
            let output = run(temp.path(), name, &script, assets);
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                !output.status.success() && stderr.contains(&why),
                "{name} {condition}: {stderr}"
            );
            assert!(!temp.path().join(job).exists(), "{name}");
        }
    }
}

/// A Cobra condition row with a wire other than 0.25 mm is a mismatched recipe.
/// The condition and tool gate refuses it before the post.
#[test]
fn a_cobra_condition_with_another_wire_diameter_is_refused() {
    let temp = tempfile::tempdir().unwrap();
    let (script, obj) = punch("AGIE_COBRA_RA030", 40, "COBRA40");
    let script = script.replacen("wire_diameter: 0.25", "wire_diameter: 0.3", 1);
    let output = run(temp.path(), "cobra.camscript", &script, &[("punch.obj", &obj)]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success() && stderr.contains("EDM condition \"AGIE_COBRA_RA030\" expects wire diameter 0.3 mm"),
        "{stderr}"
    );
    assert!(!temp.path().join("COBRA40").exists(), "{stderr}");
}
