//! #3327: the 2D Adaptive's start and exit points on the HSM plan. A
//! rectangle 60 x 40, a tool of radius 5 and a helix of radius 2. The
//! start point is where the helix goes down, and the plan grows the
//! cleared area from there under the engagement law (the shape mask's
//! progressive rim meter). The exit point is where the last pass of each
//! level ends: it runs back along its own closing ride to the point
//! nearest the pick. A point that the plan cannot hold is refused by name.

use super::{rect, region};
use camcraft_motion::cl::{MoveKind, Toolpath};
use camcraft_motion::entry::Entry3D;
use camcraft_motion::geo2d::{Point2, Region};
use camcraft_motion::links::Links;
use camcraft_motion::load::DEFAULT_OPTIMAL_LOAD;
use camcraft_motion::machining2d::{PocketParams, PocketStrategy, pocket};
use camcraft_motion::verify::crossed_pairs;

use super::common::shape_mask::{CutDepth, MaskSpec, ShapeMask};

const D: f64 = 10.0;
const HELIX: f64 = 2.0;
const FLOOR: f64 = -2.5;

fn params(start: Option<Point2>, exit: Option<Point2>) -> PocketParams {
    let mut links = Links::PLANAR;
    links.approach.start_point = start;
    links.approach.exit_point = exit;
    PocketParams {
        tool_diameter: D,
        stepover: 4.0,
        stock_to_leave: 0.0,
        clearance_z: 5.0,
        retract_z: Some(1.0),
        top_z: 0.0,
        bottom_z: 2.0 * FLOOR,
        stepdown: -FLOOR,
        feed: 1000.0,
        plunge_feed: 300.0,
        climb: true,
        strategy: PocketStrategy::Adaptive,
        entry: Entry3D::Helix,
        helix_radius: HELIX,
        optimal_load: DEFAULT_OPTIMAL_LOAD,
        blend_turns: false,
        blend_radius: None,
        links,
    }
}

fn area() -> Region {
    region(vec![rect(0.0, 0.0, 60.0, 40.0)])
}

fn cut(start: Option<Point2>, exit: Option<Point2>) -> Result<Toolpath, String> {
    pocket(&area(), &params(start, exit)).map_err(|e| e.to_string())
}

fn dist(a: Point2, x: f64, y: f64) -> f64 {
    (a.x - x).hypot(a.y - y)
}

/// The XY of the first move of each level that cuts at the level's floor,
/// and of the last feed before each climb out of a level.
fn level_ends(t: &Toolpath) -> (Vec<Point2>, Vec<Point2>) {
    let (mut firsts, mut lasts) = (Vec::new(), Vec::new());
    let mut level: Option<f64> = None;
    let mut last_feed: Option<Point2> = None;
    for m in &t.moves {
        let planar = (m.from.z - m.to.z).abs() < 1e-9 && m.to.z < -1e-6;
        if m.kind == MoveKind::Feed && planar && level.is_none_or(|z| (z - m.to.z).abs() > 1e-6) {
            if let Some(l) = last_feed.take() {
                lasts.push(l);
            }
            level = Some(m.to.z);
            firsts.push(Point2::new(m.from.x, m.from.y));
        }
        if m.kind == MoveKind::Feed && planar {
            last_feed = Some(Point2::new(m.to.x, m.to.y));
        }
    }
    lasts.extend(last_feed);
    (firsts, lasts)
}

/// The widest cut of `t` on the progressive rim meter, its entry bore at
/// `seed` (the helix's own disc, radius `r + helix`) taken as cleared
/// before the cut, as the plan counts it. No overcut past the area.
fn widest_cut(name: &str, t: &Toolpath, seed: Point2) -> f64 {
    let bore = Region::sweep(&[vec![seed]], 0.5 * D + HELIX).expect("bore");
    let mask = ShapeMask::evaluate_with_precleared(
        &MaskSpec { name, expected: &area(), path: t, tool_diameter: D, cell: 0.1, depth: CutDepth::NearZ { z: FLOOR, tol: 1e-3 }, diff_dir: None },
        &bore,
        2.0 * FLOOR,
    )
    .expect("mask");
    mask.assert_no_overcut();
    let m = mask.metrics();
    m.max_cut_width
}

#[test]
fn the_start_point_is_where_the_helix_goes_down_on_each_level() {
    let q = Point2::new(15.0, 12.0);
    let t = cut(Some(q), None).expect("a start point in the area is taken");
    let (firsts, _) = level_ends(&t);
    assert_eq!(firsts.len(), 2, "two levels");
    for f in &firsts {
        assert!(dist(*f, q.x, q.y) <= HELIX + 1e-6, "each level starts on the helix about the start point: {f:?}");
    }
    assert!(!t.notes.iter().any(|n| n.contains("exceeds the")), "no planned load over the optimal load: {:?}", t.notes);
    assert!(crossed_pairs(&t, 1e-3).is_empty(), "the plan crosses no feed of its own");
    // The automatic seed reads 1.62 mm here, the picked one 1.79: both
    // under the optimal load's step with the plan's margin and the grid's.
    let widest = widest_cut("adaptive_start_point", &t, q);
    assert!(widest <= D * DEFAULT_OPTIMAL_LOAD * 1.25 + 0.2, "no cut over the engagement limit: {widest:.3} mm");
}

#[test]
fn the_exit_point_is_where_the_last_pass_ends_on_each_level() {
    let q = Point2::new(59.0, 20.0);
    let t = cut(None, Some(q)).expect("an exit point beside a wall is taken");
    let (_, lasts) = level_ends(&t);
    assert_eq!(lasts.len(), 2, "two levels");
    for l in &lasts {
        // The closing ride runs 5 mm off the wall at X 60.
        assert!(dist(*l, 55.0, 20.0) < 0.05, "each level ends on the ride point nearest the exit point: {l:?}");
    }
    assert!(crossed_pairs(&t, 1e-3).is_empty(), "the run back crosses no feed");
}

#[test]
fn a_point_the_plan_cannot_hold_is_refused_by_name() {
    let refused = |s: Option<Point2>, e: Option<Point2>, want: &[&str]| {
        let why = cut(s, e).expect_err("refused");
        for w in want {
            assert!(why.contains(w), "{s:?} {e:?}: says {w}: {why}");
        }
    };
    // Outside the area the tool centre can reach (2 mm from two walls).
    refused(Some(Point2::new(2.0, 2.0)), None, &["start point", "outside the area"]);
    // Inside it, but the helix has no room (0.5 mm from the centre limit).
    refused(Some(Point2::new(5.5, 20.0)), None, &["helix", "no room", "start point"]);
    // An exit in the middle, far from the closing lap.
    refused(None, Some(Point2::new(30.0, 20.0)), &["exit point", "closing ride"]);
    // A start from which the spiral cannot hold the optimal load.
    refused(Some(Point2::new(40.0, 25.0)), None, &["start point", "optimal load"]);
    // A ramp entry chooses its own place.
    let mut p = params(Some(Point2::new(15.0, 12.0)), None);
    p.entry = Entry3D::Ramp;
    let why = pocket(&area(), &p).map(|_| ()).expect_err("a start point with a ramp entry");
    assert!(why.to_string().contains("Helix or Plunge"), "{why}");
}
