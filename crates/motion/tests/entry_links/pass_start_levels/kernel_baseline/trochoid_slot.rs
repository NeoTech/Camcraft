//! #4153 Trochoidal slot baseline: a 12 mm slot along a straight 20 mm
//! chain with a Ø6 cutter, the microlift return and the retract return,
//! climb and conventional, a known stock top, and a 16 mm slot. A curved
//! chain is not in the set: the old planner refuses the arcs tried (radius
//! 30, 12 mm wide) with "the centre line turns too tightly".

use super::pin;
use camcraft_motion::geo2d::Point2;
use camcraft_motion::machining2d::{TrochoidParams, TrochoidReturn, trochoidal_slot};

fn params() -> TrochoidParams {
    TrochoidParams {
        tool_diameter: 6.0,
        width: 12.0,
        loop_width: None,
        step: 1.5,
        max_engagement: 90.0,
        ret: TrochoidReturn::Microlift { height: 0.3 },
        smoothing: 0.5,
        stock_to_leave: 0.0,
        clearance_z: 5.0,
        retract_z: Some(1.0),
        top_z: 0.0,
        bottom_z: -4.0,
        stepdown: 2.0,
        feed: 800.0,
        return_feed: 2000.0,
        plunge_feed: 200.0,
        climb: true,
        stock_top: None,
    }
}

#[test]
fn kernel_baseline_trochoid_slot() {
    let straight = [Point2::new(0.0, 0.0), Point2::new(20.0, 0.0)];
    let cases: [(&str, &[Point2], TrochoidParams); 5] = [
        ("straight", &straight, params()),
        ("straight_retract", &straight, TrochoidParams { ret: TrochoidReturn::Retract, ..params() }),
        ("straight_conventional", &straight, TrochoidParams { climb: false, ..params() }),
        ("straight_stock_top", &straight, TrochoidParams { stock_top: Some(-1.0), ..params() }),
        ("wide", &straight, TrochoidParams { width: 16.0, ..params() }),
    ];
    for (case, chain, p) in cases {
        pin("trochoid_slot", case, &trochoidal_slot(chain, &p).unwrap_or_else(|e| panic!("{case}: {e}")));
    }
}
