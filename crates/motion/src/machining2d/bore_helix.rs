//! Helical boring (#2830): one continuous helix on the wall radius from
//! above the material to the hole floor, one flat circle at the floor,
//! an optional spring pass, then a move off the wall to the centre.
//!
//! Provenance: helical interpolation of a counterbore or a hole with an
//! end mill smaller than the hole - the cutter's rim on the finished
//! wall, the axis descending a fixed pitch per turn, a full circle at
//! the floor to remove the ramp step - is standard milling practice
//! described in machine-tool handbooks. Written clean-room from that
//! description and the issue's own use case; no vendor material
//! consulted.
//!
//! The law in one line: the cutter centre rides `rc = hole_radius -
//! tool_radius - leave`, the same wall circle as the ring ladder, and
//! descends `stepdown` per turn. One helix clears the whole floor only
//! while its inner edge reaches the centre (`rc <= floor reach`); a
//! wider hole would leave a core (a loose slug in a through hole), so
//! it is refused by name before any move is built. A helix steeper
//! than [`MAX_HELIX_ANGLE_DEG`] is a plunge in all but name, so it is
//! refused too, with the pitch that would fit. A cutter that is not
//! centre-cutting has an uncut core round its axis (`plunge_law::CORE`);
//! the helix cuts the hole's axis only while its wall circle stands at
//! least that core radius off it, so a smaller hole is refused for such
//! a cutter.

use std::f64::consts::PI;

use crate::cl::{lattice, MoveKind, PathBuilder, Toolpath};
use crate::geometry::Point3;

use crate::plunge_law::CORE;

use super::bore::{BoreHole, BoreParams};
use super::bore_core::{bore_floor_reach, CORE_TOL};
use super::face::GenError;
use super::pass_start::PassStart;

/// How a Bore operation cuts each hole.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BoreStrategy {
    /// The level-by-level ring ladder: a dive at the centre, then rings
    /// out to the wall at each stepdown level. Every bore before #2830.
    #[default]
    Rings,
    /// One continuous helix on the wall radius to the floor, then a flat
    /// floor circle. `spring_pass` adds a second floor circle with no
    /// extra depth.
    Helical { spring_pass: bool },
}

/// The steepest helix the strategy posts, in degrees from the
/// horizontal. Steeper, the cutter's whole bottom face meets the
/// material at once - the plunge the strategy exists to avoid.
pub const MAX_HELIX_ANGLE_DEG: f64 = 15.0;

/// The widest hole one helix clears to the centre: the cutter's own
/// diameter, twice the wall allowance, and twice its floor reach (the
/// flat bottom of a blind hole, the corner's reach at the floor plane of
/// a through hole).
#[must_use]
pub fn max_helix_bore_diameter(tool_diameter: f64, corner_radius: f64, leave: f64, through_allowance: Option<f64>) -> f64 {
    tool_diameter + 2.0 * leave + 2.0 * bore_floor_reach(tool_diameter, corner_radius, through_allowance)
}

/// Does one helix leave a core at the floor of this hole?
#[must_use]
pub fn helix_leaves_core(hole_radius: f64, tool_diameter: f64, corner_radius: f64, leave: f64, through_allowance: Option<f64>) -> bool {
    let rc = hole_radius - (tool_diameter * 0.5 + leave);
    rc > bore_floor_reach(tool_diameter, corner_radius, through_allowance) + CORE_TOL
}

/// The smallest hole a helix cuts with a cutter that is not
/// centre-cutting: the wall circle must stand at least the uncut core
/// (`CORE` of the radius) off the hole's axis, or the core rides over
/// the solid there for the whole descent.
#[must_use]
pub fn min_helix_bore_diameter_no_center_cut(tool_diameter: f64, leave: f64) -> f64 {
    tool_diameter + 2.0 * leave + CORE * tool_diameter
}

/// Why a helical bore refuses `start` (#3360), or `None`. A helix has no
/// rings, so Inside and Outside are refused by name, never ignored.
#[must_use]
pub fn helical_pass_start_refusal(strategy: BoreStrategy, start: PassStart) -> Option<String> {
    (strategy != BoreStrategy::Rings && start != PassStart::Auto).then(|| {
        format!(
            "Pass starts at {} is not available with the Helical strategy - one helix per hole has no ring order; use Auto or Rings",
            start.label()
        )
    })
}

/// The hole's helical doors (the ring ladder's mouth door has run
/// before): no core at the floor, and a helix no steeper than the limit.
pub(super) fn check(p: &BoreParams, h: &BoreHole, mouth: Point3) -> Result<(), GenError> {
    let through = h.through.then_some(p.through_allowance);
    let rc = h.radius - (p.tool_diameter * 0.5 + p.stock_to_leave);
    if helix_leaves_core(h.radius, p.tool_diameter, p.corner_radius, p.stock_to_leave, through) {
        let reach = bore_floor_reach(p.tool_diameter, p.corner_radius, through);
        return Err(GenError::invalid_at(
            format!(
                "the {:.3} mm hole at ({:.3}, {:.3}) is too wide for one helix with a {:.3} mm cutter: \
                 the helix leaves a {:.3} mm core at the floor. The widest hole one helix clears is {:.3} mm. \
                 Use the Rings strategy, or a larger tool",
                h.radius * 2.0,
                tidy(h.xy.x),
                tidy(h.xy.y),
                p.tool_diameter,
                2.0 * (rc - reach),
                max_helix_bore_diameter(p.tool_diameter, p.corner_radius, p.stock_to_leave, through),
            ),
            mouth,
        ));
    }
    // A cutter that is not centre-cutting: its uncut core must stay off
    // the hole's axis. The axis is the worst point - it stays `rc` from
    // the cutter's axis for the whole helix, the floor circle and the
    // move to the centre. Refused, not proved clear by the stock: no
    // pre-hole is known here (a conservative door).
    let core = CORE * p.tool_diameter * 0.5;
    if !p.links.approach.center_cutting && rc + CORE_TOL < core {
        return Err(GenError::invalid_at(
            format!(
                "the {:.3} mm hole at ({:.3}, {:.3}) is too small for a helix with a {:.3} mm cutter that is not \
                 centre-cutting: the helix radius {:.3} mm is inside the cutter's {:.3} mm uncut core, which would \
                 run into the solid at the hole axis. The smallest hole this cutter helixes is {:.3} mm. \
                 Use a centre-cutting tool, or a smaller tool",
                h.radius * 2.0,
                tidy(h.xy.x),
                tidy(h.xy.y),
                p.tool_diameter,
                rc,
                core,
                min_helix_bore_diameter_no_center_cut(p.tool_diameter, p.stock_to_leave),
            ),
            mouth,
        ));
    }
    let circumference = 2.0 * PI * rc;
    let angle = p.stepdown.atan2(circumference).to_degrees();
    if angle > MAX_HELIX_ANGLE_DEG + 1e-9 {
        let fit = circumference * MAX_HELIX_ANGLE_DEG.to_radians().tan();
        return Err(GenError::invalid_at(
            format!(
                "the helix in the {:.3} mm hole at ({:.3}, {:.3}) descends at {angle:.1} deg, steeper than \
                 {MAX_HELIX_ANGLE_DEG:.0} deg: set the stepdown (the helix pitch) to {fit:.3} mm or less, \
                 or use the Rings strategy",
                h.radius * 2.0,
                tidy(h.xy.x),
                tidy(h.xy.y),
            ),
            mouth,
        ));
    }
    Ok(())
}

/// A coordinate as a refusal prints it: noise under the printed
/// precision reads 0.000, never -0.000.
fn tidy(v: f64) -> f64 {
    if v.abs() < 5e-4 { 0.0 } else { v }
}

fn pt(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(lattice(x), lattice(y), lattice(z))
}

/// Helical boring over a validated hole list. Each hole: rapid over the
/// helix start at clearance, rapid down to the plunge height over the
/// material, one helix to the floor (arcs of at most half a turn), the
/// floor circle (and the spring pass), a feed to the centre, a rapid up
/// to clearance.
pub(super) fn ride(p: &BoreParams, spring_pass: bool) -> Result<Toolpath, GenError> {
    let r = p.tool_diameter * 0.5;
    let hop = p.retract_z.unwrap_or(p.clearance_z);
    let a = &p.links.approach;
    let helix_feed = if a.ramp_feed > 0.0 { a.ramp_feed } else { p.feed };
    let sense = if p.climb { 1.0 } else { -1.0 };
    let mut b = PathBuilder::approached(Point3::new(0.0, 0.0, p.clearance_z));

    for h in &p.holes {
        let rc = h.radius - (r + p.stock_to_leave);
        let bottom = if h.through { h.floor_z - p.through_allowance } else { h.floor_z };
        // The material top over this hole: its entry, or the stock over
        // it when the application knows the stock stands higher (#2004).
        let top = a.stock_top.map_or(h.top_z, |s| s.max(h.top_z));
        // The helix starts in the air: the plunge height over the
        // material, never over the hop plane (0 starts it at the hop).
        let z0 = if a.plunge_height > 0.0 { (top + a.plunge_height).min(hop) } else { hop };
        let at = |theta: f64, z: f64| pt(h.xy.x + rc * (sense * theta).cos(), h.xy.y + rc * (sense * theta).sin(), z);
        let start = at(0.0, z0);
        b.push(MoveKind::Rapid, Point3::new(start.x, start.y, p.clearance_z), None);
        b.push(MoveKind::Rapid, start, None);

        // The helix: a continuous descent of `stepdown` per turn, in arcs
        // of at most half a turn so every arc posts as one G2/G3.
        let drop = (z0 - bottom).max(0.0);
        let sweep = 2.0 * PI * drop / p.stepdown;
        let n = (sweep / PI).ceil().max(1.0);
        if !n.is_finite() || n > 1.0e6 {
            return Err(GenError::Invalid("the helix needs too many turns: raise the stepdown".into()));
        }
        let n = n as usize;
        let ccw = p.climb;
        for k in 1..=n {
            let f = k as f64 / n as f64;
            let z = if k == n { bottom } else { z0 - drop * f };
            b.push_arc(MoveKind::Feed, at(sweep * f, z), Some(helix_feed), h.xy, ccw);
        }
        // The floor circle removes the ramp step; the spring pass rides
        // it again with no extra depth. Two half turns each.
        let end = sweep;
        let circles = if spring_pass { 2 } else { 1 };
        for c in 0..circles {
            for half in 1..=2 {
                let theta = end + 2.0 * PI * c as f64 + PI * half as f64;
                b.push_arc(MoveKind::Feed, at(theta, bottom), Some(p.feed), h.xy, ccw);
            }
        }
        // Off the wall to the centre (cleared: rc <= floor reach), then up.
        b.push(MoveKind::Feed, pt(h.xy.x, h.xy.y, bottom), Some(p.feed));
        b.push(MoveKind::Rapid, Point3::new(lattice(h.xy.x), lattice(h.xy.y), p.clearance_z), None);
    }
    Ok(b.finish())
}
