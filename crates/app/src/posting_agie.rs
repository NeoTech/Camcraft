//! Native Agie job dispatch and posting preflight. Provenance: CAMCraft's
//! verified Wire Contour workflow and the local Agie controller job contract.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use camcraft_post::agie::MAX_PUNCH_WORKS;
use flecs_ecs::prelude::World;

use super::{Posted, mounts};
use crate::post::{cnc, options::OptionValue};
use crate::schema::{PostOptions, PostRef};
use crate::{model, operation};

pub use camcraft_post::agie::valid_name;
pub(crate) use camcraft_post::agie::{AgieJob, AgieQuality, AgieSide};

pub const POST: &str = "Agie native wire EDM";
const MODEL: &str = "AGIECUT CLASSIC 2S";
pub(crate) const PREVIEW_GAP: f64 = 0.005;

pub fn is_native(post: &PostRef) -> bool {
    matches!(post, PostRef::Named(name) if name == POST)
}

pub(super) fn post(
    world: &World,
    dest: &Path,
    extra: Option<&BTreeMap<String, OptionValue>>,
) -> Result<Posted, String> {
    let mount = mounts::single(world, false)?;
    let Some(machine) = mount.machine.1.as_ref() else {
        return Err(format!("The Agie job requires the {MODEL} machine profile"));
    };
    if machine.model != MODEL
        || crate::machine::kind(Some(machine)) != crate::machine::Kind::WireEdm
    {
        return Err(format!("The Agie job requires the {MODEL} machine profile"));
    }
    if let Some(axis) = machine.axes.iter().find(|axis| {
        axis.travel.is_some()
            && (axis.name.eq_ignore_ascii_case("X") || axis.name.eq_ignore_ascii_case("Y"))
    }) {
        return Err(format!(
            "Agie cannot verify the declared {} travel against the control's resident rough and trim offsets at the installed work datum",
            axis.name
        ));
    }
    let name = dest.file_name().and_then(|x| x.to_str()).filter(|s| valid_name(s))
        .ok_or("Choose a new Agie job directory named with 1 to 8 uppercase letters or digits, starting with a letter")?;
    let ops = super::program_ops(world)?;
    if !(1..=MAX_PUNCH_WORKS).contains(&ops.len()) {
        return Err(format!(
            "An Agie job requires one active contour, up to {MAX_PUNCH_WORKS} independent punches or two die openings"
        ));
    }
    let operations = ops
        .iter()
        .map(|(name, entity)| {
            model::operation_of(world, entity)
                .map(|op| (name, op))
                .ok_or("Agie operation is unavailable")
        })
        .collect::<Result<Vec<_>, _>>()?;
    let (_, first) = operations
        .first()
        .ok_or("Agie requires an active Wire Contour")?;
    let setup = first.setup.as_deref();
    if operations
        .iter()
        .any(|(_, op)| op.setup != first.setup || op.tool != first.tool)
    {
        return Err("Agie grouped works must use the same setup and wire tool".into());
    }
    if crate::setup_post_options::of(world, setup) != PostOptions::default() {
        return Err("Agie jobs require default post options and One program output".into());
    }
    if crate::units::of(world) != crate::units::Unit::Millimeters {
        return Err("Agie jobs require a millimetre document".into());
    }
    let info = super::program_info(world);
    if info.number.is_some() || info.comment.is_some() {
        return Err(
            "Agie uses the job directory name; clear the NC program number and comment".into(),
        );
    }

    // Use the ordinary post's current operation order, freshness, warning
    // records, setup laws and verification. The wire centres in the stream
    // are only verification evidence; the writer receives the nominal ring.
    for (op_name, records) in super::op_streams_by(world, false)? {
        for record in records {
            if let cnc::Record::Warn { text } = record {
                return Err(format!("Agie post blocked - {op_name}: {text}"));
            }
        }
    }
    let report = super::verification_report(world);
    if !report.problems.is_empty() {
        return Err(format!(
            "Agie post blocked - {}",
            report.problems.join("; ")
        ));
    }
    let models = model::model_entities(world);
    let jobs = match operations.as_slice() {
        [(name, op)] => {
            let sketches = crate::sketch::placed_in(world, setup);
            if let ([], [profile]) = (models.as_slice(), sketches.as_slice()) {
                vec![operation::agie_job(world, op, name, profile, 1)?]
            } else {
                if !sketches.is_empty() {
                    return Err(
                        "Agie requires a single sketch or a single modeled workpiece".into(),
                    );
                }
                let [(source, _)] = models.as_slice() else {
                    return Err("Agie requires exactly one modeled workpiece for one active contour; additional models can intersect the controller path".into());
                };
                vec![operation::agie_job(world, op, name, source, 1)?]
            }
        }
        [(_, left), (_, right)]
            if [&left.params, &right.params].iter().all(|params| {
                matches!(params, operation::OpParams::WireContour(wire) if wire.side == operation::WireSide::Die)
            }) =>
        {
            let [(plate, _)] = models.as_slice() else {
                return Err("Both Agie die operations must select through-openings on the same retained plate, with no other model".into());
            };
            if !crate::sketch::placed_in(world, setup).is_empty() {
                return Err("The grouped Agie die plate must be the only workpiece".into());
            }
            // Each selection must match one of the two physical openings.
            // The group preflight requires disjoint cut and threading envelopes,
            // so selecting the same opening twice cannot leave an uncut rim.
            operations
                .iter()
                .map(|(name, op)| operation::agie_job(world, op, name, plate, 2))
                .collect::<Result<Vec<_>, _>>()?
        }
        _ => {
            let mut owners = BTreeSet::new();
            let mut jobs = Vec::with_capacity(operations.len());
            for (name, op) in &operations {
                let operation::SrcRef::Entity(owner) = &op.src else {
                    return Err("Select each complete punch model or two through-openings on the same die plate".into());
                };
                if !matches!(&op.params, operation::OpParams::WireContour(wire) if wire.side == operation::WireSide::Punch) {
                    return Err("Every operation in an Agie punch group must be a closed punch".into());
                }
                if !owners.insert(owner.as_str()) {
                    return Err("Each Agie punch must have a distinct modeled source".into());
                }
                jobs.push(operation::agie_job(world, op, name, owner, 1)?);
            }
            if owners.len() != models.len()
                || models.iter().any(|(name, _)| !owners.contains(name.as_str()))
                || !crate::sketch::placed_in(world, setup).is_empty()
            {
                return Err("The Agie punch group must cover every modeled workpiece without an additional model or placed sketch".into());
            }
            jobs
        }
    };
    let options = super::program_options_for_script(
        world,
        super::builtin_post_text(POST)?,
        Some(&PostRef::Named(POST.into())),
        setup,
        extra,
    )?;
    let separation = options
        .get("separationLengthMm")
        .and_then(|v| v.as_f64())
        .ok_or("The Agie post must declare its separation cut length in millimetres")?;
    let bundle_files = camcraft_post::agie::export_works(&jobs, dest, name, separation)?;
    Ok(Posted {
        bundle_files,
        ops: jobs.len(),
        lines: 0,
        program: String::new(),
        line_pos: Vec::new(),
        blocks: Vec::new(),
        not_judged: report.not_judged,
    })
}
