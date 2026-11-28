//! The entries into material, used only when the user chose one: a helix
//! that ends tangent to the first cut at its start, a ramp along the first
//! cut, and a plunge (only when asked and the tool can plunge). The helix
//! and the ramp descend at no more than the user's descent angle. The helix
//! circle and the ramp stay in the legal room of the tool centre.
//! Provenance: helical and ramp entries as in public machining handbooks
//! (Machinery's Handbook, milling entry practice); the limits of the old
//! planners (`links.rs` `MAX_HELIX_TURNS`, `contour_link.rs`
//! `MAX_RAMP_LAPS`) kept as caps.

use super::proof::{Area, Entry, Proof};
use super::room::{Ctx, ROOM_TOL, disc, geo, path_of};
use super::EntryKind;
use crate::frame::{PlanePoint, Positive};
use crate::verdict::Stage;
use crate::{KERNEL_TOL, Point2, Region};
use camcraft_geometry::geo2d::arc_points;
use std::f64::consts::TAU;

/// The helix radii tried: the asked one, then halving.
const HELIX_TRIES: usize = 4;
/// The most turns of one helix (the old planners' cap).
const MAX_HELIX_TURNS: f64 = 1024.0;
/// The most passes of one ramp along the first cut (two for each of the
/// old planners' 64 laps).
const MAX_RAMP_PASSES: f64 = 128.0;
/// The most points of one descent.
const MAX_DESCENT_POINTS: usize = 1 << 18;

/// The height of the descent: from the descent start down to the cut.
fn height(ctx: &Ctx<'_>) -> f64 {
    ctx.stage.heights().start() - ctx.depth()
}

/// The helix entry, if one fits.
pub(super) fn helix(ctx: &mut Ctx<'_>) -> Result<Option<Entry>, crate::Refusal> {
    let m = ctx.choice.material();
    let (r, h, tan) = (ctx.r(), height(ctx), m.angle().to_radians().tan());
    let (s, u, tol) = (ctx.start.at(), ctx.dir, ctx.stage.chord_tol());
    let asked = m.helix_radius().unwrap_or(0.5 * r);
    let mut radius = asked;
    for _ in 0..HELIX_TRIES {
        if radius <= r + KERNEL_TOL {
            for ccw in [true, false] {
                let n = if ccw { Point2::new(-u.y, u.x) } else { Point2::new(u.y, -u.x) };
                let c = Point2::new(s.x + radius * n.x, s.y + radius * n.y);
                let ring = geo(arc_points(c, s, if ccw { TAU } else { -TAU }, tol))?;
                if !ctx.holds(&ring)? {
                    continue;
                }
                let Some(descent) = helix_points(&ring, ctx.stage.heights().start(), h, TAU * radius * tan)? else { continue };
                let pitch = Positive::new(Stage::Entry, TAU * radius * tan)?;
                let proof = Proof::IntoMaterial { area: disc(c, radius + ROOM_TOL)?, angle_or_pitch: pitch };
                let made = Entry::new(EntryKind::Helix, vec![ctx.above(s)?], descent, proof, ctx.choice, ctx.tool());
                if let Some(e) = ctx.checked(EntryKind::Helix, made) {
                    if radius < asked {
                        ctx.note(format!("entry: the helix fits only under its radius - flown at {radius:.3} mm"))?;
                    }
                    return Ok(Some(e));
                }
            }
        }
        radius *= 0.5;
    }
    Ok(None)
}

/// The points of a helix on `ring` (a full turn from the cut start back to
/// it) from depth `top` down by `h` at `pitch` per turn; it ends on the cut
/// start. `None` above the caps (the helix does not fit); a point that
/// is not finite is refused by name (#4407), not taken as "does not fit".
fn helix_points(ring: &[Point2], top: f64, h: f64, pitch: f64) -> Result<Option<Vec<PlanePoint>>, crate::Refusal> {
    let Some(per) = ring.len().checked_sub(1).filter(|n| *n > 0) else { return Ok(None) };
    let turns = (h / pitch).ceil().max(1.0);
    if !(turns.is_finite() && turns <= MAX_HELIX_TURNS) {
        return Ok(None);
    }
    // `turns` is a whole number in [1, MAX_HELIX_TURNS]: the cast is exact.
    let Some(total) = (turns as usize).checked_mul(per).filter(|t| *t <= MAX_DESCENT_POINTS) else { return Ok(None) };
    let step = h / total as f64;
    (1..=total).map(|j| PlanePoint::new(ring[j % per], top - step * j as f64)).collect::<Result<Vec<_>, _>>().map(Some)
}

/// The ramp entry along the first cut, if one fits.
pub(super) fn ramp(ctx: &mut Ctx<'_>) -> Result<Option<Entry>, crate::Refusal> {
    let path = path_of(ctx.cut);
    let len: f64 = path.windows(2).map(|w| (w[1].x - w[0].x).hypot(w[1].y - w[0].y)).sum();
    let deg = ctx.choice.material().angle();
    let drop = len * deg.to_radians().tan();
    if !(drop.is_finite() && drop > KERNEL_TOL) || !ctx.holds(&path)? {
        return Ok(None);
    }
    let closed = ctx.cut.curve().closed();
    let mut passes = (height(ctx) / drop).ceil().max(1.0);
    if !closed && passes % 2.0 == 1.0 {
        passes += 1.0;
    }
    if passes > MAX_RAMP_PASSES || path.len().saturating_mul(passes as usize) > MAX_DESCENT_POINTS {
        return Ok(None);
    }
    let (top, total) = (ctx.stage.heights().start(), passes * len);
    let mut walked = 0.0;
    let mut prev = path.first().copied().unwrap_or(ctx.start.at());
    let mut descent = Vec::new();
    // `passes` is a whole number in [1, MAX_RAMP_PASSES]: the cast is exact.
    for i in 0..passes as usize {
        let leg: Vec<Point2> = if closed || i % 2 == 0 { path.clone() } else { path.iter().rev().copied().collect() };
        for q in leg.into_iter().skip(1) {
            walked += (q.x - prev.x).hypot(q.y - prev.y);
            descent.push(PlanePoint::new(q, top - height(ctx) * (walked / total).min(1.0))?);
            prev = q;
        }
    }
    if let Some(last) = descent.last_mut() {
        *last = ctx.start;
    }
    let area = Area::new(geo(Region::sweep(&[path], ROOM_TOL))?)?;
    let proof = Proof::IntoMaterial { area, angle_or_pitch: Positive::new(Stage::Entry, deg)? };
    let made = Entry::new(EntryKind::Ramp, vec![ctx.above(ctx.start.at())?], descent, proof, ctx.choice, ctx.tool());
    Ok(ctx.checked(EntryKind::Ramp, made))
}

/// The plunge at the cut start: only when the user asked for it and the
/// tool can plunge.
pub(super) fn plunge(ctx: &mut Ctx<'_>) -> Result<Option<Entry>, crate::Refusal> {
    if ctx.choice.asked() != EntryKind::Plunge || !ctx.tool().can_plunge() {
        return Ok(None);
    }
    let s = ctx.start.at();
    let proof = Proof::IntoMaterial { area: disc(s, ROOM_TOL)?, angle_or_pitch: Positive::new(Stage::Entry, 90.0)? };
    let made = Entry::new(EntryKind::Plunge, vec![ctx.above(s)?], vec![ctx.start], proof, ctx.choice, ctx.tool());
    Ok(ctx.checked(EntryKind::Plunge, made))
}
