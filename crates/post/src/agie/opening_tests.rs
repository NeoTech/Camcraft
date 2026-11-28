//! A reentrant die opening of lines and arcs (#3178): the native G01/G02/G03 ISO, the
//! counter-clockwise order, and each refusal with its values.
//! Provenance: the original examples/agie/reentrant-die drawing of #3178.
use super::opening::{iso, prepare};
use super::*;
use crate::agie_setup::InspectionSegment;
use camcraft_geometry::geo2d::Point2;

fn p(q: [f64; 2]) -> Point2 {
    Point2::new(q[0], q[1])
}

fn l(a: [f64; 2], b: [f64; 2]) -> InspectionSegment {
    InspectionSegment { start: p(a), end: p(b), arc: None }
}

/// An arc about `c` from `a` to `b`, turning `deg` degrees (negative: clockwise).
fn a(c: [f64; 2], a: [f64; 2], b: [f64; 2], deg: f64) -> InspectionSegment {
    InspectionSegment { start: p(a), end: p(b), arc: Some((p(c), deg.to_radians())) }
}

/// The opening of examples/agie/reentrant-die.dxf with the lower tooth fillet
/// of radius `f`: R2 corners, a tooth with two sharp tips on the right wall
/// and a tongue with an R4 round tip on the top wall. Counter-clockwise.
fn opening(f: f64) -> Vec<InspectionSegment> {
    vec![
        l([-18., -12.], [18., -12.]),
        a([18., -10.], [18., -12.], [20., -10.], 90.),
        l([20., -10.], [20., -3. - f]),
        a([20. - f, -3. - f], [20., -3. - f], [20. - f, -3.], 90.),
        l([20. - f, -3.], [14., -3.]),
        l([14., -3.], [14., 3.]),
        l([14., 3.], [19., 3.]),
        a([19., 4.], [19., 3.], [20., 4.], 90.),
        l([20., 4.], [20., 10.]),
        a([18., 10.], [20., 10.], [18., 12.], 90.),
        l([18., 12.], [5., 12.]),
        a([5., 11.], [5., 12.], [4., 11.], 90.),
        l([4., 11.], [4., 4.]),
        a([0., 4.], [4., 4.], [-4., 4.], -180.),
        l([-4., 4.], [-4., 11.]),
        a([-5., 11.], [-4., 11.], [-5., 12.], 90.),
        l([-5., 12.], [-18., 12.]),
        a([-18., 10.], [-18., 12.], [-20., 10.], 90.),
        l([-20., 10.], [-20., -10.]),
        a([-18., -10.], [-20., -10.], [-18., -12.], 90.),
    ]
}

fn job(hole: [f64; 2]) -> AgieJob {
    AgieJob {
        side: AgieSide::Die,
        contour: Vec::new(),
        start: Some(hole),
        stock_min: [-40., -30., -20.],
        stock_max: [40., 30., 0.],
    }
}

fn written(job: &AgieJob, walls: &[InspectionSegment]) -> Result<String, String> {
    prepare(job, walls, "REENT20").map(|(walls, _)| iso("REENT20.ISO", &walls))
}

fn refusal(job: &AgieJob, walls: &[InspectionSegment]) -> String {
    written(job, walls).expect_err("refused")
}

/// The golden of examples/agie/reentrant-die.camscript, line by line:
/// N0001 the seam, the foot of the 5 mm lead from the hole X0 Y-7 on the
/// bottom wall; N0003 the R2 corner (G03, I/J from its start); N0005 and
/// N0009 the R1 fillets of the tooth; N0006 to N0008 the tooth with its two
/// sharp tips; N0013 and N0017 the R1 fillets of the tongue; N0015 the R4
/// round tip of the tongue, clockwise (G02) about X0 Y4; N0022 back to the seam.
const REENT20: &str = "( REENT20.ISO )\r\n\
    N0001 G00 X0.00000 Y-12.00000 ;\r\n\
    N0002 G01 X18.00000 Y-12.00000 ;\r\n\
    N0003 G03 X20.00000 Y-10.00000 I0.00000 J2.00000 ;\r\n\
    N0004 G01 X20.00000 Y-4.00000 ;\r\n\
    N0005 G03 X19.00000 Y-3.00000 I-1.00000 J0.00000 ;\r\n\
    N0006 G01 X14.00000 Y-3.00000 ;\r\n\
    N0007 G01 X14.00000 Y3.00000 ;\r\n\
    N0008 G01 X19.00000 Y3.00000 ;\r\n\
    N0009 G03 X20.00000 Y4.00000 I0.00000 J1.00000 ;\r\n\
    N0010 G01 X20.00000 Y10.00000 ;\r\n\
    N0011 G03 X18.00000 Y12.00000 I-2.00000 J0.00000 ;\r\n\
    N0012 G01 X5.00000 Y12.00000 ;\r\n\
    N0013 G03 X4.00000 Y11.00000 I0.00000 J-1.00000 ;\r\n\
    N0014 G01 X4.00000 Y4.00000 ;\r\n\
    N0015 G02 X-4.00000 Y4.00000 I-4.00000 J0.00000 ;\r\n\
    N0016 G01 X-4.00000 Y11.00000 ;\r\n\
    N0017 G03 X-5.00000 Y12.00000 I-1.00000 J0.00000 ;\r\n\
    N0018 G01 X-18.00000 Y12.00000 ;\r\n\
    N0019 G03 X-20.00000 Y10.00000 I0.00000 J-2.00000 ;\r\n\
    N0020 G01 X-20.00000 Y-10.00000 ;\r\n\
    N0021 G03 X-18.00000 Y-12.00000 I2.00000 J0.00000 ;\r\n\
    N0022 G01 X0.00000 Y-12.00000 ;\r\n\
    N0023 M02 ;\r\n";

#[test]
fn a_reentrant_opening_posts_g01_g02_and_g03_counter_clockwise_from_its_seam() {
    assert_eq!(written(&job([0., -7.]), &opening(1.)).unwrap(), REENT20);
    let (_, setup) = prepare(&job([0., -7.]), &opening(1.), "REENT20").unwrap();
    assert_eq!(setup.start_hole_um, [0, -7_000]);
    assert!(setup.encode_die().unwrap().sbl.contains("ID_PUNCH,c_die"));
}

#[test]
fn a_clockwise_drawing_posts_the_same_counter_clockwise_iso() {
    let reversed: Vec<_> = opening(1.)
        .into_iter()
        .rev()
        .map(|s| InspectionSegment {
            start: s.end,
            end: s.start,
            arc: s.arc.map(|(c, t)| (c, -t)),
        })
        .collect();
    assert_eq!(written(&job([0., -7.]), &reversed).unwrap(), REENT20);
}

#[test]
fn the_nearest_straight_wall_with_room_takes_the_entry() {
    // Near the left wall: the seam is its foot, and the cut goes down it first.
    let left = written(&job([-15., 0.]), &opening(1.)).unwrap();
    assert!(left.contains("N0001 G00 X-20.00000 Y0.00000 ;\r\nN0002 G01 X-20.00000 Y-10.00000 ;"), "{left}");
    // 3 mm before the tooth tip, with exactly 3 mm of the tip on both sides.
    let tooth = written(&job([11., 0.]), &opening(1.)).unwrap();
    assert!(tooth.contains("N0001 G00 X14.00000 Y0.00000 ;\r\nN0002 G01 X14.00000 Y3.00000 ;"), "{tooth}");
}

#[test]
fn a_concave_arc_not_larger_than_the_rough_offset_is_refused_with_its_values() {
    for f in [0.2, 0.245] {
        let why = refusal(&job([0., -7.]), &opening(f));
        assert!(why.starts_with("Native arc offset rule"), "{why}");
        assert!(why.contains(&format!("radius {f:.5} mm")) && why.contains("offset 0.245 mm"), "{why}");
    }
    // An R0.5 tooth fillet keeps 0.255 mm of radius on the rough pass.
    assert!(written(&job([0., -7.]), &opening(0.5)).is_ok());
}

#[test]
fn unequal_arc_radii_on_the_grid_are_refused_with_both_radii() {
    let mut walls = opening(1.);
    walls[1] = a([18., -10.], [18., -12.], [20.00001, -10.], 90.);
    walls[2] = l([20.00001, -10.], [20., -4.]);
    let why = refusal(&job([0., -7.]), &walls);
    assert!(why.starts_with("Native arc radius rule"), "{why}");
    assert!(why.contains("start radius 2.00000 mm and end radius 2.00001 mm"), "{why}");
}

#[test]
fn an_arc_without_a_common_tangent_is_refused_at_its_joint() {
    let mut walls = opening(1.);
    // The same tongue ends on a circle about X0 Y5: it meets the sides at an angle.
    walls[13] = a([0., 5.], [4., 4.], [-4., 4.], -(180. - 2. * (0.25f64).atan().to_degrees()));
    let why = refusal(&job([0., -7.]), &walls);
    assert!(why.starts_with("Native joint rule") && why.contains("X4.00000 Y4.00000"), "{why}");
}

#[test]
fn a_sharp_inside_corner_of_the_plate_leaves_rest_and_is_refused() {
    let mut walls = opening(1.);
    walls.splice(10..13, [l([18., 12.], [4., 12.]), l([4., 12.], [4., 4.])]);
    let why = refusal(&job([0., -7.]), &walls);
    assert!(why.starts_with("Strict cover rule") && why.contains("X4.00000 Y12.00000"), "{why}");
    assert!(why.contains("turns 90.000 degrees") && why.contains("leaves 0.1015 mm"), "{why}");
}

#[test]
fn walls_closer_than_two_rough_passes_and_wire_are_refused() {
    let mut walls = opening(1.);
    // The tooth tip at X5 comes 1.099 mm from the round tongue tip.
    walls[4] = l([19., -3.], [5., -3.]);
    walls[5] = l([5., -3.], [5., 3.]);
    walls[6] = l([5., 3.], [19., 3.]);
    let why = refusal(&job([0., -7.]), &walls);
    assert!(why.contains("are 1.09902 mm apart") && why.contains("more than 1.18297 mm"), "{why}");
}

#[test]
fn the_hole_lead_and_stock_must_clear_the_walls() {
    let walls = opening(1.);
    for (hole, why) in [
        ([0., 2.], "inside the die opening"),
        ([17., 0.], "inside the die opening"),
        ([0., -11.5], "inside the die opening"),
        ([0., -9.5], "3 mm inside it"),
    ] {
        let error = refusal(&job(hole), &walls);
        assert!(error.contains(why), "{hole:?}: {error}");
    }
    let tight = AgieJob { stock_min: [-20.3, -30., -20.], ..job([0., -7.]) };
    assert!(refusal(&tight, &walls).contains("0.4 mm of stock beyond its walls"));
    let thin = AgieJob { stock_min: [-40., -30., -15.], ..job([0., -7.]) };
    assert!(refusal(&thin, &walls).contains("20 mm"));
    let open = AgieJob { side: AgieSide::Punch, ..job([0., -7.]) };
    assert!(refusal(&open, &walls).contains("die only"));
}
