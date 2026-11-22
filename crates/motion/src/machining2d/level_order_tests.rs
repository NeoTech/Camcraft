//! Unit pins of the Outside later-entry law of `level_order` (#3567): a
//! later pass whose start has no cut floor in reach is reported as a cold
//! entry (the plan then falls back by name); a pass one stepover from the
//! cut before it lands on that cut. Ours.

use super::*;
use crate::geo2d::Loop;

fn square(h: f64) -> Region {
    let pts = vec![Point2::new(-h, -h), Point2::new(h, -h), Point2::new(h, h), Point2::new(-h, h)];
    let Ok(l) = Loop::new(pts) else { panic!("fixture loop") };
    let Ok(r) = Region::new(vec![l]) else { panic!("fixture region") };
    r
}

fn piece(level: u32, y: f64) -> Piece {
    Piece { level, pts: vec![Point2::new(-20.0, y), Point2::new(20.0, y)], closed: false, core: false }
}

fn plan(gap: f64, start: PassStart) -> LevelPlan {
    let side = StartSide::from(start);
    match order(vec![piece(0, 30.0), piece(1, 30.0 - gap)], &square(40.0), 4.0, &side) {
        Ok(p) => p,
        Err(e) => panic!("{e:?}"),
    }
}

#[test]
fn an_outside_pass_one_stepover_away_lands_on_the_cut_before_it() {
    let p = plan(4.0, PassStart::Outside);
    assert_eq!(p.cold_entry, None);
    let Some(second) = p.passes.get(1) else { panic!("two passes: {:?}", p.passes) };
    let Some(&q) = second.first() else { panic!("an empty pass") };
    assert!((q.y - 30.0).abs() < 0.01, "the lead starts on the cut at Y 30, not at {q:?}");
}

#[test]
fn an_outside_pass_with_no_cut_floor_in_reach_is_a_cold_entry() {
    let p = plan(10.0, PassStart::Outside);
    assert_eq!(p.cold_entry, Some(Point2::new(-20.0, 20.0)));
    // Inside keeps its law of #3353: no cold-entry judgement.
    assert_eq!(plan(10.0, PassStart::Inside).cold_entry, None);
}
