//! #1595: the mark law, unit by unit - the budget's arithmetic, and the
//! solid law read against the part lowered by it. Our own fixtures.

use super::*;
use crate::cl::{Move, MoveKind};
use crate::geometry::cube;
use crate::tools::ToolGeom;

fn p(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

/// A 6 mm flat end mill, 20 mm of flute.
fn cutter() -> verify::Cutter {
    verify::Cutter::flat(&ToolGeom {
        radius: 3.0,
        nose_radius: 0.0,
        flute_len: 20.0,
        shank_radius: 3.0,
        shank_len: 30.0,
        holder_radius: 15.0,
        holder_len: 40.0,
    })
}

/// A 40 x 40 x 10 block with a 10 x 40 x 5 boss standing on it along
/// x 30..40.
fn stepped() -> Vec<Triangle> {
    let mut f = cube(p(0.0, 0.0, 0.0), p(40.0, 40.0, 10.0)).facets.to_vec();
    f.extend(cube(p(30.0, 0.0, 10.0), p(40.0, 40.0, 15.0)).facets.iter().copied());
    f
}

/// One feed along y at `x`, the tip at `z`.
fn line(x: f64, z: f64) -> Toolpath {
    Toolpath::new(vec![Move::new(MoveKind::Feed, p(x, 10.0, z), p(x, 30.0, z), Some(500.0))])
}

fn hits(path: &Toolpath, budget: f64) -> Vec<Warning> {
    let mut w = Vec::new();
    mark_into(&mut w, path, &stepped(), &cutter(), budget, 0.0);
    w
}

#[test]
fn the_budget_is_the_lesser_of_the_stated_depth_and_the_cutter_radius() {
    assert_eq!(budget(8.0, 7.7, 3.0), 8.0 - 7.7);
    assert_eq!(budget(8.0, 1.0, 3.0), 3.0, "never deeper than half the kerf is wide");
    assert_eq!(budget(8.0, 5.0, 3.0), 3.0);
    assert_eq!(budget(7.0, 8.0, 3.0), 0.0, "a Bottom over the Top states no depth");
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(budget(bad, 1.0, 3.0), 0.0);
        assert_eq!(budget(8.0, bad, 3.0), 0.0);
        assert_eq!(budget(8.0, 1.0, bad), 0.0);
    }
    assert_eq!(budget(8.0, 1.0, -3.0), 0.0);
}

#[test]
fn only_engrave_and_trace_are_mark_kinds() {
    use crate::operation::{seed_params, OpType};
    for t in [OpType::Engrave, OpType::Trace] {
        assert!(is_mark_kind(&seed_params(t.ordinal())), "{t:?}");
    }
    for t in [OpType::Chamfer2D, OpType::Pocket, OpType::Contour, OpType::Slot, OpType::Circular, OpType::Drill] {
        assert!(!is_mark_kind(&seed_params(t.ordinal())), "{t:?}");
    }
}

#[test]
fn a_cut_within_the_budget_under_the_surface_is_a_mark() {
    assert_eq!(hits(&line(15.0, 9.7), 0.3), Vec::new());
    assert_eq!(hits(&line(15.0, 7.0), 3.0), Vec::new());
}

#[test]
fn a_cut_past_the_budget_is_a_gouge_on_its_move() {
    assert_eq!(hits(&line(15.0, 9.6), 0.3), vec![Warning::Gouge { at: 0 }]);
    assert_eq!(hits(&line(15.0, 9.7), 0.0), vec![Warning::Gouge { at: 0 }], "no budget, no entry");
    assert_eq!(hits(&line(15.0, 9.7), f64::NAN), vec![Warning::Gouge { at: 0 }], "an unread budget is none");
}

#[test]
fn a_wall_beside_the_line_is_the_part_not_the_mark() {
    // 0.3 mm into the low face, the centre on the boss's foot (x 30):
    // the flank stands 3 mm inside a 5 mm wall.
    assert_eq!(hits(&line(30.0, 9.7), 0.3), vec![Warning::Gouge { at: 0 }]);
    // A step no higher than the budget is within the mark.
    assert_eq!(hits(&line(30.0, 9.7), 5.3), Vec::new());
    // Clear of the wall by the radius: the same cut is a mark.
    assert_eq!(hits(&line(26.9, 9.7), 0.3), Vec::new());
}

#[test]
fn a_conviction_another_law_already_made_is_not_repeated() {
    let mut w = vec![Warning::Gouge { at: 0 }];
    assert_eq!(mark_into(&mut w, &line(15.0, 9.0), &stepped(), &cutter(), 0.3, 0.0), 0);
    assert_eq!(w, vec![Warning::Gouge { at: 0 }]);
}

#[test]
fn the_note_names_the_budget_in_the_documents_unit() {
    let n = note("Engrave", 3.0, Unit::default());
    assert!(n.starts_with("Engrave cuts deeper than a mark") && n.contains("3.000 mm") && n.contains("model the groove"), "{n}");
}
