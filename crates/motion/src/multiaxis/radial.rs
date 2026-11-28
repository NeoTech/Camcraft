//! The rotary kinds' radial heights (#2774): a height on a turned part is a
//! RADIUS about the fitted axis. This module holds the resolved numbers the
//! generators read and the contour's axial radial-relink law.
//!
//! The parallel and wrap ladders already build their links radially in their
//! laid/developed frames ([`super::rotary_links::radial_relink`]); the
//! contour's ladder lifts along the AXIS, so [`radial_relink_z`] rebuilds
//! those lifts as radial retracts and plunges about the same line.
//!
//! Provenance: radial retract / index / infeed linking of 4-axis rotary
//! milling from general machining practice; the star-shaped ball-dilation
//! argument is elementary geometry. Nothing of any vendor's consulted.

use crate::cl::{Move, MoveKind, Toolpath};
use crate::geometry::Point3;
use crate::machining2d::{GenError, ARC_FIT_TOL};

/// The radial heights of one rotary op, resolved to millimetres about the
/// fitted axis. `None` keeps the op's Z-plane law on that field.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RadialHeights {
    pub clearance: Option<f64>,
    pub retract: Option<f64>,
    pub top: Option<f64>,
    pub bottom: Option<f64>,
}

impl RadialHeights {
    /// No radial field: the pre-#2774 Z-plane behavior, byte for byte.
    pub const NONE: Self = RadialHeights { clearance: None, retract: None, top: None, bottom: None };

    /// Whether any field switched the op to cylinders.
    #[must_use]
    pub fn any(&self) -> bool {
        self.clearance.is_some() || self.retract.is_some() || self.top.is_some() || self.bottom.is_some()
    }
}

/// Rebuild a standing rotary ladder's links RADIAL about the axis (#2774).
/// `base` is the contour's planning-frame path in its vertical-ball tip
/// convention: a stored point is the TIP, and its ball center stands `r` up
/// the frame's own z (the axis), so a point's radius about the axis is
/// `hypot(x, y)`. `clear`/`hop` are STATED TIP radii about that axis - the
/// radii the operator named. The pose law of `posed` backs every emitted
/// point off one ball radius along the pose it holds, so each link places
/// the BALL CENTER one ball radius OUTSIDE the stated radius (the same
/// tip-to-center conversion `rotary_links::radial_relink` applies to the
/// parallel twin): the posed tip then stands at, never one ball under, the
/// stated radius. Each link becomes a radial retract to the `hop` cylinder,
/// an index on it, and a radial plunge into the next cut; the cut moves are
/// kept verbatim. Refuses by name when a radius does not clear the work.
#[allow(clippy::too_many_arguments)] // one relink's flat read
pub(super) fn radial_relink_z(
    base: &Toolpath,
    r: f64,
    clear: f64,
    hop: f64,
    rho_part: f64,
    feed: f64,
    plunge_feed: f64,
    kind: &str,
) -> Result<Toolpath, GenError> {
    for (name, v) in [("clearance", clear), ("retract", hop)] {
        if !(v.is_finite() && v > rho_part + ARC_FIT_TOL) {
            return Err(GenError::Invalid(format!(
                "{kind} reads its {name} height as a radius about the bar's axis: {v:.3} mm does not clear the work's {rho_part:.3} mm radius - raise it"
            )));
        }
    }
    let ms = &base.moves;
    let link = |i: usize| -> bool {
        let m = &ms[i];
        match m.kind {
            MoveKind::Rapid | MoveKind::Plunge => true,
            // The lift off a pass: straight up the frame's z (the axis),
            // into air, then a rapid across.
            MoveKind::Feed => {
                m.arc.is_none()
                    && m.from.x == m.to.x
                    && m.from.y == m.to.y
                    && m.to.z > m.from.z
                    && ms.get(i + 1).is_some_and(|n| n.kind == MoveKind::Rapid)
            }
        }
    };
    let mut runs: Vec<&[Move]> = Vec::new();
    let mut i = 0usize;
    while i < ms.len() {
        if link(i) {
            i += 1;
            continue;
        }
        let s = i;
        while i < ms.len() && !link(i) {
            i += 1;
        }
        runs.push(&ms[s..i]);
    }
    let (Some(first), Some(last)) = (runs.first(), runs.last()) else {
        return Err(GenError::Empty(format!("the {kind} ladder left no pass to link")));
    };
    let (Some(b0), Some(end)) = (first.first().map(|m| m.from), last.last().map(|m| m.to)) else {
        return Err(GenError::Empty(format!("the {kind} ladder left no pass to link")));
    };
    let mut out: Vec<Move> = Vec::with_capacity(ms.len());
    let start = lift(b0, r, clear)?;
    out.push(Move::new(MoveKind::Plunge, start, b0, Some(plunge_feed)));
    for (k, run) in runs.iter().enumerate() {
        if k > 0
            && let (Some(a), Some(b)) = (out.last().map(|m| m.to), run.first().map(|m| m.from))
        {
            link_into(&mut out, a, b, r, hop, feed, plunge_feed)
                .map_err(|e| e.with_ghost_moves(&out))?;
        }
        out.extend(run.iter().cloned());
    }
    let exit = lift(end, r, clear).map_err(|e| e.with_ghost_moves(&out))?;
    out.push(Move::new(MoveKind::Rapid, end, exit, None));
    let mut path = Toolpath::new(out);
    path.notes = base.notes.clone();
    Ok(path)
}

/// Retract from tip `a` along its radial to the `hop` cylinder, index on it,
/// then plunge along tip `b`'s radial into `b`. `hop` is the stated TIP
/// radius: the index ring rides the ball CENTER one ball radius outside it,
/// so the posed tips stand on the stated cylinder (`posed` backs each point
/// off one ball radius along the pose).
fn link_into(out: &mut Vec<Move>, a: Point3, b: Point3, r: f64, hop: f64, feed: f64, plunge_feed: f64) -> Result<(), GenError> {
    let ua = lift(a, r, hop)?;
    let ub = lift(b, r, hop)?;
    out.push(Move::new(MoveKind::Feed, a, ua, Some(feed)));
    let rc = hop + r;
    let (pa, pb) = (polar(ua), polar(ub));
    let dphi = (pb.1 - pa.1 + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI;
    let cap = 2.0 * (1.0 - (ARC_FIT_TOL / rc).min(2.0)).clamp(-1.0, 1.0).acos();
    let n = if cap > 1e-12 { (dphi.abs() / cap).ceil() as usize } else { 1 };
    let n = n.clamp(1, 8192);
    let mut at = ua;
    for j in 1..=n {
        let t = j as f64 / n as f64;
        let to = if j == n {
            ub
        } else {
            let (z, ph) = (pa.0 + (pb.0 - pa.0) * t, pa.1 + dphi * t);
            Point3::new(rc * ph.cos(), rc * ph.sin(), z)
        };
        if to != at {
            out.push(Move::new(MoveKind::Rapid, at, to, None));
            at = to;
        }
    }
    out.push(Move::new(MoveKind::Plunge, ub, b, Some(plunge_feed)));
    Ok(())
}

/// The stored point whose ball center stands one ball radius OUTSIDE tip
/// radius `rho` about the axis: the center's radius is `rho + r`, so the
/// posed tip (`posed` backs the point off `r` along its radial) stands at
/// `rho` - the radius the operator stated, never one ball under it.
fn lift(p: Point3, r: f64, rho: f64) -> Result<Point3, GenError> {
    let l = p.x.hypot(p.y);
    if !(l.is_finite() && l > 1e-9) {
        return Err(GenError::Invalid(format!(
            "a rotary pass ends on the axis line at {p:?} - no radius to retract along"
        )));
    }
    let k = (rho + r) / l;
    Ok(Point3::new(p.x * k, p.y * k, p.z))
}

/// (station, azimuth) of tip `p`.
fn polar(p: Point3) -> (f64, f64) {
    (p.z, p.y.atan2(p.x))
}