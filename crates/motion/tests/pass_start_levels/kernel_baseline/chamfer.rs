//! #4153 2D Chamfer baseline: the ride along a 60 x 60 square, outside
//! and inside, climb and conventional, a 30 degree slope, and an open
//! run on a block edge.

use super::{pin, rect, region};
use camcraft_motion::geo2d::Point2;
use camcraft_motion::machining2d::{ChamferParams, chamfer, chamfer_open_runs};

fn params() -> ChamferParams {
    ChamferParams {
        width: 2.0,
        angle_deg: 45.0,
        minimum_cone_radius: 0.0,
        clearance_z: 25.0,
        retract_z: None,
        top_z: 20.0,
        feed: 500.0,
        plunge_feed: 100.0,
        climb: true,
        inside: false,
    }
}

#[test]
fn kernel_baseline_chamfer() {
    let square = region(vec![rect(0.0, 0.0, 60.0, 60.0)]);
    let cases: [(&str, ChamferParams); 5] = [
        ("outside", params()),
        ("outside_conventional", ChamferParams { climb: false, ..params() }),
        ("outside_30deg", ChamferParams { width: 3.0, angle_deg: 30.0, ..params() }),
        ("inside", ChamferParams { inside: true, ..params() }),
        ("inside_retract", ChamferParams { inside: true, retract_z: Some(22.0), ..params() }),
    ];
    for (case, p) in cases {
        pin("chamfer", case, &chamfer(&square, &p).unwrap_or_else(|e| panic!("{case}: {e}")));
    }
    let run = vec![vec![Point2::new(-20.0, -20.0), Point2::new(20.0, -20.0)]];
    let path = chamfer_open_runs(&run, &params(), &|_| Ok(Some(region(vec![rect(-20.0, -20.0, 20.0, 20.0)]))))
        .expect("a legal open run");
    pin("chamfer", "open_run", &path);
}
