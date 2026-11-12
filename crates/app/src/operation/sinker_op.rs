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
use crate::sinker::{burn_to_depth, SinkerDatum, SinkerParams};
use crate::sinker_undersize::{inverted, judge, SinkerPass, Undersize};
use crate::tools::{Tool, ToolKind};
use crate::verify::Warning;

use super::kinds::Carried;
use super::sinker_pass::PassRead;
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
        pass: super::SinkerPass::Single,
        cavity_x: 0.0,
        cavity_y: 0.0,
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
    let c = condition(world, setup, op, &[ConditionField::Gap])?;
    let label = crate::setup_mount::machine_label(&crate::machine::of_setup(world, setup));
    c.gap.ok_or_else(|| format!("machine {label}: EDM condition {:?} states no spark gap", op.condition.trim()))
}

/// The burn's condition with every field in `need` stated, read from the
/// op's setup's machine - the one door #541's gap and #543's undersize
/// share. Refused by name: a setup not on a sinker EDM, no condition
/// named, no EDM section, or a condition missing or short of a field.
///
/// # Errors
/// Each refusal above, in words.
pub fn condition(
    world: &World,
    setup: Option<&str>,
    op: &SinkerBurnOp,
    need: &[ConditionField],
) -> Result<crate::machine::Condition, String> {
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
    edm.condition(&label, id, need).cloned()
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
        stand: None,
    })
}

/// Generate the burn. A rough or finish (#543) is first judged by the
/// undersize law (`sinker_undersize::judge`), which also places the tip
/// one undersize above the finished floor; a finish whose undersize is
/// not smaller than the rough's it follows is warned by name.
pub(super) fn generate_sinker(
    op: &SinkerBurnOp,
    tool: &Tool,
    hs: &ResolvedHeights,
    gap: f64,
    pass: Option<Result<PassRead, String>>,
    facets: &[Triangle],
) -> Result<(Toolpath, Vec<Warning>), GenError> {
    let mut p = sinker_params(op, tool, hs, gap, facets)?;
    let read = match (op.pass, pass) {
        (SinkerPass::Single, _) => None,
        (_, Some(r)) => Some(r.map_err(GenError::Invalid)?),
        (_, None) => {
            return Err(GenError::Invalid(format!("sinker {}: its condition's undersize was never read", op.pass.label())));
        }
    };
    if let Some(r) = &read {
        let u = Undersize {
            pass: op.pass,
            cavity: (op.cavity_x, op.cavity_y),
            electrode: (tool.diameter, tool.width),
            gap,
            undersize: r.undersize,
        };
        p.stand = Some(judge(&u, op.condition.trim())?);
    }
    let path = burn_to_depth(&p)?;
    let mut warns = Vec::new();
    if let Some(PassRead { undersize, rough: Some((_, rough)) }) = read
        && op.pass == SinkerPass::Finish
        && inverted(undersize, rough)
    {
        let at = path.moves.iter().position(|m| m.kind == crate::cl::MoveKind::Feed).unwrap_or(0);
        warns.push(Warning::SinkerFinishOrder { at, finish_mm: undersize, rough_mm: rough });
    }
    Ok((path, warns))
}
