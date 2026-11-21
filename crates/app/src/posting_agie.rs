//! Native Agie job dispatch and posting preflight. Provenance: CAMCraft's
//! verified Wire Contour workflow and the local Agie controller job contract.

use std::collections::BTreeMap;
use std::path::Path;

use flecs_ecs::prelude::World;

use super::{Posted, mounts};
use crate::post::{cnc, options::OptionValue};
use crate::schema::{PostOptions, PostRef};
use crate::{model, operation};

pub(crate) use camcraft_post::agie::AgieJob;
pub use camcraft_post::agie::valid_name;

pub const POST: &str = "Agie native wire EDM";
const MODEL: &str = "AGIECUT CLASSIC 2S";
pub(crate) const CONDITION: &str = "AGIE_QN2";
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
    let [(op_name, entity)] = ops.as_slice() else {
        return Err("An Agie job requires exactly one active Wire Contour operation".into());
    };
    let op = model::operation_of(world, entity).ok_or("Agie operation is unavailable")?;
    let setup = op.setup.as_deref();
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
    for (_, records) in super::op_streams_by(world, false)? {
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
    let job = operation::agie_job(world, &op, op_name)?;
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
    let bundle = camcraft_post::agie::export_with_review(&job, dest, name, separation)?;
    Ok(Posted {
        bundle_files: bundle.files,
        controller_review: Some(bundle.controller_review.into()),
        ops: 1,
        lines: 0,
        program: String::new(),
        line_pos: Vec::new(),
        blocks: Vec::new(),
        not_judged: report.not_judged,
    })
}
