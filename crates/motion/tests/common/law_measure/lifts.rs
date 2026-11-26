//! The lifts and re-entries of each cut level (the link rule of #4023).
//! Provenance: our own design record (#4024, "Link rule"); the same count
//! as the one-level reader of `pass_start_levels/ring_links.rs`, made for
//! each level of a path.

use camcraft_motion::cl::{MoveKind, Toolpath};
use camcraft_motion::geo2d::Point2;

use super::Reading;

const Z_TOL: f64 = 1e-6;

/// The cut levels: the Z values of planar feeds below the stock top.
fn levels(path: &Toolpath, top: f64) -> Vec<f64> {
    let mut out: Vec<f64> = Vec::new();
    for m in &path.moves {
        let planar = (m.from.z - m.to.z).abs() <= Z_TOL;
        if m.kind == MoveKind::Rapid || !planar || !m.to.z.is_finite() || m.to.z >= top - Z_TOL {
            continue;
        }
        if !out.iter().any(|z| (z - m.to.z).abs() <= Z_TOL) {
            out.push(m.to.z);
        }
    }
    out
}

/// The lifts of every level (a Z-up move that leaves the level between
/// its first and its last cut) and the re-entries (each run of cuts on
/// the level after the first one).
pub(super) fn level_lifts(path: &Toolpath, top: f64) -> (Reading, usize) {
    let mut lifts = Reading::default();
    let mut re_entries = 0usize;
    for z in levels(path, top) {
        let on = |i: usize| {
            path.moves.get(i).is_some_and(|m| {
                m.kind != MoveKind::Rapid
                    && (m.from.z - z).abs() <= Z_TOL
                    && (m.to.z - z).abs() <= Z_TOL
            })
        };
        let floor: Vec<usize> = (0..path.moves.len()).filter(|&i| on(i)).collect();
        let (Some(&first), Some(&last)) = (floor.first(), floor.last()) else {
            continue;
        };
        let mut runs = 0usize;
        let mut was = false;
        for i in first..=last {
            let now = on(i);
            if now && !was {
                runs = runs.saturating_add(1);
            }
            was = now;
        }
        re_entries = re_entries.saturating_add(runs.saturating_sub(1));
        for m in path.moves.get(first..last).unwrap_or(&[]) {
            if m.to.z > z + Z_TOL && (m.from.z - z).abs() <= Z_TOL {
                lifts.add(Point2::new(m.from.x, m.from.y));
            }
        }
    }
    (lifts, re_entries)
}
