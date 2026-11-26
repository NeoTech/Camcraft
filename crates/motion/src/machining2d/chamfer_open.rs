//! Open-run 2D Chamfer planning and posting, split from `chamfer.rs`
//! (#3237, the 400-line law). The wall classification is this
//! codebase's own selected-edge law; no vendor material was consulted.

use crate::cl::{EdgeBreak, MoveKind, PathBuilder, Toolpath};
use crate::geo2d::Point2;
use crate::geometry::Point3;

use super::chamfer::ChamferParams;
use super::chamfer_lead::{legacy_open_ride, plan_walks};
use super::chamfer_lead_check::ChamferLeadSafety;
use super::chamfer_side::boundary_inside;
use super::contour_run_side::{
    LIFT, SectionAt, Words, material_side, offset_run_side, simple,
};
use super::face::GenError;
use super::levels::{Motion, run_level_walks, run_levels};

const CHAMFER_WORDS: Words = Words {
    body: "selected model",
    both: "a chamfer on that edge would meet material on both sides; pick a boundary edge with air on one side",
};

/// Chamfer joined open edge chains, one approach and exit per chain.
/// An absent lead is the pre-v83 fixed-width compatibility path.
pub fn chamfer_open_runs(
    runs: &[Vec<Point2>],
    p: &ChamferParams,
    section: &SectionAt<'_>,
) -> Result<Toolpath, GenError> {
    chamfer_open_runs_impl(runs, p, section, None)
}

/// Open Chamfer with exact collision inputs for explicit leads.
pub fn chamfer_open_runs_with_lead_safety(
    runs: &[Vec<Point2>],
    p: &ChamferParams,
    section: &SectionAt<'_>,
    safety: &ChamferLeadSafety<'_>,
) -> Result<Toolpath, GenError> {
    chamfer_open_runs_impl(runs, p, section, Some(safety))
}

fn chamfer_open_runs_impl(
    runs: &[Vec<Point2>],
    p: &ChamferParams,
    section: &SectionAt<'_>,
    safety: Option<&ChamferLeadSafety<'_>>,
) -> Result<Toolpath, GenError> {
    p.validate()?;
    if runs.is_empty() {
        return Err(GenError::Empty(
            "a 2D Chamfer needs a selected edge chain".into(),
        ));
    }
    let z = p.tip_z();
    if !z.is_finite() {
        return Err(GenError::Invalid(
            "virtual cone tip left the finite horizon".into(),
        ));
    }
    let corner_z = p.corner_z();
    let probe = LIFT.min(p.break_depth() * 0.25);
    let mut passes: Vec<Vec<Point2>> = Vec::with_capacity(runs.len());
    let mut breaks: Vec<EdgeBreak> = Vec::with_capacity(runs.len());
    for pts in runs {
        for pair in pts.windows(2) {
            tangent(pair[0], pair[1])?;
        }
        let run = simple(pts).map_err(|e| match e {
            GenError::Empty(_) => {
                GenError::Empty("a chamfer edge collapses to one point - re-pick the edge".into())
            }
            other => other,
        })?;
        let mut levels = Vec::new();
        for at in [corner_z + probe, p.top_z - probe] {
            if let Some(region) = section(at)?
                && !region.is_empty()
            {
                levels.push((at, region));
            }
        }
        let material_left = material_side(&run, &levels, &CHAMFER_WORDS)?.ok_or_else(|| {
            GenError::Invalid(
                "the selected open chamfer edge has no model wall at the programmed chamfer depth - pick an edge on the part, or change Top of Part Z"
                    .into(),
            )
        })?;
        let pass = oriented_pass(run.as_slice(), material_left, p, &levels)?;
        let ride = if p.legacy_leads() {
            legacy_open_ride(&pass, p, &levels)?
        } else {
            pass
        };
        passes.push(ride);
        breaks.push(EdgeBreak {
            edge: run
                .iter()
                .map(|q| Point3::new(q.x, q.y, p.top_z))
                .collect(),
            depth: p.break_depth(),
            slope: p.angle_deg.to_radians().tan(),
            volume: None,
        });
    }

    let mut b = PathBuilder::approached(Point3::new(0.0, 0.0, p.clearance_z));
    let motion = Motion::new(p.clearance_z, p.retract_z, p.feed, p.plunge_feed);
    if p.legacy_leads() {
        run_levels(&mut b, &passes, &[z], &motion, None, None, None);
    } else {
        let planned = plan_walks(&passes, p, safety)?;
        run_level_walks(
            &mut b,
            &planned.passes,
            &[z],
            &motion,
            Some(&planned.runs),
            None,
            None,
        );
        for note in planned.notes {
            b.note_once(&note);
        }
    }
    let cur = b.at();
    if cur.z < p.clearance_z {
        b.push(
            MoveKind::Rapid,
            Point3::new(cur.x, cur.y, p.clearance_z),
            None,
        );
    }
    let mut path = b.finish();
    if !path.all_finite() {
        return Err(GenError::Invalid(
            "open chamfer path left the finite horizon".into(),
        ));
    }
    path.breaks = breaks.into_iter().filter(EdgeBreak::is_sane).collect();
    Ok(path)
}

fn oriented_pass(
    run: &[Point2],
    material_left: bool,
    p: &ChamferParams,
    levels: &[(f64, crate::geo2d::Region)],
) -> Result<Vec<Point2>, GenError> {
    if p.minimum_cone_radius == 0.0 {
        let mut pass = run.to_vec();
        // Material stays on the RIGHT for climb, independent of how the
        // selected edge chain happened to be stored.
        if p.climb == material_left {
            pass.reverse();
        }
        return Ok(pass);
    }
    let air_left = !material_left;
    let Some(inside) = boundary_inside(run, levels, air_left)? else {
        return Err(GenError::Invalid(
            "the selected open model wall cannot classify Side for the tip offset; re-pick a boundary edge".into(),
        ));
    };
    if inside != p.inside {
        let side = if p.inside { "Inside" } else { "Outside" };
        return Err(GenError::Invalid(format!(
            "Side {side} names the wrong class of selected model wall for the tip offset; use Side Auto or the other side"
        )));
    }
    let offset = offset_run_side(run, p.minimum_cone_radius, air_left)?
        .ok_or_else(|| {
            GenError::Invalid(format!(
                "the tip offset of {} mm folds onto itself or disappears; pick a gentler run or a smaller offset",
                p.minimum_cone_radius
            ))
        })?;
    let offset_forward = !air_left;
    let desired_forward = p.climb != material_left;
    let mut pass = offset;
    if offset_forward != desired_forward {
        pass.reverse();
    }
    Ok(pass)
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
