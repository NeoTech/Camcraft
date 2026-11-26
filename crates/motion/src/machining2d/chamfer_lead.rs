//! 2D Chamfer lead planning: shared lead shapes, the Chamfer-specific
//! air/transition policy, whole-tool collision checks and hard refusal.
//!
//! Provenance: the five shape constructors are the shared machining
//! handbook geometry in `arc_leads`; the fit, move-crossing and full
//! tool-assembly checks are this codebase's safety laws. No vendor source
//! was consulted.

use crate::cl::{Move, PathBuilder, Toolpath};
use crate::geo2d::{Point2, Region};
use crate::geometry::Point3;

use super::arc_cross::moves_cross_prior;
use super::arc_leads::{lead_in, lead_out};
use super::arcs::{Seg, push_segs, walk_tangent};
use super::chamfer::ChamferParams;
use super::chamfer_lead_check::{ChamferLeadSafety, LeadProbe, full_tool_holds};
use super::contour::{Lead, air_side};
use super::face::GenError;

/// Four half-steps are enough to keep a lead local to the picked edge.
const LEAD_SHRINKS: usize = 4;

/// One accepted pass for `run_level_walks`.
pub(super) struct PlannedWalk {
    pub pass: Vec<Point2>,
    pub segs: Vec<Seg>,
}

/// All accepted Chamfer runs, plus the named half-step notes they earned.
pub(super) struct PlannedWalks {
    pub passes: Vec<Vec<Point2>>,
    pub runs: Vec<Vec<Seg>>,
    pub notes: Vec<String>,
}

/// Plan explicit lead-in/out motion for each selected path. A missing
/// outer lead-in is the v82-and-earlier compatibility marker, not an
/// explicit `Lead::None`, and is rejected here by construction.
pub(super) fn plan_walks(
    paths: &[Vec<Point2>],
    p: &ChamferParams,
    safety: Option<&ChamferLeadSafety<'_>>,
) -> Result<PlannedWalks, GenError> {
    let lead = p.lead.ok_or_else(|| {
        GenError::Invalid("legacy Chamfer motion cannot enter the explicit lead planner".into())
    })?;
    let lead_out_shape = p.lead_out.unwrap_or(lead);
    let mut prior: Vec<Move> = Vec::new();
    let mut passes = Vec::with_capacity(paths.len());
    let mut runs = Vec::with_capacity(paths.len());
    let mut notes = Vec::new();
    for path in paths {
        let (walk, mut said) =
            plan_one(path, lead, lead_out_shape, p, safety, &prior)?;
        let probe = feed_path(&walk.pass, &walk.segs, p.tip_z(), p.feed);
        prior.extend(probe.moves);
        notes.append(&mut said);
        passes.push(walk.pass);
        runs.push(walk.segs);
    }
    Ok(PlannedWalks { passes, runs, notes })
}

fn plan_one(
    path: &[Point2],
    requested_in: Lead,
    requested_out: Lead,
    p: &ChamferParams,
    safety: Option<&ChamferLeadSafety<'_>>,
    prior: &[Move],
) -> Result<(PlannedWalk, Vec<String>), GenError> {
    let Some(&first) = path.first() else {
        return Err(GenError::Empty("a Chamfer lead needs a selected path".into()));
    };
    let Some(&last) = path.last() else {
        return Err(GenError::Empty("a Chamfer lead needs a selected path".into()));
    };
    if path.len() < 2 {
        return Err(GenError::Empty("a Chamfer lead needs at least two path points".into()));
    }
    let path_segs = vec![Seg::Line(path[1..].to_vec())];
    let t_in = walk_tangent(&path_segs, path, true);
    let t_out = walk_tangent(&path_segs, path, false);
    let mut last_reason = "the candidate did not fit".to_owned();
    for budget in 0..=LEAD_SHRINKS {
        for in_step in 0..=budget {
            for out_step in 0..=budget {
                if in_step != budget && out_step != budget {
                    continue;
                }
                let shaped_in =
                    requested_in.scaled(0.5f64.powi(in_step as i32));
                let shaped_out =
                    requested_out.scaled(0.5f64.powi(out_step as i32));
                let (entry, segs_in) =
                    lead_in(first, t_in, air_side(t_in, p.climb), shaped_in);
                let segs_out =
                    lead_out(last, t_out, air_side(t_out, p.climb), shaped_out);
                let mut segs = Vec::with_capacity(segs_in.len() + segs_out.len() + 1);
                segs.extend(segs_in.iter().cloned());
                segs.push(Seg::Line(path[1..].to_vec()));
                segs.extend(segs_out.iter().cloned());
                let pass = endpoints(entry, &segs);
                let probe = feed_path(&pass, &segs, p.tip_z(), p.feed);
                if !probe.all_finite() {
                    last_reason = "the candidate left the finite horizon".into();
                    continue;
                }
                if crosses_itself(&probe.moves) {
                    last_reason = "the lead crossed its own Chamfer ride".into();
                    continue;
                }
                if moves_cross_prior(&probe.moves, prior) {
                    last_reason = "the lead crossed an earlier Chamfer ride".into();
                    continue;
                }
                match full_tool_holds(
                    LeadProbe {
                        path,
                        entry,
                        segs_in: &segs_in,
                        segs_out: &segs_out,
                        shaped_in,
                        shaped_out,
                    },
                    p,
                    safety,
                ) {
                    Ok(()) => {
                        let mut notes = Vec::new();
                        shrink_note(
                            &mut notes,
                            "lead-in",
                            requested_in,
                            shaped_in,
                            in_step,
                        );
                        shrink_note(
                            &mut notes,
                            "lead-out",
                            requested_out,
                            shaped_out,
                            out_step,
                        );
                        return Ok((PlannedWalk { pass, segs }, notes));
                    }
                    Err(why) => last_reason = why,
                }
            }
        }
    }
    Err(GenError::Invalid(format!(
        "2D Chamfer {} lead-in and {} lead-out cannot fit after four halvings: {last_reason}; shorten the requested lead or move the selected edge into air",
        requested_in.kind(),
        requested_out.kind(),
    )))
}

fn feed_path(
    pass: &[Point2],
    segs: &[Seg],
    z: f64,
    feed: f64,
) -> Toolpath {
    let at = pass.first().copied().unwrap_or(Point2::new(0.0, 0.0));
    let mut b = PathBuilder::new(Point3::new(at.x, at.y, z));
    push_segs(&mut b, segs, z, feed);
    b.finish()
}

fn endpoints(entry: Point2, segs: &[Seg]) -> Vec<Point2> {
    let mut points = vec![entry];
    for seg in segs {
        match seg {
            Seg::Line(qs) => points.extend(qs.iter().copied()),
            Seg::Arc { to, .. } => points.push(*to),
        }
    }
    points
}

fn crosses_itself(moves: &[Move]) -> bool {
    moves
        .iter()
        .enumerate()
        .any(|(i, m)| moves_cross_prior(std::slice::from_ref(m), &moves[..i]))
}

fn shrink_note(notes: &mut Vec<String>, end: &str, requested: Lead, used: Lead, steps: usize) {
    if steps == 0 || used.size() == 0.0 {
        return;
    }
    let note = format!(
        "2D Chamfer {end} {} lead was halved {} time(s): requested {:.3} mm, used {:.3} mm",
        requested.kind(),
        steps,
        requested.size(),
        used.size(),
    );
    if !notes.contains(&note) {
        notes.push(note);
    }
}

/// Preserve the pre-v83 open-chain air extension exactly. This is the
/// compatibility path selected by an absent persisted `lead` key.
pub(super) fn legacy_open_ride(
    pass: &[Point2],
    p: &ChamferParams,
    levels: &[(f64, Region)],
) -> Result<Vec<Point2>, GenError> {
    let (Some(&first), Some(&second), Some(&last), Some(&before_last)) = (
        pass.first(),
        pass.get(1),
        pass.last(),
        pass.get(pass.len().saturating_sub(2)),
    ) else {
        return Err(GenError::Empty("a chamfer edge needs at least two points".into()));
    };
    let entry_air = air_side(tangent(first, second)?, p.climb);
    let exit_air = air_side(tangent(before_last, last)?, p.climb);
    let mut ride = Vec::with_capacity(pass.len() + 2);
    ride.push(Point2::new(
        first.x + entry_air.x * p.width,
        first.y + entry_air.y * p.width,
    ));
    ride.extend_from_slice(pass);
    ride.push(Point2::new(
        last.x + exit_air.x * p.width,
        last.y + exit_air.y * p.width,
    ));
    if p.minimum_cone_radius > 0.0 {
        let swept = Region::sweep(&[ride.clone()], 1e-3).map_err(GenError::Geo)?;
        for (z, region) in levels {
            let hit = swept.intersect(region).map_err(GenError::Geo)?;
            if !hit.is_empty()
                && let Some(q) = hit.loops().first().and_then(|l| l.points().first())
            {
                return Err(GenError::invalid_at(
                    format!(
                        "the tip offset reaches selected-model material on the air side at Z {z:.3} near ({:.3}, {:.3}); pick a smaller offset or a different Side",
                        q.x, q.y
                    ),
                    Point3::new(q.x, q.y, *z),
                ));
            }
        }
    }
    Ok(ride)
}

fn tangent(a: Point2, b: Point2) -> Result<Point2, GenError> {
    let len = (b.x - a.x).hypot(b.y - a.y);
    if !len.is_finite() || len <= 1e-9 {
        return Err(GenError::Empty(
            "a chamfer edge holds a collapsed segment - re-pick the edge".into(),
        ));
    }
    Ok(Point2::new((b.x - a.x) / len, (b.y - a.y) / len))
}
