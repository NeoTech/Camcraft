//! A wide tool's short move cuts on the field engine as it does on the
//! column engine. The field walk's per-move cell ceiling bounded the
//! whole clipped window, so a tool whose own footprint already passed
//! it was refused on a 1 mm move, and "segment it" could not help.
//!
//! A move longer than one field update is cut in pieces by the simulator
//! itself (#4179): through `Stock` the field engine cuts what the column
//! engine cuts, and `SdfField::subtract_batch` keeps the refusal.
//!
//! Gate map (#4079): covers sim::stock_sim::field_cut (SdfField::subtract_batch's
//! ceiling). Provenance: our own fixtures; synthetic sizes.

use camcraft_sim::cl::{ArcMove, Move, MoveKind};
use camcraft_sim::geo2d::Point2;
use camcraft_sim::geometry::Point3;
use camcraft_sim::stock_sim::{Backend, SdfField, SdfTool, SimOp, Stock, FIELD_BAND};
use camcraft_tools::undercut::{Head, Undercut};

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

/// The 120 mm chord of a 50.8 mm flat tool across a 120 x 120 x 30 mm
/// block at z 25: past the ceiling as one update.
fn long_chord() -> Move {
    Move::new(MoveKind::Feed, Point3::new(0.0, 60.0, 25.0), Point3::new(120.0, 60.0, 25.0), Some(500.0))
}

/// The faced top is air, and stock stays below the face and beside the pass.
fn faced_by_the_long_chord(f: &SdfField) {
    assert!(f.distance(Point3::new(100.0, 60.0, 27.5)) > 0.0, "the faced top is air");
    assert!(f.distance(Point3::new(100.0, 60.0, 22.5)) < 0.0, "below the face is stock");
    assert!(f.distance(Point3::new(100.0, 100.0, 27.5)) < 0.0, "beside the pass is stock");
}

/// The ceiling still bounds one field update: the long chord refuses
/// through `subtract_batch`, and `subtract_batch_fitted` cuts it in
/// pieces.
#[test]
fn a_wide_tools_long_chord_still_refuses_and_its_segments_cut() {
    let field = || {
        SdfField::from_box(Point3::new(0.0, 0.0, 0.0), Point3::new(120.0, 120.0, 30.0), CELL, FIELD_BAND)
            .unwrap_or_else(|e| panic!("block: {e}"))
    };
    let tool = SdfTool::Flat { r: 25.4, flank: 40.0 };
    match field().subtract_batch(&[long_chord()], &tool, 0.01) {
        Ok(()) => panic!("a 120 mm chord of a 50.8 mm tool must still refuse one update"),
        Err(e) => assert!(e.to_string().contains("segment it"), "{e}"),
    }
    let mut f = field();
    f.subtract_batch_fitted(&[long_chord()], &tool, 0.01).unwrap_or_else(|e| panic!("the fitted chord must cut: {e}"));
    faced_by_the_long_chord(&f);
}

/// `Stock`'s single-move field arms, `subtract_flat` and `cut_chord`,
/// cut the long chord in pieces too, as their column arms do.
#[test]
fn the_single_move_stock_arms_cut_the_long_chord() {
    let block = || {
        Stock::block(Backend::Field { band: FIELD_BAND }, 0.0, 0.0, 120.0, 120.0, 30.0, CELL)
            .unwrap_or_else(|e| panic!("block: {e}"))
    };
    let mut s = block();
    s.subtract_flat(&long_chord(), 25.4, 0.01).unwrap_or_else(|e| panic!("subtract_flat refused the long chord: {e}"));
    faced_by_the_long_chord(s.field().expect("a field"));
    let mut s = block();
    let m = long_chord();
    s.cut_chord(m.from, m.to, 25.4).unwrap_or_else(|e| panic!("cut_chord refused the long chord: {e}"));
    faced_by_the_long_chord(s.field().expect("a field"));
}

/// One feed move cut through `Stock::simulate` with a flat tool of `r`.
fn one_move(b: Backend, size: [f64; 3], r: f64, from: Point3, to: Point3) -> Result<Stock, String> {
    let mut s = Stock::block(b, 0.0, 0.0, size[0], size[1], size[2], CELL).map_err(|e| format!("block: {e}"))?;
    let moves = [Move::new(MoveKind::Feed, from, to, Some(500.0))];
    s.simulate(&[SimOp { moves: &moves, radius: r, ball: false, cone: None, corner: None }])
        .map_err(|e| e.to_string())?;
    Ok(s)
}

/// #4179's steps: a 12.7 mm endmill's straight 300 mm pass, 5 mm deep in
/// a 30 mm block, cuts on both engines. On the field engine it refused
/// (the ceiling) from about 300 mm; 200 mm cut.
#[test]
fn an_endmills_300_mm_pass_cuts_on_both_engines() {
    let pass = |b| one_move(b, [340.0, 100.0, 30.0], 6.35, Point3::new(20.0, 50.0, 25.0), Point3::new(320.0, 50.0, 25.0));
    assert!(pass(Backend::Column).is_ok(), "the column engine cuts it");
    let s = pass(Backend::Field { band: FIELD_BAND }).unwrap_or_else(|e| panic!("the field engine refused the pass: {e}"));
    let f = s.field().expect("a field");
    assert!(f.distance(Point3::new(170.0, 50.0, 27.5)) > 0.0, "the slot is air");
    assert!(f.distance(Point3::new(170.0, 50.0, 22.5)) < 0.0, "below the slot is stock");
    assert!(f.distance(Point3::new(170.0, 60.0, 27.5)) < 0.0, "beside the slot is stock");
}

/// #4179's steps: a 127 mm flat tool plunging 6 mm into a 200 x 200 x 30
/// mm block cuts on both engines; the field engine refused from 6 mm.
#[test]
fn a_127_mm_tools_6_mm_plunge_cuts_on_both_engines() {
    let plunge = |b| one_move(b, [200.0, 200.0, 30.0], 63.5, Point3::new(100.0, 100.0, 30.0), Point3::new(100.0, 100.0, 24.0));
    assert!(plunge(Backend::Column).is_ok(), "the column engine cuts it");
    let s = plunge(Backend::Field { band: FIELD_BAND }).unwrap_or_else(|e| panic!("the field engine refused the plunge: {e}"));
    let f = s.field().expect("a field");
    assert!(f.distance(Point3::new(150.0, 100.0, 27.0)) > 0.0, "inside the tool is air");
    assert!(f.distance(Point3::new(100.0, 100.0, 22.0)) < 0.0, "below the plunge is stock");
    assert!(f.distance(Point3::new(100.0, 170.0, 27.0)) < 0.0, "outside the tool is stock");
}

/// #4179's steps: an undercut disc head 127 mm across (height 3, neck
/// 12.7 mm) plunging 6 mm through `simulate_undercut` cuts; it refused
/// from 6 mm.
#[test]
fn an_undercut_discs_6_mm_plunge_cuts_through_simulate_undercut() {
    let mut s = Stock::block(Backend::Field { band: FIELD_BAND }, 0.0, 0.0, 200.0, 200.0, 30.0, CELL)
        .unwrap_or_else(|e| panic!("block: {e}"));
    let disc = Undercut { r: 63.5, height: 3.0, neck_r: 6.35, head: Head::Disc };
    let moves = [Move::new(MoveKind::Feed, Point3::new(100.0, 100.0, 30.0), Point3::new(100.0, 100.0, 24.0), Some(500.0))];
    s.simulate_undercut(&moves, &disc, 30.0).unwrap_or_else(|e| panic!("the disc's plunge refused: {e}"));
    let f = s.field().expect("a field");
    assert!(f.distance(Point3::new(150.0, 100.0, 25.5)) > 0.0, "the head's path is air");
    assert!(f.distance(Point3::new(100.0, 100.0, 22.0)) < 0.0, "below the plunge is stock");
    assert!(f.distance(Point3::new(100.0, 170.0, 25.5)) < 0.0, "outside the head is stock");
}

/// #4179: an arc piece past the ceiling is cut in equal parts on its
/// circle (`ArcPiece::split`). A 127 mm flat tool's helical entry: half a
/// turn of radius 0.2 mm about (100, 100), 16 mm down from the top of a
/// 200 x 200 x 30 mm block. At the chord tolerance that is two quarter
/// turn pieces, each 8 mm deep - past the ceiling, as the 6 mm plunge is.
/// A helical piece is searched cell by cell (`field_arc`), so the test
/// takes about 22 min in debug and 5.5 min in release: heavy lane.
#[test]
#[ignore = "heavy lane (release, #4179): a helical piece past the ceiling, millions of searched cells - run with --release -- --ignored"]
fn a_127_mm_tools_helical_entry_cuts_on_both_engines() {
    let helix = |b| -> Result<Stock, String> {
        let mut s = Stock::block(b, 0.0, 0.0, 200.0, 200.0, 30.0, CELL).map_err(|e| format!("block: {e}"))?;
        let m = Move {
            rotary: None,
            kind: MoveKind::Feed,
            from: Point3::new(100.2, 100.0, 30.0),
            to: Point3::new(99.8, 100.0, 14.0),
            feed: Some(500.0),
            arc: Some(ArcMove::xy(Point2::new(100.0, 100.0), true)),
            axis: None,
        };
        s.simulate(&[SimOp { moves: &[m], radius: 63.5, ball: false, cone: None, corner: None }])
            .map_err(|e| e.to_string())?;
        Ok(s)
    };
    assert!(helix(Backend::Column).is_ok(), "the column engine cuts it");
    let s = helix(Backend::Field { band: FIELD_BAND }).unwrap_or_else(|e| panic!("the field engine refused the helix: {e}"));
    let f = s.field().expect("a field");
    assert!(f.distance(Point3::new(150.0, 100.0, 20.0)) > 0.0, "inside the tool is air");
    assert!(f.distance(Point3::new(100.0, 100.0, 12.0)) < 0.0, "below the entry is stock");
    assert!(f.distance(Point3::new(100.0, 170.0, 20.0)) < 0.0, "outside the tool is stock");
}
