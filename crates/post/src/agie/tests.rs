//! Native Agie bundle integrity, nominal seam, and no-clobber checks on an original rectangular punch.
use super::*;

fn punch() -> AgieJob {
    AgieJob {
        contour: vec![[-20., -10.], [20., -10.], [20., 10.], [-20., 10.]],
        start: [-30., 0.],
        stock_min: [-40., -30., -20.],
        stock_max: [40., 30., 0.],
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn native_punch_links_its_three_files_and_uses_the_uncompensated_contour() {
    let temp = tempfile::tempdir().unwrap();
    let out = temp.path().join("new-job");
    let paths = export(&punch(), &out, "PUNCH40", 1.0).unwrap();
    assert_eq!(paths.len(), 3);
    let iso = std::fs::read_to_string(out.join("PUNCH40.ISO")).unwrap();
    assert_eq!(iso.lines().count(), 8);
    assert!(iso.contains("G00 X-20.00000 Y0.00000 ;\r\n"));
    assert!(iso.contains("G01 X20.00000 Y10.00000 ;\r\n"));
    assert!(iso.ends_with("M02 ;\r\n"));
    let map = std::fs::read_to_string(out.join("PUNCH40.SBR")).unwrap();
    assert_eq!(map, "AGIE.USING_PUNCH40.ISO IMPORT PUNCH40.ISO;\r\n");
    let script = std::fs::read_to_string(out.join("PUNCH40.SBL")).unwrap();
    for field in [
        "AGIE.USING_PUNCH40.ISO",
        "ID_PUNCH,c_punch",
        "ID_SEPCUT,\"1.0\"",
        "ID_DIAMETER,\"2.0\"",
        "ID_POSX,\"-30.000\"",
        "0.30,1.50,1.00,F,F,F",
        "ASDELETE",
    ] {
        assert!(script.contains(field), "{field}");
    }
    assert!(!script.contains("ASAGIE"));
    assert!(export(&punch(), &out, "PUNCH40", 1.0).is_err());
    assert_eq!(
        std::fs::read_to_string(out.join("PUNCH40.ISO")).unwrap(),
        iso
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn final_rename_cannot_replace_an_empty_directory() {
    let temp = tempfile::tempdir().unwrap();
    let from = temp.path().join("stage");
    let to = temp.path().join("destination");
    std::fs::create_dir(&from).unwrap();
    std::fs::create_dir(&to).unwrap();
    assert!(rename_new(&from, &to).is_err());
    assert!(from.is_dir());
    assert!(to.is_dir());
}

#[test]
fn stock_hole_and_entry_are_validated_before_output() {
    let mut job = punch();
    job.start = [0., 0.];
    assert!(contour::prepare(&job, "PUNCH40").is_err());
    job.start = [-20.5, 0.];
    assert!(contour::prepare(&job, "PUNCH40").is_err());
    job.start = [-39.5, 0.];
    assert!(contour::prepare(&job, "PUNCH40").is_err());
    job = punch();
    job.contour.swap(0, 1);
    assert!(contour::prepare(&job, "PUNCH40").is_err());
    job = punch();
    job.stock_min[2] = -12.;
    assert!(contour::prepare(&job, "PUNCH40").is_err());
    assert!(!valid_name("A/B"));
    assert!(!valid_name("NUL"));
}

#[test]
fn the_native_pass_envelope_requires_a_rectangle_and_straight_face_for_separation() {
    let mut job = punch();
    job.contour[2][0] = 18.0;
    assert!(
        contour::prepare(&job, "PUNCH40")
            .unwrap_err()
            .contains("axis-aligned rectangular")
    );
    let mut job = punch();
    job.start = [-30., 9.5];
    assert!(
        contour::prepare(&job, "PUNCH40")
            .unwrap_err()
            .contains("straight face on both sides")
    );
    job.start = [-30., 7.];
    assert!(contour::prepare(&job, "PUNCH40").is_ok());
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn stock_origin_and_fractional_setup_coordinates_are_refused_before_publication() {
    let temp = tempfile::tempdir().unwrap();
    let out = temp.path().join("REFUSED");
    let mut displaced = punch();
    for vertex in &mut displaced.contour {
        vertex[0] += 340.0;
    }
    displaced.start[0] += 340.0;
    displaced.stock_min[0] += 340.0;
    displaced.stock_max[0] += 340.0;
    let mut fractional = punch();
    fractional.start[1] = 0.0004;
    for (job, why) in [
        (displaced, "stock and work origin"),
        (fractional, "0.001 mm"),
    ] {
        let error = export_with_review(&job, &out, "REFUSED", 1.0).unwrap_err();
        assert!(error.contains(why), "{error}");
        assert!(!out.exists());
    }
}
