//! A user-picked start point on a closed 2D contour (#1448): the ring
//! begins at the pick's projection on every level, the lead-in runs onto
//! that point, the automatic seam is unchanged without a pick, and every
//! case the first slice does not certify is refused by name.
//!
//! Gate map (M4.4): covers camcraft_motion::machining2d (contour,
//! contour_start, contour_chain), camcraft_motion::approach.

use camcraft_motion::cl::{MoveKind, Toolpath};
use camcraft_motion::entry::Entry3D;
use camcraft_motion::geo2d::{Loop, Point2, Region};
use camcraft_motion::links::{Approach, Compensation, Links};
use camcraft_motion::machining2d::{contour, contour_along_chain, ContourParams, Lead, Side};

fn square() -> Region {
    let pts = [(0.0, 0.0), (40.0, 0.0), (40.0, 40.0), (0.0, 40.0)];
    Region::new(vec![Loop::new(pts.iter().map(|&(x, y)| Point2::new(x, y)).collect()).unwrap()]).unwrap()
}

fn params(pick: Option<Point2>) -> ContourParams {
    ContourParams {
        tool_diameter: 6.0,
        stock_to_leave: 0.0,
        clearance_z: 5.0,
        retract_z: 2.0,
        top_z: 0.0,
        bottom_z: -4.0,
        stepdown: 2.0,
        feed: 500.0,
        plunge_feed: 100.0,
        climb: true,
        side: Side::Outside,
        lead: Lead::Perpendicular { length: 5.0 },
        tolerance: 0.01,
        tab_count: 0,
        tab_width: 0.0,
        tab_height: 0.0,
        entry: Entry3D::Plunge,
        helix_radius: 1.0,
        blend_turns: false,
        links: Links { approach: Approach { start_point: pick, ..Approach::DEFAULT }, ..Links::PLANAR },
    }
}

/// Distance from `p` to the 40 x 40 square (outside it).
fn to_square(p: Point2) -> f64 {
    let ox = if p.x < 0.0 { -p.x } else if p.x > 40.0 { p.x - 40.0 } else { 0.0 };
    let oy = if p.y < 0.0 { -p.y } else if p.y > 40.0 { p.y - 40.0 } else { 0.0 };
    ox.hypot(oy)
}

/// Per cut level, where the cutter first reaches the cutter-centre ring
/// (one radius off the square): the end of the lead-in.
fn ring_starts(t: &Toolpath) -> Vec<Point2> {
    let mut levels: Vec<f64> = Vec::new();
    let mut out = Vec::new();
    for m in &t.moves {
        if m.kind != MoveKind::Feed || m.from.z != m.to.z || levels.contains(&m.to.z) {
            continue;
        }
        let q = Point2::new(m.to.x, m.to.y);
        if (to_square(q) - 3.0).abs() < 1e-6 {
            levels.push(m.to.z);
            out.push(q);
        }
    }
    out
}

fn refusal(p: &ContourParams) -> String {
    match contour(&square(), p) {
        Ok(_) => panic!("expected a refusal"),
        Err(e) => e.to_string(),
    }
}

#[test]
fn a_picked_start_begins_every_level_at_its_projection() {
    // A click 1 mm outside the cutter path on the right wall.
    let t = contour(&square(), &params(Some(Point2::new(44.0, 20.0)))).unwrap();
    let starts = ring_starts(&t);
    assert!(starts.len() >= 2, "a multi-level ladder: {starts:?} {:?}", t.notes);
    for s in &starts {
        assert!((s.x - 43.0).abs() < 1e-9 && (s.y - 20.0).abs() < 1e-9, "starts at the pick's foot: {s:?}");
    }
    // The perpendicular lead-in comes off the air side, square to the wall.
    let plunge = t.moves.iter().find(|m| m.kind == MoveKind::Plunge).unwrap();
    assert!((plunge.to.y - 20.0).abs() < 1e-9 && plunge.to.x > 43.0, "the dive sits on the lead: {:?}", plunge.to);
}

#[test]
fn no_pick_keeps_the_automatic_seam() {
    let auto = contour(&square(), &params(None)).unwrap();
    let base = contour(&square(), &ContourParams { links: Links::PLANAR, ..params(None) }).unwrap();
    assert_eq!(auto.moves, base.moves, "an unset start point is the default path, move for move");
    let picked = contour(&square(), &params(Some(Point2::new(44.0, 20.0)))).unwrap();
    assert_ne!(ring_starts(&auto)[0], ring_starts(&picked)[0], "the pick moved the start");
}

#[test]
fn a_pick_far_from_every_wall_is_refused_by_name() {
    let why = refusal(&params(Some(Point2::new(80.0, 20.0))));
    assert!(why.contains("start point (80.000, 20.000)") && why.contains("farther than"), "{why}");
}

#[test]
fn a_non_finite_pick_is_refused_before_any_move() {
    let why = refusal(&params(Some(Point2::new(f64::NAN, 0.0))));
    assert!(why.contains("start point"), "{why}");
}

#[test]
fn tabs_and_control_compensation_with_a_pick_are_refused_by_name() {
    let pick = Some(Point2::new(44.0, 20.0));
    let tabs = refusal(&ContourParams { tab_count: 2, tab_width: 4.0, tab_height: 1.0, ..params(pick) });
    assert!(tabs.contains("with tabs"), "{tabs}");
    let mut p = params(pick);
    p.links.approach.compensation = Compensation::Control;
    let comp = refusal(&p);
    assert!(comp.contains("compensation"), "{comp}");
}

#[test]
fn an_open_chain_with_a_pick_is_refused_by_name() {
    let run = [Point2::new(0.0, 0.0), Point2::new(20.0, 0.0)];
    let why = match contour_along_chain(&run, &params(Some(Point2::new(10.0, 0.0)))) {
        Ok(_) => panic!("expected a refusal"),
        Err(e) => e.to_string(),
    };
    assert!(why.contains("open chain"), "{why}");
}
