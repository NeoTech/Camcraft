//! #1595: the operation owns its mark. An Engrave or Trace on the
//! finished part is judged against the model with the declared mark cut
//! (`stock_sim::credit_marks`): what the mark's own cutter swept, no
//! deeper than its budget under the model's top, is the finished part;
//! past the budget it is still a gouge charged at its depth under the
//! budget; a column the mark never swept, or one another operation cut
//! deeper, keeps its charge.
//!
//! Gate map (#1595): covers camcraft_sim::stock_sim (mark, edge_break's
//! credit_floors, deviate).

use camcraft_sim::cl::{Move, MoveKind};
use camcraft_sim::geometry::{cube, Point3, Triangle};
use camcraft_sim::stock_sim::{self, credit_marks, deviate, Dexel, Mark, SimOp, DEV_TOL};

fn p(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

fn feed(from: Point3, to: Point3) -> Move {
    Move { rotary: None, kind: MoveKind::Feed, from, to, feed: Some(500.0), arc: None, axis: None }
}

/// The finished part: a 20 x 20 x 10 block.
fn part() -> Vec<Triangle> {
    cube(p(0.0, 0.0, 0.0), p(20.0, 20.0, 10.0)).facets.to_vec()
}

/// One engraved line across the top face, y = `y`, the tip at `z`.
fn line(y: f64, z: f64) -> Vec<Move> {
    vec![feed(p(4.0, y, z), p(16.0, y, z))]
}

fn op(moves: &[Move], ball: bool) -> SimOp<'_> {
    SimOp { moves, radius: 1.0, ball, cone: None, corner: None }
}

/// The part as stock, cut by `ops` in order.
fn cut(ops: &[SimOp]) -> Dexel {
    let mut d = Dexel::from_facets(&part(), 0.125).expect("the finished part as stock");
    stock_sim::simulate(&mut d, ops).expect("the program cuts");
    d
}

#[test]
fn a_mark_within_its_budget_is_the_finished_part_not_a_gouge() {
    for ball in [false, true] {
        let moves = line(10.0, 9.7);
        let d = cut(&[op(&moves, ball)]);
        let facets = part();
        let mut dev = deviate(&d, &facets, DEV_TOL).expect("verdict");
        assert!(dev.gouge_cells > 0, "the sharp model convicts every engraving - the bug's own reading");
        let credited = credit_marks(&mut dev, &d, &facets, &[Mark { op: op(&moves, ball), budget: 0.3 }]);
        assert!(credited > 0);
        assert_eq!(dev.gouge_cells, 0, "ball {ball}: worst {} at {:?}", dev.max_gouge, dev.max_gouge_at);
        assert!(dev.is_clean());
    }
}

#[test]
fn a_mark_past_its_budget_is_still_a_gouge_at_its_excess() {
    // Cut 1.0 deep on a 0.3 budget: 0.7 under the declared floor.
    let moves = line(10.0, 9.0);
    let d = cut(&[op(&moves, false)]);
    let facets = part();
    let mut dev = deviate(&d, &facets, DEV_TOL).expect("verdict");
    assert!((dev.max_gouge - 1.0).abs() < 1e-9);
    credit_marks(&mut dev, &d, &facets, &[Mark { op: op(&moves, false), budget: 0.3 }]);
    assert!(dev.gouge_cells > 0, "an over-deep mark passed");
    assert!((dev.max_gouge - 0.7).abs() < 1e-9, "charged {}", dev.max_gouge);
    assert!(dev.max_gouge_at.is_some(), "the worst gouge still locates itself");
}

#[test]
fn a_column_the_mark_never_swept_keeps_the_models_verdict() {
    // Two lines cut alike; only the first is a declared mark.
    let (marked, other) = (line(6.0, 9.7), line(14.0, 9.7));
    let d = cut(&[op(&marked, false), op(&other, false)]);
    let facets = part();
    let mut dev = deviate(&d, &facets, DEV_TOL).expect("verdict");
    let before = dev.gouge_cells;
    let credited = credit_marks(&mut dev, &d, &facets, &[Mark { op: op(&marked, false), budget: 0.3 }]);
    assert!(credited > 0 && dev.gouge_cells > 0 && dev.gouge_cells < before, "{} of {before}", dev.gouge_cells);
    let at = dev.max_gouge_at.expect("located");
    assert!((at.y - 14.0).abs() <= 1.0 + 0.125, "the charge left stands on the undeclared line: {at:?}");
    assert!((dev.max_gouge - 0.3).abs() < 1e-9);
}

#[test]
fn another_operations_cut_under_the_mark_is_charged_under_the_marks_floor() {
    // The mark is 0.3 deep; another op then cuts the same line to 0.8.
    let (mark, deep) = (line(10.0, 9.7), line(10.0, 9.2));
    let d = cut(&[op(&mark, false), op(&deep, false)]);
    let facets = part();
    let mut dev = deviate(&d, &facets, DEV_TOL).expect("verdict");
    credit_marks(&mut dev, &d, &facets, &[Mark { op: op(&mark, false), budget: 0.3 }]);
    assert!(dev.gouge_cells > 0, "a gouge under a mark was forgiven");
    assert!((dev.max_gouge - 0.5).abs() < 1e-9, "charged {} under the mark's own floor", dev.max_gouge);
}

#[test]
fn a_budget_wider_than_the_cut_credits_only_what_the_cutter_swept() {
    // A 3 mm budget over a 0.3 mm ball mark: the floor is the ball's own
    // groove, so another op's flat cut to the same depth beside the
    // centre line (deeper than the ball reached there) is still charged.
    let mark = line(10.0, 9.7);
    let flat = line(10.0, 9.7);
    let d = cut(&[op(&mark, true), op(&flat, false)]);
    let facets = part();
    let mut dev = deviate(&d, &facets, DEV_TOL).expect("verdict");
    let before = dev.gouge_cells;
    credit_marks(&mut dev, &d, &facets, &[Mark { op: op(&mark, true), budget: 3.0 }]);
    assert!(dev.gouge_cells > 0, "the flat cutter's corners beside the ball's groove were forgiven");
    assert!(dev.gouge_cells < before, "the ball's own groove is credited: {} of {before}", dev.gouge_cells);
    // The ball (radius 1, 0.3 deep) leaves the surface 0.714 mm off the
    // line; the worst charge left stands outside that, at the flat's edge.
    let at = dev.max_gouge_at.expect("located");
    assert!((at.y - 10.0).abs() > 0.7, "charged beside the ball's groove: {at:?}");
    assert!((dev.max_gouge - 0.3).abs() < 1e-9, "at the model's own depth there: {}", dev.max_gouge);
}

#[test]
fn a_mark_with_no_budget_declares_nothing() {
    let moves = line(10.0, 9.7);
    let d = cut(&[op(&moves, false)]);
    let facets = part();
    for budget in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        let mut dev = deviate(&d, &facets, DEV_TOL).expect("verdict");
        let before = (dev.gouge_cells, dev.max_gouge);
        assert_eq!(credit_marks(&mut dev, &d, &facets, &[Mark { op: op(&moves, false), budget }]), 0, "{budget}");
        assert_eq!((dev.gouge_cells, dev.max_gouge), before, "{budget}");
    }
}
