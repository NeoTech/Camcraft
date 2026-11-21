//! Native Agie bundle integrity, nominal seam, and no-clobber checks on an original rectangular punch.
use super::*;

fn punch() -> AgieJob {
    AgieJob {
        quality: AgieQuality::FineRa030,
        side: AgieSide::Punch,
        round_profile: None,
        contour: vec![[-20., -10.], [20., -10.], [20., 10.], [-20., 10.]],
        start: Some([-30., 0.]),
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
    job.start = Some([0., 0.]);
    assert!(contour::prepare(&job).is_err());
    job.start = Some([-20.5, 0.]);
    assert!(contour::prepare(&job).is_err());
    job.start = Some([-39.5, 0.]);
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
    job.start = Some([-30., 9.5]);
    assert!(
        contour::prepare(&job)
            .unwrap_err()
            .contains("straight face on both sides")
    );
    job.start = Some([-30., 7.]);
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
    job.start = Some([-20.5, 6.]);
    let prepared = contour::prepare(&job).unwrap();
    assert_eq!(prepared[0], [-12.5, 0.]);
    assert_eq!(&prepared[1..], job.contour.as_slice());
    job.contour.reverse();
    assert_eq!(contour::prepare(&job).unwrap(), prepared);

    // 3.125 mm along the 3:4 face, even though each coordinate difference
    // is smaller than 3 mm. A 2.5 mm length cannot hold the transition.
    job.start = Some([-26.125, -1.5]);
    assert_eq!(contour::prepare(&job).unwrap()[0], [-18.125, -7.5]);
    job.start = Some([-26.5, -2.]);
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
    job.start = Some([-20.5, 6.]);
    let prepared = contour::prepare(&job).unwrap();
    assert_eq!(prepared[0], [-12.64765, 0.50336]);
    assert_eq!(&prepared[1..], job.contour.as_slice());
    job.contour.reverse();
    assert_eq!(contour::prepare(&job).unwrap(), prepared);

    // Nearest-coordinate rounding here would cut inside the punch.
    job.start.as_mut().unwrap()[1] = 6.003;
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
    job.start = Some([-15.5, 1.659]);
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
    job.start = Some([15., 9.7]);
    assert!(contour::prepare(&job).unwrap_err().contains("start hole"));
    // This hole is outside, but the extension of the locally visible sloping
    // face crosses the other bank. Entry requires a supporting exterior face.
    job.start = Some([5., 16.]);
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
    job.start = Some([0., 0.]);
    job.stock_min[2] = -40.;
    assert!(contour::prepare(&job).unwrap_err().contains("20 mm"));
    job.stock_min[2] = -20.;
    // The requested opening has square corners. An inward QN2 rough pass
    // cannot reach these corners to the 0.002 mm Wire Contour tolerance.
    assert!(contour::prepare(&job).unwrap_err().contains("0.002 mm"));
}

#[test]
fn the_open_cut_includes_air_tails_wire_and_work_origin_in_travel() {
    let mut job = punch();
    job.side = AgieSide::Right;
    job.contour = vec![[6., -120.], [6., 120.]];
    job.stock_min[1] = -120.;
    job.stock_max[1] = 120.;
    job.start = None;
    let geometry = cutoff::prepare(&job).unwrap();
    assert_eq!(geometry.entry, [9., -124.]);
    assert_eq!(geometry.exit, [9., 124.]);
    // A 242 mm bar fits the 250 mm Y stroke. The two 4 mm tails
    // would put the wire centre on its travel limits.
    job.stock_min[1] = -121.;
    job.stock_max[1] = 121.;
    job.contour = vec![[6., -121.], [6., 121.]];
    assert!(cutoff::prepare(&job).err().unwrap().contains("travel"));
    job.stock_min[1] = -120.;
    job.stock_max[1] = 120.;
    job.contour = vec![[39., -120.], [39., 120.]];
    assert!(cutoff::prepare(&job).err().unwrap().contains("corners"));
    // A displaced WCS can put the controller's travel-to-origin outside the stroke.
    job.stock_min[0] = 180.;
    job.stock_max[0] = 250.;
    job.contour = vec![[210., -120.], [210., 120.]];
    assert!(cutoff::prepare(&job).err().unwrap().contains("origin"));
}

#[test]
fn a_group_rejects_an_unqualified_height_or_reserved_geometry_alias() {
    let temp = tempfile::tempdir().unwrap();
    for (height, name, expected) in [
        (-40., "PAIR", "20 mm"),
        (-20., "LPT", "non-reserved"),
        (-20., "ABCDEFGH", "seven"),
    ] {
        let mut a = punch();
        let mut b = punch();
        a.stock_min[2] = height;
        b.stock_min[2] = height;
        let dir = temp.path().join(name);
        assert!(
            export_works(&[a, b], &dir, name, 1.0)
                .unwrap_err()
                .contains(expected)
        );
        assert!(!dir.exists());
    }
}

#[test]
fn fine_plus_uses_only_its_measured_height_and_single_punch_cycle() {
    let temp = tempfile::tempdir().unwrap();
    let destination = temp.path().join("FINE20");
    let mut job = punch();
    job.quality = AgieQuality::FinePlusRa035;
    assert_eq!(contour::stock_offset_um(&job).unwrap(), 245);
    for height in [15., 19.999, 20.001, 40., 80.] {
        job.stock_min[2] = -height;
        assert!(
            export(&job, &destination, "FINE20", 1.0)
                .unwrap_err()
                .contains("thickness")
        );
        assert!(!destination.exists());
    }
    job.stock_min[2] = -20.;
    job.side = AgieSide::Die;
    assert!(
        export(&job, &destination, "FINE20", 1.0)
            .unwrap_err()
            .contains("single closed punch")
    );
    job.side = AgieSide::Punch;
    assert!(
        export_works(&[punch(), job], &destination, "FINE20", 1.0)
            .unwrap_err()
            .contains("single closed punch")
    );
    assert!(!destination.exists());
}

#[test]
fn qualified_circle_radii_and_cardinal_entries_keep_the_exact_diameter() {
    use camcraft_geometry::{
        geo2d::Point2,
        wire_round_profile::{RoundProfile, Spine},
    };
    let center = [2.0, -3.0];
    for radius in [6.0, 8.375, 10.0, 15.0] {
        for axis in [0, 1] {
            for sign in [-1.0, 1.0] {
                let mut job = punch();
                job.contour.clear();
                job.round_profile = Some(RoundProfile {
                    spine: Spine::Circle(Point2::new(center[0], center[1])),
                    radius,
                });
                let mut hole = center;
                let mut seam = center;
                let mut opposite = center;
                hole[axis] += sign * (radius + 7.0);
                seam[axis] += sign * radius;
                opposite[axis] -= sign * radius;
                job.start = Some(hole);
                let work = Work::prepare(&job, "CIRCLE".into(), 1.0).unwrap();
                assert_eq!(work.entry, hole);
                assert_eq!(work.points, [seam, opposite, seam]);
                assert_eq!(work.arc_centers, [None, Some(center), Some(center)]);
            }
        }
    }
}
