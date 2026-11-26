//! #4153 Slot baseline: a 60 mm slot as wide as the tool and a wider one
//! (with side passes), each entry kind, climb and conventional, and a
//! slot that leaves stock.

use super::pin;
use camcraft_motion::entry::Entry3D;
use camcraft_motion::geo2d::Point2;
use camcraft_motion::links::Links;
use camcraft_motion::machining2d::{SlotParams, slot};

fn params() -> SlotParams {
    SlotParams {
        links: Links::PLANAR,
        tool_diameter: 10.0,
        width: 10.0,
        stock_to_leave: 0.0,
        clearance_z: 5.0,
        retract_z: None,
        top_z: 0.0,
        bottom_z: -6.0,
        stepdown: 3.0,
        stepover: 5.0,
        feed: 500.0,
        plunge_feed: 100.0,
        climb: true,
        entry: Entry3D::Plunge,
        helix_radius: 1.0,
        blend_turns: false,
        blend_radius: None,
    }
}

#[test]
fn kernel_baseline_slot() {
    let (a, b) = (Point2::new(0.0, 0.0), Point2::new(60.0, 0.0));
    let wide = || SlotParams { width: 18.0, stepover: 3.0, retract_z: Some(2.0), ..params() };
    let cases: [(&str, SlotParams); 8] = [
        ("plunge", params()),
        ("ramp", SlotParams { entry: Entry3D::Ramp, ..params() }),
        ("conventional", SlotParams { climb: false, ..params() }),
        ("wide", wide()),
        ("wide_conventional", SlotParams { climb: false, ..wide() }),
        ("wide_helix", SlotParams { entry: Entry3D::Helix, helix_radius: 2.0, ..wide() }),
        ("wide_arc", SlotParams { entry: Entry3D::Arc, helix_radius: 2.0, ..wide() }),
        ("wide_stock_to_leave", SlotParams { stock_to_leave: 0.5, ..wide() }),
    ];
    for (case, p) in cases {
        pin("slot", case, &slot(a, b, &p).unwrap_or_else(|e| panic!("{case}: {e}")));
    }
}
