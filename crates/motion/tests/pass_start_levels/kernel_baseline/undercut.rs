//! #4153 Slot Undercut baseline, beside the four goldens of
//! `golden_undercut.rs` (T-slot, keyseat, dovetail, lollipop, one
//! direction each): the T-slot straight run conventional, the keyseat
//! straight run climb, the dovetail on an arc, and the lollipop straight.

use super::pin;
use camcraft_motion::geo2d::Point2;
use camcraft_motion::machining2d::{UndercutParams, undercut};
use camcraft_motion::tools::ToolKind;
use camcraft_tools::undercut::{Undercut, cutter};

fn params(cutter: Undercut, width: f64, bulge: f64) -> UndercutParams {
    UndercutParams {
        cutter,
        width,
        bulge,
        run_in: 15.0,
        neck_slot: 0.0,
        clearance_z: 10.0,
        retract_z: Some(2.0),
        bottom_z: -12.0,
        stepover: 2.0,
        feed: 120.0,
        plunge_feed: 60.0,
        climb: true,
    }
}

#[test]
fn kernel_baseline_undercut() {
    let head = |kind, d, h, neck, angle| Undercut::of(&cutter(kind, d, h, neck, 30.0, angle)).expect("a head");
    let tslot = head(ToolKind::TSlot, 20.0, 6.0, 8.0, 0.0);
    let keyseat = head(ToolKind::Keyseat, 16.0, 3.0, 6.0, 0.0);
    let dovetail = head(ToolKind::Dovetail, 20.0, 5.0, 8.0, 60.0);
    let lollipop = head(ToolKind::Lollipop, 10.0, 5.0, 4.0, 0.0);
    let cases: [(&str, UndercutParams); 4] = [
        ("tslot_conventional", UndercutParams { climb: false, neck_slot: 14.0, ..params(tslot, 26.0, 0.0) }),
        ("keyseat_straight", params(keyseat, 20.0, 0.0)),
        ("dovetail_arc", params(dovetail, 24.0, 0.3)),
        ("lollipop_straight", params(lollipop, 14.0, 0.0)),
    ];
    let (a, b) = (Point2::new(0.0, 0.0), Point2::new(60.0, 0.0));
    for (case, p) in cases {
        pin("undercut", case, &undercut(a, b, &p).unwrap_or_else(|e| panic!("{case}: {e}")));
    }
}
