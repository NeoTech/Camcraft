//! Public entry points for closed-region 2D Contour with "Do not touch" protection (#3344).
//! Provenance: clean-room avoid-aware contour orchestration reusing `contour` and `contour_depth`; no vendor material consulted.

use camcraft_sim::verify::AvoidSpec;

use crate::cl::{MoveKind, PathBuilder, Toolpath};
use crate::geo2d::{Loop, Region};
use crate::geometry::{Point3, Triangle};

use super::contour::{ContourParams, finish_contour, prepare_band};
use super::contour_avoid_clip::finalize_avoid;
use super::contour_avoid_emit::{AvoidSink, emit_band_avoid};
use super::contour_comp;
use super::contour_depth::{Ladder, ladder, ladder_picked, picked_law};
use super::contour_entry::say;
use super::contour_region::{PartRef, contour, contour_on_bodies, contour_on_part, contour_on_picked};
use super::contour_start;
use super::face::GenError;

/// Shared finalization after one or more bands have been emitted into `b`.
pub(super) fn finish_avoid_contour(
    mut b: PathBuilder,
    mut sink: AvoidSink,
    p: &ContourParams,
    avoid: &AvoidSpec,
) -> Result<Toolpath, GenError> {
    if sink.blocked_any || sink.clipped_any {
        say(
            &mut sink.lead_notes,
            format!(
                "Do not touch ({}): skipped or trimmed blocked contour portion(s); remaining path stays clear",
                avoid.names_list()
            ),
        );
    }
    sink.lifted.say(&mut b, "ring-to-ring");
    let path = finish_contour(b, sink.lead_notes, &sink.tally, p)?;
    finalize_avoid(path, avoid)
}

/// [`contour`] with "Do not touch" protection across flutes, shank, and holder.
pub fn contour_avoid(region: &Region, p: &ContourParams, avoid: &AvoidSpec) -> Result<Toolpath, GenError> {
    if avoid.is_empty() {
        return contour(region, p);
    }
    contour_impl_avoid(region, p, None, avoid)
}

/// [`contour_on_part`] with "Do not touch" protection.
pub fn contour_on_part_avoid(
    region: &Region,
    part: &[Triangle],
    p: &ContourParams,
    avoid: &AvoidSpec,
) -> Result<Toolpath, GenError> {
    if avoid.is_empty() {
        return contour_on_part(region, part, p);
    }
    contour_impl_avoid(region, p, Some((part, &[], false)), avoid)
}

/// [`contour_on_bodies`] with "Do not touch" protection.
pub fn contour_on_bodies_avoid(
    region: &Region,
    part: &[Triangle],
    bodies: &[std::ops::Range<usize>],
    p: &ContourParams,
    avoid: &AvoidSpec,
) -> Result<Toolpath, GenError> {
    if avoid.is_empty() {
        return contour_on_bodies(region, part, bodies, p);
    }
    contour_impl_avoid(region, p, Some((part, bodies, false)), avoid)
}

/// [`contour_on_picked`] with "Do not touch" protection.
pub fn contour_on_picked_avoid(
    region: &Region,
    part: &[Triangle],
    bodies: &[std::ops::Range<usize>],
    p: &ContourParams,
    avoid: &AvoidSpec,
) -> Result<Toolpath, GenError> {
    if avoid.is_empty() {
        return contour_on_picked(region, part, bodies, p);
    }
    contour_impl_avoid(region, p, Some((part, bodies, true)), avoid)
}

fn contour_impl_avoid(
    region: &Region,
    p: &ContourParams,
    part: Option<PartRef<'_>>,
    avoid: &AvoidSpec,
) -> Result<Toolpath, GenError> {
    p.validate()?;
    contour_comp::door(p)?;
    let band = prepare_band(region, p)?;
    let refs: Vec<&Loop> = band.loops.iter().collect();
    let pick = contour_start::pick_ring(&refs, p)?;
    let exit = contour_start::pick_exit(&refs, p)?;
    let flat = p.ring_levels()?;
    let r = p.tool_diameter * 0.5;
    let d = r + p.stock_to_leave;
    let mut sink = AvoidSink::default();
    let lad = match part {
        None => Ladder::uniform(&flat, band.loops.len()),
        Some((f, bodies, true)) => {
            ladder_picked(&band.plan, f, bodies, &flat, p.top_z, r, p.stock_to_leave, &mut sink.lead_notes)
        }
        Some((f, bodies, false)) => ladder(&band.plan, f, bodies, &flat, p.top_z, r, p.stock_to_leave, &mut sink.lead_notes),
    };
    if let Some((f, _, true)) = part {
        picked_law(&band.plan, f, &flat, &lad, d, p.side)?;
    }
    if lad.zs.is_empty() {
        return Err(GenError::Invalid(format!(
            "every contour ring cuts into the part at its first level - nothing to machine without a gouge ({})",
            sink.lead_notes.join("; ")
        )));
    }
    let mut b = PathBuilder::approached(Point3::new(0.0, 0.0, p.clearance_z));
    emit_band_avoid(&mut b, &band, p, &lad, pick, exit, avoid, &mut sink)?;
    let cur = b.at();
    b.push(MoveKind::Rapid, Point3::new(cur.x, cur.y, p.clearance_z), None);
    finish_avoid_contour(b, sink, p, avoid)
}
