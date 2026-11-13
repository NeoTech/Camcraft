//! Detect holes' Bore filter (#1556): the holes a Bore op's circular
//! ride can clear with its tool, and a named note for each it cannot.
//!
//! Provenance: circular-interpolation boring (a cutter riding one ring
//! inside the wall, machine-tool handbook practice) clears the centre
//! the dive opens and the ring's band - nothing between them. The
//! verdict is the motion kernel's own (`machining2d::bore_leaves_core`,
//! the same call its generate door makes, corner radius and through
//! floor included), so detection and the refusal can never disagree.

use flecs_ecs::prelude::World;

use crate::drilling::{Hole, THROUGH_CLEAR};
use crate::machining2d::{bore_leaves_core, max_bore_diameter};
use crate::model;
use crate::operation::OpParams;

/// Detect holes' answer: the params to store, and one sentence per hole
/// the op's kind could not take (#1556).
#[derive(Clone, Debug, PartialEq)]
pub struct DetectedHoles {
    pub params: OpParams,
    pub skipped: Vec<String>,
}

/// A Bore op's freshly detected list, cut to the holes its tool's ring
/// clears; every other kind passes untouched. All holes too wide is a
/// refusal naming each one - an empty Bore op is never stored.
pub(super) fn skip_unborable(world: &World, name: &str, params: &mut OpParams) -> Result<Vec<String>, String> {
    let OpParams::Bore { holes, stock_to_leave, .. } = params else {
        return Ok(Vec::new());
    };
    let Some((d, corner)) = op_tool_size(world, name) else {
        return Ok(Vec::new());
    };
    let skipped = keep_borable(holes, d, corner, *stock_to_leave);
    if holes.is_empty() {
        return Err(format!("no hole this Bore op can make: {}", skipped.join("; ")));
    }
    Ok(skipped)
}

/// The diameter and corner radius of the tool the named operation
/// holds, when both exist.
fn op_tool_size(world: &World, name: &str) -> Option<(f64, f64)> {
    let op = model::find_by_name(world, name).and_then(|e| model::operation_of(world, &e))?;
    model::find_by_name(world, &op.tool)
        .and_then(|t| model::tool_of(world, &t))
        .map(|t| (t.diameter, t.corner_r))
}

/// Drop the holes wider than one ring of a `tool_diameter` cutter with
/// a `corner` radius clears (leaving `leave` on the wall), and name each
/// one dropped. A through hole is judged at its floor plane, where the
/// generator's tip runs [`THROUGH_CLEAR`] under it (the same allowance
/// generate hands the kernel). A non-finite or non-positive diameter, or
/// a corner that does not fit it, filters nothing: the kernel's own doors
/// refuse that tool at generate, by name.
pub(super) fn keep_borable(holes: &mut Vec<Hole>, tool_diameter: f64, corner: f64, leave: f64) -> Vec<String> {
    let sane = tool_diameter.is_finite()
        && tool_diameter > 0.0
        && corner.is_finite()
        // The kernel's own corner door (its float allowance included).
        && corner >= 0.0
        && corner <= tool_diameter * 0.5 + 1e-6
        && leave.is_finite()
        && leave >= 0.0;
    if !sane {
        return Vec::new();
    }
    let mut skipped = Vec::new();
    holes.retain(|h| {
        let through = h.through.then_some(THROUGH_CLEAR);
        let keep = !h.radius.is_finite() || !bore_leaves_core(h.radius, tool_diameter, corner, leave, through);
        if !keep {
            let max = max_bore_diameter(tool_diameter, corner, leave, through);
            skipped.push(format!(
                "the {:.3} mm hole at ({:.3}, {:.3}) was skipped: wider than the {max:.3} mm a \
                 {tool_diameter:.3} mm tool bores without leaving a core - clear it with a pocket or \
                 adaptive operation, or use a larger tool",
                h.radius * 2.0,
                tidy(h.xy.x),
                tidy(h.xy.y)
            ));
        }
        keep
    });
    skipped
}

/// A coordinate as the note prints it: float noise under the printed
/// precision reads 0.000, never -0.000.
fn tidy(v: f64) -> f64 {
    if v.abs() < 5e-4 { 0.0 } else { v }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geo2d::Point2;

    fn hole(dia: f64, through: bool) -> Hole {
        Hole { xy: Point2::new(5.0, -3.0), radius: dia * 0.5, entry_z: 0.0, floor_z: -4.0, through, stepped: false }
    }

    #[test]
    fn detect_keeps_the_limit_hole_with_its_leave_and_names_the_one_past_it() {
        // 4 mm square end mill, 0.5 mm leave: the limit is 3 x 4 + 2 x 0.5
        // = 13 mm. The kernel bores Ø13.000 (core gap exactly 0), so detect
        // keeps it; Ø13.02 is named. Ignoring the leave (a 12 mm limit)
        // would drop the Ø13 hole, a looser verdict would keep Ø13.02.
        let mut holes = vec![hole(13.0, false), hole(13.02, false), hole(12.5, true)];
        let skipped = keep_borable(&mut holes, 4.0, 0.0, 0.5);
        let kept: Vec<f64> = holes.iter().map(|h| h.radius * 2.0).collect();
        assert_eq!(kept, vec![13.0, 12.5], "the limit hole and the narrower one stay");
        assert_eq!(skipped.len(), 1, "{skipped:?}");
        assert!(skipped[0].contains("13.020 mm hole at (5.000, -3.000)"), "{skipped:?}");
        assert!(skipped[0].contains("wider than the 13.000 mm"), "{skipped:?}");
    }

    #[test]
    fn detect_judges_a_bull_nose_through_hole_at_its_floor_reach() {
        // Ø12 bull nose, r2 corner, no leave: through, the tip runs 1 mm
        // under the floor, where the corner reaches 4 + sqrt(3) = R5.732,
        // so the limit is 12 + 4 x 5.732 = 34.928 mm. A blind hole keeps
        // the 36 mm square-end limit (its floor stays under the ridge).
        let limit = 12.0 + 4.0 * (4.0 + 3f64.sqrt());
        let mut holes = vec![hole(limit, true), hole(limit + 0.02, true), hole(36.0, false)];
        let skipped = keep_borable(&mut holes, 12.0, 2.0, 0.0);
        assert_eq!(holes.len(), 2, "{skipped:?}");
        assert!((holes[0].radius * 2.0 - limit).abs() < 1e-12 && !holes[1].through, "{holes:?}");
        assert_eq!(skipped.len(), 1, "{skipped:?}");
        assert!(skipped[0].contains("34.948 mm hole") && skipped[0].contains("wider than the 34.928 mm"), "{skipped:?}");
    }
}
