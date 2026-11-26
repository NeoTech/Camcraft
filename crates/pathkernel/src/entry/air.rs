//! The entry in air (#3746): the tool feeds down in the air outside the
//! stock, clear of it by the approach distance, then moves in the cut plane
//! onto the start of the cut - a straight lead in air where one is needed,
//! then one arc of the asked radius and sweep, tangent to the cut at its
//! start. The arc and the lead stay in the legal room; the straight lead
//! does not enter the cutter-centre area. Where no arc fits at a smaller
//! radius either, a cut that starts in the air zone is entered straight
//! down; else the stage tries the next kind.
//!
//! Provenance: the tangential arc lead-in of general machining practice
//! (RS-274 / ISO 6983 G17 G2/G3), entered from outside the stock as public
//! machining handbooks describe it. A port of the rule of
//! `crates/motion/src/machining2d/air_entry.rs` (#3746, #3810) onto the
//! region of K1; the old code reads a stock box, this reads the air band.

use super::proof::{Area, Entry, Lead, Proof};
use super::room::{Ctx, ROOM_TOL, air_zone, geo};
use super::EntryKind;
use camcraft_geometry::geo2d::arc_points;
use crate::{Point2, Region};

/// The radii tried for the arc: the asked one, then halving.
const RADIUS_TRIES: usize = 5;
/// The steps of the search back along the arc's tangent for a descent in air.
const MARCH_STEPS: u32 = 512;

/// The arc of `radius` and `sweep` that ends at `s` tangent to `u`, turning
/// counter-clockwise when `ccw`: its start and its points (the last is `s`).
pub(super) fn arc_onto(s: Point2, u: Point2, radius: f64, sweep: f64, ccw: bool, tol: f64) -> Result<(Point2, Point2, Vec<Point2>), crate::Refusal> {
    let n = if ccw { Point2::new(-u.y, u.x) } else { Point2::new(u.y, -u.x) };
    let c = Point2::new(s.x + radius * n.x, s.y + radius * n.y);
    let back = if ccw { -sweep } else { sweep };
    let (vx, vy) = (s.x - c.x, s.y - c.y);
    let (sin, cos) = back.sin_cos();
    let a = Point2::new(c.x + vx * cos - vy * sin, c.y + vx * sin + vy * cos);
    let mut pts = geo(arc_points(c, a, -back, tol))?;
    if let Some(last) = pts.last_mut() {
        *last = s;
    }
    Ok((a, c, pts))
}

/// The direction of travel at `a` on the arc about `c`.
fn tangent(a: Point2, c: Point2, ccw: bool) -> Option<Point2> {
    let (vx, vy) = (a.x - c.x, a.y - c.y);
    super::room::unit(if ccw { Point2::new(-vy, vx) } else { Point2::new(vy, -vx) })
}

/// The first point in `zone` back along `-t` from `a`, within `reach`.
fn march(zone: &Region, a: Point2, t: Point2, reach: f64) -> Option<Point2> {
    let step = reach / f64::from(MARCH_STEPS);
    (1..=MARCH_STEPS).map(|i| f64::from(i) * step).map(|l| Point2::new(a.x - t.x * l, a.y - t.y * l)).find(|q| zone.contains(*q))
}

/// The air entry at the start of the first cut, if one fits.
pub(super) fn enter(ctx: &mut Ctx<'_>) -> Result<Option<Entry>, crate::Refusal> {
    let p = ctx.region;
    if p.air().is_empty() {
        return Ok(None);
    }
    let (zone, capped) = air_zone(p, ctx.choice.approach())?;
    let Ok(zone_area) = Area::new(zone.clone()) else {
        return Ok(None);
    };
    let inner = geo(p.center().offset(-ROOM_TOL))?;
    let into_center = |a: Point2, b: Point2| match inner.clip_polyline(&[a, b], true) {
        Ok(pieces) => pieces.iter().any(|s| s.windows(2).any(|w| (w[1].x - w[0].x).hypot(w[1].y - w[0].y) > ROOM_TOL)),
        Err(_) => true,
    };
    let (s, u, arc, tol) = (ctx.start.at(), ctx.dir, ctx.stage.arc(), ctx.stage.chord_tol());
    let reach = 3.0 * ctx.r() + ctx.choice.approach() + 2.0 * arc.radius();
    let mut radius = arc.radius();
    let mut with_line: Option<(Point2, Vec<Point2>, f64)> = None;
    for _ in 0..RADIUS_TRIES {
        for ccw in [true, false] {
            let (a, c, pts) = arc_onto(s, u, radius, arc.sweep(), ccw, tol)?;
            if !ctx.holds(&pts) {
                continue;
            }
            if zone.contains(a) {
                return found(ctx, a, pts, radius, capped, zone_area);
            }
            if with_line.is_some() {
                continue;
            }
            let Some(q) = tangent(a, c, ccw).and_then(|t| march(&zone, a, t, reach)) else { continue };
            if ctx.holds(&[q, a]) && !into_center(q, a) {
                let mut lead = vec![q];
                lead.extend(pts);
                with_line = Some((q, lead, radius));
            }
        }
        if with_line.is_some() {
            break;
        }
        radius *= 0.5;
    }
    if let Some((q, lead, radius)) = with_line {
        return found(ctx, q, lead, radius, capped, zone_area);
    }
    if zone.contains(s) {
        let entry = Entry::new(EntryKind::Air, vec![ctx.above(s)?], vec![ctx.start], Proof::InAir(zone_area), ctx.choice, ctx.tool());
        return Ok(entry.ok());
    }
    Ok(None)
}

/// The arc-lead entry: down at `at` in the air zone, then the lead `xy`.
fn found(ctx: &mut Ctx<'_>, at: Point2, xy: Vec<Point2>, radius: f64, capped: Option<f64>, zone: Area) -> Result<Option<Entry>, crate::Refusal> {
    let pts = xy.iter().map(|q| ctx.at_depth(*q)).collect::<Result<Vec<_>, _>>()?;
    let Ok(lead) = Lead::new(pts, &ctx.legal_area) else {
        return Ok(None);
    };
    let (approach, descent) = (vec![ctx.above(at)?], vec![ctx.at_depth(at)?]);
    let Ok(entry) = Entry::led(EntryKind::ArcLead, approach, descent, Some(lead), Proof::InAir(zone), ctx.choice, ctx.tool()) else {
        return Ok(None);
    };
    if let Some(a) = capped {
        ctx.note(format!("entry: the approach distance is limited to {a:.3} mm by the air band round the stock"))?;
    }
    if radius < ctx.stage.arc().radius() {
        ctx.note(format!("entry: the arc lead fits only under its radius - flown at {radius:.3} mm"))?;
    }
    Ok(Some(entry))
}
