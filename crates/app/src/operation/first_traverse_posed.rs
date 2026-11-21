//! #3084: generate's seam to the posed first-traverse law (`verify::first_traverse_low_posed`): which operations, on which mounted machines, read their work through every pose of the machine's work-turning rotaries; our own reading of the shop rule "never traverse through uncut stock or steel", no vendor material consulted.
//!
//! The post sends Z, then X/Y at that height, then the move with the
//! rotary words (`traverseZ`, `xyFirst`). Before that move the
//! work-turning rotaries stand where the previous program left them.
//! The #2965 laws read one fitted axis (the rotary kinds) or the part
//! frame (the multi-axis tip law), so they miss:
//! 1. a rotary kind on a machine where two rotaries turn the work;
//! 2. a multi-axis kind on a machine whose table tilts the work.
//!
//! Each line of a work-turning rotary passes through the WCS origin
//! (the indexed post's setup law, as the index-swing clause reads it).
//! A stated non-zero `pivot` or `offset` moves it off: refused (#3127).
//! Decision #3084 has the premises and the limits.

use crate::cl::{TiltedFrame, Toolpath};
use crate::fixture::{FixtureBox, MACHINE_TABLE};
use crate::geometry::Point3;
use crate::machine::AxisLimits;
use crate::posting::index_swing::{RotaryLine, off_origin, rotary_line};
use crate::verify::{self, PosedTurn, Warning};

use super::OpParams;
use super::kinds::Section;
use super::material::Z_TOL;

/// What the posed law reads: the pose set, the work's points (world),
/// the setup's steel and its names (`fixture::named_boxes_for`), and the
/// operation's clearance height in the path's frame.
pub(super) struct Posed<'a> {
    pub turn: Option<PosedTurn>,
    pub rides: &'a [Point3],
    pub fixtures: &'a [FixtureBox],
    pub names: &'a [String],
    pub clearance: f64,
}

/// A rotary whose line can lean the work: a stated axis with an X or Y
/// part, or no stated axis (the rule of `first_traverse::tilting_all`).
fn tilts(a: &AxisLimits) -> bool {
    a.axis.is_none_or(|d| d[0].abs() > 1e-9 || d[1].abs() > 1e-9)
}

fn multi_axis(params: &OpParams) -> bool {
    let info = params.kind().info();
    info.section == Section::MultiAxis || info.mode == verify::Mode::Rotary
}

/// The machine's work-turning rotaries (`turns_part == Some(true)`).
fn work_turning(rotaries: &[AxisLimits]) -> Vec<&AxisLimits> {
    rotaries.iter().filter(|a| a.rotary && a.turns_part == Some(true)).collect()
}

/// The pose set the posed law reads for this operation on a machine that
/// posts the mill order (`mill`), or `Ok(None)` when the law does not
/// apply: not a multi-axis or rotary kind, no machine, no work-turning
/// rotary that tilts, or a rotary kind on a machine where one rotary
/// turns the work (#2965's swept law reads it). Two or more
/// work-turning rotaries give the sphere about the WCS origin; one gives
/// its line through it. A lone tilting rotary whose line is not known
/// refuses by name: the work it turns cannot be bounded.
pub(super) fn turn_of(params: &OpParams, mill: bool, rotaries: Option<&[AxisLimits]>, wcs_origin: [f64; 3]) -> Result<Option<PosedTurn>, String> {
    if !mill || !multi_axis(params) {
        return Ok(None);
    }
    let turning = rotaries.map(work_turning).unwrap_or_default();
    if !turning.iter().any(|a| tilts(a)) {
        return Ok(None);
    }
    let origin = Point3::new(wcs_origin[0], wcs_origin[1], wcs_origin[2]);
    // #3127: the sphere and the line both pass through the WCS origin.
    let off = |a: &AxisLimits| {
        format!(
            "{}: the work-turning rotary {} states a pivot or offset that moves its line off the WCS origin, so the first traverse over the work it turns cannot be judged",
            params.kind().info().label,
            a.name
        )
    };
    if turning.len() >= 2 {
        if let Some(a) = turning.iter().find(|a| off_origin(a)) {
            return Err(off(a));
        }
        return Ok(Some(PosedTurn::About { origin }));
    }
    if params.kind().info().mode == verify::Mode::Rotary {
        return Ok(None);
    }
    let Some(a) = turning.first() else { return Ok(None) };
    if off_origin(a) && rotary_line(a) != RotaryLine::Vertical {
        return Err(off(a));
    }
    let stated = a.axis.filter(|d| d.iter().all(|c| c.is_finite()) && d.iter().any(|c| c.abs() > 1e-12));
    match (rotary_line(a), stated) {
        (RotaryLine::Vertical, _) => Ok(None),
        (RotaryLine::Horizontal(dir), _) | (RotaryLine::Leaning, Some(dir)) => Ok(Some(PosedTurn::Line { origin, dir })),
        (RotaryLine::Leaning | RotaryLine::Unknown, _) => Err(format!(
            "{}: the work-turning rotary {} states no usable axis line, so the first traverse over the work it turns cannot be judged",
            params.kind().info().label,
            a.name
        )),
    }
}

/// #3084: whether the posed law reads this operation on its setup's
/// machine (a refusal counts: the law reads it too). Its op stamp then
/// takes one more word, so a cache judged without the law reads Stale
/// once; no other stamp moves. The stamp knows neither the tool nor the
/// path, and a mill-turn posts an unstamped rotary kind in the mill order
/// too (#3096, `rotary_traverse::mill_post_rotary`). So it reads each mill
/// and mill-turn setup as the mill order: a word too many only reads a
/// cache Stale once, a word too few would keep an unjudged cache fresh.
pub(super) fn newly_judged(world: &flecs_ecs::prelude::World, setup: Option<&str>, params: &OpParams) -> bool {
    use crate::machine::Kind;
    let mill = matches!(crate::machine::kind_of_setup(world, setup), Kind::Mill | Kind::MillTurn);
    let rotaries: Vec<AxisLimits> = crate::machine::of_setup(world, setup).1.map(|m| m.axes).unwrap_or_default();
    !matches!(turn_of(params, mill, Some(&rotaries), [0.0; 3]), Ok(None))
}

/// The height the post traverses X/Y at, in the path's frame: the higher
/// of the clearance and the first move's own height (`traverseZ`). A
/// rotary kind's first move is a radial pose, not that height, so it
/// reads the clearance alone (the post never traverses under it).
fn traverse_height(path: &Toolpath, params: &OpParams, clearance: f64) -> Option<f64> {
    let z = path.moves.first()?.from.z;
    let rotary = params.kind().info().mode == verify::Mode::Rotary;
    Some(match (clearance.is_finite(), rotary) {
        (true, true) => clearance,
        (true, false) => clearance.max(z),
        (false, _) => z,
    })
}

/// Add the posed verdict to `warns` when the traverse height stands
/// under the work or the tagged steel read through every pose. The
/// machine table stays under the rotaries and is not read. A verdict
/// replaces each first-traverse verdict at a lower top: one verdict
/// names the highest top that the clearance must clear.
pub(super) fn judge(path: &Toolpath, params: &OpParams, frame: Option<&TiltedFrame>, posed: &Posed<'_>, warns: &mut Vec<Warning>) {
    let Some(turn) = posed.turn else { return };
    let Some(height) = traverse_height(path, params, posed.clearance) else { return };
    let steel: Vec<FixtureBox> = posed.fixtures.iter().zip(posed.names).filter(|(_, n)| n.as_str() != MACHINE_TABLE).map(|(b, _)| *b).collect();
    let work = verify::PosedWork { turn, rides: posed.rides, steel: &steel };
    let (up, base) = frame.map_or(([0.0, 0.0, 1.0], Point3::new(0.0, 0.0, 0.0)), |f| (f.axis, f.origin));
    let Some(w) = verify::first_traverse_low_posed(height, &work, up, base, Z_TOL) else { return };
    let Warning::FirstTraverseLow { top_mm, .. } = w else { return };
    warns.retain(|o| !matches!(o, Warning::FirstTraverseLow { top_mm: t, .. } if *t <= top_mm));
    warns.push(w);
}
