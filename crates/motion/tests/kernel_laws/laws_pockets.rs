//! #4154: the readings of the OLD planners of 2D Pocket, 2D Adaptive, 2D
//! Offset and 3D Pocket round the disc and the dog bone, Inside and
//! Outside. The tables pin today's readings; the third column names each
//! law that a reading breaks today (the kernel must improve it).

use super::fixture::{
    R, STEP, TOP, Z, band, parts, pin, planned_deg, pocket_params, prism, rect, region, side,
    spec_round,
};
use camcraft_motion::entry::Entry3D;
use camcraft_motion::links::Links;
use camcraft_motion::load::DEFAULT_OPTIMAL_LOAD;
use camcraft_motion::machining2d::{CornerRelief, PassStart, PocketStrategy, pocket_started_on};
use camcraft_motion::machining3d::{Pocket3DParams, pocket3d};

const SIDES: [(&str, PassStart); 2] = [("Inside", PassStart::Inside), ("Outside", PassStart::Outside)];

/// The 2D ring kinds round each part, Inside and Outside.
fn ring_cases(kind: &str, strategy: PocketStrategy) -> Vec<(String, camcraft_motion::cl::Toolpath, super::common::law_measure::Spec)> {
    let mut out = Vec::new();
    for (pname, part) in parts() {
        for (sname, start) in SIDES {
            let p = pocket_params(strategy);
            let path = pocket_started_on(&band(&part), &p, &[], &CornerRelief::default(), &side(start))
                .unwrap_or_else(|e| panic!("{kind} {pname} {sname}: {e}"));
            out.push((format!("{pname} {sname}"), path, spec_round(&part, planned_deg(STEP, R))));
        }
    }
    out
}

#[test]
fn pocket_2d_readings_of_the_old_planner() {
    const TABLE: &[super::fixture::Row] = &[
        ("disc Inside",
         "lifts 18 named 0 re 18 | cross 0 leave 0 | entry air 1 hole 0 col 0 solid 18 (plunge 9 ramp 0 helix 9 rapid 0) | over 20 worst 178 full 0 | fallback 0",
         "link L4 L5 L6"),
        ("disc Outside",
         "lifts 18 named 0 re 18 | cross 0 leave 0 | entry air 1 hole 0 col 0 solid 18 (plunge 9 ramp 0 helix 9 rapid 0) | over 20 worst 178 full 0 | fallback 0",
         "link L4 L5 L6"),
        ("dog bone Inside",
         "lifts 26 named 0 re 26 | cross 0 leave 0 | entry air 1 hole 0 col 0 solid 26 (plunge 13 ramp 0 helix 13 rapid 0) | over 32 worst 180 full 1 | fallback 0",
         "link L4 L5 L6"),
        ("dog bone Outside",
         "lifts 26 named 0 re 26 | cross 0 leave 0 | entry air 1 hole 0 col 0 solid 26 (plunge 13 ramp 0 helix 13 rapid 0) | over 32 worst 180 full 1 | fallback 0",
         "link L4 L5 L6"),
    ];
    pin("2D Pocket", false, TABLE, ring_cases("2D Pocket", PocketStrategy::Auto));
}

#[test]
fn adaptive_2d_readings_of_the_old_planner() {
    const TABLE: &[super::fixture::Row] = &[
        ("disc Inside",
         "lifts 0 named 0 re 0 | cross 0 leave 0 | entry air 0 hole 0 col 0 solid 1 (plunge 0 ramp 0 helix 1 rapid 0) | over 33 worst 180 full 1 | fallback 1",
         "L6"),
        ("disc Outside",
         "lifts 0 named 0 re 0 | cross 0 leave 0 | entry air 1 hole 0 col 0 solid 1 (plunge 0 ramp 0 helix 1 rapid 0) | over 11 worst 118 full 0 | fallback 1",
         "L6"),
        ("dog bone Inside",
         "lifts 0 named 0 re 0 | cross 0 leave 0 | entry air 0 hole 0 col 0 solid 1 (plunge 0 ramp 0 helix 1 rapid 0) | over 54 worst 180 full 1 | fallback 1",
         "L6"),
        ("dog bone Outside",
         "lifts 0 named 0 re 0 | cross 0 leave 0 | entry air 1 hole 0 col 0 solid 1 (plunge 0 ramp 0 helix 1 rapid 0) | over 39 worst 167 full 0 | fallback 1",
         "L6"),
    ];
    pin("2D Adaptive", false, TABLE, ring_cases("2D Adaptive", PocketStrategy::Adaptive));
}

#[test]
fn offset_2d_readings_of_the_old_planner() {
    const TABLE: &[super::fixture::Row] = &[
        ("disc Inside",
         "lifts 0 named 0 re 0 | cross 0 leave 0 | entry air 0 hole 0 col 0 solid 1 (plunge 0 ramp 0 helix 1 rapid 0) | over 33 worst 180 full 1 | fallback 0",
         "L6"),
        ("disc Outside",
         "lifts 0 named 0 re 0 | cross 0 leave 0 | entry air 1 hole 0 col 0 solid 1 (plunge 0 ramp 0 helix 1 rapid 0) | over 11 worst 118 full 0 | fallback 0",
         "L6"),
        ("dog bone Inside",
         "lifts 0 named 0 re 0 | cross 0 leave 0 | entry air 0 hole 0 col 0 solid 1 (plunge 0 ramp 0 helix 1 rapid 0) | over 54 worst 180 full 1 | fallback 0",
         "L6"),
        ("dog bone Outside",
         "lifts 0 named 0 re 0 | cross 0 leave 0 | entry air 1 hole 0 col 0 solid 1 (plunge 0 ramp 0 helix 1 rapid 0) | over 39 worst 167 full 0 | fallback 0",
         "L6"),
    ];
    pin("2D Offset", false, TABLE, ring_cases("2D Offset", PocketStrategy::Waterline));
}

#[test]
fn pocket_3d_readings_of_the_old_planner() {
    const TABLE: &[super::fixture::Row] = &[
        ("disc Inside",
         "lifts 18 named 0 re 18 | cross 0 leave 0 | entry air 0 hole 0 col 1 solid 18 (plunge 0 ramp 1 helix 17 rapid 0) | over 65 worst 180 full 1 | fallback 1",
         "link L6"),
        ("disc Outside",
         "lifts 18 named 0 re 18 | cross 0 leave 0 | entry air 0 hole 0 col 1 solid 18 (plunge 0 ramp 1 helix 17 rapid 0) | over 65 worst 180 full 1 | fallback 1",
         "link L6"),
        ("dog bone Inside",
         "lifts 26 named 0 re 26 | cross 0 leave 0 | entry air 0 hole 0 col 1 solid 26 (plunge 0 ramp 1 helix 25 rapid 0) | over 79 worst 180 full 2 | fallback 1",
         "link L6"),
        ("dog bone Outside",
         "lifts 26 named 0 re 26 | cross 0 leave 0 | entry air 0 hole 0 col 1 solid 26 (plunge 0 ramp 1 helix 25 rapid 0) | over 79 worst 180 full 2 | fallback 1",
         "link L6"),
    ];
    let footprint = region(vec![rect(-40.0, -25.0, 40.0, 25.0)]);
    let mut cases = Vec::new();
    for (pname, part) in parts() {
        let facets = prism(&part, -10.0, TOP);
        for (sname, start) in SIDES {
            let p = Pocket3DParams {
                pass_start: start,
                links: Links::ROUGH3D,
                tool_diameter: 2.0 * R,
                stepover: STEP,
                stock_to_leave: 0.0,
                clearance_z: 10.0,
                top_z: TOP,
                bottom_z: Z,
                stepdown: 2.0,
                feed: 1000.0,
                plunge_feed: 300.0,
                climb: true,
                strategy: PocketStrategy::Auto,
                retract_z: Some(2.0),
                entry: Entry3D::Helix,
                helix_radius: 1.5,
                optimal_load: DEFAULT_OPTIMAL_LOAD,
            };
            let path = pocket3d(&facets, &footprint, &p)
                .unwrap_or_else(|e| panic!("3D Pocket {pname} {sname}: {e}"));
            cases.push((format!("{pname} {sname}"), path, spec_round(&part, planned_deg(STEP, R))));
        }
    }
    pin("3D Pocket", false, TABLE, cases);
}
