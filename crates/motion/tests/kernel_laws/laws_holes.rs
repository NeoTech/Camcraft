//! #4154: the readings of the OLD planners of Circular, Bore and Thread on
//! the disc (a hole of radius 12 at the disc). Circular and Bore have a
//! start side (Inside and Outside); Thread rides a pre-drilled hole.

use super::common::law_measure::{EntryKind, Spec};
use super::fixture::{R, STEP, TOP, Z, disc, pin, planned_deg, rect, region, stock};
use camcraft_motion::cl::ThreadHand;
use camcraft_motion::entry::Entry3D;
use camcraft_motion::geo2d::Point2;
use camcraft_motion::links::Links;
use camcraft_motion::machining2d::{
    BoreHole, BoreParams, CircularParams, PassStart, ThreadHole, ThreadParams, ThreadPasses,
    bore_started, circular, thread_mill,
};

const SIDES: [(&str, PassStart); 2] = [("Inside", PassStart::Inside), ("Outside", PassStart::Outside)];
const HOLE_R: f64 = 12.0;

/// The spec of a hole in solid stock: the centre stays in the hole.
fn hole_spec(limit: f64) -> Spec {
    let mut s = Spec::new(R, stock(), TOP, limit);
    s.reach = Some(disc().offset(-R).expect("reach"));
    s.asked_entry = Some(EntryKind::Helix);
    s
}

#[test]
fn circular_readings_of_the_old_planner() {
    const TABLE: &[super::fixture::Row] = &[
        ("disc Inside",
         "lifts 3 named 0 re 3 | cross 0 leave 0 | entry air 0 hole 0 col 0 solid 4 (plunge 0 ramp 0 helix 4 rapid 0) | over 8 worst 180 full 1 | fallback 0",
         "link L6"),
        ("disc Outside",
         "lifts 3 named 0 re 3 | cross 0 leave 0 | entry air 0 hole 0 col 1 solid 3 (plunge 0 ramp 0 helix 3 rapid 0) | over 3 worst 180 full 3 | fallback 0",
         "link L6"),
    ];
    let square = region(vec![rect(-12.0, -12.0, 12.0, 12.0)]);
    let mut cases = Vec::new();
    for (sname, start) in SIDES {
        let p = CircularParams {
            pass_start: start,
            links: Links::PLANAR,
            center: Point2::new(0.0, 0.0),
            radius: HOLE_R,
            vault: false,
            bottom_z: Z,
            top_z: TOP,
            tool_diameter: 2.0 * R,
            stock_to_leave: 0.0,
            stepover: STEP,
            stepdown: 2.0,
            clearance_z: 10.0,
            retract_z: Some(2.0),
            feed: 1000.0,
            plunge_feed: 300.0,
            climb: true,
            entry: Entry3D::Helix,
            helix_radius: 1.5,
        };
        let path = circular(&square, &p).unwrap_or_else(|e| panic!("circular {sname}: {e}"));
        cases.push((format!("disc {sname}"), path, hole_spec(planned_deg(STEP, R))));
    }
    pin("Circular", false, TABLE, cases);
}

#[test]
fn bore_readings_of_the_old_planner() {
    const TABLE: &[super::fixture::Row] = &[
        ("disc Inside",
         "lifts 1 named 0 re 1 | cross 0 leave 0 | entry air 0 hole 0 col 1 solid 1 (plunge 0 ramp 0 helix 1 rapid 0) | over 0 worst 180 full 1 | fallback 0",
         "link L6"),
        ("disc Outside",
         "lifts 1 named 0 re 1 | cross 0 leave 0 | entry air 0 hole 0 col 1 solid 1 (plunge 0 ramp 0 helix 1 rapid 0) | over 0 worst 180 full 1 | fallback 0",
         "link L6"),
    ];
    let mut cases = Vec::new();
    for (sname, start) in SIDES {
        let p = BoreParams {
            holes: vec![BoreHole {
                xy: Point2::new(0.0, 0.0),
                radius: HOLE_R,
                top_z: TOP,
                floor_z: Z,
                through: false,
                stepped: false,
            }],
            tool_diameter: 2.0 * R,
            corner_radius: 0.0,
            stock_to_leave: 0.0,
            stepdown: 2.0,
            through_allowance: 0.0,
            clearance_z: 10.0,
            retract_z: Some(2.0),
            feed: 1000.0,
            plunge_feed: 300.0,
            climb: true,
            tolerance: 0.01,
            entry: Entry3D::Helix,
            helix_radius: 1.5,
            links: Links::PLANAR,
        };
        let path = bore_started(&p, start).unwrap_or_else(|e| panic!("bore {sname}: {e}"));
        // Bore has no planned step: the limit is the full width.
        cases.push((format!("disc {sname}"), path, hole_spec(180.0)));
    }
    pin("Bore", false, TABLE, cases);
}

#[test]
fn thread_readings_of_the_old_planner() {
    const TABLE: &[super::fixture::Row] = &[
        ("disc hole",
         "lifts 0 named 0 re 0 | cross 0 leave 0 | entry air 0 hole 3 col 0 solid 0 (plunge 0 ramp 0 helix 0 rapid 0) | over 0 worst 0 full 0 | fallback 0",
         "-"),
    ];
    let p = ThreadParams {
        through_run: None,
        holes: vec![ThreadHole {
            xy: Point2::new(0.0, 0.0),
            radius: HOLE_R,
            top_z: TOP,
            floor_z: -6.0,
            stepped: false,
            through: false,
        }],
        pitch: 1.5,
        tool_diameter: 2.0 * R,
        rows: 1,
        infeed: 0.3,
        external: false,
        hand: ThreadHand::Right,
        climb: true,
        passes: ThreadPasses::default(),
        clearance_z: 10.0,
        r_z: None,
        retract_z: None,
        feed: 400.0,
        plunge_feed: 120.0,
    };
    let path = thread_mill(&p).expect("thread");
    // The hole is drilled before: no stock in it. The thread form is not
    // a flat disc, so the engagement column is a reading only.
    let mut spec = Spec::new(R, stock(), TOP, 180.0);
    spec.holes = Some(disc());
    pin("Thread", true, TABLE, vec![("disc hole".into(), path, spec)]);
}
