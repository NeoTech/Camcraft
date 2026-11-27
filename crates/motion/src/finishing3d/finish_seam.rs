//! The user's start and exit points on the 3D Finish (#3329, the 3D
//! Finish slice of #1448). The picks are read on the Parallel pattern,
//! the one-direction zigzag raster, at any pass direction. The raster's
//! passes are kept as they are; the pick only chooses the corner where
//! the cut starts and the corner where the last pass ends, by the
//! raster seam's law (`raster_seam::order`, shared with the 5-Axis
//! Parallel):
//!
//! - **Start:** the raster corner nearest the start pick. The pass
//!   order is reversed when that corner is on the last pass, and each
//!   pass runs the other way when that corner is the far end of its
//!   pass.
//! - **Exit:** the order whose last pass ends nearest the exit pick.
//!   With a start pick, the exit must be near the end that the start's
//!   order gives.
//!
//! What a pick cannot keep is refused by name, never moved:
//!
//! - a pick on a pattern other than Parallel;
//! - a pick farther than its reach from every corner, and an exit that
//!   the start's order cannot end at (`raster_seam`);
//! - a plunge column at the picked start, or a retract column at the
//!   picked exit, that comes within the ball radius of the part. The
//!   check is exact: the distance from the column's segment to each
//!   facet (`support::column_clear`), no sampling;
//! - a start or an exit that a later stage moves: the waterline rings
//!   that cut after the raster, or (in the application) the area
//!   controls that clip it. The finished path is read again, and its
//!   first cut and its last cut must be the picked corners.
//!
//! No pick is the automatic raster, byte for byte.
//!
//! Provenance: first principles - the four end corners of a serpentine
//! raster, the swept ball of a vertical move as its segment inflated by
//! the radius; the entry and exit position is the common vocabulary of
//! published CAM practice. No vendor material consulted.

use crate::cl::{MoveKind, PathBuilder, Toolpath};
use crate::geometry::{Point3, Triangle};
use crate::machining2d::GenError;
use crate::path_picks::{PathPick, PickRole};

use super::grid::SupSet;
use super::params::{validate, Finish3DParams, FinishPattern, TOUCH_EPS};
use super::raster::{parallel_angle_refusal, parallel_with_angle, raster_pass_seamed, rings_and_tail};
use super::raster_seam::{RasterEnds, RasterSeam};
use super::support::{build_sups, column_clear};

/// The unit pass direction of the Parallel pattern at `direction_angle`
/// degrees counterclockwise from +X; the four axis angles are exact.
pub(super) fn parallel_direction(direction_angle: f64) -> (f64, f64) {
    match direction_angle {
        0.0 => (1.0, 0.0),
        90.0 => (0.0, 1.0),
        180.0 => (-1.0, 0.0),
        270.0 => (0.0, -1.0),
        _ => {
            let (sin, cos) = direction_angle.to_radians().sin_cos();
            (cos, sin)
        }
    }
}

/// "start point (x, y)" or "exit point (x, y, z)", the refusals' lead.
fn named(role: PickRole, p: &PathPick) -> String {
    match p.z {
        Some(z) => format!("{} point ({:.3}, {:.3}, {:.3})", role.word(), p.xy.x, p.xy.y, z),
        None => format!("{} point ({:.3}, {:.3})", role.word(), p.xy.x, p.xy.y),
    }
}

fn at(q: Point3) -> String {
    format!("({:.3}, {:.3}, {:.3})", q.x, q.y, q.z)
}

/// The first pick of the seam, with its role: the subject of a refusal
/// that is not about one role.
fn any_pick(seam: &RasterSeam) -> Option<(PickRole, PathPick)> {
    seam.start.map(|q| (PickRole::Start, q)).or(seam.exit.map(|q| (PickRole::Exit, q)))
}

/// The 3D Finish's Parallel raster at `direction_angle` degrees with the
/// user's start and exit points: the passes ordered for the picks, then
/// the waterline rings and the clearance exit, as the automatic path.
/// Returns the path and where the raster starts and ends. An empty
/// `seam` is [`parallel_with_angle`], byte for byte, with no ends.
pub fn finish_seamed(
    facets: &[Triangle],
    p: &Finish3DParams,
    direction_angle: f64,
    seam: &RasterSeam,
) -> Result<(Toolpath, Option<RasterEnds>), GenError> {
    let Some((role, q)) = any_pick(seam) else {
        return Ok((parallel_with_angle(facets, p, direction_angle)?, None));
    };
    if !matches!(p.pattern, FinishPattern::Parallel) {
        return Err(GenError::Invalid(format!(
            "{}: the 3D Finish reads a start or exit point on the Parallel pattern only, not on {:?} - pick \
             the Parallel pattern, or clear the {} point",
            named(role, &q),
            p.pattern,
            role.word()
        )));
    }
    if let Some(why) = parallel_angle_refusal(direction_angle) {
        return Err(GenError::Invalid(why.into()));
    }
    validate(p, facets)?;
    let r = p.tool_diameter * 0.5;
    let sups = build_sups(facets, r);
    let mut b = PathBuilder::approached(Point3::new(0.0, 0.0, p.clearance_z));
    let dir = parallel_direction(direction_angle);
    let Some(ends) = raster_pass_seamed(&mut b, p, r, &sups, dir, false, p.tolerance, seam)? else {
        return Err(GenError::Invalid(format!(
            "{}: the raster has no pass to read it on - no shallow ground stands inside the depths",
            named(role, &q)
        )));
    };
    ends_clear(ends, seam, r, p.clearance_z, &sups)?;
    let path = rings_and_tail(b, p, r, facets, true)?;
    raster_ends_kept(
        &path,
        ends,
        seam,
        "the waterline rings cut after the raster",
        "turn off the waterline rings, or clear the exit point",
    )?;
    Ok((path, Some(ends)))
}

/// The exact column law at each picked end of the ordered raster: the
/// ball's plunge from the clearance plane down to the first tip, and
/// its retract from the last tip up to the clearance plane, keep a full
/// radius from every facet (`support::column_clear`: the distance from
/// the column's segment to each facet, no sampling). The raster's own
/// ground law gives the same answer for each sample it keeps; this is
/// the seam's own proof at the ends that a pick moves.
pub(super) fn ends_clear(ends: RasterEnds, seam: &RasterSeam, r: f64, clearance_z: f64, sups: &SupSet) -> Result<(), GenError> {
    let top = clearance_z + r;
    let picked = [(seam.start, ends.start, PickRole::Start, "plunge column to"), (seam.exit, ends.exit, PickRole::Exit, "retract column from")];
    for (pick, tip, role, what) in picked {
        let Some(q) = pick else { continue };
        if !column_clear(tip.x, tip.y, tip.z + r, top, r - TOUCH_EPS, sups) {
            return Err(GenError::Invalid(format!(
                "{}: the {what} the raster corner {} comes within the ball radius {r:.3} mm of the part - \
                 pick another corner, or clear the {} point",
                named(role, &q),
                at(tip),
                role.word()
            )));
        }
    }
    Ok(())
}

/// The finished path keeps the picked corners: its first cut lands on
/// the raster's first tip when a start is picked, and its last cut ends
/// on the raster's last tip when an exit is picked. Else the refusal
/// names the pick, where the path starts or ends instead, the `cause`
/// (what moved it) and the `fix`. A pick is never moved in silence.
pub fn raster_ends_kept(path: &Toolpath, ends: RasterEnds, seam: &RasterSeam, cause: &str, fix: &str) -> Result<(), GenError> {
    let same = |a: Point3, b: Point3| (a.x - b.x).abs() <= 1e-9 && (a.y - b.y).abs() <= 1e-9 && (a.z - b.z).abs() <= 1e-9;
    let first = path.moves.iter().find(|m| m.kind != MoveKind::Rapid).map(|m| m.to);
    let last = path.moves.iter().rev().find(|m| m.kind == MoveKind::Feed).map(|m| m.to);
    let checks = [(seam.start, first, ends.start, PickRole::Start, "starts"), (seam.exit, last, ends.exit, PickRole::Exit, "ends")];
    for (pick, got, want, role, verb) in checks {
        let Some(q) = pick else { continue };
        if got.is_some_and(|g| same(g, want)) {
            continue;
        }
        let instead = got.map_or_else(|| "nowhere".to_owned(), at);
        return Err(GenError::Invalid(format!(
            "{}: {cause}, so the cut {verb} at {instead}, not at the picked raster corner {} - {fix}",
            named(role, &q),
            at(want)
        )));
    }
    Ok(())
}
