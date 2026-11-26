//! #4153 Trace baseline: the plain trace of a plate with a hole and of a
//! round loop, climb and conventional, one level and stepped levels, and
//! with a known stock top.

use super::engrave::trace_params;
use super::{disc, pin, rect, region};
use camcraft_motion::machining2d::{TraceParams, trace};

#[test]
fn kernel_baseline_trace() {
    let plate = region(vec![rect(-30.0, -20.0, 30.0, 20.0), rect(-6.0, -6.0, 6.0, 6.0)]);
    let round = region(vec![disc(0.0, 0.0, 15.0)]);
    let cases: [(&str, &_, TraceParams); 5] = [
        ("plate_climb", &plate, trace_params()),
        ("plate_conventional", &plate, TraceParams { climb: false, ..trace_params() }),
        ("plate_stepped", &plate, TraceParams { bottom_z: -3.0, stepdown: 1.0, ..trace_params() }),
        ("plate_stock_top", &plate, TraceParams { bottom_z: -3.0, stepdown: 1.0, stock_top: Some(-1.0), ..trace_params() }),
        ("round", &round, trace_params()),
    ];
    for (case, reg, p) in cases {
        pin("trace", case, &trace(reg, &p).unwrap_or_else(|e| panic!("{case}: {e}")));
    }
}
