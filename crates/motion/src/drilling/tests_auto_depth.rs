//! #2443: Auto reads a hole's depth in material - from its mouth, not
//! from the R plane above it - and `Deep` is the user's word for the
//! full-retract cycle (G83). A spot under a tall R plane drills (G81);
//! it is not a chip-break peck whose step is longer than the hole.

use super::tests::drill_tool;
use super::*;
use crate::cl::CycleKind;
use crate::geo2d::Point2;
use crate::geometry::Point3;

fn hole(entry: f64, floor: f64) -> Hole {
    Hole { xy: Point2::new(8.0, 8.0), radius: 3.0, entry_z: entry, floor_z: floor, through: false, stepped: false }
}

fn axial<'a>(tool: &'a crate::tools::Tool, holes: &'a [Hole], r_z: f64, cycle: CycleChoice) -> CycleParams<'a> {
    CycleParams {
        through_clear: THROUGH_CLEAR,
        break_through_from: BreakThroughFrom::default(),
        clearance_z: r_z + 5.0,
        r_z,
        hop_z: None,
        bottom_z: None,
        cycle,
        peck: 0.0,
        dwell: 0.0,
        feed: 300.0,
        plunge_feed: 100.0,
        tool,
        holes,
    }
}

fn kinds(t: &crate::cl::Toolpath) -> Vec<CycleKind> {
    t.cycles.iter().map(|c| c.kind).collect()
}

/// The issue's case: the R plane at the model top (z 15 here 17), holes
/// whose mouths are at z 10 and 7, and a 3.5 mm deep spot with a 6 mm
/// drill. The old law read 10.5 and 13.5 mm and chose the G73 peck.
#[test]
fn auto_measures_a_spot_from_its_mouth_not_from_the_r_plane() {
    let tool = drill_tool(6.0);
    let holes = [hole(10.0, 6.5), hole(7.0, 3.5)];
    let t = drill(&axial(&tool, &holes, 17.0, CycleChoice::Auto)).expect("drills");
    assert_eq!(kinds(&t), [CycleKind::Drill, CycleKind::Drill]);
    // The plunge still runs from the R plane to the floor: only the
    // choice of cycle reads the mouth.
    assert_eq!(t.cycles[0].r_z, 17.0);

    // Past one diameter of material the peck stays, and past four the
    // full retract stays, whatever the air above.
    let deeper = [hole(10.0, 2.0), hole(10.0, -20.0)];
    let t = drill(&axial(&tool, &deeper, 17.0, CycleChoice::Auto)).expect("drills");
    assert_eq!(kinds(&t), [CycleKind::Peck, CycleKind::Deep]);
}

/// An R plane below the mouth (a hole drilled from inside a pocket wall
/// line) reads the depth from the R plane: the feed starts there.
#[test]
fn auto_reads_a_lower_r_plane_as_the_start_of_the_depth() {
    let tool = drill_tool(6.0);
    let holes = [hole(10.0, 2.0)];
    let t = drill(&axial(&tool, &holes, 7.0, CycleChoice::Auto)).expect("drills");
    assert_eq!(kinds(&t), [CycleKind::Drill], "5 mm from the R plane on a 6 mm drill");
}

/// `Deep` posts the full-retract cycle on a hole that Auto would drill
/// or chip-break.
#[test]
fn the_deep_choice_is_the_full_retract_cycle() {
    let tool = drill_tool(6.0);
    let holes = [hole(10.0, 6.5), hole(10.0, 2.0)];
    let t = drill(&axial(&tool, &holes, 12.0, CycleChoice::Deep)).expect("drills");
    assert_eq!(kinds(&t), [CycleKind::Deep, CycleKind::Deep]);
    assert!((t.cycles[1].peck - 6.0).abs() < 1e-9, "the auto G83 step is one diameter");
}

fn cross_hole(floor_d: f64) -> CrossHole {
    CrossHole { origin: Point3::new(15.0, 0.0, 10.0), dir: Point3::new(-1.0, 0.0, 0.0), radius: 1.0, entry_d: 0.0, floor_d, through: false }
}

fn radial<'a>(tool: &'a crate::tools::Tool, holes: &'a [CrossHole], cycle: CycleChoice) -> CrossParams<'a> {
    CrossParams {
        clearance_d: 2.0,
        r_d: 1.0,
        hop_d: None,
        bottom_d: None,
        cycle,
        peck: 0.0,
        dwell: 0.0,
        feed: 500.0,
        plunge_feed: 100.0,
        stock_r: 15.0,
        stock_z_max: None,
        tool,
        holes,
    }
}

fn plunges(t: &crate::cl::Toolpath) -> usize {
    t.moves.iter().filter(|m| m.kind == crate::cl::MoveKind::Plunge).count()
}

/// The radial law: the R standoff outside the mouth is air too. 8 mm in
/// material on a 2 mm drill is four diameters: a chip-break peck (step
/// 6 mm, two plunges), where the old reading (9 mm from the standoff)
/// chose full retracts. The `Deep` choice reaches the cross-hole cycle
/// (no unreachable arm) and pecks one diameter at a time.
#[test]
fn a_cross_hole_reads_its_depth_from_the_mouth_and_takes_deep() {
    let tool = drill_tool(2.0);
    let holes = [cross_hole(-8.0)];
    let t = cross_drill(&radial(&tool, &holes, CycleChoice::Auto)).expect("drills");
    assert_eq!(plunges(&t), 2, "a chip-break peck of 6 mm steps");
    let t = cross_drill(&radial(&tool, &holes, CycleChoice::Deep)).expect("drills");
    assert_eq!(plunges(&t), 5, "full retracts of 2 mm steps from the standoff: -1, -3, -5, -7, -8");
}

/// The review case: the depth in material picks the Auto cycle, but the
/// pecks still run from the R standoff. A far standoff is refused by the
/// peck budget, never looped over: 30000 mm gives 10,000 pecks, and
/// 1e20 is a step that cannot move `t`. This holds for an explicit Peck
/// and for the Peck that Auto chooses.
#[test]
fn a_far_radial_standoff_is_refused_by_the_peck_budget() {
    let tool = drill_tool(2.0);
    let holes = [cross_hole(-8.0)];
    for cycle in [CycleChoice::Peck, CycleChoice::Auto, CycleChoice::Deep] {
        for r_d in [30_000.0, 1e20] {
            let p = CrossParams { r_d, ..radial(&tool, &holes, cycle) };
            let e = cross_drill(&p).expect_err("a far standoff is refused");
            assert!(e.to_string().contains("peck plan unusable"), "{cycle:?} at {r_d}: {e}");
        }
    }
}
