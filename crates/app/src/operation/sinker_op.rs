//! The Sinker Burn operation's seed, condition read and generation
//! (#541): the op's stored numbers resolved against the world (heights,
//! the electrode's length, the setup machine's spark gap) and
//! `sinker::burn_to_depth` run over them.
//!
//! Provenance: our own glue over the motion crate's sinker generator,
//! whose module head cites its public sources (general die-sink EDM
//! practice); the condition table is #531's user data (#290 D4).

use flecs_ecs::prelude::World;

use crate::cl::Toolpath;
use crate::geometry::Triangle;
use crate::machine::{ConditionField, Kind};
use crate::machining2d::GenError;
use crate::sinker::{SinkerDatum, SinkerParams};
use crate::sinker_wear::burn_compensated;
use crate::tools::{Tool, ToolKind};
use crate::verify::Warning;

use super::kinds::Carried;
use super::types::{OpParams, SinkerBurnOp};
use super::ResolvedHeights;

/// The sinker draft: the carried heights (work top, and the model's
/// bottom as the floor datum), a 5 mm burn from the work top at the
/// origin, a 50 mm/min servo feed, and no condition named - the user
/// names one of the machine's own (#290 D4: none ship).
pub(super) fn seed_sinker_burn(c: &Carried) -> OpParams {
    OpParams::SinkerBurn(SinkerBurnOp {
        clearance_z: c.clearance,
        top_z: c.top,
        floor_z: c.bottom,
        datum: SinkerDatum::WorkTop,
        depth: 5.0,
        x: 0.0,
        y: 0.0,
        feed: 50.0,
        condition: String::new(),
        layer: None,
        wear_lost: None,
        wear_over: None,
    })
}

/// The spark gap per side of the burn's condition, read from the op's
/// setup's machine (#531's door). Refused by name: a setup not on a
/// sinker EDM, no condition named, or a condition that is missing or
/// states no gap. Only the sinker's own field (the gap) is asked for -
/// never a wire field.
///
/// # Errors
/// Each refusal above, in words.
pub fn condition_gap(world: &World, setup: Option<&str>, op: &SinkerBurnOp) -> Result<f64, String> {
    let kind = crate::machine::kind_of_setup(world, setup);
    if kind != Kind::SinkerEdm {
        return Err(format!(
            "a Sinker Burn burns on a sinker EDM; the setup's machine is a {}",
            kind.word()
        ));
    }
    let mounted = crate::machine::of_setup(world, setup);
    let label = crate::setup_mount::machine_label(&mounted);
    let id = op.condition.trim();
    if id.is_empty() {
        return Err(format!(
            "the Sinker Burn names no EDM condition - name one of machine {label}'s conditions (its spark gap places the electrode)"
        ));
    }
    let edm = mounted
        .1
        .as_ref()
        .and_then(|m| m.edm.as_ref())
        .ok_or_else(|| format!("machine {label} states no EDM section, so no condition {id:?}"))?;
    let c = edm.condition(&label, id, &[ConditionField::Gap])?;
    c.gap.ok_or_else(|| format!("machine {label}: EDM condition {id:?} states no spark gap"))
}

/// The generator's numbers from the op's, the electrode's and the world's.
///
/// # Errors
/// A tool that is not a sound electrode, or heights that do not resolve.
pub fn sinker_params(
    op: &SinkerBurnOp,
    tool: &Tool,
    hs: &ResolvedHeights,
    gap: f64,
    facets: &[Triangle],
) -> Result<SinkerParams, GenError> {
    if tool.kind != ToolKind::Electrode {
        return Err(GenError::Invalid(format!(
            "a Sinker Burn mounts a sinker electrode, not a {}",
            tool.kind
        )));
    }
    if let Some(why) = tool.torch_refusal() {
        return Err(GenError::Invalid(why));
    }
    let (Some(top), Some(floor)) = (hs.top, hs.bottom) else {
        return Err(GenError::Invalid("sinker burn: the work top and the cavity floor must both resolve".into()));
    };
    let model_floor = facets
        .iter()
        .flat_map(|t| [t.a.z, t.b.z, t.c.z])
        .fold(f64::INFINITY, f64::min);
    Ok(SinkerParams {
        x: op.x,
        y: op.y,
        approach_z: hs.clearance,
        top_z: top,
        floor_z: floor,
        model_floor_z: model_floor.is_finite().then_some(model_floor),
        datum: op.datum,
        depth: op.depth,
        electrode_len: tool.flute_len,
        feed: op.feed,
        gap,
    })
}

/// Generate the burn, wear-compensated in Z when a rate is stated
/// (#546: measured on the op, else the condition's `cond_wear`).
pub(super) fn generate_sinker(
    op: &SinkerBurnOp,
    tool: &Tool,
    hs: &ResolvedHeights,
    gap: f64,
    cond_wear: Option<f64>,
    facets: &[Triangle],
) -> Result<(Toolpath, Vec<Warning>), GenError> {
    if let Some(why) = op.wear_refusal() {
        return Err(GenError::Invalid(why));
    }
    let p = sinker_params(op, tool, hs, gap, facets)?;
    let w = super::sinker_wear_op::sinker_wear(op, cond_wear);
    Ok((burn_compensated(&p, &w)?, Vec::new()))
}
