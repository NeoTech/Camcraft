//! M13.1 (#23): dual contouring keeps a machined pocket's sharp interior
//! corner. A flat endmill slots a box; the slot's floor/wall edge is a
//! true 90-degree concave line. The dual mesh passes within a small
//! twentieth of a cell of that line, where marching tetrahedra chamfers
//! it by about a third of a cell; both stay closed. Degenerate fields
//! (everything cut away, a body smaller than a cell) answer cleanly.
//! Provenance: our own fixtures over the M7.1 field engine and the
//! extraction of Ju, Losasso, Schaefer & Warren (SIGGRAPH 2002); nothing
//! from vendor binaries.
//!
//! Gate map (M4.4): covers camcraft_sim::stock_sim::iso, stock_sim::dual, stock_sim::qef.
//! #4107: a cut floor on the lattice meshes to a valid solid by marching
//! tetrahedra (tet's crossing snaps to the sample).

use std::collections::HashMap;

use camcraft_sim::cl::{Move, MoveKind};
use camcraft_sim::geometry::{cube, Point3, Triangle};
use camcraft_sim::stock_sim::iso::{extract, extract_with, open_edges, unpaired_edges, IsoMethod};
use camcraft_sim::stock_sim::{dist_seg_tri, Backend, SdfField, SdfTool, SimOp, Stock, FIELD_BAND};

const CELL: f64 = 1.0;
/// The slot: a flat endmill of radius 2 at tip z 6.2 along y = 5.3, so
/// its walls stand at y = 3.3 and 7.3 - off the lattice's samples and
/// faces both, the generic case.
const R: f64 = 2.0;
const FLOOR: f64 = 6.2;
const AXIS_Y: f64 = 5.3;

fn feed(from: Point3, to: Point3) -> Move {
    Move { kind: MoveKind::Feed, from, to, feed: Some(500.0), arc: None, axis: None, rotary: None }
}

fn slotted() -> SdfField {
    let m = cube(Point3::new(0.0, 0.0, 0.0), Point3::new(10.0, 10.0, 10.0));
    let mut f = SdfField::from_mesh(&m.facets, CELL, FIELD_BAND).expect("box field");
    let tool = SdfTool::Flat { r: R, flank: 20.0 };
    let moves = [
        feed(Point3::new(2.0, AXIS_Y, 14.0), Point3::new(2.0, AXIS_Y, FLOOR)),
        feed(Point3::new(2.0, AXIS_Y, FLOOR), Point3::new(8.0, AXIS_Y, FLOOR)),
    ];
    f.subtract_batch(&moves, &tool, 0.01).expect("the slot cuts");
    f
}

/// The farthest the mesh stands from the slot's straight floor/wall
/// edges, sampled along their middle (clear of the round ends).
fn edge_gap(tris: &[Triangle]) -> f64 {
    let mut worst = 0.0f64;
    for y in [AXIS_Y - R, AXIS_Y + R] {
        for s in 0..=16 {
            let e = Point3::new(3.5 + 3.0 * s as f64 / 16.0, y, FLOOR);
            let d = tris.iter().map(|t| dist_seg_tri(e, e, t)).fold(f64::INFINITY, f64::min);
            worst = worst.max(d);
        }
    }
    worst
}

#[test]
fn dual_contouring_keeps_the_slot_floor_wall_edge_sharp() {
    let f = slotted();
    let dual = extract_with(&f, CELL, IsoMethod::Dual).expect("dual mesh");
    let mt = extract_with(&f, CELL, IsoMethod::Marching).expect("marching mesh");
    assert_eq!(open_edges(&dual), 0, "the dual shell is closed");
    assert_eq!(open_edges(&mt), 0, "the marching shell is closed");
    let (gd, gm) = (edge_gap(&dual), edge_gap(&mt));
    eprintln!("floor/wall edge gap: dual {gd:.4}, marching {gm:.4} (cells of {CELL})");
    assert!(gm > 0.2 * CELL, "marching tetrahedra chamfers the corner: {gm}");
    assert!(gd < 0.05 * CELL, "the dual mesh keeps the corner within a twentieth of a cell: {gd}");
    assert!(gd < 0.5 * gm, "dual {gd} vs marching {gm}");
}

#[test]
fn dual_is_the_default_extractor() {
    let f = slotted();
    let a = extract(&f).expect("default mesh");
    let b = extract_with(&f, CELL, IsoMethod::Dual).expect("dual mesh");
    assert_eq!(a.len(), b.len());
    assert_eq!(IsoMethod::default(), IsoMethod::Dual);
}

#[test]
fn a_fully_cut_field_extracts_nothing_without_error() {
    let m = cube(Point3::new(0.0, 0.0, 0.0), Point3::new(4.0, 4.0, 4.0));
    let mut f = SdfField::from_mesh(&m.facets, CELL, FIELD_BAND).expect("box field");
    let tool = SdfTool::Flat { r: 10.0, flank: 30.0 };
    f.subtract(&feed(Point3::new(2.0, 2.0, 20.0), Point3::new(2.0, 2.0, -5.0)), &tool, 0.01).expect("cut");
    for method in [IsoMethod::Dual, IsoMethod::Marching] {
        let tris = extract_with(&f, CELL, method).expect("an empty answer, not an error");
        assert!(tris.is_empty(), "{method:?}: nothing is left, {} triangles", tris.len());
    }
}

#[test]
fn a_body_under_one_cell_never_panics_and_never_opens() {
    let m = cube(Point3::new(0.1, 0.1, 0.1), Point3::new(0.6, 0.6, 0.6));
    let Ok(f) = SdfField::from_mesh(&m.facets, CELL, FIELD_BAND) else {
        return; // refused by the field gate: a clean answer too
    };
    for method in [IsoMethod::Dual, IsoMethod::Marching] {
        if let Ok(tris) = extract_with(&f, CELL, method) {
            assert_eq!(open_edges(&tris), 0, "{method:?}: closed or empty");
            // #4122: closed is not enough; the dual mesh here was three
            // flat fins on one edge, volume 0, with no odd edge.
            assert_eq!(unpaired_edges(&tris), 0, "{method:?}: every edge on two facets");
            assert!(tris.iter().all(|t| t.a.is_finite() && t.b.is_finite() && t.c.is_finite()));
        }
    }
    for s in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(extract_with(&f, s, IsoMethod::Dual).is_err(), "step {s} must refuse");
    }
}


#[test]
fn a_curved_ball_pocket_stays_closed_and_never_bulges() {
    // A ball nose of radius 2.5 plunged 3 deep: a sphere-capped hole,
    // curved everywhere at under three cells of radius. The dual mesh
    // borrows no corner planes here and must agree with the field's
    // volume as closely as marching does.
    let m = cube(Point3::new(0.0, 0.0, 0.0), Point3::new(10.0, 10.0, 10.0));
    let mut f = SdfField::from_mesh(&m.facets, CELL, FIELD_BAND).expect("box field");
    let tool = SdfTool::Shanked { r: 2.5, shank: 20.0 };
    f.subtract(&feed(Point3::new(5.1, 4.9, 14.0), Point3::new(5.1, 4.9, 7.0)), &tool, 0.01).expect("cut");
    let vol = |t: &[Triangle]| t.iter().map(|t| t.a.dot(t.b.cross(t.c)) / 6.0).sum::<f64>();
    let dual = extract_with(&f, CELL, IsoMethod::Dual).expect("dual mesh");
    let mt = extract_with(&f, CELL, IsoMethod::Marching).expect("marching mesh");
    assert_eq!(open_edges(&dual), 0);
    let r = 2.5f64;
    // Box minus the cylinder above the ball centre (z 9.5..10) and the
    // lower hemisphere (centre z 9.5).
    let truth = 1000.0 - std::f64::consts::PI * r * r * 0.5 - 2.0 / 3.0 * std::f64::consts::PI * r.powi(3);
    let (vd, vm) = (vol(&dual), vol(&mt));
    eprintln!("ball pocket volume: dual {vd:.3}, marching {vm:.3}, true {truth:.3}");
    assert!((vd - truth).abs() <= (vm - truth).abs().max(0.005 * truth), "dual {vd} vs marching {vm}, truth {truth}");
}

/// #4107: a 30 x 30 x 20 mm block at cell 0.5, faced by four passes of a
/// flat tool 6 mm across at height `z`. The lattice samples stand at
/// (k + 0.5) * 0.5: 15.25 is one, 15.1 is not.
fn faced(z: f64) -> Stock {
    let mut s = Stock::block(Backend::Field { band: FIELD_BAND }, 0.0, 0.0, 30.0, 30.0, 20.0, 0.5)
        .unwrap_or_else(|e| panic!("block: {e}"));
    let moves: Vec<Move> = (0..4)
        .map(|k| {
            let y = 8.0 + 4.0 * k as f64;
            feed(Point3::new(4.0, y, z), Point3::new(26.0, y, z))
        })
        .collect();
    s.simulate(&[SimOp { moves: &moves, radius: 3.0, ball: false, cone: None, corner: None }])
        .unwrap_or_else(|e| panic!("cut: {e}"));
    s
}

/// Facets that lose a corner at the 1e-9 weld, and edges that are not on
/// exactly two facets after it. `open_edges` counts only odd edges; an
/// edge on four facets passes it, so this counts every edge.
fn weld_defects(tris: &[Triangle]) -> (usize, usize) {
    let key = |p: Point3| [(p.x * 1e9).round() as i64, (p.y * 1e9).round() as i64, (p.z * 1e9).round() as i64];
    let mut collapsed = 0;
    let mut uses: HashMap<([i64; 3], [i64; 3]), u32> = HashMap::new();
    for t in tris {
        let k = [key(t.a), key(t.b), key(t.c)];
        if k[0] == k[1] || k[1] == k[2] || k[0] == k[2] {
            collapsed += 1;
        }
        for (a, b) in [(k[0], k[1]), (k[1], k[2]), (k[2], k[0])] {
            *uses.entry(if a <= b { (a, b) } else { (b, a) }).or_insert(0) += 1;
        }
    }
    (collapsed, uses.values().filter(|&&n| n != 2).count())
}

#[test]
fn a_floor_on_the_lattice_meshes_to_a_valid_solid_by_marching_tetrahedra() {
    let s = faced(15.25);
    let f = s.field().expect("a field");
    assert!(f.distance(Point3::new(15.25, 15.25, 15.25)).abs() < 1e-9, "the floor stands on a lattice sample");
    let tris = extract_with(f, f.cell(), IsoMethod::Marching).unwrap_or_else(|e| panic!("mesh: {e}"));
    assert_eq!(weld_defects(&tris), (0, 0), "collapsed facets, edges not on two facets");
}

#[test]
fn a_floor_off_the_lattice_stays_valid_by_both_methods() {
    let s = faced(15.1);
    let f = s.field().expect("a field");
    for m in [IsoMethod::Marching, IsoMethod::Dual] {
        let tris = extract_with(f, f.cell(), m).unwrap_or_else(|e| panic!("mesh: {e}"));
        assert_eq!(weld_defects(&tris), (0, 0), "{m:?}");
    }
}
