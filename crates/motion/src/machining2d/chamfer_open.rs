//! Open-chain 2D Chamfer rides, including the end transition for a
//! positive cone-band tip offset.

use crate::cl::{EdgeBreak, MoveKind, PathBuilder, Toolpath};
use crate::geo2d::{Point2, Region};
use crate::geometry::Point3;

use super::chamfer::ChamferParams;
use super::chamfer_side::boundary_inside;
use super::contour::air_side;
use super::contour_run_side::{LIFT, SectionAt, Words, material_side, offset_run_side, simple};
use super::face::GenError;
use super::levels::{Motion, run_levels};

const CHAMFER_WORDS: Words = Words {
    body: "selected model",
    both: "a chamfer on that edge would meet material on both sides; pick a boundary edge with air on one side",
};

/// Chamfer joined open edge chains, one safe approach and exit per
/// chain. At zero radius the pass stands on the selected wall. A positive
/// radius offsets the selected run toward Side's air side. The entry and
/// exit caps return to the zero-radius cone as separate passes, joined
/// to the offset band by retracts in air; no feed changes Z between the
/// two cone planes.
pub fn chamfer_open_runs(
    runs: &[Vec<Point2>],
    p: &ChamferParams,
    section: &SectionAt<'_>,
) -> Result<Toolpath, GenError> {
    p.validate()?;
    if runs.is_empty() {
        return Err(GenError::Empty(
            "a 2D Chamfer needs a selected edge chain".into(),
        ));
    }
    let z = p.tip_z();
    if !z.is_finite() {
        return Err(GenError::Invalid(
            "virtual cone tip left the finite horizon".into(),
        ));
    }
    let corner_z = p.corner_z();
    let depth = p.break_depth();
    let probe = LIFT.min(depth * 0.25);
    let mut passes: Vec<Vec<Point2>> = Vec::with_capacity(runs.len());
    let mut offset_rides: Vec<Vec<Point3>> = Vec::with_capacity(runs.len());
    let mut breaks: Vec<EdgeBreak> = Vec::with_capacity(runs.len());
    for pts in runs {
        let tangent = |a: Point2, b: Point2| -> Result<Point2, GenError> {
            let len = (b.x - a.x).hypot(b.y - a.y);
            if !len.is_finite() || len <= 1e-9 {
                return Err(GenError::Empty(
                    "a chamfer edge holds a collapsed segment - re-pick the edge".into(),
                ));
            }
            Ok(Point2::new((b.x - a.x) / len, (b.y - a.y) / len))
        };
        for pair in pts.windows(2) {
            tangent(pair[0], pair[1])?;
        }
        let run = simple(pts).map_err(|e| match e {
            GenError::Empty(_) => {
                GenError::Empty("a chamfer edge collapses to one point - re-pick the edge".into())
            }
            other => other,
        })?;
        let mut levels = Vec::new();
        for at in [corner_z + probe, p.top_z - probe] {
            if let Some(region) = section(at)?
                && !region.is_empty()
            {
                levels.push((at, region));
            }
        }
        let material_left = material_side(&run, &levels, &CHAMFER_WORDS)?.ok_or_else(|| {
            GenError::Invalid(
                "the selected open chamfer edge has no model wall at the programmed chamfer depth - pick an edge on the part, or change Top of Part Z"
                    .into(),
            )
        })?;
        let mut original = run.clone();
        let pass = if p.minimum_cone_radius == 0.0 {
            let mut pass = run.clone();
            if p.climb == material_left {
                pass.reverse();
                original.reverse();
            }
            pass
        } else {
            let air_left = !material_left;
            let Some(inside) = boundary_inside(&run, &levels, air_left)? else {
                return Err(GenError::Invalid(
                    "the selected open model wall cannot classify Side for the tip offset; re-pick a boundary edge".into(),
                ));
            };
            if inside != p.inside {
                let side = if p.inside { "Inside" } else { "Outside" };
                return Err(GenError::Invalid(format!(
                    "Side {side} names the wrong class of selected model wall for the tip offset; use Side Auto or the other side"
                )));
            }
            let offset_left = air_left;
            let offset = offset_run_side(&run, p.minimum_cone_radius, offset_left)?
                .ok_or_else(|| {
                    GenError::Invalid(format!(
                        "the tip offset of {} mm folds onto itself or disappears; pick a gentler run or a smaller offset",
                        p.minimum_cone_radius
                    ))
                })?;
            let offset_forward = !offset_left;
            let desired_forward = p.climb != material_left;
            let mut pass = offset;
            if offset_forward != desired_forward {
                pass.reverse();
                original.reverse();
            }
            pass
        };
        let (Some(&first), Some(&second), Some(&last), Some(&before_last)) = (
            pass.first(),
            pass.get(1),
            pass.last(),
            pass.get(pass.len().saturating_sub(2)),
        ) else {
            return Err(GenError::Empty(
                "a chamfer edge needs at least two points".into(),
            ));
        };
        let entry_air = air_side(tangent(first, second)?, p.climb);
        let exit_air = air_side(tangent(before_last, last)?, p.climb);
        let mut ride = Vec::with_capacity(pass.len() + 2);
        ride.push(Point2::new(
            first.x + entry_air.x * p.width,
            first.y + entry_air.y * p.width,
        ));
        ride.extend(pass.iter().copied());
        ride.push(Point2::new(
            last.x + exit_air.x * p.width,
            last.y + exit_air.y * p.width,
        ));
        if p.minimum_cone_radius > 0.0 {
            let swept = Region::sweep(&[ride.clone()], 1e-3).map_err(GenError::Geo)?;
            for (z, region) in &levels {
                let hit = swept.intersect(region).map_err(GenError::Geo)?;
                if !hit.is_empty()
                    && let Some(q) = hit.loops().first().and_then(|l| l.points().first())
                {
                    return Err(GenError::invalid_at(
                        format!(
                            "the tip offset reaches selected-model material on the air side at Z {z:.3} near ({:.3}, {:.3}); pick a smaller offset or a different Side",
                            q.x, q.y
                        ),
                        Point3::new(q.x, q.y, *z),
                    ));
                }
            }
            offset_rides.extend(offset_rides_for(&original, &pass, p)?);
        } else {
            passes.push(ride);
        }
        breaks.push(EdgeBreak {
            edge: run.iter().map(|q| Point3::new(q.x, q.y, p.top_z)).collect(),
            depth: p.break_depth(),
            slope: p.angle_deg.to_radians().tan(),
            volume: None,
        });
    }

    let mut b = PathBuilder::approached(Point3::new(0.0, 0.0, p.clearance_z));
    if p.minimum_cone_radius == 0.0 {
        let motion = Motion::new(p.clearance_z, p.retract_z, p.feed, p.plunge_feed);
        run_levels(&mut b, &passes, &[z], &motion, None, None, None);
    } else {
        emit_offset_rides(&mut b, &offset_rides, p);
    }
    let cur = b.at();
    if cur.z < p.clearance_z {
        b.push(
            MoveKind::Rapid,
            Point3::new(cur.x, cur.y, p.clearance_z),
            None,
        );
    }
    let mut path = b.finish();
    if !path.all_finite() {
        return Err(GenError::Invalid(
            "open chamfer path left the finite horizon".into(),
        ));
    }
    path.breaks = breaks.into_iter().filter(EdgeBreak::is_sane).collect();
    Ok(path)
}

/// Keep the zero-radius cone for each end cap and cut the translated cone
/// only in the run interior. A vertical plunge at the original endpoint
/// cuts the whole cap; a retract separates it from the different tip plane.
fn offset_rides_for(
    original: &[Point2],
    offset: &[Point2],
    p: &ChamferParams,
) -> Result<Vec<Vec<Point3>>, GenError> {
    let blend = (p.width * (p.width + 2.0 * p.minimum_cone_radius)).sqrt() - p.width;
    let original_length = length(original);
    let offset_length = length(offset);
    if !blend.is_finite() || original_length <= 2.0 * blend || offset_length <= 2.0 * blend {
        return Err(GenError::Invalid(format!(
            "the tip offset of {} mm has no room to join the end caps of this chamfer run; pick a longer edge or a smaller offset",
            p.minimum_cone_radius
        )));
    }
    let middle = section(offset, blend, offset_length - blend)?;
    let Some(first) = original.first() else {
        return Err(GenError::Empty("open chamfer has no first point".into()));
    };
    let Some(last) = original.last() else {
        return Err(GenError::Empty("open chamfer has no last point".into()));
    };
    let band: Vec<_> = middle
        .into_iter()
        .map(|q| Point3::new(q.x, q.y, p.tip_z()))
        .collect();
    Ok(vec![
        vec![Point3::new(first.x, first.y, p.corner_z())],
        band,
        vec![Point3::new(last.x, last.y, p.corner_z())],
    ])
}

fn emit_offset_rides(b: &mut PathBuilder, rides: &[Vec<Point3>], p: &ChamferParams) {
    let hop = p.retract_z.unwrap_or(p.clearance_z).min(p.clearance_z);
    for (index, ride) in rides.iter().enumerate() {
        let Some(first) = ride.first() else {
            continue;
        };
        let above = if index == 0 { p.clearance_z } else { hop };
        b.push(MoveKind::Rapid, Point3::new(first.x, first.y, above), None);
        b.push(MoveKind::Plunge, *first, Some(p.plunge_feed));
        for q in &ride[1..] {
            b.push(MoveKind::Feed, *q, Some(p.feed));
        }
        let Some(last) = ride.last() else {
            continue;
        };
        let above = if index + 1 == rides.len() {
            p.clearance_z
        } else {
            hop
        };
        b.push(MoveKind::Rapid, Point3::new(last.x, last.y, above), None);
    }
}

fn point_at(points: &[Point2], distance: f64) -> Result<Point2, GenError> {
    let Some(&first) = points.first() else {
        return Err(GenError::Empty("chamfer run has no points".into()));
    };
    let Some(&last) = points.last() else {
        return Err(GenError::Empty("chamfer run has no points".into()));
    };
    let total = length(points);
    if distance <= 0.0 {
        return Ok(first);
    }
    if distance >= total {
        return Ok(last);
    }
    let mut walked = 0.0;
    for pair in points.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let len = (b.x - a.x).hypot(b.y - a.y);
        if distance <= walked + len {
            let t = if len > 0.0 {
                (distance - walked) / len
            } else {
                0.0
            };
            return Ok(Point2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t));
        }
        walked += len;
    }
    Ok(last)
}

fn section(points: &[Point2], start: f64, end: f64) -> Result<Vec<Point2>, GenError> {
    let a = point_at(points, start)?;
    let b = point_at(points, end)?;
    let mut section = vec![a];
    let mut walked = 0.0;
    for pair in points.windows(2) {
        let next = walked + (pair[1].x - pair[0].x).hypot(pair[1].y - pair[0].y);
        if next > start && next < end && section.last().is_none_or(|q| !same_point(*q, pair[1])) {
            section.push(pair[1]);
        }
        walked = next;
    }
    if section.last().is_none_or(|q| !same_point(*q, b)) {
        section.push(b);
    }
    Ok(section)
}

fn same_point(a: Point2, b: Point2) -> bool {
    (a.x - b.x).powi(2) + (a.y - b.y).powi(2) <= 1e-20
}

fn length(points: &[Point2]) -> f64 {
    points
        .windows(2)
        .map(|p| (p[1].x - p[0].x).hypot(p[1].y - p[0].y))
        .sum()
}
