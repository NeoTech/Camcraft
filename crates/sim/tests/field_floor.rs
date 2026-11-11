//! #637 steps B and E1: a field column's floor is read exactly from the
//! shell where no cut reaches it, and under a cut by the monotone law - a
//! spanning tool lowers the top, never raises the floor, so a column is
//! one interval `[floor, tool bottom]` or nothing. Only a bounded tool
//! (a truncated taper) keeps the refusal.
//!
//! Gate map (#637): covers sim::stock_sim::field_floor (SdfField::floor_at,
//! SdfField::column_span).
//! Provenance: our own fixtures.

use camcraft_sim::cl::{Move, MoveKind};
use camcraft_sim::geometry::{cube, Point3};
use camcraft_sim::stock_sim::{ColumnSpan, FloorRead, SdfField, Stock, SimOp, FIELD_BAND};

/// A real 20 x 12 x 6 block (z -6..0) as a closed field.
fn plate() -> SdfField {
    match SdfField::from_mesh(&cube(Point3::new(0.0, 0.0, -6.0), Point3::new(20.0, 12.0, 0.0)).facets, 0.5, FIELD_BAND) {
        Ok(f) => f,
        Err(e) => panic!("plate: {e:?}"),
    }
}

/// A slot along y 6 at depth `z`, radius `r`, flat or tapered (`cot`).
fn slot(f: SdfField, z: f64, r: f64, cone: Option<f64>) -> SdfField {
    let mut s = Stock::Field(f);
    let p = |x: f64, y: f64, z: f64| Point3::new(x, y, z);
    let moves = [
        Move::new(MoveKind::Rapid, p(-4.0, 6.0, 5.0), p(-4.0, 6.0, z), None),
        Move::new(MoveKind::Feed, p(-4.0, 6.0, z), p(24.0, 6.0, z), Some(500.0)),
    ];
    if let Err(e) = s.simulate(&[SimOp { moves: &moves, radius: r, ball: false, cone, corner: None }]) {
        panic!("cut: {e:?}");
    }
    s.field().expect("a field").clone()
}

fn cut(f: SdfField, z: f64) -> SdfField {
    slot(f, z, 2.0, None)
}

#[test]
fn an_uncut_floor_reads_the_shell_exactly() {
    let f = plate();
    assert_eq!(f.floor_at(10.25, 6.25), FloorRead::Floor(-6.0));
    assert_eq!(f.column_span(10.25, 6.25), ColumnSpan::Material { top: 0.0, floor: -6.0 });
    // A shallow slot from the top leaves the floor untouched.
    let g = cut(f, -2.0);
    assert_eq!(g.floor_at(10.25, 6.25), FloorRead::Floor(-6.0), "a top cut does not reach the floor");
    assert_eq!(g.column_span(10.25, 6.25), ColumnSpan::Material { top: -2.0, floor: -6.0 });
    assert_eq!(g.floor_at(40.0, 6.25), FloorRead::NoMaterial, "beside the stock");
}

#[test]
fn a_slot_through_the_plate_reads_empty_and_its_neighbours_exact() {
    // A slot cut through the plate (y 4..8): nothing stands in it, and
    // the columns beside it - within a cell of the wall too - keep the
    // shell floor (#637 E1: the monotone law, not step B's refusal).
    let g = cut(plate(), -7.0);
    assert_eq!(g.floor_at(10.25, 6.25), FloorRead::NoMaterial);
    assert_eq!(g.column_span(10.25, 6.25), ColumnSpan::Empty);
    assert_eq!(g.floor_at(10.25, 3.75), FloorRead::Floor(-6.0), "beside the wall");
    assert_eq!(g.column_span(10.25, 3.75), ColumnSpan::Material { top: 0.0, floor: -6.0 });
    assert_eq!(g.floor_at(10.25, 0.75), FloorRead::Floor(-6.0));
}

#[test]
fn a_column_reads_at_its_cell_centre_near_a_cut_wall() {
    // Review of #649: a slot through the plate, r 2.2 at y 6 (y 3.8..8.2),
    // its walls off the cell boundaries. A column answers at its cell's
    // centre - the field's own law (`distance`, `column_top`): every
    // column whose centre is inside the slot is empty, and a column whose
    // centre stands outside is whole even where the query point itself is
    // a hair inside the cut (y 3.81, centre 3.75) - material over-reported
    // at the lattice's resolution, the blessed direction, never air where
    // steel stands.
    let g = slot(plate(), -7.0, 2.2, None);
    for y in [4.01, 4.25, 6.0, 7.75, 7.99] {
        assert_eq!(g.floor_at(10.25, y), FloorRead::NoMaterial, "y {y}: its centre is inside the slot");
    }
    for y in [3.75, 3.81, 3.99, 8.1, 8.19, 8.25] {
        // The shell crossing of an off-centre ray is exact to rounding.
        match g.column_span(10.25, y) {
            ColumnSpan::Material { top, floor } => {
                assert!(top.abs() <= 1e-9 && (floor + 6.0).abs() <= 1e-9, "y {y}: top {top} floor {floor}");
            }
            other => panic!("y {y}: its centre is outside, read {other:?}"),
        }
    }
    assert_eq!(g.floor_at(10.25, 0.75), FloorRead::Floor(-6.0));
}

#[test]
fn a_sliver_on_the_floor_is_read_from_the_record() {
    // The slot's floor at -5.8 leaves 0.2 of stock under it: thinner than
    // the 0.5 cell and holding no cell centre (the floor cell's centre,
    // -5.75, is inside the tool). The down-march finds no material; the
    // cell below the floor plane holds the ray's entry into the swept
    // solid, closed-form under a flat floor, so the sliver reads exactly.
    let g = cut(plate(), -5.8);
    match g.column_span(10.25, 6.25) {
        ColumnSpan::Material { top, floor } => {
            assert!((top + 5.8).abs() <= 1e-12, "top {top}");
            assert_eq!(floor, -6.0);
        }
        other => panic!("the sliver: {other:?}"),
    }
    assert_eq!(g.floor_at(10.25, 6.25), FloorRead::Floor(-6.0));
    // A tool bottom exactly on the floor leaves a zero-height column -
    // the column body's own `top < floor` law, same strictness.
    let h = cut(plate(), -6.0);
    assert_eq!(h.column_span(10.25, 6.25), ColumnSpan::Material { top: -6.0, floor: -6.0 });
    // A hair below it is cut through.
    let i = cut(plate(), -6.0 - 1e-9);
    assert_eq!(i.column_span(10.25, 6.25), ColumnSpan::Empty);
}

#[test]
fn a_bounded_tool_keeps_the_refusal_by_name() {
    // A truncated taper does not span the world above its tip, so the
    // monotone law does not hold: a floor its cut came within a cell of
    // is refused, as step B refused every cut. Far from it, exact.
    let g = slot(plate(), -7.0, 2.0, Some(1.0));
    assert_eq!(g.floor_at(10.25, 6.25), FloorRead::CutFromBelow);
    assert_eq!(g.column_span(10.25, 6.25), ColumnSpan::CutFromBelow);
    assert_eq!(g.floor_at(10.25, 0.75), FloorRead::Floor(-6.0));
    // A taper cut that stays well above the floor refuses nothing.
    let h = slot(plate(), -2.0, 2.0, Some(1.0));
    assert_eq!(h.floor_at(10.25, 6.25), FloorRead::Floor(-6.0));
}
