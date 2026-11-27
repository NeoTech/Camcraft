//! The arcs of a cut whose curve holds an exact circle: the plan's points
//! must lie on the circle at one depth and turn one way round its centre;
//! the arcs end on plan points, each a quarter turn or a little more, so a
//! full ring is never one arc with the same start and end (the builder
//! drops a zero-sweep arc).
//! Provenance: circular interpolation G2/G3 of RS-274 / ISO 6983 (an arc is
//! its end point, its centre and its sense); our own design record
//! `docs/decisions/4269-kernel-plan-converter.md`.

use super::pen::EMIT_TOL;
use super::{ARC_NOT_MONOTONE, ARC_OFF_CIRCLE};
use crate::geometry::Point3;
use camcraft_pathkernel::guide::Circle;

/// How far (mm) a point of a circle cut may be off the circle: the chord
/// points of the guide stand on the circle, so this is a rounding margin.
pub(super) const ARC_TOL: f64 = 1e-3;

/// The arc ends after this much turn (radians): a quarter turn.
const RUN: f64 = std::f64::consts::FRAC_PI_2;

/// The arcs along `pts` (the first point is where the tool stands; a closed
/// curve repeats its start at the end): each item is an arc end point and
/// its sense (true = counter-clockwise, G3).
pub(super) fn arc_runs(pts: &[Point3], c: Circle) -> Result<Vec<(Point3, bool)>, &'static str> {
    let (o, r) = (c.center(), c.radius());
    let Some(z) = pts.first().map(|p| p.z) else {
        return Err(ARC_OFF_CIRCLE);
    };
    let off = |p: &Point3| ((p.x - o.x).hypot(p.y - o.y) - r).abs() > ARC_TOL || (p.z - z).abs() > EMIT_TOL;
    if pts.iter().any(off) {
        return Err(ARC_OFF_CIRCLE);
    }
    let mut sense: Option<bool> = None;
    let mut turned = 0.0f64;
    let mut pending: Option<(Point3, bool)> = None;
    let mut out = Vec::new();
    // A chord shorter than the tolerance (a repeated point) adds no turn:
    // the arc runs on to the next point.
    for w in pts.windows(2).filter(|w| (w[1].x - w[0].x).hypot(w[1].y - w[0].y) > EMIT_TOL) {
        let (a, b) = (w[0], w[1]);
        let (ax, ay, bx, by) = (a.x - o.x, a.y - o.y, b.x - o.x, b.y - o.y);
        let cross = ax * by - ay * bx;
        let dot = ax * bx + ay * by;
        let ccw = cross > 0.0;
        if cross.abs() <= f64::EPSILON * r * r || sense.is_some_and(|s| s != ccw) {
            return Err(ARC_NOT_MONOTONE);
        }
        sense = Some(ccw);
        turned += cross.abs().atan2(dot);
        pending = Some((b, ccw));
        if turned >= RUN {
            out.extend(pending.take());
            turned = 0.0;
        }
    }
    out.extend(pending);
    if out.is_empty() {
        return Err(ARC_NOT_MONOTONE);
    }
    Ok(out)
}
