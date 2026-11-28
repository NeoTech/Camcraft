//! #2830: the Helical Bore strategy. One continuous helix on the wall
//! radius from above the material to the floor, one flat floor circle
//! (two with the spring pass), a move to the centre, a retract - and no
//! axial plunge anywhere. A hole one helix cannot clear to the centre,
//! a helix steeper than the limit, a helix whose radius lies inside the
//! uncut core of a cutter that is not centre-cutting, and a pass order
//! other than Auto are refused by name.
//!
//! Provenance: our own tests over the kernel's helical law (helical
//! interpolation with an end mill, RS-274 G2/G3 with Z); the numbers are
//! the issue's counterbore (Ø11 x 7 deep, Ø6 flat end mill, 1 mm per
//! turn). No vendor material consulted.
//!
//! Gate map: covers camcraft_motion::machining2d (bore, bore_helix).

use std::f64::consts::PI;

use camcraft_motion::cl::{Move, MoveKind, Toolpath};
use camcraft_motion::entry::Entry3D;
use camcraft_motion::geo2d::Point2;
use camcraft_motion::links::Links;
use camcraft_motion::machining2d::{
    BoreHole, BoreParams, BoreStrategy, MAX_HELIX_ANGLE_DEG, PassStart, bore, bore_started, helical_entry_refusal, helical_pass_start_refusal,
    helix_leaves_core, max_helix_bore_diameter, min_helix_bore_diameter_no_center_cut,
};

const FEED: f64 = 600.0;
const TOP: f64 = 0.0;
const FLOOR: f64 = -7.0;
/// The approach's default plunge height: the helix starts this far over
/// the material.
const AIR: f64 = 2.0;

fn hole(diameter: f64, through: bool) -> BoreHole {
    BoreHole { xy: Point2::new(10.0, 5.0), radius: diameter * 0.5, top_z: TOP, floor_z: FLOOR, through, stepped: false }
}

/// The issue's case: a Ø11 counterbore, 7 deep, a Ø6 flat end mill,
/// 1 mm per turn, climb.
fn params(h: BoreHole, spring_pass: bool) -> BoreParams {
    BoreParams {
        holes: vec![h],
        tool_diameter: 6.0,
        corner_radius: 0.0,
        stock_to_leave: 0.0,
        stepdown: 1.0,
        through_allowance: 1.0,
        clearance_z: 10.0,
        retract_z: Some(5.0),
        feed: FEED,
        plunge_feed: 200.0,
        climb: true,
        tolerance: 0.01,
        entry: Entry3D::Plunge,
        helix_radius: 1.0,
        strategy: BoreStrategy::Helical { spring_pass },
        links: Links::PLANAR,
    }
}

fn path(p: &BoreParams) -> Toolpath {
    match bore(p) {
        Ok(t) => t,
        Err(e) => panic!("a legal helical bore: {e}"),
    }
}

fn refusal(p: &BoreParams) -> String {
    match bore(p) {
        Ok(_) => panic!("the bore must be refused"),
        Err(e) => e.to_string(),
    }
}

fn xy_dist(m: &Move, c: Point2) -> (f64, f64) {
    ((m.from.x - c.x).hypot(m.from.y - c.y), (m.to.x - c.x).hypot(m.to.y - c.y))
}

/// The signed sweep of an arc move about its centre, radians (CCW positive).
fn sweep(m: &Move) -> f64 {
    let Some(a) = m.arc else { return 0.0 };
    let t0 = (m.from.y - a.center.y).atan2(m.from.x - a.center.x);
    let t1 = (m.to.y - a.center.y).atan2(m.to.x - a.center.x);
    let mut d = t1 - t0;
    if a.ccw {
        while d <= 0.0 {
            d += 2.0 * PI;
        }
    } else {
        while d >= 0.0 {
            d -= 2.0 * PI;
        }
    }
    d
}

#[test]
fn the_counterbore_is_one_helix_on_the_wall_then_a_floor_circle_and_never_a_plunge() {
    let h = hole(11.0, false);
    let t = path(&params(h, false));
    let rc = 5.5 - 3.0;

    // No axial feed into material anywhere: every feed move that goes
    // down also moves in XY.
    for m in &t.moves {
        let down = m.to.z < m.from.z - 1e-9;
        let xy = (m.to.x - m.from.x).hypot(m.to.y - m.from.y);
        assert!(!(down && m.kind != MoveKind::Rapid && xy < 1e-6), "an axial plunge: {m:?}");
        assert_ne!(m.kind, MoveKind::Plunge, "the helical bore posts no plunge: {m:?}");
        // Rapids never go under the material top.
        if m.kind == MoveKind::Rapid {
            assert!(m.to.z >= TOP + AIR - 1e-9, "a rapid under the plunge height: {m:?}");
        }
    }

    // The helix: arcs on the wall radius about the hole, descending
    // monotonically from the plunge height to the floor.
    let helix: Vec<&Move> = t.moves.iter().filter(|m| m.arc.is_some() && m.to.z < m.from.z).collect();
    let Some(first) = helix.first() else { panic!("no helix in {:?}", t.moves) };
    assert!((first.from.z - (TOP + AIR)).abs() < 1e-6, "the helix starts in the air: {first:?}");
    let mut turned = 0.0;
    for m in &helix {
        let (r0, r1) = xy_dist(m, h.xy);
        assert!((r0 - rc).abs() < 2e-3 && (r1 - rc).abs() < 2e-3, "off the wall radius: {m:?}");
        let s = sweep(m);
        assert!(s > 0.0 && s <= PI + 1e-6, "climb arcs are CCW, at most half a turn: {s}");
        turned += s;
    }
    let Some(last) = helix.last() else { unreachable!() };
    assert!((last.to.z - FLOOR).abs() < 1e-9, "the helix ends on the floor: {last:?}");
    let turns = turned / (2.0 * PI);
    assert!((turns - (TOP + AIR - FLOOR)).abs() < 1e-3, "1 mm per turn over 9 mm: {turns} turns");

    // One flat circle at the floor, then a feed to the centre, then up.
    let floor: Vec<&Move> = t.moves.iter().filter(|m| m.arc.is_some() && m.from.z == FLOOR && m.to.z == FLOOR).collect();
    let around: f64 = floor.iter().map(|m| sweep(m)).sum();
    assert!((around - 2.0 * PI).abs() < 1e-6, "one full floor circle: {around}");
    let n = t.moves.len();
    let out = &t.moves[n - 2];
    assert_eq!(out.kind, MoveKind::Feed);
    assert!((out.to.x - h.xy.x).abs() < 1e-9 && (out.to.y - h.xy.y).abs() < 1e-9 && out.to.z == FLOOR, "off the wall to the centre: {out:?}");
    let up = &t.moves[n - 1];
    assert!(up.kind == MoveKind::Rapid && up.to.z == 10.0, "retract to clearance: {up:?}");
}

#[test]
fn the_spring_pass_is_a_second_floor_circle() {
    let floor_sweep = |t: &Toolpath| -> f64 {
        t.moves.iter().filter(|m| m.arc.is_some() && m.from.z == FLOOR && m.to.z == FLOOR).map(sweep).sum()
    };
    let one = floor_sweep(&path(&params(hole(11.0, false), false)));
    let two = floor_sweep(&path(&params(hole(11.0, false), true)));
    assert!((one - 2.0 * PI).abs() < 1e-6 && (two - 4.0 * PI).abs() < 1e-6, "{one} then {two}");
}

#[test]
fn conventional_milling_turns_the_helix_clockwise() {
    let mut p = params(hole(11.0, false), false);
    p.climb = false;
    let t = path(&p);
    assert!(t.moves.iter().filter(|m| m.arc.is_some()).all(|m| sweep(m) < 0.0), "every arc is CW");
}

#[test]
fn a_through_hole_helix_runs_on_by_the_break_through() {
    let t = path(&params(hole(11.0, true), false));
    let low = t.moves.iter().map(|m| m.to.z).fold(f64::INFINITY, f64::min);
    assert!((low - (FLOOR - 1.0)).abs() < 1e-9, "the floor less the 1 mm allowance: {low}");
}

#[test]
fn a_hole_one_helix_cannot_clear_is_refused_with_the_widest_it_can() {
    // A Ø6 flat end mill clears 6 + 2 x 3 = 12 mm (twice its diameter)
    // with one helix.
    assert!((max_helix_bore_diameter(6.0, 0.0, 0.0, None) - 12.0).abs() < 1e-12);
    assert!(!helix_leaves_core(6.0, 6.0, 0.0, 0.0, None));
    assert!(helix_leaves_core(6.01, 6.0, 0.0, 0.0, None));
    path(&params(hole(12.0, false), false));
    // Ø14: the helix's inner edge stops 1 mm short of the centre.
    let why = refusal(&params(hole(14.0, false), false));
    assert!(why.contains("too wide for one helix") && why.contains("2.000 mm core") && why.contains("12.000 mm"), "{why}");
    assert!(why.contains("Rings strategy"), "{why}");
}

#[test]
fn a_helix_steeper_than_the_limit_is_refused_with_the_pitch_that_fits() {
    // Ø6.5 with Ø6: rc 0.25, one turn is 1.571 mm long - 1 mm per turn
    // descends at 32.5 deg.
    let why = refusal(&params(hole(6.5, false), false));
    let fit = 2.0 * PI * 0.25 * MAX_HELIX_ANGLE_DEG.to_radians().tan();
    assert!(why.contains("32.5 deg") && why.contains(&format!("{fit:.3} mm or less")), "{why}");
    let mut p = params(hole(6.5, false), false);
    p.stepdown = fit;
    path(&p);
}

#[test]
fn the_rings_strategy_is_the_default_and_keeps_its_centre_dive() {
    assert_eq!(BoreStrategy::default(), BoreStrategy::Rings);
    let mut p = params(hole(11.0, false), false);
    p.strategy = BoreStrategy::Rings;
    let t = path(&p);
    let axial = t.moves.iter().any(|m| m.kind != MoveKind::Rapid && m.to.z < m.from.z && m.to.x == m.from.x && m.to.y == m.from.y);
    assert!(axial, "the ladder still dives at the centre");
}

#[test]
fn a_helix_inside_the_uncut_core_of_a_non_centre_cutting_tool_is_refused() {
    // The review's case: Ø6.5 with a Ø6 tool and 0.4 mm per turn gives a
    // 14.3 deg helix of radius 0.25 mm. A centre-cutting tool may take
    // it; a tool that is not centre-cutting has a 1.5 mm uncut core
    // (plunge_law::CORE = half the radius), and that core runs into the
    // solid at the hole axis for the whole descent.
    let mut p = params(hole(6.5, false), false);
    p.stepdown = 0.4;
    path(&p);
    p.links.approach.center_cutting = false;
    let why = refusal(&p);
    assert!(
        why.contains("not centre-cutting") && why.contains("0.250 mm") && why.contains("1.500 mm uncut core") && why.contains("9.000 mm"),
        "{why}"
    );
    // The smallest hole that tool helixes: the wall circle 1.5 mm off the axis.
    assert!((min_helix_bore_diameter_no_center_cut(6.0, 0.0) - 9.0).abs() < 1e-12);
    let mut edge = params(hole(9.0, false), false);
    edge.links.approach.center_cutting = false;
    path(&edge);
    // The issue's counterbore keeps its helix with that tool (rc 2.5).
    let mut cb = params(hole(11.0, false), false);
    cb.links.approach.center_cutting = false;
    path(&cb);
}

#[test]
fn a_helical_bore_refuses_a_pass_order_by_name() {
    let p = params(hole(11.0, false), false);
    assert_eq!(helical_pass_start_refusal(p.strategy, PassStart::Auto), None);
    assert_eq!(helical_pass_start_refusal(BoreStrategy::Rings, PassStart::Outside), None);
    for start in [PassStart::Inside, PassStart::Outside] {
        let why = match bore_started(&p, start) {
            Ok(_) => panic!("{start:?}: a helix has no ring order"),
            Err(e) => e.to_string(),
        };
        assert!(why.contains("Helical strategy") && why.contains(start.label()), "{why}");
    }
    assert_eq!(bore_started(&p, PassStart::Auto).ok(), Some(path(&p)));
}

#[test]
fn a_helical_bore_refuses_a_ramp_or_helix_entry_by_name() {
    let p = params(hole(11.0, false), false);
    assert_eq!(helical_entry_refusal(p.strategy, Entry3D::Plunge), None, "Plunge is the default entry word");
    assert_eq!(helical_entry_refusal(BoreStrategy::Rings, Entry3D::Ramp), None, "the rings take the entry");
    for (entry, name) in [(Entry3D::Ramp, "Ramp"), (Entry3D::Helix, "Helix")] {
        let q = BoreParams { entry, ..p.clone() };
        let why = match bore(&q) {
            Ok(_) => panic!("{name}: the helix is the entry, so the word is refused"),
            Err(e) => e.to_string(),
        };
        assert!(why.contains("Helical strategy") && why.contains(name), "{why}");
    }
    assert_eq!(bore(&p).ok(), Some(path(&p)), "a Plunge entry keeps the helix");
}
