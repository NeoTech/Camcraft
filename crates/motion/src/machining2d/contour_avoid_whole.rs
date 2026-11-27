//! Whole-model 2D Contour planning with "Do not touch" protection (#3344).
//! Provenance: clean-room per-band region fencing and avoid emission extending `contour_whole`; no vendor material consulted.

use std::ops::Range;

use camcraft_sim::verify::AvoidSpec;

use crate::cl::{MoveKind, PathBuilder, Toolpath};
use crate::geo2d::Region;
use crate::geometry::{Point3, Triangle};

use super::contour::prepare_band;
use super::contour_avoid::{contour_on_bodies_avoid, finish_avoid_contour};
use super::contour_avoid_emit::{AvoidSink, emit_band_avoid};
use super::contour_comp;
use super::contour_depth::ladder;
use super::contour_entry::say;
use super::contour_params::{ContourParams, Side};
use super::contour_start;
use super::contour_whole::{
    SAME_LEVEL, check_unread, first_split, forward_band_notes, match_pick, note_rings_without_floor, stopped, trim_ladder,
};
use super::contour_whole_fence::{Planned, room};
use super::contour_whole_reach::{band_sweep, reach_up};
use super::contour_whole_slice::next_region;
use super::face::GenError;

fn flush_avoid(
    b: &mut PathBuilder,
    planned: &mut Vec<Planned>,
    p: &ContourParams,
    avoid: &AvoidSpec,
    sink: &mut AvoidSink,
) -> Result<(), GenError> {
    let rooms = (0..planned.len()).map(|i| room(planned, i)).collect::<Result<Vec<_>, _>>()?;
    for (mut pl, rm) in planned.drain(..).zip(rooms) {
        pl.band.room = rm;
        emit_band_avoid(b, &pl.band, p, &pl.lad, pl.pick, pl.exit, avoid, sink)?;
    }
    Ok(())
}

/// Whole-model [`super::contour_whole::contour_whole_bodies_with`] with "Do not touch" protection.
pub fn contour_whole_bodies_with_avoid(
    top_region: &Region,
    part: &[Triangle],
    bodies: &[Range<usize>],
    p: &ContourParams,
    avoid: &AvoidSpec,
    area_at: &mut dyn FnMut(Region) -> Result<Region, GenError>,
) -> Result<(Toolpath, Vec<Region>), GenError> {
    if avoid.is_empty() {
        return super::contour_whole::contour_whole_bodies_with(top_region, part, bodies, p, area_at);
    }
    if p.side == Side::Inside {
        let path = contour_on_bodies_avoid(top_region, part, bodies, p, avoid)?;
        return Ok((path, vec![top_region.clone()]));
    }
    p.validate()?;
    contour_comp::door(p)?;
    if top_region.is_empty() {
        return Err(GenError::Empty("empty footprint".into()));
    }
    let flat = p.ring_levels()?;
    let r = p.tool_diameter * 0.5;
    let mut b = PathBuilder::approached(Point3::new(0.0, 0.0, p.clearance_z));
    let mut sink = AvoidSink::default();
    let mut walked = Vec::new();
    let mut cover = Region::empty();
    let mut remaining: &[f64] = &flat;
    let mut current_region = top_region.clone();
    let mut ceiling = p.top_z;
    let mut used_fallback = false;
    let mut pick_matched = p.picks.first(crate::path_picks::PickRole::Start).map(|k| k.xy).is_none();
    let mut exit_matched = p.picks.first(crate::path_picks::PickRole::Exit).map(|k| k.xy).is_none();
    let mut deferred_pick = None;
    let mut deferred_exit = None;
    let mut planned: Vec<Planned> = Vec::new();

    let mut mid = false;
    let max_bands = flat.len().saturating_mul(2).saturating_add(part.len().saturating_mul(3)).saturating_add(2);
    let mut band_idx = 0usize;
    while !remaining.is_empty() && band_idx <= max_bands {
        let last_z = remaining[remaining.len() - 1];
        let band = prepare_band(&current_region, p)?;
        let refs: Vec<&crate::geo2d::Loop> = band.loops.iter().collect();
        let pick = match_pick(contour_start::pick_ring(&refs, p), &mut pick_matched, &mut deferred_pick)?;
        let exit = match_pick(contour_start::pick_exit(&refs, p), &mut exit_matched, &mut deferred_exit)?;

        let mut ring_notes = Vec::new();
        let lad = ladder(&band.plan, part, bodies, remaining, ceiling, r, p.stock_to_leave, &mut ring_notes);
        if check_unread(&lad, remaining, p, &b).is_err() {
            flush_avoid(&mut b, &mut planned, p, avoid, &mut sink)?;
            check_unread(&lad, remaining, p, &b)?;
        }
        let lad = reach_up(&band, lad, &cover, &flat, part, bodies, p)?;
        let split = if mid { Some(0) } else { first_split(&band, &lad, remaining, part, bodies, p)? };
        match split {
            Some(0) if mid || (band_idx == 0 && !used_fallback) => {
                let trimmed = trim_ladder(&lad, remaining[0]);
                let emitted = trimmed.zs.last().is_some_and(|&z| z < ceiling - SAME_LEVEL);
                if emitted {
                    note_rings_without_floor(&band, &lad, &trimmed, remaining[0], ceiling, &mut sink.lead_notes);
                    forward_band_notes(&band, &lad, last_z, true, ring_notes, &mut sink.lead_notes);
                    walked.push(current_region.clone());
                    cover = cover.union(&band_sweep(&band.center, r)?)?;
                    ceiling = trimmed.zs.last().copied().unwrap_or(ceiling);
                    planned.push(Planned { band, lad: trimmed, pick, exit });
                } else if mid {
                    say(
                        &mut sink.lead_notes,
                        format!(
                            "whole-model contour: the outline changes between Z {:.3} and Z {:.3} but has no floor level there - that wall is not cut; reduce the stepdown",
                            remaining[0], ceiling
                        ),
                    );
                }
                used_fallback = true;
                let (raw, next_mid) = next_region(part, bodies, p, ceiling, remaining[0], emitted)?;
                mid = next_mid;
                current_region = area_at(raw)?;
            }
            Some(0) => {
                let msg = format!(
                    "whole-model contour cannot cut Z {:.3} (Bottom Z {:.3}) without gouging ({})",
                    remaining[0],
                    p.bottom_z,
                    ring_notes.join("; ")
                );
                flush_avoid(&mut b, &mut planned, p, avoid, &mut sink)?;
                return Err(stopped(msg, b));
            }
            Some(j) => {
                let z_next = remaining[j];
                let trimmed = trim_ladder(&lad, z_next);
                if trimmed.zs.is_empty() {
                    return Err(GenError::Invalid(format!(
                        "whole-model contour produced no cut levels above Z {z_next:.3}"
                    )));
                }
                forward_band_notes(&band, &lad, last_z, true, ring_notes, &mut sink.lead_notes);
                walked.push(current_region.clone());
                cover = cover.union(&band_sweep(&band.center, r)?)?;
                ceiling = trimmed.zs.last().copied().unwrap_or(remaining[j - 1]);
                planned.push(Planned { band, lad: trimmed, pick, exit });
                remaining = &remaining[j..];
                used_fallback = true;
                let (raw, next_mid) = next_region(part, bodies, p, ceiling, remaining[0], true)?;
                mid = next_mid;
                current_region = match area_at(raw) {
                    Ok(r) => r,
                    Err(e) => {
                        let msg = format!(
                            "whole-model contour stopped at Z {ceiling:.3} (Bottom Z {:.3}): no machinable region at Z {:.3} ({e})",
                            p.bottom_z, remaining[0]
                        );
                        flush_avoid(&mut b, &mut planned, p, avoid, &mut sink)?;
                        return Err(stopped(msg, b));
                    }
                };
            }
            None => {
                forward_band_notes(&band, &lad, last_z, false, ring_notes, &mut sink.lead_notes);
                planned.push(Planned { band, lad, pick, exit });
                walked.push(current_region);
                remaining = &[];
                break;
            }
        }
        band_idx += 1;
    }
    if let Some(z) = remaining.first() {
        let msg = format!(
            "whole-model contour stopped at Z {ceiling:.3} (Bottom Z {:.3}): too many outline changes above Z {z:.3}",
            p.bottom_z
        );
        flush_avoid(&mut b, &mut planned, p, avoid, &mut sink)?;
        return Err(stopped(msg, b));
    }
    flush_avoid(&mut b, &mut planned, p, avoid, &mut sink)?;
    let cur = b.at();
    b.push(MoveKind::Rapid, Point3::new(cur.x, cur.y, p.clearance_z), None);

    if !pick_matched
        && let Some(e) = deferred_pick
    {
        return Err(e);
    }
    if !exit_matched
        && let Some(e) = deferred_exit
    {
        return Err(e);
    }
    let path = finish_avoid_contour(b, sink, p, avoid)?;
    Ok((path, walked))
}
