//! Native Agie bundle integrity, nominal seam, and no-clobber checks on an original rectangular punch.
use super::*;

fn punch() -> AgieJob {
    AgieJob {
        side: AgieSide::Punch,
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
        "ASAGIE",
    ] {
        assert!(script.contains(field), "{field}");
    }
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
    assert!(contour::prepare(&job).is_err());
    job.start = [-20.5, 0.];
    assert!(contour::prepare(&job).is_err());
    job.start = [-39.5, 0.];
    assert!(contour::prepare(&job).is_err());
    job = punch();
    job.contour.swap(0, 1);
    assert!(contour::prepare(&job).is_err());
    job = punch();
    job.stock_min[2] = -12.;
    assert!(contour::prepare(&job).is_err());
    assert!(!valid_name("A/B"));
    assert!(!valid_name("NUL"));
}

#[test]
fn separation_needs_three_millimetres_on_each_side_of_the_entry() {
    let mut job = punch();
    job.start = [-30., 9.5];
    assert!(
        contour::prepare(&job)
            .unwrap_err()
            .contains("straight face on both sides")
    );
    job.start = [-30., 7.];
    assert!(contour::prepare(&job).is_ok());
}

#[test]
fn oblique_punch_preserves_nominal_vertices_and_perpendicular_seam() {
    let mut job = punch();
    job.contour = vec![
        [-20., -10.],
        [16., -10.],
        [22., -2.],
        [18., 10.],
        [-5., 10.],
    ];
    job.start = [-20.5, 6.];
    let prepared = contour::prepare(&job).unwrap();
    assert_eq!(prepared[0], [-12.5, 0.]);
    assert_eq!(&prepared[1..], job.contour.as_slice());
    job.contour.reverse();
    assert_eq!(contour::prepare(&job).unwrap(), prepared);

    // 3.125 mm along the 3:4 face, even though each coordinate difference
    // is smaller than 3 mm. A 2.5 mm length cannot hold the transition.
    job.start = [-26.125, -1.5];
    assert_eq!(contour::prepare(&job).unwrap()[0], [-18.125, -7.5]);
    job.start = [-26.5, -2.];
    assert!(contour::prepare(&job).is_err());
}

#[test]
fn oblique_seam_rounds_to_the_exterior_without_moving_nominal_vertices() {
    let mut job = punch();
    job.contour = vec![
        [-20., -10.],
        [16., -10.],
        [22., -2.],
        [18., 10.],
        [-6., 10.],
    ];
    job.start = [-20.5, 6.];
    let prepared = contour::prepare(&job).unwrap();
    assert_eq!(prepared[0], [-12.64765, 0.50336]);
    assert_eq!(&prepared[1..], job.contour.as_slice());
    job.contour.reverse();
    assert_eq!(contour::prepare(&job).unwrap(), prepared);

    // Nearest-coordinate rounding here would cut inside the punch.
    job.start[1] = 6.003;
    assert_eq!(contour::prepare(&job).unwrap()[0], [-12.64625, 0.50537]);
}

#[test]
fn outward_rounding_cannot_consume_the_required_lead_clearance() {
    let mut job = punch();
    job.contour = vec![
        [-19.99933, -10.],
        [16., -10.],
        [22., -2.],
        [18., 10.],
        [-5.99933, 10.],
    ];
    job.start = [-15.5, 1.659];
    // The exact approach measures 3.000002716 mm. Every exterior seam
    // inside the ISO rounding budget leaves less than 3 mm to a new face.
    let why = contour::prepare(&job).unwrap_err();
    assert!(why.contains("0.00001 mm ISO grid"), "{why}");
}

#[test]
fn acute_corner_clearance_uses_the_largest_resident_offset_at_the_selected_height() {
    let mut job = punch();
    job.contour = vec![[-20., -10.], [30., 0.], [-20., 10.]];
    // The miter at X30 grows with the rough offset. All three stock bounds
    // pass the 0.4 mm straight-face check; only the height-specific miter distinguishes them.
    for (height, right, fits) in [
        (15., 31.430, true),
        (20., 31.430, true),
        (40., 31.430, false),
        (40., 31.480, true),
        (80., 31.480, false),
        (80., 31.500, true),
    ] {
        job.stock_min[2] = -height;
        job.stock_max[0] = right;
        let result = contour::prepare(&job);
        if fits {
            result.unwrap();
        } else {
            assert!(result.unwrap_err().contains("rough corner"));
        }
    }
}

#[test]
fn an_acute_miter_needs_more_stock_than_a_round_corner() {
    let mut job = punch();
    job.contour = vec![[-20., -10.], [30., 0.], [-20., 10.]];
    job.stock_max[0] = 30.4;
    let why = contour::prepare(&job).unwrap_err();
    assert!(why.contains("rough corner"), "{why}");
    job.stock_max[0] = 32.;
    assert!(contour::prepare(&job).is_ok());

    job.contour = vec![[-20., -10.], [20., -10.], [0., 0.], [20., 10.], [-20., 10.]];
    assert!(
        contour::prepare(&job)
            .unwrap_err()
            .contains("reentrant corner")
    );
}

#[test]
fn a_broad_notch_keeps_its_nominal_corner_and_requires_an_exterior_entry() {
    let mut job = punch();
    job.contour
        .splice(3..3, [[10., 10.], [0., 9.], [-10., 10.]]);
    let points = contour::prepare(&job).unwrap();
    assert_eq!(points[0], [-20., 0.]);
    assert_eq!(&points[1..], job.contour.as_slice());
    // The drilled hole cannot sit in the retained right-hand shoulder.
    job.start = [15., 9.7];
    assert!(contour::prepare(&job).unwrap_err().contains("start hole"));
    // This hole is outside, but the extension of the locally visible sloping
    // face crosses the other bank. Entry requires a supporting exterior face.
    job.start = [5., 16.];
    assert!(contour::prepare(&job).unwrap_err().contains("start hole"));
}

#[test]
fn a_gentle_but_narrow_notch_cannot_hold_the_rough_corner_joins() {
    let mut job = punch();
    job.contour
        .splice(3..3, [[0.4, 10.], [0., 9.98], [-0.4, 10.]]);
    let why = contour::prepare(&job).unwrap_err();
    assert!(why.contains("segment"), "{why}");
}

#[test]
fn a_die_requires_the_qualified_thickness_and_reachable_inside_corners() {
    let mut job = punch();
    job.side = AgieSide::Die;
    job.start = [0., 0.];
    job.stock_min[2] = -40.;
    assert!(contour::prepare(&job).unwrap_err().contains("20 mm"));
    job.stock_min[2] = -20.;
    // The requested opening has square corners. An inward QN2 rough pass
    // cannot reach these corners to the 0.002 mm Wire Contour tolerance.
    assert!(contour::prepare(&job).unwrap_err().contains("0.002 mm"));
}
