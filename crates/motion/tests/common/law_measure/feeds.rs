//! Planar feeds that cross another planar feed, or leave the reach of the
//! tool centre (the link rule of #4023).
//! Provenance: our own design record (#4024, "Link rule"); the crossing is
//! the verifier's own law (`verify::crossed_pairs`), as the one-level
//! reader `assert_no_crossing` of `pass_start_levels/ring_links.rs` uses it.

use camcraft_motion::cl::{MoveKind, Toolpath};
use camcraft_motion::geo2d::{Point2, Region};

use super::Reading;
use super::stock::points;

const Z_TOL: f64 = 1e-9;
/// The chord tolerance of the verifier's crossing law in the old tests.
const CHORD_TOL: f64 = 0.01;
/// How far a centre may stand outside the reach before it counts.
const REACH_TOL: f64 = 0.01;

fn planar(path: &Toolpath, i: usize) -> bool {
    path.moves
        .get(i)
        .is_some_and(|m| m.kind != MoveKind::Rapid && (m.from.z - m.to.z).abs() < Z_TOL)
}

/// Each pair of planar feeds that cross; the place is the end of the first.
pub(super) fn crossings(path: &Toolpath) -> Reading {
    let mut out = Reading::default();
    for pr in camcraft_motion::verify::crossed_pairs(path, CHORD_TOL) {
        if !(planar(path, pr[0]) && planar(path, pr[1])) {
            continue;
        }
        if let Some(m) = path.moves.get(pr[0]) {
            out.add(Point2::new(m.to.x, m.to.y));
        }
    }
    out
}

/// Each planar feed whose centre goes outside `reach` (grown by a small
/// tolerance); the place is the first point outside.
pub(super) fn leaves(path: &Toolpath, reach: &Region, step: f64) -> Result<Reading, String> {
    let grown = reach.offset(REACH_TOL).map_err(|e| format!("reach: {e:?}"))?;
    let mut out = Reading::default();
    for (i, m) in path.moves.iter().enumerate() {
        if !planar(path, i) {
            continue;
        }
        if let Some(p) = points(m, step).into_iter().find(|p| !grown.contains(Point2::new(p.x, p.y))) {
            out.add(Point2::new(p.x, p.y));
        }
    }
    Ok(out)
}
