//! #4153 V-Carve baseline: a 90 degree V-bit in a rectangle, a hexagon
//! and a rectangle too wide for the cone (with and without the flat
//! clear), and a 60 degree bit.

use super::{pin, poly, rect, region};
use camcraft_motion::machining2d::{VCarveParams, v_carve};

fn params() -> VCarveParams {
    VCarveParams {
        angle_deg: 90.0,
        tool_diameter: 12.0,
        clearance_z: 10.0,
        retract_z: Some(3.0),
        top_z: 0.0,
        bottom_z: -6.0,
        stepdown: 6.0,
        feed: 600.0,
        plunge_feed: 200.0,
        flat_clear: false,
        flat_stepover: 0.25,
    }
}

fn hexagon() -> super::Loop {
    let pts: Vec<(f64, f64)> = (0..6)
        .map(|i| {
            let a = std::f64::consts::TAU * f64::from(i) / 6.0;
            (8.0 * a.cos(), 8.0 * a.sin())
        })
        .collect();
    poly(&pts)
}

#[test]
fn kernel_baseline_v_carve() {
    let narrow = region(vec![rect(0.0, 0.0, 30.0, 8.0)]);
    let hex = region(vec![hexagon()]);
    let wide = region(vec![rect(0.0, 0.0, 40.0, 20.0)]);
    let shallow = VCarveParams { bottom_z: -3.0, stepdown: 3.0, ..params() };
    let cases: [(&str, &_, VCarveParams); 5] = [
        ("narrow", &narrow, params()),
        ("hexagon", &hex, params()),
        ("wide", &wide, shallow),
        ("wide_flat_clear", &wide, VCarveParams { flat_clear: true, ..shallow }),
        ("narrow_60deg", &narrow, VCarveParams { angle_deg: 60.0, ..params() }),
    ];
    for (case, reg, p) in cases {
        pin("v_carve", case, &v_carve(reg, &p).unwrap_or_else(|e| panic!("{case}: {e}")));
    }
}
