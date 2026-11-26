//! #4153 Bore baseline: a blind Ø20 hole with a Ø6 cutter for Auto,
//! Inside and Outside, each entry kind, conventional, and a through hole.

use super::pin;
use camcraft_motion::entry::Entry3D;
use camcraft_motion::geo2d::Point2;
use camcraft_motion::links::Links;
use camcraft_motion::machining2d::{BoreHole, BoreParams, PassStart, bore_started};

fn params() -> BoreParams {
    BoreParams {
        holes: vec![BoreHole {
            xy: Point2::new(5.0, 5.0),
            radius: 10.0,
            top_z: 0.0,
            floor_z: -4.0,
            through: false,
            stepped: false,
        }],
        tool_diameter: 6.0,
        corner_radius: 0.0,
        stock_to_leave: 0.0,
        stepdown: 2.0,
        through_allowance: 0.0,
        clearance_z: 5.0,
        retract_z: None,
        feed: 500.0,
        plunge_feed: 250.0,
        climb: true,
        tolerance: 0.01,
        entry: Entry3D::Plunge,
        helix_radius: 1.0,
        links: Links::PLANAR,
    }
}

#[test]
fn kernel_baseline_bore() {
    let with = |entry: Entry3D| BoreParams { entry, ..params() };
    let mut through = params();
    through.holes[0].through = true;
    through.through_allowance = 1.0;
    let cases: [(&str, BoreParams, PassStart); 9] = [
        ("auto", params(), PassStart::Auto),
        ("inside", params(), PassStart::Inside),
        ("outside", params(), PassStart::Outside),
        ("conventional", BoreParams { climb: false, ..params() }, PassStart::Auto),
        ("ramp", with(Entry3D::Ramp), PassStart::Auto),
        ("helix", with(Entry3D::Helix), PassStart::Auto),
        ("arc", with(Entry3D::Arc), PassStart::Auto),
        ("helix_outside", with(Entry3D::Helix), PassStart::Outside),
        ("through", through, PassStart::Auto),
    ];
    for (case, p, start) in cases {
        pin("bore", case, &bore_started(&p, start).unwrap_or_else(|e| panic!("{case}: {e}")));
    }
}
