//! 2D Adaptive's trochoidal method, pinned (issue #1509; Gate map:
//! covers motion::machining2d::adaptive_troch, trochoid_bore): the path
//! stays in the region and under the engagement limit, the opened bore
//! leaves no post, and every refusal is by name.

use super::*;
use crate::entry::Entry3D;
use crate::geo2d::Loop;
use crate::links::Links;
use crate::quality::{QualitySpec, measure, rest_area, stock_for};

const D: f64 = 6.0;

fn poly(pts: &[(f64, f64)]) -> Loop {
    let Ok(l) = Loop::new(pts.iter().map(|&(x, y)| Point2::new(x, y)).collect()) else { panic!("loop") };
    l
}

fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Region {
    let Ok(r) = Region::new(vec![poly(&[(x0, y0), (x1, y0), (x1, y1), (x0, y1)])]) else { panic!("region") };
    r
}

/// A pocket face, 34 x 16 about the origin: a slim twin of the
/// maintainer's 50 x 25 (#1503; that one is the app suite's), its band
/// still wider than the tool.
fn face() -> Region {
    rect(-17.0, -8.0, 17.0, 8.0)
}

fn params() -> PocketParams {
    PocketParams {
        tool_diameter: D,
        stepover: 1.0,
        stock_to_leave: 0.0,
        clearance_z: 10.0,
        top_z: 0.0,
        bottom_z: -6.0,
        stepdown: 3.0,
        feed: 800.0,
        plunge_feed: 200.0,
        climb: true,
        strategy: PocketStrategy::Adaptive,
        retract_z: Some(2.0),
        entry: Entry3D::Helix,
        helix_radius: 1.0,
        optimal_load: crate::load::DEFAULT_OPTIMAL_LOAD,
        blend_turns: false,
        links: Links::PLANAR,
    }
}

fn refusal(region: &Region, p: &PocketParams) -> String {
    match adaptive_trochoidal(region, p) {
        Err(e) => e.to_string(),
        Ok(t) => panic!("not refused: {} moves, {:?}", t.moves.len(), t.notes),
    }
}

fn cut(region: &Region, p: &PocketParams) -> Toolpath {
    match adaptive_trochoidal(region, p) {
        Ok(t) => t,
        Err(e) => panic!("refused: {e}"),
    }
}

/// Every cutting move's cutter centre, sampled.
fn cut_points(t: &Toolpath) -> Vec<Point3> {
    t.moves.iter().filter(|m| m.kind != MoveKind::Rapid).flat_map(|m| m.segments3(0.01)).flat_map(|(a, b)| [a, b]).collect()
}

#[test]
fn the_pocket_face_is_cleared_inside_its_region_under_the_engagement_limit() {
    let t = cut(&face(), &params());
    assert!(t.has_arcs() && t.all_finite());
    // The cutter centre never leaves the face eroded by the tool radius
    // (read on the rectangle's own numbers, not through `confined`).
    let worst = cut_points(&t).iter().map(|q| (q.x.abs() - 14.0).max(q.y.abs() - 5.0)).fold(f64::MIN, f64::max);
    assert!(worst <= 2e-3, "the cutter centre leaves the region by {worst:.4} mm");
    assert!(cut_points(&t).iter().all(|q| q.z >= -6.0 - 1e-9), "never below the bottom");
    let spec = QualitySpec::new(D);
    let Ok(mut dex) = stock_for(&face(), 0.0, -6.0, D, &spec) else { panic!("stock") };
    let Ok(q) = measure(&t, &mut dex, &spec) else { panic!("measure") };
    assert!(q.max_deg <= DEFAULT_MAX_ENGAGEMENT, "engagement {:.1} deg at {:?}", q.max_deg, q.max_at);
    assert!(q.slot_len < 1e-6, "no slotting: {:.3} mm", q.slot_len);
    assert!(t.notes.iter().any(|n| n.starts_with(WHO) && n.contains("loops of radius")), "{:?}", t.notes);
    // The four corners are out of the loops' reach, and said.
    assert!(t.notes.iter().any(|n| n.contains("outside the loops' reach") && n.contains("mm2")), "{:?}", t.notes);
    assert!(crate::verify::crossed_pairs(&t, super::super::ARC_FIT_TOL).is_empty(), "no feed crosses another");
    eprintln!("trochoidal 34x16 D6: {:.1} deg, {} moves, {:?}", q.max_deg, t.moves.len(), t.notes);
}

/// A band wider than the tool: the plain bore (the stand-alone slot's)
/// leaves a post at the lane's start; the opened bore clears it.
#[test]
fn the_opened_bore_leaves_no_post_at_the_lane_start() {
    let p = PocketParams { bottom_z: -3.0, ..params() };
    let Ok(center) = face().offset(-0.5 * D) else { panic!("center") };
    let Ok((chain, b)) = lane(&face(), &center, &p) else { panic!("lane") };
    assert!(b > 4.5 && b < 5.0 && chain.len() == 2, "band {b}, {} points", chain.len());
    // Where the post would stand: a disc well inside the bore.
    let Ok(core) = Region::sweep(&[vec![chain[0]]], b - 0.5 * D - 0.5) else { panic!("core") };
    let spec = QualitySpec::new(D);
    let rest = |t: &Toolpath| {
        let Ok(mut dex) = stock_for(&face(), 0.0, -3.0, D, &spec) else { panic!("stock") };
        let Ok(_) = measure(t, &mut dex, &spec) else { panic!("measure") };
        let Ok(a) = rest_area(&dex, &core, -3.0) else { panic!("rest") };
        a
    };
    let t = match cut_lane(&center, &chain, b, &p) {
        Ok(t) => t,
        Err(e) => panic!("cut: {e}"),
    };
    assert!(rest(&t) < 0.5, "the opened bore leaves {:.2} mm2 standing", rest(&t));
    let tp = TrochoidParams {
        tool_diameter: D,
        width: 2.0 * (b + 0.5 * D),
        loop_width: None,
        step: load_step(D, p.optimal_load),
        max_engagement: DEFAULT_MAX_ENGAGEMENT,
        ret: TrochoidReturn::Microlift { height: DEFAULT_MICROLIFT },
        smoothing: 1.5,
        stock_to_leave: 0.0,
        clearance_z: 10.0,
        retract_z: Some(2.0),
        top_z: 0.0,
        bottom_z: -3.0,
        stepdown: 3.0,
        feed: 800.0,
        return_feed: 800.0,
        plunge_feed: 200.0,
        climb: true,
    };
    let Ok(plain) = plan_lane(&chain, &tp, false) else { panic!("plain") };
    assert!(rest(&plain.path) > 4.0, "the plain bore's post: {:.2} mm2", rest(&plain.path));
}

#[test]
fn an_empty_region_is_nothing_to_machine() {
    assert_eq!(refusal(&Region::empty(), &params()), "nothing to machine: empty pocket region");
}

#[test]
fn a_region_narrower_than_the_tool_is_refused_by_name() {
    let why = refusal(&rect(0.0, 0.0, 40.0, 5.0), &params());
    assert!(why.contains(WHO) && why.contains("narrower than the 6.000 mm tool"), "{why}");
}

#[test]
fn a_region_too_narrow_to_loop_in_is_refused_by_name_with_the_numbers() {
    let why = refusal(&rect(0.0, 0.0, 40.0, 6.1), &params());
    assert!(why.contains(WHO) && why.contains("too narrow for a 6.000 mm tool to loop in") && why.contains("6.260"), "{why}");
}

#[test]
fn a_ring_round_an_island_is_refused_by_name() {
    // A 10 mm band round a 60 x 50 part (corners rounded by the offset).
    let part = rect(0.0, 0.0, 60.0, 50.0);
    let (Ok(o), Ok(i)) = (part.offset(12.0), part.offset(2.0)) else { panic!("offsets") };
    let Ok(ring) = o.difference(&i) else { panic!("ring") };
    let why = refusal(&ring, &params());
    assert!(why.contains(WHO) && why.contains("closes round an island") && why.contains("use the Spiral method"), "{why}");
}

#[test]
fn a_region_with_no_single_medial_line_is_refused_by_name() {
    let Ok(isl) = Region::new(vec![
        poly(&[(0.0, 0.0), (66.0, 0.0), (66.0, 46.0), (0.0, 46.0)]),
        poly(&[(27.0, 17.0), (39.0, 17.0), (39.0, 29.0), (27.0, 29.0)]),
    ]) else {
        panic!("island")
    };
    let why = refusal(&isl, &params());
    assert!(why.contains(WHO) && why.contains("no single channel") && why.contains("use the Spiral method"), "{why}");
}

#[test]
fn a_lane_that_would_leave_most_of_the_region_standing_is_refused_by_name() {
    let Ok(tri) = Region::new(vec![poly(&[(0.0, 0.0), (60.0, 0.0), (0.0, 60.0)])]) else { panic!("triangle") };
    let why = refusal(&tri, &params());
    assert!(why.contains(WHO) && why.contains("% of the region standing (at most 25 %)"), "{why}");
}

#[test]
fn a_zero_or_oversized_load_is_refused_by_name_never_looped_forever() {
    for load in [0.0, 0.6, f64::NAN] {
        let why = refusal(&face(), &PocketParams { optimal_load: load, ..params() });
        assert!(why.contains("optimal load") || why.contains("optimal-load") || why.contains("finite"), "{load}: {why}");
    }
    let why = refusal(&face(), &PocketParams { stepdown: 0.0, ..params() });
    assert!(why.contains("stepdown"), "{why}");
}

/// The law behind the construction: a band wider than the room is
/// stopped by the check on the finished path, with its ghost.
#[test]
fn a_loop_outside_the_region_is_refused_by_the_check_on_the_path() {
    let Ok(center) = face().offset(-0.5 * D) else { panic!("center") };
    let chain = [Point2::new(-9.0, 0.0), Point2::new(9.0, 0.0)];
    let err = match cut_lane(&center, &chain, 6.5, &PocketParams { bottom_z: -3.0, ..params() }) {
        Err(e) => e,
        Ok(_) => panic!("a band 1.5 mm past the walls must not pass"),
    };
    assert!(err.to_string().contains("would leave the region - nothing is cut"), "{err}");
    assert!(err.location().is_some() && err.ghost().is_some(), "the refusal names its place and shows the path");
}

#[test]
fn a_drop_past_the_helix_turn_cap_is_refused_by_name() {
    let why = refusal(&face(), &PocketParams { bottom_z: -4000.0, stepdown: 4000.0, ..params() });
    assert!(why.contains(WHO) && why.contains("bore helix") && why.contains("half the tool radius"), "{why}");
}

#[test]
fn without_lift_room_the_returns_retract_and_a_plunge_entry_is_said_unread() {
    let p = PocketParams { clearance_z: 0.1, retract_z: None, bottom_z: -3.0, entry: Entry3D::Plunge, ..params() };
    let t = cut(&face(), &p);
    let rapids = t.moves.iter().filter(|m| m.kind == MoveKind::Rapid).count();
    assert!(rapids > 40, "retract returns: {rapids} rapids");
    assert!(cut_points(&t).iter().all(|q| q.z <= 0.1 + 1e-9), "no feed above the clearance plane");
    assert!(t.notes.iter().any(|n| n.contains("the Entry setting is not read")), "{:?}", t.notes);
    let helix = cut(&face(), &PocketParams { bottom_z: -3.0, ..params() });
    assert!(!helix.notes.iter().any(|n| n.contains("Entry setting")), "{:?}", helix.notes);
}
