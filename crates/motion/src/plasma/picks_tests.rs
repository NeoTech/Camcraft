//! The plasma profile's start and exit picks (#3334): the lead-in lands
//! on the start pick, the cut runs on to the exit pick, each other
//! contour keeps its automatic cut, and each pick that a law cannot hold
//! is refused by name.
//!
//! Provenance: our own tests over our own fixtures.

use super::picks::run_on;
use super::*;
use crate::cl::{ContourRole, Toolpath};
use crate::path_picks::PathPick;

fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Loop {
    Loop::new(vec![Point2::new(x0, y0), Point2::new(x1, y0), Point2::new(x1, y1), Point2::new(x0, y1)]).unwrap()
}

fn circle(cx: f64, cy: f64, r: f64) -> Loop {
    Loop::new((0..96).map(|k| {
        let t = std::f64::consts::TAU * f64::from(k) / 96.0;
        Point2::new(cx + r * t.cos(), cy + r * t.sin())
    }).collect()).unwrap()
}

fn params() -> PlasmaParams {
    PlasmaParams {
        kerf: 1.5,
        control_comp: false,
        reversed: false,
        lead_in: 4.0,
        lead_out: 3.0,
        small_hole_dia: SMALL_HOLE_DIA,
        small_hole_feed: SMALL_HOLE_FEED,
        overburn: 2.0,
        thickness: 6.0,
        feed: Some(2500.0),
        clearance_z: 10.0,
        cut_z: 0.0,
        pierce_height: None,
        cut_height: None,
        pierce_delay: None,
        model_excess: 0.0,
        file_tolerance: false,
    }
}

/// A 100 x 60 plate with a Ø40 hole at (30, 30) and a Ø10 hole at (75, 30).
fn plate() -> (Vec<Loop>, Region) {
    let loops = vec![rect(0.0, 0.0, 100.0, 60.0), circle(30.0, 30.0, 20.0), circle(75.0, 30.0, 5.0)];
    let part = Region::new(loops.clone()).unwrap();
    (loops, part)
}

fn picks(start: Option<(f64, f64)>, exit: Option<(f64, f64)>) -> PathPicks {
    let one = |p: Option<(f64, f64)>| p.map(|(x, y)| PathPick::xy(Point2::new(x, y))).into_iter().collect();
    PathPicks { start: one(start), exit: one(exit) }
}

fn cut(p: &PlasmaParams, k: &PathPicks) -> Result<Toolpath, String> {
    let (loops, part) = plate();
    plasma_profile_picked(&loops, &[], &part, p, k).map_err(|e| e.to_string())
}

/// The lit moves of the contour whose first lit move is nearest `near`
/// (the three contours of the plate stand far from each other).
fn span(path: &Toolpath, small: bool, role: ContourRole) -> (usize, usize) {
    let rec = path.plasma.as_ref().unwrap();
    let t = rec.torches.iter().find(|t| t.role == role && t.small.is_some() == small).unwrap();
    (t.on, t.off)
}

fn at(p: crate::geometry::Point3, x: f64, y: f64, tol: f64) -> bool {
    (p.x - x).hypot(p.y - y) < tol
}

fn stretch(path: &Toolpath, (on, off): (usize, usize)) -> Vec<(crate::geometry::Point3, crate::geometry::Point3)> {
    path.moves[on - 1..=off].iter().map(|m| (m.from, m.to)).collect()
}

#[test]
fn no_pick_is_the_automatic_path_move_for_move() {
    let (loops, part) = plate();
    let plain = plasma_profile(&loops, &[], &part, &params()).unwrap();
    let picked = plasma_profile_picked(&loops, &[], &part, &params(), &PathPicks::default()).unwrap();
    assert_eq!(plain.moves, picked.moves);
    assert_eq!(plain.notes, picked.notes);
    assert_eq!(plain.plasma, picked.plasma);
}

/// The outline is cut clockwise: along the bottom edge the torch runs in
/// -X, at y = -0.75.
#[test]
fn the_lead_in_lands_on_the_start_pick_and_the_other_contours_keep_their_cut() {
    let auto = cut(&params(), &PathPicks::default()).unwrap();
    let path = cut(&params(), &picks(Some((50.0, 0.0)), None)).unwrap();
    let (on, off) = span(&path, false, ContourRole::Outer);
    assert!(at(path.moves[on].to, 50.0, -0.75, 1e-6), "{:?}", path.moves[on].to);
    assert!(path.moves[on].from.y < -0.75 - 1.0, "the pierce is in the scrap: {:?}", path.moves[on].from);
    assert!(at(path.moves[off].from, 50.0, -0.75, 1e-6), "no exit pick: the lead-out leaves at the start");
    assert!(path.moves[on + 1].to.x < 50.0, "the cut direction is not changed");
    let (a_on, _) = span(&auto, false, ContourRole::Outer);
    assert!(!at(auto.moves[a_on].to, 50.0, -0.75, 1.0), "the automatic seam is elsewhere");
    for (small, role) in [(false, ContourRole::Inner), (true, ContourRole::Inner)] {
        assert_eq!(stretch(&auto, span(&auto, small, role)), stretch(&path, span(&path, small, role)));
    }
    assert!(path.notes.iter().any(|n| n.contains("starts at the start point (50.000, 0.000)")), "{:?}", path.notes);
    assert!(path.plasma.as_ref().unwrap().is_sane(&path.moves));
}

#[test]
fn the_cut_runs_on_to_the_exit_pick_and_the_lead_out_leaves_there() {
    let path = cut(&params(), &picks(Some((50.0, 0.0)), Some((40.0, 0.0)))).unwrap();
    let (on, off) = span(&path, false, ContourRole::Outer);
    assert!(at(path.moves[on].to, 50.0, -0.75, 1e-6));
    assert!(at(path.moves[off].from, 40.0, -0.75, 1e-6), "{:?}", path.moves[off].from);
    assert!(path.moves[off].to.y < -0.75 - 1.0, "the torch goes off in the scrap: {:?}", path.moves[off].to);
    let laps = path.moves[on..=off].iter().filter(|m| at(m.to, 50.0, -0.75, 1e-6)).count();
    assert_eq!(laps, 2, "the whole outline is cut before the run to the exit");
    assert!(path.notes.iter().any(|n| n.contains("runs on 10.000 mm past its start to the exit point (40.000, 0.000)")), "{:?}", path.notes);
    // An exit pick alone: the automatic start, and the same exit.
    let auto = cut(&params(), &PathPicks::default()).unwrap();
    let alone = cut(&params(), &picks(None, Some((40.0, 0.0)))).unwrap();
    let ((a_on, _), (e_on, e_off)) = (span(&auto, false, ContourRole::Outer), span(&alone, false, ContourRole::Outer));
    assert_eq!(auto.moves[a_on].to, alone.moves[e_on].to, "the start stays automatic");
    assert!(at(alone.moves[e_off].from, 40.0, -0.75, 1e-6), "{:?}", alone.moves[e_off].from);
    // An exit pick on the start adds no second lap.
    let same = cut(&params(), &picks(Some((50.0, 0.0)), Some((50.0, 0.0)))).unwrap();
    let start_only = cut(&params(), &picks(Some((50.0, 0.0)), None)).unwrap();
    assert_eq!(same.moves, start_only.moves);
}

/// The Ø40 hole is not small: it is cut counter-clockwise with arc leads.
#[test]
fn a_pick_on_a_hole_lands_on_the_hole_and_pierces_in_its_slug() {
    let path = cut(&params(), &picks(Some((50.0, 30.0)), Some((30.0, 50.0)))).unwrap();
    let (on, off) = span(&path, false, ContourRole::Inner);
    assert!(at(path.moves[on].to, 49.25, 30.0, 0.02), "{:?}", path.moves[on].to);
    let pierce = path.moves[on].from;
    assert!((pierce.x - 30.0).hypot(pierce.y - 30.0) < 19.25 - 1.0, "the pierce is in the slug: {pierce:?}");
    assert!(at(path.moves[off].from, 30.0, 49.25, 0.02), "the lead-out leaves at the exit pick: {:?}", path.moves[off].from);
    let off_at = path.moves[off].to;
    assert!((off_at.x - 30.0).hypot(off_at.y - 30.0) < 19.25 - 1.0, "the torch goes off in the slug: {off_at:?}");
}

/// The Ø10 hole takes the small-hole rule: the pierce at its centre, no
/// lead-out, and the overburn before the torch goes off.
#[test]
fn a_small_hole_keeps_its_rule_and_its_overburn() {
    let start = cut(&params(), &picks(Some((80.0, 30.0)), None)).unwrap();
    let (on, off) = span(&start, true, ContourRole::Inner);
    assert!(at(start.moves[on].from, 75.0, 30.0, 0.01), "the pierce stays at the centre: {:?}", start.moves[on].from);
    assert!(at(start.moves[on].to, 79.25, 30.0, 0.02), "{:?}", start.moves[on].to);
    let end = start.moves[off].to;
    assert!(end.y > 30.5 && (end.x - 75.0).hypot(end.y - 30.0) < 4.3, "the 2 mm overburn runs on past the start: {end:?}");
    // The exit a quarter turn on is past the overburn: the torch goes off there.
    let exit = cut(&params(), &picks(Some((80.0, 30.0)), Some((75.0, 35.0)))).unwrap();
    let (_, off) = span(&exit, true, ContourRole::Inner);
    assert!(at(exit.moves[off].to, 75.0, 34.25, 0.02), "{:?}", exit.moves[off].to);
    assert!(exit.notes.iter().any(|n| n.contains("with no lead-out")), "{:?}", exit.notes);
    // An exit inside the overburn is refused by name.
    let near = (75.0 + 5.0 * 0.2f64.cos(), 30.0 + 5.0 * 0.2f64.sin());
    let why = cut(&params(), &picks(Some((80.0, 30.0)), Some(near))).unwrap_err();
    assert!(why.contains("exit point") && why.contains("inside its 2.000 mm overburn"), "{why}");
}

/// A small U-shaped hole: its centre is in the base, and the straight
/// lead-in from the centre to the inner wall of an arm crosses the tongue
/// of the part between the arms.
#[test]
fn a_small_hole_lead_in_that_would_cross_the_part_is_refused_by_name() {
    let p = |x: f64, y: f64| Point2::new(x, y);
    let u = Loop::new(vec![p(0.0, 0.0), p(20.0, 0.0), p(20.0, 14.0), p(16.0, 14.0), p(16.0, 8.0), p(4.0, 8.0), p(4.0, 14.0), p(0.0, 14.0)]).unwrap();
    let loops = vec![rect(-20.0, -20.0, 40.0, 34.0), u];
    let part = Region::new(loops.clone()).unwrap();
    let cut = |k: &PathPicks| plasma_profile_picked(&loops, &[], &part, &params(), k).map_err(|e| e.to_string());
    assert!(cut(&PathPicks::default()).is_ok(), "the hole is cut without a pick");
    // The base wall is in sight of the centre: the pick is held.
    assert!(cut(&picks(Some((10.0, 0.0)), None)).is_ok(), "a pick with a clear lead-in");
    // The inner wall of the left arm is behind the tongue.
    let why = cut(&picks(Some((4.0, 13.0)), None)).unwrap_err();
    assert!(why.contains("start point (4.000, 13.000)") && why.contains("would cross the part"), "{why}");
}

#[test]
fn a_pick_that_no_law_can_hold_is_refused_by_name() {
    // Far from each closed profile.
    let why = cut(&params(), &picks(Some((50.0, -10.0)), None)).unwrap_err();
    assert!(why.contains("plasma profile: start point (50.000, -10.000) is 10.000 mm from the nearest closed profile"), "{why}");
    assert!(why.contains("farther than the 2.250 mm"), "{why}");
    let why = cut(&params(), &picks(None, Some((50.0, -10.0)))).unwrap_err();
    assert!(why.contains("exit point (50.000, -10.000)"), "{why}");
    // More than one pick of a role, and a pick that is not a number.
    let mut two = picks(Some((50.0, 0.0)), None);
    two.start.push(PathPick::xy(Point2::new(40.0, 0.0)));
    assert!(cut(&params(), &two).unwrap_err().contains("only one start point"));
    assert!(cut(&params(), &picks(Some((f64::NAN, 0.0)), None)).unwrap_err().contains("finite"));
    // An open run takes no pick.
    let run = vec![Point2::new(0.0, 0.0), Point2::new(20.0, 0.0)];
    let e = plasma_profile_picked(&[], &[run], &Region::empty(), &params(), &picks(Some((10.0, 0.0)), None)).unwrap_err().to_string();
    assert!(e.contains("start point (10.000, 0.000)") && e.contains("closed contour only"), "{e}");
}

#[test]
fn control_compensation_holds_the_start_pick_and_refuses_the_exit_pick() {
    let p = PlasmaParams { control_comp: true, lead_in: 2.0, lead_out: 2.0, ..params() };
    let path = cut(&p, &picks(Some((50.0, 0.0)), None)).unwrap();
    let (on, off) = span(&path, false, ContourRole::Outer);
    assert!(at(path.moves[on].to, 50.0, -0.75, 1e-6), "{:?}", path.moves[on].to);
    let comp = path.comps.iter().find(|c| c.first == on && c.last == off).expect("the outline's G41/G42 span");
    assert!(comp.is_sane(&path.moves));
    assert!(at(comp.programmed[0].to, 50.0, 0.0, 1e-6), "the programmed contour starts at the pick: {:?}", comp.programmed[0].to);
    let why = cut(&p, &picks(None, Some((40.0, 0.0)))).unwrap_err();
    assert!(why.contains("exit point (40.000, 0.000)") && why.contains(EXIT_WITH_CONTROL_COMP), "{why}");
}

#[test]
fn the_run_on_follows_the_walk_and_never_adds_a_lap() {
    let p = |x: f64, y: f64| Point2::new(x, y);
    let walk = [p(5.0, 0.0), p(10.0, 0.0), p(10.0, 10.0), p(0.0, 10.0), p(0.0, 0.0), p(5.0, 0.0)];
    assert!(run_on(&walk, p(5.0, 0.0)).is_empty(), "the exit at the start");
    assert_eq!(run_on(&walk, p(8.0, 0.0)), vec![p(8.0, 0.0)]);
    assert_eq!(run_on(&walk, p(10.0, 0.0)), vec![p(10.0, 0.0)]);
    assert_eq!(run_on(&walk, p(10.0, 4.0)), vec![p(10.0, 0.0), p(10.0, 4.0)]);
    assert_eq!(run_on(&walk, p(2.0, 0.0)), vec![p(10.0, 0.0), p(10.0, 10.0), p(0.0, 10.0), p(0.0, 0.0), p(2.0, 0.0)]);
    assert!(run_on(&[], p(1.0, 1.0)).is_empty() && run_on(&[p(0.0, 0.0)], p(1.0, 1.0)).is_empty());
}
