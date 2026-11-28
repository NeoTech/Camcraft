//! The Laser Profile operation's seed and generation (#4081): the op's
//! stored numbers resolved against the world (heights, the laser head's
//! kerf), the picked contours or the model's through-silhouette as the
//! geometry, and `laser::laser_profile` run over them.
//!
//! Provenance: our own glue over the motion crate's laser generator,
//! whose module head cites the public sources (general laser-cutting
//! practice; RS-274 / ISO 6983).

use crate::cl::Toolpath;
use crate::geo2d::{Loop, Point2, Region};
use crate::geometry::slice;
use crate::geometry::Triangle;
use crate::laser::{laser_profile, JetTabs, LaserParams, LeadShape, OpenSide};
use crate::machining2d::GenError;
use crate::tools::{Tool, ToolKind};
use crate::verify::Warning;

use super::kinds::Carried;
use super::material::FOOTPRINT_SAMPLES;
use super::types::{JetLead, JetSide, LaserOp, OpParams};
use super::ResolvedHeights;

/// The laser draft: the carried heights (sheet top and bottom), the
/// head's own kerf, computer compensation, arc leads of 2 mm in and
/// 1 mm out, open runs on their centre line, no tabs, arcs kept, and a
/// low feed and power for the user to state from the cut data.
pub(super) fn seed_laser_profile(c: &Carried) -> OpParams {
    OpParams::LaserProfile(LaserOp {
        clearance_z: c.clearance,
        top_z: c.top,
        bottom_z: c.bottom,
        kerf: None,
        control_comp: false,
        reversed: false,
        lead: JetLead::Arc,
        lead_in: 2.0,
        lead_out: 1.0,
        open_side: JetSide::Center,
        tabs: None,
        feed: 600.0,
        power: 50.0,
        pierce_time: 0.5,
        linearize: None,
        picks: crate::path_picks::PathPicks::default(),
    })
}

/// The generator's numbers from the op's and the world's.
///
/// `model_excess` is how much coarser than promised the model the part
/// silhouette is read from is (#1053): the kerf law gives it back.
pub fn laser_params(op: &LaserOp, tool: &Tool, hs: &ResolvedHeights, model_excess: f64) -> Result<LaserParams, GenError> {
    if tool.kind != ToolKind::Laser {
        return Err(GenError::Invalid(format!(
            "a laser profile mounts a laser head, not a {} - its diameter is the kerf width",
            tool.kind
        )));
    }
    if let Some(why) = crate::laser_tools::refusal(tool) {
        return Err(GenError::Invalid(format!("laser profile: {why}")));
    }
    let (Some(top), Some(bottom)) = (hs.top, hs.bottom) else {
        return Err(GenError::Invalid("laser profile: the sheet's top and bottom must both resolve".into()));
    };
    if !(top.is_finite() && bottom.is_finite()) || bottom > top {
        return Err(GenError::Invalid(format!(
            "laser profile: the sheet's bottom ({bottom:.3}) stands above its top ({top:.3})"
        )));
    }
    Ok(LaserParams {
        kerf: op.kerf.unwrap_or(tool.diameter),
        control_comp: op.control_comp,
        reversed: op.reversed,
        lead_shape: match op.lead {
            JetLead::Arc => LeadShape::Arc,
            JetLead::Straight => LeadShape::Straight,
        },
        lead_in: op.lead_in,
        lead_out: op.lead_out,
        open_side: match op.open_side {
            JetSide::Center => OpenSide::Center,
            JetSide::Left => OpenSide::Left,
            JetSide::Right => OpenSide::Right,
        },
        tabs: op.tabs.as_ref().map(|t| JetTabs {
            count: t.count,
            width: t.width,
            at: t.at.iter().map(|p| Point2::new(p[0], p[1])).collect(),
        }),
        power: op.power,
        pierce_time: op.pierce_time,
        linearize: op.linearize,
        feed: op.feed,
        clearance_z: hs.clearance,
        cut_z: top,
        model_excess,
    })
}

/// Generate: `picks` is the picked contours (closed rings, open runs)
/// when the source is a selection; otherwise `base` - the model's
/// through-silhouette - is cut whole. The kerf may never reach
/// `facets`' silhouette, judged with that model's excess given back.
pub(super) fn generate_laser(
    op: &LaserOp,
    tool: &Tool,
    hs: &ResolvedHeights,
    picks: Option<(Vec<Loop>, Vec<Vec<Point2>>)>,
    base: impl FnOnce() -> Result<Region, GenError>,
    (facets, model_excess): (&[Triangle], f64),
) -> Result<(Toolpath, Vec<Warning>), GenError> {
    let p = laser_params(op, tool, hs, model_excess)?;
    let part = if facets.is_empty() {
        Region::empty()
    } else {
        slice::project(facets, FOOTPRINT_SAMPLES)
            .map_err(|e| GenError::Invalid(format!("laser profile: the part's silhouette is unreadable: {e}")))?
    };
    let (loops, chains) = match picks {
        Some(geom) => geom,
        None => (base()?.loops().to_vec(), Vec::new()),
    };
    let path = laser_profile(&loops, &chains, &part, &p)?;
    Ok((path, Vec::new()))
}
