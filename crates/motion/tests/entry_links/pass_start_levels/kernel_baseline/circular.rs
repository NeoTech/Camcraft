//! #4153 Circular baseline: a Ø30 hole with a Ø8 cutter for Auto, Inside
//! and Outside, each entry kind, climb and conventional, and the vault.

use super::{pin, rect, region};
use camcraft_motion::entry::Entry3D;
use camcraft_motion::geo2d::Point2;
use camcraft_motion::links::Links;
use camcraft_motion::machining2d::{CircularParams, PassStart, circular};

fn params() -> CircularParams {
    CircularParams {
        pass_start: PassStart::Auto,
        links: Links::PLANAR,
        center: Point2::new(0.0, 0.0),
        radius: 15.0,
        vault: false,
        bottom_z: -4.0,
        top_z: 0.0,
        tool_diameter: 8.0,
        stock_to_leave: 0.0,
        stepover: 3.0,
        stepdown: 2.0,
        clearance_z: 10.0,
        retract_z: Some(2.0),
        feed: 800.0,
        plunge_feed: 200.0,
        climb: true,
        entry: Entry3D::Plunge,
        helix_radius: 1.0,
    }
}

#[test]
fn kernel_baseline_circular() {
    let reg = region(vec![rect(-20.0, -20.0, 20.0, 20.0)]);
    let cases: [(&str, CircularParams); 9] = [
        ("auto", params()),
        ("inside", CircularParams { pass_start: PassStart::Inside, ..params() }),
        ("outside", CircularParams { pass_start: PassStart::Outside, ..params() }),
        ("conventional", CircularParams { climb: false, ..params() }),
        ("ramp", CircularParams { entry: Entry3D::Ramp, ..params() }),
        ("helix", CircularParams { entry: Entry3D::Helix, ..params() }),
        ("arc", CircularParams { entry: Entry3D::Arc, ..params() }),
        ("vault", CircularParams { vault: true, top_z: 0.0, bottom_z: -6.0, ..params() }),
        ("vault_outside", CircularParams { vault: true, top_z: 0.0, bottom_z: -6.0, pass_start: PassStart::Outside, ..params() }),
    ];
    for (case, p) in cases {
        pin("circular", case, &circular(&reg, &p).unwrap_or_else(|e| panic!("{case}: {e}")));
    }
}
