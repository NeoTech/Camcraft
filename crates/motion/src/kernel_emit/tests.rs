//! Pins of the converter (#4269) on hand-made plans: each step kind gives
//! its moves, a lift gives a retract and a note with its reason, a step
//! that cannot be emitted is refused by name, and no cut is dropped.

use super::*;
use crate::cl::{Move, MoveKind};
use crate::geo2d::{Loop, Point2, Region};
use crate::geometry::Point3;
use camcraft_pathkernel::entry::{Area, Entry, EntryChoice, EntryKind, Exit, Proof};
use camcraft_pathkernel::guide::{Circle, Curve, Family};
use camcraft_pathkernel::link::{CUTS_PART, Hop};
use camcraft_pathkernel::plan::Cut;
use camcraft_pathkernel::{PlanePoint, ToolShape};

const LEVEL: f64 = -2.0;
const CLEAR: f64 = 10.0;

fn pp(x: f64, y: f64, d: f64) -> PlanePoint {
    PlanePoint::new(Point2::new(x, y), d).unwrap()
}

fn air() -> Area {
    let sq = vec![Point2::new(-50.0, -50.0), Point2::new(50.0, -50.0), Point2::new(50.0, 50.0), Point2::new(-50.0, 50.0)];
    Area::new(Region::new(vec![Loop::new(sq).unwrap()]).unwrap()).unwrap()
}

/// An air entry at `(x, y)`: the approach 3 above the level (1 above the
/// stock top at Z 0), straight down to the level.
fn entry_at(x: f64, y: f64, approach_depth: f64) -> Step {
    let choice = EntryChoice::new(EntryKind::Air, 1.0, vec![]).unwrap();
    let tool = ToolShape::disc(4.0, 0.0, true, 30.0).unwrap();
    let e = Entry::new(EntryKind::Air, vec![pp(x, y, approach_depth)], vec![pp(x, y, 0.0)], Proof::InAir(air()), &choice, tool);
    Step::Entry(e.unwrap())
}

fn line(a: (f64, f64), b: (f64, f64)) -> Step {
    let c = Curve::new(vec![pp(a.0, a.1, 0.0), pp(b.0, b.1, 0.0)], None, false, 0, Family::Row).unwrap();
    Step::Cut(Cut::new(c, None).unwrap())
}

fn ring(r: f64, n: u32, circle: bool) -> Step {
    let pts = (0..n)
        .map(|k| {
            let a = f64::from(k) * std::f64::consts::TAU / f64::from(n);
            pp(r * a.cos(), r * a.sin(), 0.0)
        })
        .collect();
    let arc = circle.then(|| Circle::new(Point2::new(0.0, 0.0), r).unwrap());
    Step::Cut(Cut::new(Curve::new(pts, arc, true, 0, Family::Ring).unwrap(), None).unwrap())
}

/// The exit of a plan on the level Z -2: straight up to the clearance
/// height (12 above the level).
fn exit() -> Step {
    exit_to(CLEAR - LEVEL)
}

/// An exit to `retract` above the level; the stock top (Z 0) is 2 above
/// the level Z -2.
fn exit_to(retract: f64) -> Step {
    Step::Exit(Exit::new(None, retract, -LEVEL).unwrap())
}

fn run(steps: Vec<Step>) -> Result<Emitted, EmitRefusal> {
    let plan = Plan::new(steps).unwrap();
    emit_plan(&plan, WorkPlane::setup_xy(LEVEL).unwrap(), EmitHeights::new(CLEAR, 0.0).unwrap(), EmitFeeds::new(1000.0, 300.0).unwrap())
}

/// The safety rules of the module doc, on each move of `path`.
fn assert_move_rules(moves: &[Move]) {
    for (i, m) in moves.iter().enumerate() {
        if m.kind != MoveKind::Rapid {
            continue;
        }
        let lateral = (m.to.x - m.from.x).hypot(m.to.y - m.from.y) > 1e-9;
        if lateral {
            assert!(m.from.z >= CLEAR - 1e-9 && m.to.z >= CLEAR - 1e-9, "move {i}: a lateral rapid under the clearance height: {m:?}");
        } else {
            assert!(m.to.z >= m.from.z, "move {i}: a rapid goes only up along the axis: {m:?}");
        }
    }
    for w in moves.windows(2) {
        assert_eq!(w[0].to, w[1].from, "the moves chain");
    }
}

#[test]
fn each_step_kind_gives_its_moves() {
    let steps = vec![
        entry_at(0.0, 0.0, 3.0),
        line((0.0, 0.0), (10.0, 0.0)),
        Step::Link(Hop::stay(vec![Point2::new(10.0, 0.0), Point2::new(10.0, 5.0)]).unwrap()),
        line((10.0, 5.0), (0.0, 5.0)),
        Step::Link(Hop::lift(CUTS_PART).unwrap()),
        entry_at(0.0, 10.0, 3.0),
        line((0.0, 10.0), (10.0, 10.0)),
        exit(),
    ];
    let out = run(steps).unwrap();
    let m = &out.path.moves;
    assert_move_rules(m);
    let kinds: Vec<MoveKind> = m.iter().map(|m| m.kind).collect();
    use MoveKind::{Feed, Plunge, Rapid};
    // Down to the approach and the level, the cut, the stay link, the cut,
    // the lift, the rapid over, down again, the cut, the exit's retract.
    assert_eq!(kinds, vec![Plunge, Plunge, Feed, Feed, Feed, Rapid, Rapid, Plunge, Plunge, Feed, Rapid]);
    assert_eq!(m[0].to, Point3::new(0.0, 0.0, 1.0), "the approach 3 above the level");
    assert_eq!(m[1].to.z, LEVEL, "the descent to the level");
    assert_eq!(m[3].to, Point3::new(10.0, 5.0, LEVEL), "the stay-down link stays at depth");
    assert_eq!(m[5].to, Point3::new(0.0, 5.0, CLEAR), "the lift is a retract to the clearance height");
    assert_eq!(m[10].to, Point3::new(10.0, 10.0, CLEAR), "the exit retracts to its height, the clearance height");
    assert_eq!((out.entries, out.stays, out.lifts, out.exits), (2, 1, 1, 1));
    assert_eq!(out.cut_moves, vec![2..3, 4..5, 9..10]);
    assert_eq!(m[2].feed, Some(1000.0));
    assert_eq!(m[1].feed, Some(300.0));
}

#[test]
fn a_lift_writes_its_reason_to_the_notes() {
    let steps = vec![
        entry_at(0.0, 0.0, 3.0),
        line((0.0, 0.0), (10.0, 0.0)),
        Step::Link(Hop::lift(CUTS_PART).unwrap()),
        entry_at(0.0, 5.0, 3.0),
        line((0.0, 5.0), (10.0, 5.0)),
        Step::Link(Hop::lift(CUTS_PART).unwrap()),
        entry_at(0.0, 10.0, 3.0),
        line((0.0, 10.0), (10.0, 10.0)),
        exit(),
    ];
    let out = run(steps).unwrap();
    assert_eq!(out.lifts, 2);
    assert_eq!(out.path.notes.len(), 1, "one note for each reason: {:?}", out.path.notes);
    let note = &out.path.notes[0];
    assert!(note.starts_with("link: the tool lifted to the clearance height 2 time(s) instead of the keep-down in-plane link - "), "{note}");
    assert!(note.contains(CUTS_PART), "{note}");
}

#[test]
fn an_exact_circle_becomes_arcs() {
    let out = run(vec![entry_at(5.0, 0.0, 3.0), ring(5.0, 64, true), exit()]).unwrap();
    let span = out.cut_moves[0].clone();
    let arcs = &out.path.moves[span];
    assert_eq!(arcs.len(), 4, "a full ring is four quarter arcs");
    assert!(arcs.iter().all(|m| m.arc.is_some_and(|a| a.ccw) && m.kind == MoveKind::Feed));
    assert!(arcs.last().is_some_and(|m| (m.to.x - 5.0).abs() < 1e-9 && m.to.y.abs() < 1e-9), "the ring closes on its start");
    // The same ring without the circle stays chords.
    let out = run(vec![entry_at(5.0, 0.0, 3.0), ring(5.0, 64, false), exit()]).unwrap();
    let chords = &out.path.moves[out.cut_moves[0].clone()];
    assert_eq!(chords.len(), 64);
    assert!(chords.iter().all(|m| m.arc.is_none()));
}

#[test]
fn levels_are_emitted_in_the_given_order() {
    // The caller plans each level: the approach of the lower level is 2
    // deeper in plan units, so that it stays 1 above the stock top.
    let upper = Plan::new(vec![entry_at(0.0, 0.0, 3.0), line((0.0, 0.0), (10.0, 0.0)), exit()]).unwrap();
    let lower = Plan::new(vec![entry_at(0.0, 0.0, 5.0), line((0.0, 0.0), (10.0, 0.0)), exit_to(CLEAR + 4.0)]).unwrap();
    let levels = [
        LevelPlan { plan: &upper, plane: WorkPlane::setup_xy(-2.0).unwrap() },
        LevelPlan { plan: &lower, plane: WorkPlane::setup_xy(-4.0).unwrap() },
    ];
    let out = emit_levels(&levels, EmitHeights::new(CLEAR, 0.0).unwrap(), EmitFeeds::new(1000.0, 300.0).unwrap()).unwrap();
    assert_move_rules(&out.path.moves);
    let zs: Vec<f64> = out.cut_moves.iter().map(|r| out.path.moves[r.start].to.z).collect();
    assert_eq!(zs, vec![-2.0, -4.0]);
    assert_eq!((out.entries, out.exits), (2, 2));
}

#[test]
fn nothing_is_dropped() {
    let steps = vec![
        entry_at(5.0, 0.0, 3.0),
        ring(5.0, 64, true),
        Step::Link(Hop::stay(vec![Point2::new(5.0, 0.0), Point2::new(8.0, 0.0)]).unwrap()),
        ring(8.0, 96, false),
        Step::Link(Hop::lift(CUTS_PART).unwrap()),
        entry_at(20.0, 0.0, 3.0),
        line((20.0, 0.0), (20.0, 0.5)),
        exit(),
    ];
    let cuts = steps.iter().filter(|s| matches!(s, Step::Cut(_))).count();
    let out = run(steps).unwrap();
    assert_eq!(out.cut_moves.len(), cuts, "one move group for each cut");
    assert!(out.cut_moves.iter().all(|r| !r.is_empty()));
    let cut_moves: usize = out.cut_moves.iter().map(|r| r.len()).sum();
    assert_eq!(cut_moves, 4 + 96 + 1);
}

#[test]
fn a_step_that_cannot_be_emitted_is_refused_by_name() {
    let reason = |r: Result<Emitted, EmitRefusal>| r.map(|_| ()).unwrap_err().reason();
    // A cut that does not start where the entry ended.
    assert_eq!(reason(run(vec![entry_at(0.0, 0.0, 3.0), line((1.0, 0.0), (10.0, 0.0)), exit()])), STEP_OFF_TOOL);
    // A stay link that does not start at the end of the cut.
    let off = Hop::stay(vec![Point2::new(9.0, 0.0), Point2::new(10.0, 5.0)]).unwrap();
    let steps = vec![entry_at(0.0, 0.0, 3.0), line((0.0, 0.0), (10.0, 0.0)), Step::Link(off), line((10.0, 5.0), (0.0, 5.0)), exit()];
    assert_eq!(reason(run(steps)), STEP_OFF_TOOL);
    // An approach under the stock top (Z 0 is 2 above the level).
    assert_eq!(reason(run(vec![entry_at(0.0, 0.0, 1.0), line((0.0, 0.0), (10.0, 0.0)), exit()])), APPROACH_IN_MATERIAL);
    // A circle cut whose points are not on its circle.
    let bad = Curve::new(vec![pp(5.0, 0.0, 0.0), pp(0.0, 5.0, 0.0), pp(-5.0, 0.0, 0.0)], Some(Circle::new(Point2::new(0.0, 0.0), 6.0).unwrap()), true, 0, Family::Ring);
    let steps = vec![entry_at(5.0, 0.0, 3.0), Step::Cut(Cut::new(bad.unwrap(), None).unwrap()), exit()];
    assert_eq!(reason(run(steps)), ARC_OFF_CIRCLE);
    // A circle cut that turns back.
    let zig = Curve::new(vec![pp(5.0, 0.0, 0.0), pp(0.0, 5.0, 0.0), pp(5.0, 0.0, 0.0)], Some(Circle::new(Point2::new(0.0, 0.0), 5.0).unwrap()), false, 0, Family::Ring);
    let steps = vec![entry_at(5.0, 0.0, 3.0), Step::Cut(Cut::new(zig.unwrap(), None).unwrap()), exit()];
    assert_eq!(reason(run(steps)), ARC_NOT_MONOTONE);
    // A cut with a tilted tool axis.
    let c = Curve::new(vec![pp(0.0, 0.0, 0.0), pp(10.0, 0.0, 0.0)], None, false, 0, Family::Row).unwrap();
    let tilted = Cut::new(c, Some([0.0, 0.6, 0.8])).unwrap();
    assert_eq!(reason(run(vec![entry_at(0.0, 0.0, 3.0), Step::Cut(tilted), exit()])), AXIS_NOT_NORMAL);
    // A plane that is not the setup's XY plane.
    let plan = Plan::new(vec![entry_at(0.0, 0.0, 3.0), line((0.0, 0.0), (10.0, 0.0)), exit()]).unwrap();
    let side = WorkPlane::planar([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]).unwrap();
    let r = emit_plan(&plan, side, EmitHeights::new(CLEAR, 0.0).unwrap(), EmitFeeds::new(1000.0, 300.0).unwrap());
    assert_eq!(reason(r), NOT_SETUP_XY);
    // The refusal names its level and step.
    let e = run(vec![entry_at(0.0, 0.0, 3.0), line((1.0, 0.0), (10.0, 0.0)), exit()]).map(|_| ()).unwrap_err();
    assert_eq!((e.level(), e.step()), (Some(0), Some(1)));
    assert!(e.to_string().contains("level 1, step 2"), "{e}");
    // Bad heights and feeds.
    assert_eq!(EmitHeights::new(-1.0, 0.0).unwrap_err().reason(), BAD_HEIGHTS);
    assert_eq!(EmitHeights::new(f64::NAN, 0.0).unwrap_err().reason(), BAD_HEIGHTS);
    assert_eq!(EmitFeeds::new(0.0, 300.0).unwrap_err().reason(), BAD_FEEDS);
    assert_eq!(EmitFeeds::new(1000.0, f64::INFINITY).unwrap_err().reason(), BAD_FEEDS);
}

#[test]
fn an_empty_plan_gives_an_empty_path() {
    let out = run(vec![]).unwrap();
    assert!(out.path.moves.is_empty() && out.cut_moves.is_empty());
}

/// #4407: the converter never raises a retract in silence. An exit under
/// the clearance height is refused by name; at it or above, the tool goes
/// to the exit's own height.
#[test]
fn an_exit_under_the_clearance_height_is_refused_by_name() {
    let low = run(vec![entry_at(0.0, 0.0, 3.0), line((0.0, 0.0), (10.0, 0.0)), exit_to(5.0)]).map(|_| ()).unwrap_err();
    assert_eq!((low.reason(), low.step()), (EXIT_UNDER_CLEARANCE, Some(2)));
    let high = run(vec![entry_at(0.0, 0.0, 3.0), line((0.0, 0.0), (10.0, 0.0)), exit_to(15.0)]).unwrap();
    assert_eq!(high.path.moves.last().map(|m| m.to.z), Some(13.0), "the exit goes to its own height");
}

/// #4407: an exit with no lead-out where one was asked writes its named
/// fallback to the notes.
#[test]
fn an_exit_fallback_goes_to_the_notes() {
    use camcraft_pathkernel::entry::NO_LEAD_OUT_ROOM;
    let x = Exit::new(None, CLEAR - LEVEL, -LEVEL).unwrap().without_lead_out(NO_LEAD_OUT_ROOM).unwrap();
    let out = run(vec![entry_at(0.0, 0.0, 3.0), line((0.0, 0.0), (10.0, 0.0)), Step::Exit(x)]).unwrap();
    assert_eq!(out.path.notes, vec![format!("exit: straight retract instead of the arc lead-out - {NO_LEAD_OUT_ROOM}")]);
    let plain = run(vec![entry_at(0.0, 0.0, 3.0), line((0.0, 0.0), (10.0, 0.0)), exit()]).unwrap();
    assert!(plain.path.notes.is_empty(), "{:?}", plain.path.notes);
}

/// #4407: a circle cut whose chords are all under the tolerance gives no
/// arc. It is refused as empty, not as a turn fault.
#[test]
fn a_circle_cut_with_no_chord_above_the_tolerance_is_refused_as_empty() {
    let r = 2e-6;
    let out = run(vec![entry_at(r, 0.0, 3.0), ring(r, 8, true), exit()]).map(|_| ()).unwrap_err();
    assert_eq!(out.reason(), ARC_EMPTY);
}
