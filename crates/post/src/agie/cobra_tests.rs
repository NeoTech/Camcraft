//! The Cobra Cut D 0.25 qualities (#2986): one 40 mm polygon punch only, with the wire
//! set on the piece and on the work, and the measured offsets of each quality.
use super::tests::punch;
use super::*;
use crate::agie_setup::{PunchSetup20, SetupError};

fn read(dir: &Path, name: &str) -> String {
    std::fs::read_to_string(dir.join(name)).unwrap()
}

/// The test punch on 40 mm stock.
fn punch40() -> AgieJob {
    let mut job = punch();
    job.stock_min[2] = -40.;
    job
}

/// Quality, its `JE_CreateQuality` line, and its offsets text.
const COBRA: [(Quality, &str, &str); 2] = [
    (
        Quality::Ccd030,
        "JE_CreateQuality(je_piece,\"CCD030\",\"0.30,1.50,1.00,F,F,F\",je_q1)",
        "250, 170, 145 and 135",
    ),
    (
        Quality::Ccd070,
        "JE_CreateQuality(je_piece,\"CCD070\",\"0.70,1.50,1.00,F,F,F\",je_q1)",
        "245, 165 and 140",
    ),
];

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn cobra_cut_d_sets_the_wire_and_quality_of_a_40_mm_punch() {
    let temp = tempfile::tempdir().unwrap();
    let base = temp.path().join("BASE");
    let berco = export_with_review(&punch40(), &base, "PUNCH40", 1.0).unwrap();
    let old = "JE_CreateQuality(je_piece,\"CCB030\",\"0.30,1.50,1.00,F,F,F\",je_q1)";
    for (quality, line, offsets) in COBRA {
        let out = temp.path().join(quality.name());
        let review = export_punch_with_review(&punch40(), quality, &out, "PUNCH40", 1.0).unwrap();
        for name in ["PUNCH40.ISO", "PUNCH40.SBR"] {
            assert_eq!(read(&out, name), read(&base, name), "{name}");
        }
        // Only the quality line and the wire of the piece and the work change.
        let sbl = read(&out, "PUNCH40.SBL");
        assert_eq!(
            sbl,
            read(&base, "PUNCH40.SBL")
                .replace(old, line)
                .replace("\"Berco Cut 0.25\"", "\"Cobra Cut D 0.25\"")
        );
        assert_eq!(sbl.matches("\"Cobra Cut D 0.25\"").count(), 2, "{sbl}");
        assert!(sbl.contains("je_piece,NULL,NULL,NULL,ID_THREAD,\"Cobra Cut D 0.25\""));
        assert!(sbl.contains("\"CUT1\",NULL,ID_THREAD,\"Cobra Cut D 0.25\""));
        assert!(!sbl.contains("Berco"));
        assert_eq!(sbl.matches("JE_CreateQuality").count(), 1);
        assert!(sbl.contains("ID_SEPCUT,\"1.0\""));
        let text = &review.controller_review;
        assert!(text.contains("Verify steel 0001, Cobra Cut D 0.25 and 40 mm"), "{text}");
        assert!(text.contains(&format!("Check offsets of {offsets} µm in order")), "{text}");
        assert!(!text.contains("Berco"), "{text}");
    }
    // The Berco job at 40 mm keeps its wire and its QN2 offsets.
    assert!(read(&base, "PUNCH40.SBL").contains("\"Berco Cut 0.25\""));
    assert!(berco.controller_review.contains("Check offsets of 260, 170, 140 and 130 µm"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn cobra_cut_d_refuses_other_heights_and_sides_without_files() {
    let temp = tempfile::tempdir().unwrap();
    let out = temp.path().join("COBRA40");
    for (quality, ..) in COBRA {
        let refuse = |job: &AgieJob| {
            let error =
                export_punch_with_review(job, quality, &out, "COBRA40", 1.0).unwrap_err();
            assert!(!out.exists(), "{error}");
            error
        };
        for height in [15., 20., 39.999, 40.001, 80.] {
            let mut job = punch();
            job.stock_min[2] = -height;
            let error = refuse(&job);
            assert!(
                error.contains(&format!("{} quality has no measured offset", quality.name()))
                    && error.ends_with("Supported thicknesses in mm are 40"),
                "{error}"
            );
        }
        for side in [AgieSide::Die, AgieSide::Left, AgieSide::Right] {
            let job = AgieJob { side, ..punch40() };
            assert!(refuse(&job).contains("one closed punch only"));
        }
        let mut job = punch40();
        job.stock_max[2] = 1.0;
        assert!(refuse(&job).contains("Cobra Cut D"), "stock top");
    }
}

#[test]
fn the_setup_encoder_holds_cobra_cut_d_to_40_mm() {
    let at = |height_um| PunchSetup20 {
        stem: "COBRA40",
        stock_min_um: [-40_000, -30_000],
        stock_max_um: [40_000, 30_000],
        start_hole_um: [-30_000, 0],
        height_um,
    };
    for (quality, ..) in COBRA {
        for height_um in [15_000, 20_000, 80_000, 41_000] {
            assert_eq!(
                at(height_um).encode_quality(quality).unwrap_err(),
                SetupError::UnqualifiedCobraHeight
            );
        }
        assert!(at(40_000).encode_quality(quality).is_ok());
    }
}

#[test]
fn cobra_cut_d_bounds_the_cut_with_its_own_rough_offset() {
    assert_eq!(
        Quality::Ccd030.offsets_um(40_000),
        Some(&[250, 170, 145, 135][..])
    );
    assert_eq!(Quality::Ccd070.offsets_um(40_000), Some(&[245, 165, 140][..]));
    assert_eq!(Quality::Ccd030.wire(), "Cobra Cut D 0.25");
    assert_eq!(Quality::Ccb030.wire(), "Berco Cut 0.25");
    // The miter reach of a square corner is the rough offset times √2.
    for (quality, rough_mm) in [
        (Quality::Ccd030, 0.250),
        (Quality::Ccd070, 0.245),
        (Quality::Ccb030, 0.260),
    ] {
        let (_, _, reach) = contour::prepare_with_reach(&punch40(), "COBRA40", quality).unwrap();
        assert!((reach - rough_mm * 2f64.sqrt()).abs() < 1e-9, "{quality:?}: {reach}");
    }
}
