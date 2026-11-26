//! #4153 2D Adaptive baseline: the spiral Adaptive (`PocketStrategy::
//! Adaptive`) on a 40 x 30 pocket with an 8 x 8 island for Auto, Inside and
//! Outside, and the trochoidal Adaptive on the same pocket.

use super::{pin, rect, region};
use camcraft_motion::entry::Entry3D;
use camcraft_motion::links::Links;
use camcraft_motion::load::DEFAULT_OPTIMAL_LOAD;
use camcraft_motion::machining2d::{CornerRelief, PassStart, PocketParams, PocketStrategy, adaptive_trochoidal, pocket_started};

fn params() -> PocketParams {
    PocketParams {
        tool_diameter: 10.0,
        stepover: 4.0,
        stock_to_leave: 0.0,
        clearance_z: 5.0,
        retract_z: Some(1.0),
        top_z: 0.0,
        bottom_z: -5.0,
        stepdown: 5.0,
        feed: 1000.0,
        plunge_feed: 300.0,
        climb: true,
        strategy: PocketStrategy::Adaptive,
        entry: Entry3D::Helix,
        helix_radius: 2.0,
        optimal_load: DEFAULT_OPTIMAL_LOAD,
        blend_turns: false,
        blend_radius: None,
        links: Links::PLANAR,
    }
}

#[test]
fn kernel_baseline_adaptive() {
    let island = region(vec![rect(0.0, 0.0, 40.0, 30.0), rect(16.0, 11.0, 24.0, 19.0)]);
    for (case, start) in [("auto", PassStart::Auto), ("inside", PassStart::Inside), ("outside", PassStart::Outside)] {
        let path = pocket_started(&island, &params(), &[], &CornerRelief::default(), start)
            .unwrap_or_else(|e| panic!("{case}: {e}"));
        pin("adaptive", case, &path);
    }
}

#[test]
fn kernel_baseline_adaptive_trochoidal() {
    let island = region(vec![rect(0.0, 0.0, 40.0, 30.0), rect(16.0, 11.0, 24.0, 19.0)]);
    let troch = adaptive_trochoidal(&island, &params()).expect("a legal trochoidal adaptive");
    pin("adaptive", "trochoidal", &troch);
}
