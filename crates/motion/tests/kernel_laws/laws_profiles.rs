//! #4154: the readings of the OLD planners of Face, 2D Contour, Engrave,
//! Trace, V-Carve and 2D Chamfer on the disc and the dog bone. These kinds
//! have no start side. Engrave, Trace, V-Carve and Chamfer cut along the
//! curve at the tool width by plan (a named slot for law L6); V-Carve and
//! Chamfer cut with a cone, which the flat-disc stock grid does not model,
//! so their engagement column is a reading only.

use super::common::law_measure::{EntryKind, Spec};
use super::fixture::{R, TOP, Z, parts, pin, planned_deg, rect, region, stock};
use camcraft_motion::entry::Entry3D;
use camcraft_motion::links::Links;
use camcraft_motion::machining2d::{
    ChamferParams, ContourParams, FaceParams, Lead, Side, TraceParams, VCarveParams, chamfer, contour,
    face, trace, v_carve,
};

/// The finish stock that a contour cuts on the wall.
const WALL: f64 = 1.0;

#[test]
fn face_readings_of_the_old_planner() {
    const TABLE: &[super::fixture::Row] = &[
        ("box (both parts)",
         "lifts 0 named 0 re 0 | cross 0 leave 0 | entry air 1 hole 0 col 0 solid 0 (plunge 0 ramp 0 helix 0 rapid 0) | over 1 worst 162 full 0 | fallback 0",
         "L6"),
    ];
    let box_ = region(vec![rect(-40.0, -25.0, 40.0, 25.0)]);
    let p = FaceParams {
        holes: Default::default(),
        tool_diameter: 2.0 * R,
        stepover: R,
        clearance_z: 10.0,
        cut_z: -1.0,
        top_z: None,
        stepdown: None,
        feed: 1000.0,
        plunge_feed: 300.0,
        climb: true,
        blend_turns: false,
        blend_radius: None,
        stock_top: Some(TOP),
    };
    let path = face(&box_, &p).expect("face");
    let spec = Spec::new(R, stock(), TOP, planned_deg(R, R));
    pin("Face", false, TABLE, vec![("box (both parts)".into(), path, spec)]);
}

#[test]
fn contour_2d_readings_of_the_old_planner() {
    const TABLE: &[super::fixture::Row] = &[
        ("disc Outside wall",
         "lifts 0 named 0 re 0 | cross 0 leave 0 | entry air 0 hole 0 col 0 solid 1 (plunge 0 ramp 1 helix 0 rapid 0) | over 0 worst 30 full 0 | fallback 1",
         "-"),
        ("dog bone Outside wall",
         "lifts 0 named 0 re 0 | cross 0 leave 0 | entry air 0 hole 0 col 0 solid 1 (plunge 0 ramp 1 helix 0 rapid 0) | over 4 worst 80 full 0 | fallback 1",
         "L6"),
    ];
    let mut cases = Vec::new();
    for (pname, part) in parts() {
        let p = ContourParams {
            tool_diameter: 2.0 * R,
            stock_to_leave: 0.0,
            clearance_z: 10.0,
            retract_z: 2.0,
            top_z: TOP,
            bottom_z: Z,
            stepdown: 2.0,
            feed: 1000.0,
            plunge_feed: 300.0,
            climb: true,
            side: Side::Outside,
            lead: Lead::None,
            tolerance: 0.01,
            tab_count: 0,
            tab_width: 0.0,
            tab_height: 0.0,
            entry: Entry3D::Helix,
            helix_radius: 1.5,
            blend_turns: false,
            blend_radius: None,
            links: Links::PLANAR,
            picks: Default::default(),
        };
        let path = contour(&part, &p).unwrap_or_else(|e| panic!("contour {pname}: {e}"));
        // The wall stock: the part grown by the finish stock.
        let mut spec = Spec::new(R, part.offset(WALL).expect("wall"), TOP, planned_deg(WALL, R));
        spec.part = Some(part.clone());
        spec.asked_entry = Some(EntryKind::Helix);
        cases.push((format!("{pname} Outside wall"), path, spec));
    }
    pin("2D Contour", false, TABLE, cases);
}

fn trace_params(bottom: f64) -> TraceParams {
    TraceParams {
        clearance_z: 10.0,
        retract_z: Some(2.0),
        top_z: TOP,
        bottom_z: bottom,
        stepdown: 2.0,
        feed: 600.0,
        plunge_feed: 200.0,
        climb: true,
        stock_top: Some(TOP),
    }
}

/// Engrave and Trace share one generator; Engrave uses a small tool and
/// a shallow depth, Trace the ring tool at the cut level.
fn trace_cases(tool_r: f64, bottom: f64) -> Vec<(String, camcraft_motion::cl::Toolpath, Spec)> {
    parts()
        .into_iter()
        .map(|(pname, part)| {
            let path = trace(&part, &trace_params(bottom)).unwrap_or_else(|e| panic!("trace {pname}: {e}"));
            (format!("{pname} outline"), path, Spec::new(tool_r, stock(), TOP, 180.0))
        })
        .collect()
}

#[test]
fn engrave_readings_of_the_old_planner() {
    const TABLE: &[super::fixture::Row] = &[
        ("disc outline",
         "lifts 0 named 0 re 0 | cross 0 leave 0 | entry air 0 hole 0 col 0 solid 1 (plunge 1 ramp 0 helix 0 rapid 0) | over 0 worst 180 full 126 | fallback 0",
         "L5"),
        ("dog bone outline",
         "lifts 0 named 0 re 0 | cross 0 leave 0 | entry air 0 hole 0 col 0 solid 1 (plunge 1 ramp 0 helix 0 rapid 0) | over 0 worst 180 full 230 | fallback 0",
         "L5"),
    ];
    pin("Engrave", true, TABLE, trace_cases(0.5, -0.5));
}

#[test]
fn trace_readings_of_the_old_planner() {
    const TABLE: &[super::fixture::Row] = &[
        ("disc outline",
         "lifts 0 named 0 re 0 | cross 0 leave 0 | entry air 0 hole 0 col 0 solid 1 (plunge 1 ramp 0 helix 0 rapid 0) | over 0 worst 180 full 114 | fallback 0",
         "L5"),
        ("dog bone outline",
         "lifts 0 named 0 re 0 | cross 0 leave 0 | entry air 0 hole 0 col 0 solid 1 (plunge 1 ramp 0 helix 0 rapid 0) | over 0 worst 180 full 210 | fallback 0",
         "L5"),
    ];
    pin("Trace", true, TABLE, trace_cases(R, Z));
}

#[test]
fn v_carve_readings_of_the_old_planner() {
    const TABLE: &[super::fixture::Row] = &[
        ("disc carved",
         "lifts 0 named 0 re 0 | cross 0 leave 0 | entry air 0 hole 0 col 0 solid 1 (plunge 1 ramp 0 helix 0 rapid 0) | over 0 worst 180 full 50 | fallback 0",
         "L5"),
        ("dog bone carved",
         "lifts 2 named 0 re 3 | cross 0 leave 0 | entry air 0 hole 0 col 1 solid 2 (plunge 1 ramp 1 helix 0 rapid 0) | over 0 worst 180 full 121 | fallback 0",
         "link L5"),
    ];
    let mut cases = Vec::new();
    for (pname, part) in parts() {
        let p = VCarveParams {
            angle_deg: 90.0,
            tool_diameter: 12.0,
            clearance_z: 10.0,
            retract_z: Some(2.0),
            top_z: TOP,
            bottom_z: -6.0,
            stepdown: 6.0,
            feed: 600.0,
            plunge_feed: 200.0,
            flat_clear: false,
            flat_stepover: 0.25,
        };
        let path = v_carve(&part, &p).unwrap_or_else(|e| panic!("v-carve {pname}: {e}"));
        cases.push((format!("{pname} carved"), path, Spec::new(6.0, stock(), TOP, 180.0)));
    }
    pin("V-Carve", true, TABLE, cases);
}

#[test]
fn chamfer_2d_readings_of_the_old_planner() {
    const TABLE: &[super::fixture::Row] = &[
        ("disc edge",
         "lifts 0 named 0 re 0 | cross 0 leave 0 | entry air 0 hole 0 col 0 solid 1 (plunge 1 ramp 0 helix 0 rapid 0) | over 0 worst 92 full 0 | fallback 0",
         "L5"),
        ("dog bone edge",
         "lifts 0 named 0 re 0 | cross 0 leave 0 | entry air 0 hole 0 col 0 solid 1 (plunge 1 ramp 0 helix 0 rapid 0) | over 0 worst 162 full 0 | fallback 0",
         "L5"),
    ];
    let mut cases = Vec::new();
    for (pname, part) in parts() {
        let p = ChamferParams {
            width: 1.0,
            angle_deg: 45.0,
            minimum_cone_radius: 0.0,
            clearance_z: 10.0,
            retract_z: None,
            top_z: TOP,
            feed: 500.0,
            plunge_feed: 100.0,
            climb: true,
            inside: false,
        };
        let path = chamfer(&part, &p).unwrap_or_else(|e| panic!("chamfer {pname}: {e}"));
        // The stock is the finished part: the chamfer cuts its top edge.
        cases.push((format!("{pname} edge"), path, Spec::new(1.0, part.clone(), TOP, 180.0)));
    }
    pin("2D Chamfer", true, TABLE, cases);
}
