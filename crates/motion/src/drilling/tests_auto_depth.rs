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
        material_top: &[],
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

/// The review case: uncut stock over a mouth is material. The caller's
/// `material_top` (the part and the stock as the program ahead leaves
/// them) counts from the higher of it and the mouth, never above the R
/// plane; an empty list or a non-finite entry is the mouth alone.
#[test]
fn auto_counts_the_material_over_the_mouth() {
    let tool = drill_tool(6.0);
    let holes = [hole(10.0, 6.5)];
    let with = |tops: &[f64]| {
        let p = CycleParams { material_top: tops, ..axial(&tool, &holes, 19.0, CycleChoice::Auto) };
        kinds(&drill(&p).expect("drills"))
    };
    assert_eq!(with(&[17.0]), [CycleKind::Peck], "7 mm of stock over the mouth: 10.5 mm deep");
    assert_eq!(with(&[10.0]), [CycleKind::Drill], "the stock is cut to the mouth: 3.5 mm");
    assert_eq!(with(&[]), [CycleKind::Drill], "no reading: the mouth");
    assert_eq!(with(&[f64::NAN]), [CycleKind::Drill], "a non-finite reading: the mouth");
    assert_eq!(with(&[40.0]), [CycleKind::Peck], "clamped to the R plane z 19: 12.5 mm");
}

/// The radial twin: the raw stock (`stock_r`) over a mouth on the part is
/// material, the standoff outside the stock is not. A 2 mm drill, 8 mm in
/// the part: with the stock flush at the mouth it chip-breaks (4 x d);
/// with 10 mm of raw bar over the mouth it is 18 mm deep, full retracts.
#[test]
fn a_cross_hole_counts_the_raw_stock_over_its_mouth() {
    let tool = drill_tool(2.0);
    let holes = [cross_hole(-8.0)];
    let at = |stock_r: f64, r_d: f64| {
        let p = CrossParams { stock_r, r_d, clearance_d: r_d + 1.0, ..radial(&tool, &holes, CycleChoice::Auto) };
        let t = cross_drill(&p).expect("drills");
        t.moves.iter().filter(|m| m.kind == crate::cl::MoveKind::Rapid).count()
    };
    let flush = at(15.0, 12.0);
    let buried = at(25.0, 12.0);
    assert!(buried > flush, "full retracts take more rapids than chip breaks: {buried} vs {flush}");
}

/// Review of 4cbc56ee9: a depth that sits on an Auto limit up to float
/// roundoff picks the same cycle on every system. Both limits are
/// inclusive within `AUTO_TOL`; two microns past a limit is the next
/// cycle. The radial case puts 10 mm of stock over a 6 mm hole on a
/// 4 mm drill, so the depth is exactly 4 x d.
#[test]
fn an_auto_limit_reads_the_same_under_roundoff() {
    use crate::tools::ToolKind::Drill as D;
    let up = |x: f64| f64::from_bits(x.to_bits() + 4);
    let down = |x: f64| f64::from_bits(x.to_bits() - 4);
    for depth in [down(16.0), 16.0, up(16.0)] {
        assert_eq!(auto_kind(D, depth, 4.0), CycleKind::Peck, "4 x d under roundoff: {depth:e}");
    }
    for depth in [down(4.0), 4.0, up(4.0)] {
        assert_eq!(auto_kind(D, depth, 4.0), CycleKind::Drill, "1 x d under roundoff: {depth:e}");
    }
    assert_eq!(auto_kind(D, 16.0 + 2.0 * AUTO_TOL, 4.0), CycleKind::Deep);
    assert_eq!(auto_kind(D, 4.0 + 2.0 * AUTO_TOL, 4.0), CycleKind::Peck);
    // The radial reading: a mouth at r 15, the stock at r 25 (or its
    // roundoff, as the reference chain reads 25.000000000000004).
    let tool = drill_tool(4.0);
    let holes = [cross_hole(-6.0)];
    let kinds_at = |stock_r: f64| {
        let p = CrossParams { stock_r, r_d: 12.0, clearance_d: 13.0, ..radial(&tool, &holes, CycleChoice::Auto) };
        let t = cross_drill(&p).expect("drills");
        t.moves.iter().map(|m| m.kind).collect::<Vec<_>>()
    };
    let exact = kinds_at(25.0);
    assert_eq!(kinds_at(up(25.0)), exact, "a roundoff over the stock radius keeps the cycle");
    assert_eq!(kinds_at(down(25.0)), exact, "a roundoff under the stock radius keeps the cycle");
}
