//! A wide tool's short move cuts on the field engine as it does on the
//! column engine. The field walk's per-move cell ceiling bounded the
//! whole clipped window, so a tool whose own footprint already passed
//! it was refused on a 1 mm move, and "segment it" could not help.
//!
//! Gate map (#4079): covers sim::stock_sim::field_cut (SdfField::subtract_batch's
//! ceiling). Provenance: our own fixtures; synthetic sizes.

use camcraft_sim::cl::{Move, MoveKind};
use camcraft_sim::geometry::Point3;
use camcraft_sim::stock_sim::{Backend, SimOp, Stock, FIELD_BAND};

const CELL: f64 = 0.254;

/// A 120 x 120 x 30 mm block, faced 5 mm deep by a 1 mm move of a flat
/// tool 50.8 mm across: its footprint alone is about 201 x 201 cells,
/// and with the block's height in the window more than 1 << 22 cells.
fn face(b: Backend) -> Result<Stock, String> {
    let mut s = Stock::block(b, 0.0, 0.0, 120.0, 120.0, 30.0, CELL).map_err(|e| format!("block: {e}"))?;
    let p = |x: f64| Point3::new(x, 60.0, 25.0);
    let moves = [Move::new(MoveKind::Feed, p(60.0), p(61.0), Some(500.0))];
    s.simulate(&[SimOp { moves: &moves, radius: 25.4, ball: false, cone: None, corner: None }])
        .map_err(|e| e.to_string())?;
    Ok(s)
}

#[test]
fn a_wide_tools_short_move_cuts_on_the_field_engine_as_on_the_column_engine() {
    assert!(face(Backend::Column).is_ok(), "the column engine cuts it");
    let s = match face(Backend::Field { band: FIELD_BAND }) {
        Ok(s) => s,
        Err(e) => panic!("the field engine refused a 1 mm move: {e}"),
    };
    let f = s.field().expect("a field");
    assert!(f.distance(Point3::new(60.5, 60.0, 27.5)) > 0.0, "the faced top is air");
    assert!(f.distance(Point3::new(60.5, 60.0, 22.5)) < 0.0, "below the face is stock");
    assert!(f.distance(Point3::new(110.0, 110.0, 27.5)) < 0.0, "outside the tool is stock");
}

/// The ceiling still bounds a long chord, and segmenting is the answer
/// it names: the same wide tool across the whole block in one chord
/// refuses, and in 5 mm pieces it cuts.
#[test]
fn a_wide_tools_long_chord_still_refuses_and_its_segments_cut() {
    let cut = |pieces: usize| {
        let mut s = Stock::block(Backend::Field { band: FIELD_BAND }, 0.0, 0.0, 120.0, 120.0, 30.0, CELL)
            .unwrap_or_else(|e| panic!("block: {e}"));
        let x = |k: usize| 120.0 * k as f64 / pieces as f64;
        let moves: Vec<Move> = (0..pieces)
            .map(|k| Move::new(MoveKind::Feed, Point3::new(x(k), 60.0, 25.0), Point3::new(x(k + 1), 60.0, 25.0), Some(500.0)))
            .collect();
        s.simulate(&[SimOp { moves: &moves, radius: 25.4, ball: false, cone: None, corner: None }]).map(|()| s)
    };
    match cut(1) {
        Ok(_) => panic!("a 120 mm chord of a 50.8 mm tool must still refuse"),
        Err(e) => assert!(e.to_string().contains("segment it"), "{e}"),
    }
    let s = cut(24).unwrap_or_else(|e| panic!("5 mm pieces must cut: {e}"));
    let f = s.field().expect("a field");
    assert!(f.distance(Point3::new(100.0, 60.0, 27.5)) > 0.0, "the faced top is air");
}
