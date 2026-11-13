//! Detect holes' Bore filter (#1556): the holes a Bore op's circular
//! ride can clear with its tool, and a named note for each it cannot.
//!
//! Provenance: circular-interpolation boring (a cutter riding one ring
//! inside the wall, machine-tool handbook practice) clears the centre
//! the dive opens and the ring's band - nothing between them. The limit
//! is the motion kernel's own number (`machining2d::max_bore_diameter`),
//! so detection and the generate refusal can never disagree.

use flecs_ecs::prelude::World;

use crate::drilling::Hole;
use crate::machining2d::max_bore_diameter;
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
    let Some(d) = op_tool_diameter(world, name) else {
        return Ok(Vec::new());
    };
    let skipped = keep_borable(holes, d, *stock_to_leave);
    if holes.is_empty() {
        return Err(format!("no hole this Bore op can make: {}", skipped.join("; ")));
    }
    Ok(skipped)
}

/// The diameter of the tool the named operation holds, when both exist.
fn op_tool_diameter(world: &World, name: &str) -> Option<f64> {
    let op = model::find_by_name(world, name).and_then(|e| model::operation_of(world, &e))?;
    model::find_by_name(world, &op.tool)
        .and_then(|t| model::tool_of(world, &t))
        .map(|t| t.diameter)
}

/// Drop the holes wider than one ring of a `tool_diameter` cutter
/// clears (leaving `leave` on the wall), and name each one dropped.
/// A non-finite or non-positive diameter filters nothing: the kernel's
/// own doors refuse that tool at generate, by name.
pub(super) fn keep_borable(holes: &mut Vec<Hole>, tool_diameter: f64, leave: f64) -> Vec<String> {
    if !(tool_diameter.is_finite() && tool_diameter > 0.0 && leave.is_finite() && leave >= 0.0) {
        return Vec::new();
    }
    let max = max_bore_diameter(tool_diameter, leave);
    let mut skipped = Vec::new();
    holes.retain(|h| {
        let dia = h.radius * 2.0;
        // The kernel's own tolerance on the radius is 1e-6; a hole at
        // the limit stays, one past it is named.
        let keep = !dia.is_finite() || dia - max <= 2e-6;
        if !keep {
            skipped.push(format!(
                "the {dia:.3} mm hole at ({:.3}, {:.3}) was skipped: wider than the {max:.3} mm a \
                 {tool_diameter:.3} mm tool bores without leaving a core - clear it with a pocket or \
                 adaptive operation, or use a larger tool",
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
