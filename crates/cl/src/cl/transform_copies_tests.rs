//! The copy join's laws (#3012): each copy is the translated path, the
//! links are rapids over the link plane, the index records follow their
//! copy, and what a join cannot carry is refused by name.

use super::*;
use crate::cl::transform::tests::{contour, p};
use crate::cl::{Cycle, CycleKind, MoveKind, PathBuilder};
use crate::geo2d::Point2;

#[test]
fn two_copies_are_the_path_and_its_shift_linked_at_the_plane() {
    let tp = contour();
    let n = tp.moves.len();
    let out = copies(&tp, &[[0.0; 3], [40.0, 0.0, 0.0]], 25.0).expect("joins");
    // The contour starts and ends at (-20, -20, 20): up to 25, across, down.
    assert_eq!(out.moves.len(), 2 * n + 3);
    assert!(out.is_chained(), "the joined path is one chain");
    assert_eq!(&out.moves[..n], &tp.moves[..]);
    let link = &out.moves[n..n + 3];
    assert!(link.iter().all(|m| m.kind == MoveKind::Rapid && m.feed.is_none()));
    assert_eq!(link[0].to, p(-20.0, -20.0, 25.0));
    assert_eq!(link[1].to, p(20.0, -20.0, 25.0));
    assert_eq!(link[2].to, p(20.0, -20.0, 20.0));
    let shifted = translate(&tp, [40.0, 0.0, 0.0]).expect("shifts");
    assert_eq!(&out.moves[n + 3..], &shifted.moves[..]);
    // The compensated span of copy 2 indexes copy 2's moves.
    assert_eq!(out.comps.len(), 2);
    assert_eq!(out.comps[1].first, tp.comps[0].first + n + 3);
    assert_eq!(out.comps[1].last, tp.comps[0].last + n + 3);
    assert_eq!(out.comps[1].programmed, shifted.comps[0].programmed);
}

#[test]
fn a_low_link_plane_never_dips_below_the_path_ends() {
    let tp = contour();
    let n = tp.moves.len();
    let out = copies(&tp, &[[0.0; 3], [0.0, 30.0, 0.0]], -50.0).expect("joins");
    // End and start are both at Z 20, so the link stays at Z 20: one move.
    assert_eq!(out.moves.len(), 2 * n + 1);
    assert_eq!(out.moves[n].from, p(-20.0, -20.0, 20.0));
    assert_eq!(out.moves[n].to, p(-20.0, 10.0, 20.0));
}

#[test]
fn one_copy_is_the_translation() {
    let tp = contour();
    let out = copies(&tp, &[[5.0, 6.0, 0.0]], 25.0).expect("joins");
    assert_eq!(out, translate(&tp, [5.0, 6.0, 0.0]).expect("shifts"));
}

#[test]
fn cycle_anchors_follow_their_copy() {
    let mut b = PathBuilder::new(p(0.0, 0.0, 10.0));
    b.push(MoveKind::Rapid, p(5.0, 5.0, 10.0), None);
    b.push(MoveKind::Rapid, p(5.0, 5.0, 2.0), None);
    b.push(MoveKind::Feed, p(5.0, 5.0, -8.0), Some(100.0));
    b.push(MoveKind::Rapid, p(5.0, 5.0, 10.0), None);
    let mut tp = b.finish();
    tp.cycles.push(Cycle {
        kind: CycleKind::Drill,
        xy: Point2::new(5.0, 5.0),
        r_z: 2.0,
        bottom_z: -8.0,
        entry_z: 0.0,
        peck: 0.0,
        dwell: 0.0,
        first_move: 1,
    });
    let n = tp.moves.len();
    let out = copies(&tp, &[[0.0; 3], [20.0, 0.0, 0.0], [40.0, 0.0, 0.0]], 10.0).expect("joins");
    assert_eq!(out.cycles.len(), 3);
    // Each copy ends at (x, 5, 10) and starts at (x', 0, 10): one cross.
    assert_eq!(out.cycles[1].first_move, 1 + n + 1);
    assert_eq!(out.cycles[2].first_move, 1 + 2 * (n + 1));
    assert_eq!(out.cycles[2].xy, Point2::new(45.0, 5.0));
    assert!(out.is_chained());
}

#[test]
fn bad_inputs_are_refused_by_name() {
    let tp = contour();
    assert_eq!(copies(&tp, &[], 25.0), Err(TransformError::BadInput("the copy count")));
    let many = vec![[0.0; 3]; MAX_COPIES + 1];
    assert_eq!(copies(&tp, &many, 25.0), Err(TransformError::BadInput("the copy count")));
    assert_eq!(copies(&tp, &[[0.0; 3]], f64::NAN), Err(TransformError::BadInput("the link plane")));
    assert_eq!(
        copies(&tp, &[[0.0; 3], [f64::INFINITY, 0.0, 0.0]], 25.0),
        Err(TransformError::BadInput("the translation"))
    );
}

#[test]
fn a_tilted_move_is_refused() {
    let mut tp = contour();
    tp.moves[4] = tp.moves[4].with_axis([0.0, 0.6, 0.8]);
    assert_eq!(copies(&tp, &[[0.0; 3], [10.0, 0.0, 0.0]], 25.0), Err(TransformError::TiltedMove));
}
