//! #4153 Engrave baseline: the trace with a Do-not-touch spec (the path
//! the Engrave arm posts when the job has avoid items): two engraved
//! squares beside a protected tower, climb and conventional, and the
//! empty spec.

use super::{pin, rect, region};
use camcraft_motion::geometry::{Point3, Triangle};
use camcraft_motion::machining2d::{TraceParams, trace_avoid};
use camcraft_motion::tools::ToolGeom;
use camcraft_motion::verify::{AvoidItem, AvoidSpec, Cutter};

fn box_facets(lo: [f64; 3], hi: [f64; 3]) -> Vec<Triangle> {
    let v = |i: usize| Point3::new([lo[0], hi[0]][i & 1], [lo[1], hi[1]][(i >> 1) & 1], [lo[2], hi[2]][(i >> 2) & 1]);
    let quad = |a: usize, b: usize, c: usize, d: usize| [Triangle::new(v(a), v(b), v(c)), Triangle::new(v(a), v(c), v(d))];
    [quad(4, 5, 7, 6), quad(0, 2, 3, 1), quad(0, 1, 5, 4), quad(2, 6, 7, 3), quad(1, 3, 7, 5), quad(2, 0, 4, 6)]
        .into_iter()
        .flatten()
        .collect()
}

fn cutter() -> Cutter {
    Cutter::flat(&ToolGeom {
        radius: 3.0,
        nose_radius: 0.0,
        flute_len: 8.0,
        shank_radius: 5.5,
        shank_len: 10.0,
        holder_radius: 13.0,
        holder_len: 20.0,
    })
}

pub(super) fn trace_params() -> TraceParams {
    TraceParams {
        clearance_z: 20.0,
        retract_z: Some(5.0),
        top_z: 0.0,
        bottom_z: -2.0,
        stepdown: 2.0,
        feed: 600.0,
        plunge_feed: 150.0,
        climb: true,
        stock_top: None,
    }
}

#[test]
fn kernel_baseline_engrave() {
    let reg = region(vec![rect(-28.0, -12.0, -12.0, 12.0), rect(12.0, -12.0, 28.0, 12.0)]);
    let tower = AvoidSpec {
        items: vec![AvoidItem {
            label: "tower".into(),
            tris: box_facets([-4.0, -15.0, -6.0], [4.0, 15.0, 12.0]),
            edges: vec![([-25.0, -14.5, -1.0], [-15.0, -14.5, -1.0])],
        }],
        cutter: cutter(),
        clearance: 1.0,
    };
    let empty = AvoidSpec { items: vec![], cutter: cutter(), clearance: 0.0 };
    let conventional = TraceParams { climb: false, ..trace_params() };
    for (case, spec, p) in
        [("avoid_climb", &tower, trace_params()), ("avoid_conventional", &tower, conventional), ("avoid_empty", &empty, trace_params())]
    {
        pin("engrave", case, &trace_avoid(&reg, &p, spec).unwrap_or_else(|e| panic!("{case}: {e}")));
    }
}
