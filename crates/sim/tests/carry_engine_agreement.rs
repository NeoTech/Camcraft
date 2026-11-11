//! #637 step D: a carried stock reads the same on both engines. The same
//! block, cut by the same slots, carried through every lattice turn: the
//! column carry (`Body::carry`, re-indexed) and the field carry
//! (`CarriedField::column_body`, read back) agree column for column on
//! the tops, the floors and which columns are empty.
//!
//! Gate map (#637): covers sim::stock_sim::field_body (CarriedField::
//! column_body), carry::Body (from_parts) against carry::Body::carry,
//! field_turn (SdfField::turned against the view, #779).
//! Provenance: our own fixtures.

use camcraft_sim::cl::{Move, MoveKind};
use camcraft_sim::geometry::Point3;
use camcraft_sim::stock_sim::{Backend, Body, CarriedField, Lattice, LatticeTurn, SimOp, Stock, FIELD_BAND};

const TOP: f64 = 5.0;
const CELL: f64 = 0.5;
/// The two engines read a top at a cell centre each: equal to rounding
/// on the mesh planes and the tool's flat floor.
const EPS: f64 = 1e-6;

fn turns() -> Vec<LatticeTurn> {
    let mut out = Vec::new();
    for (r, t) in [
        ([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], [3.0, -2.0, 1.0]),
        ([[0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]], [10.0, 0.0, 0.0]),
        ([[-1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0]], [20.0, 12.0, 0.0]),
        ([[0.0, 1.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 0.0, 1.0]], [0.0, 20.0, 0.0]),
        ([[1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, -1.0]], [0.0, 12.0, 5.0]),
        ([[-1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, -1.0]], [20.0, 0.0, 5.0]),
        ([[0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, -1.0]], [0.0, 0.0, 5.0]),
        ([[0.0, -1.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 0.0, -1.0]], [12.0, 20.0, 5.0]),
    ] {
        out.push(LatticeTurn::from_matrix(r, t).unwrap_or_else(|e| panic!("turn {r:?}: {e}")));
    }
    out
}

/// A flat-end slot across the 20 x 12 block at depth `z`.
fn cut(b: Backend, z: f64) -> Stock {
    cut_with(b, 6.0, z, 2.0, false)
}

/// A slot across the block along y = `y`, depth `z`, radius `r`.
fn cut_with(b: Backend, y: f64, z: f64, r: f64, ball: bool) -> Stock {
    let mut s = Stock::block(b, 0.0, 0.0, 20.0, 12.0, TOP, CELL).expect("block");
    let p = |x: f64, y: f64, z: f64| Point3::new(x, y, z);
    let moves = [
        Move::new(MoveKind::Rapid, p(-4.0, y, 8.0), p(-4.0, y, z), None),
        Move::new(MoveKind::Feed, p(-4.0, y, z), p(24.0, y, z), Some(500.0)),
    ];
    s.simulate(&[SimOp { moves: &moves, radius: r, ball, cone: None, corner: None }]).expect("cut");
    s
}

fn agree(slot_z: f64) {
    agree_with(cut(Backend::Column, slot_z), cut(Backend::Field { band: FIELD_BAND }, slot_z), EPS);
}

fn agree_with(col: Stock, field: Stock, eps: f64) {
    let col = match col {
        Stock::Column(d) => d,
        Stock::Field(_) => panic!("a column stock"),
    };
    let field = field.field().expect("a field").clone();
    // The block stock's floor as the field built it (an uncut corner).
    let floor = match field.floor_at(0.25, 0.25) {
        camcraft_sim::stock_sim::FloorRead::Floor(z) => z,
        r => panic!("the block's floor: {r:?}"),
    };
    let body = Body::on_floor(col, floor);
    for t in turns() {
        let a = body.carry(&t).unwrap_or_else(|e| panic!("{t:?}: column carry: {e}"));
        let d = &a.dex;
        let lat = Lattice { x0: d.x0, y0: d.y0, cell: d.cell, nx: d.nx, ny: d.ny };
        let b = CarriedField::new(field.clone(), t).column_body(lat, false).unwrap_or_else(|e| panic!("{t:?}: field carry: {e}"));
        assert_eq!(a.floor_judged(), b.floor_judged(), "{t:?}");
        for k in 0..d.nx * d.ny {
            assert_eq!(a.empty_at(k), b.empty_at(k), "{t:?} column {k}: empty");
            if a.empty_at(k) {
                continue;
            }
            let (ta, tb) = (a.dex.top_at(k), b.dex.top_at(k));
            let (fa, fb) = (a.floor_at(k), b.floor_at(k));
            assert!((ta - tb).abs() <= eps, "{t:?} column {k}: top {ta} vs {tb}");
            assert!((fa - fb).abs() <= eps, "{t:?} column {k}: floor {fa} vs {fb}");
        }
        let cells = (d.nx * d.ny) as f64 * d.cell * d.cell;
        assert!((a.volume() - b.volume()).abs() <= eps * cells, "{t:?}: volume {} vs {}", a.volume(), b.volume());
    }
}

#[test]
fn a_slot_from_the_top_carries_the_same_on_both_engines() {
    agree(3.0);
}

#[test]
fn a_deep_slot_carries_the_same_on_both_engines() {
    // A slot 4 deep: the cut columns stand on the block's floor on both.
    agree(1.0);
}

#[test]
fn a_slot_through_the_block_carries_the_same_on_both_engines() {
    // #637 E1: a slot below the block's floor - cut through on both
    // engines, its columns empty through every turn, the columns beside
    // it (within a cell of the wall) whole.
    agree(-1.0);
}

#[test]
fn a_sliver_on_the_floor_carries_the_same_on_both_engines() {
    // #637 E1: 0.2 of stock left under the slot, thinner than the 0.5
    // cell and holding no cell centre: the field reads it from the cut
    // layer's ray entry, the column engine from its lowered top.
    agree(0.2);
}

#[test]
fn an_off_grid_ball_slot_carries_the_same_on_both_engines() {
    // Review of #656: walls off the cell boundaries and a curved floor -
    // both engines read the ball's surface at the same cell centres; the
    // field's march brackets its top within MARCH_EPS.
    const MARCH_EPS: f64 = 1e-3;
    let (y, z, r) = (6.3, 3.2, 1.7);
    agree_with(
        cut_with(Backend::Column, y, z, r, true),
        cut_with(Backend::Field { band: FIELD_BAND }, y, z, r, true),
        MARCH_EPS,
    );
}

/// #637 E2a (#779): a field with the turn baked into its bricks, seen
/// through the translation alone, is the view. Through all eight turns the
/// distance at every column centre of the carried lattice, at every cell
/// centre height, is bit-identical; for the four upright turns the whole
/// column read-back is too (a flipped bake's floor under a cut is E2b's
/// up-march - until then it refuses by name, never guesses).
fn baked_agrees(field: Stock) {
    let f = field.field().expect("a field").clone();
    let col = match cut(Backend::Column, 3.0) {
        Stock::Column(d) => d,
        Stock::Field(_) => panic!("a column stock"),
    };
    let body = Body::on_floor(col, 0.0);
    for t in turns() {
        let d = body.carry(&t).unwrap_or_else(|e| panic!("{t:?}: column carry: {e}")).dex;
        let lat = Lattice { x0: d.x0, y0: d.y0, cell: d.cell, nx: d.nx, ny: d.ny };
        let (g, shift) = f.turned(&t);
        let view = CarriedField::new(f.clone(), t);
        let only = LatticeTurn::from_matrix([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], [shift.x, shift.y, shift.z])
            .unwrap_or_else(|e| panic!("a translation: {e}"));
        let baked = CarriedField::new(g, only);
        for jy in 0..d.ny {
            for jx in 0..d.nx {
                let (x, y) = (d.x0 + (jx as f64 + 0.5) * d.cell, d.y0 + (jy as f64 + 0.5) * d.cell);
                for k in -4..16 {
                    let q = Point3::new(x, y, (f64::from(k) + 0.5) * CELL);
                    assert_eq!(baked.distance(q).to_bits(), view.distance(q).to_bits(), "{t:?} at {q:?}");
                }
            }
        }
        if t.flips() {
            continue;
        }
        let a = view.column_body(lat, false).unwrap_or_else(|e| panic!("{t:?}: view: {e}"));
        let b = baked.column_body(lat, false).unwrap_or_else(|e| panic!("{t:?}: baked: {e}"));
        assert_eq!(a.floor_judged(), b.floor_judged(), "{t:?}");
        for k in 0..d.nx * d.ny {
            assert_eq!(a.empty_at(k), b.empty_at(k), "{t:?} column {k}: empty");
            assert_eq!(a.dex.top_at(k).to_bits(), b.dex.top_at(k).to_bits(), "{t:?} column {k}: top");
            assert_eq!(a.floor_at(k).to_bits(), b.floor_at(k).to_bits(), "{t:?} column {k}: floor");
        }
    }
}

#[test]
fn a_baked_turn_reads_what_the_view_reads_through_every_turn() {
    let field = Backend::Field { band: FIELD_BAND };
    baked_agrees(cut(field, 3.0));
    baked_agrees(cut(field, -1.0));
    baked_agrees(cut(field, 0.2));
    baked_agrees(cut_with(field, 6.3, 3.2, 1.7, true));
}
