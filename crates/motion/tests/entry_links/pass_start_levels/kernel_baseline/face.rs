//! #4153 Face baseline: the raster on a 40 x 20 plate (climb and
//! conventional, one level and stepped levels, blended turns), the plate
//! with a hole, and a start pick and an exit pick.

use super::{pin, rect, region};
use camcraft_motion::geo2d::Point2;
use camcraft_motion::machining2d::{FaceHoles, FaceParams, face, face_picked};
use camcraft_motion::path_picks::{PathPick, PathPicks};

fn p() -> FaceParams {
    FaceParams {
        tool_diameter: 6.0,
        stepover: 3.0,
        clearance_z: 5.0,
        cut_z: -2.0,
        top_z: None,
        stepdown: None,
        feed: 500.0,
        plunge_feed: 100.0,
        climb: true,
        blend_turns: false,
        blend_radius: None,
        holes: FaceHoles::default(),
        stock_top: None,
    }
}

#[test]
fn kernel_baseline_face() {
    let plate = region(vec![rect(-20.0, -10.0, 20.0, 10.0)]);
    let holed = region(vec![rect(-20.0, -10.0, 20.0, 10.0), rect(-4.0, -3.0, 4.0, 3.0)]);
    let conventional = FaceParams { climb: false, ..p() };
    let stepped = FaceParams { top_z: Some(0.0), stepdown: Some(1.0), ..p() };
    let blended = FaceParams { blend_turns: true, ..p() };
    for (case, reg, fp) in [
        ("climb", &plate, p()),
        ("conventional", &plate, conventional),
        ("stepped", &plate, stepped),
        ("blended", &plate, blended),
        ("hole", &holed, p()),
    ] {
        pin("face", case, &face(reg, &fp).expect("a legal face"));
    }
    let one = |x: f64, y: f64| vec![PathPick::xy(Point2::new(x, y))];
    let picks = PathPicks { start: one(25.0, 12.0), exit: one(25.0, -12.0) };
    pin("face", "picked", &face_picked(&plate, &p(), 0.0, None, None, &picks).expect("a legal picked face"));
}
