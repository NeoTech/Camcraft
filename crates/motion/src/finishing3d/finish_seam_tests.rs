//! #3329: the 3D Finish seam's exact column law, driven inside the
//! crate. The raster's own ground law never keeps a sample under a
//! canopy, so a whole raster cannot reach this refusal: the test gives
//! the law the ends directly. Fixture: a floor at z = 0 and a canopy
//! block over one corner, below the clearance plane.

use crate::geo2d::Point2;
use crate::geometry::{Point3, Triangle};
use crate::path_picks::PathPick;

use super::finish_seam::ends_clear;
use super::raster_seam::{RasterEnds, RasterSeam};
use super::support::build_sups;

const R: f64 = 3.0;
const CLEARANCE: f64 = 20.0;

fn p3(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

/// A 40 x 30 floor at z = 0 and a closed canopy block x, y -4..6 at
/// z 10..12 over the corner (1, 1).
fn floor_and_canopy() -> Vec<Triangle> {
    let mut f = vec![
        Triangle::new(p3(0.0, 0.0, 0.0), p3(40.0, 0.0, 0.0), p3(40.0, 30.0, 0.0)),
        Triangle::new(p3(0.0, 0.0, 0.0), p3(40.0, 30.0, 0.0), p3(0.0, 30.0, 0.0)),
    ];
    let (lo, hi) = (p3(-4.0, -4.0, 10.0), p3(6.0, 6.0, 12.0));
    let c = |i: usize| p3(if i & 1 == 0 { lo.x } else { hi.x }, if i & 2 == 0 { lo.y } else { hi.y }, if i & 4 == 0 { lo.z } else { hi.z });
    for q in [[0, 2, 3], [0, 3, 1], [4, 5, 7], [4, 7, 6], [0, 1, 5], [0, 5, 4], [2, 6, 7], [2, 7, 3], [0, 4, 6], [0, 6, 2], [1, 3, 7], [1, 7, 5]] {
        f.push(Triangle::new(c(q[0]), c(q[1]), c(q[2])));
    }
    f
}

fn pick(x: f64, y: f64) -> Option<PathPick> {
    Some(PathPick::xy(Point2::new(x, y)))
}

fn refusal(ends: RasterEnds, seam: RasterSeam) -> String {
    let sups = build_sups(&floor_and_canopy(), R);
    ends_clear(ends, &seam, R, CLEARANCE, &sups).err().map(|e| format!("{e:?}")).unwrap_or_default()
}

#[test]
fn a_picked_end_under_a_canopy_is_refused_by_its_column() {
    let (under, open) = (p3(1.0, 1.0, 0.0), p3(20.0, 15.0, 0.0));
    // The picked start under the canopy: its plunge column is refused.
    let e = refusal(RasterEnds { start: under, exit: open }, RasterSeam { start: pick(1.5, 1.5), exit: None });
    assert!(e.contains("start point (1.500, 1.500)") && e.contains("plunge column") && e.contains("ball radius 3.000"), "{e}");
    // The picked exit under the canopy: its retract column is refused.
    let e = refusal(RasterEnds { start: open, exit: under }, RasterSeam { start: None, exit: pick(1.5, 1.5) });
    assert!(e.contains("exit point (1.500, 1.500)") && e.contains("retract column"), "{e}");
    // Clear columns pass, and an end that no pick states is not judged.
    assert_eq!(refusal(RasterEnds { start: open, exit: open }, RasterSeam { start: pick(20.0, 15.0), exit: pick(20.0, 15.0) }), "");
    assert_eq!(refusal(RasterEnds { start: open, exit: under }, RasterSeam { start: pick(20.0, 15.0), exit: None }), "");
}

#[test]
fn the_column_law_is_exact_at_the_ball_radius() {
    // The canopy's edge at x = 6: a column 3 mm off it grazes and
    // passes, one 2.999 mm off it is refused. No sampling step can
    // tell these apart; the segment-to-facet distance does.
    let seam = RasterSeam { start: pick(9.0, 1.0), exit: None };
    let grazes = p3(9.0, 1.0, 0.0);
    assert_eq!(refusal(RasterEnds { start: grazes, exit: grazes }, seam), "", "a column at the radius passes");
    let bites = p3(8.999, 1.0, 0.0);
    assert!(refusal(RasterEnds { start: bites, exit: bites }, seam).contains("plunge column"), "a column inside the radius is refused");
}
