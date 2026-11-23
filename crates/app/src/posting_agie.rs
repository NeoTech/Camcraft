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

/// The registry name of the native wire EDM post, the one place it is spelled (#3537).
pub const POST: &str = "Native wire EDM job files";
/// #3537: the name this post had before; a hidden alias that no list shows.
/// A saved project, an option key `builtin:<name>`, a script or an MCP call
/// that still says it selects [`POST`].
pub const LEGACY_POST: &str = "Agie native wire EDM";
const MODEL: &str = "AGIECUT CLASSIC 2S";
pub(crate) const CONDITION: &str = "AGIE_QN2";
pub(crate) const PREVIEW_GAP: f64 = 0.005;

/// The registry name that `name` selects: [`POST`] for its hidden alias, else `name`.
#[must_use]
pub fn current_name(name: &str) -> &str {
    if name == LEGACY_POST { POST } else { name }
}

/// `post` with a hidden alias name replaced by the registry name it selects.
#[must_use]
pub fn current_ref(post: PostRef) -> PostRef {
    match post {
        PostRef::Named(name) if name == LEGACY_POST => PostRef::Named(POST.to_owned()),
        other => other,
    }
}

/// An option key `builtin:<alias>` rewritten to the key of the registry name.
#[must_use]
pub fn current_key(key: &str) -> String {
    match key.strip_prefix("builtin:") {
        Some(name) if name == LEGACY_POST => format!("builtin:{POST}"),
        _ => key.to_owned(),
    }
}

pub fn is_native(post: &PostRef) -> bool {
    matches!(post, PostRef::Named(name) if current_name(name) == POST)
}

pub(super) fn post(
    world: &World,
    dest: &Path,
    extra: Option<&BTreeMap<String, OptionValue>>,
) -> Result<Posted, String> {
    let mount = mounts::single(world, false)?;
    let Some(machine) = mount.machine.1.as_ref() else {
        return Err(format!("The native wire EDM job requires the {MODEL} machine profile"));
    };
    if machine.model != MODEL
        || crate::machine::kind(Some(machine)) != crate::machine::Kind::WireEdm
    {
        return Err(format!("The native wire EDM job requires the {MODEL} machine profile"));
    }
    if let Some(axis) = machine.axes.iter().find(|axis| {
        axis.travel.is_some()
            && (axis.name.eq_ignore_ascii_case("X") || axis.name.eq_ignore_ascii_case("Y"))
    }) {
        return Err(format!(
            "The native wire EDM post cannot verify the declared {} travel against the control's resident rough and trim offsets at the installed work datum",
            axis.name
        ));
    }
    let name = dest.file_name().and_then(|x| x.to_str()).filter(|s| valid_name(s))
        .ok_or("Choose a new native wire EDM job directory named with 1 to 8 uppercase letters or digits, starting with a letter")?;
    let ops = super::program_ops(world)?;
    let [(op_name, entity)] = ops.as_slice() else {
        return Err("A native wire EDM job requires exactly one active Wire Contour operation".into());
    };
    let op = model::operation_of(world, entity).ok_or("The Wire Contour operation is unavailable")?;
    let setup = op.setup.as_deref();
    if crate::setup_post_options::of(world, setup) != PostOptions::default() {
        return Err("Native wire EDM jobs require default post options and One program output".into());
    }
    if crate::units::of(world) != crate::units::Unit::Millimeters {
        return Err("Native wire EDM jobs require a millimetre document".into());
    }
    let info = super::program_info(world);
    if info.number.is_some() || info.comment.is_some() {
        return Err(
            "The native wire EDM job uses the job directory name; clear the NC program number and comment".into(),
        );
    }

    // Use the ordinary post's current operation order, freshness, warning
    // records, setup laws and verification. The wire centres in the stream
    // are only verification evidence; the writer receives the nominal ring.
    for (_, records) in super::op_streams_by(world, false)? {
        for record in records {
            if let cnc::Record::Warn { text } = record {
                return Err(format!("Native wire EDM post blocked - {op_name}: {text}"));
            }
        }
    }
    let report = super::verification_report(world);
    if !report.problems.is_empty() {
        return Err(format!(
            "Native wire EDM post blocked - {}",
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
        .ok_or("The native wire EDM post must declare its separation cut length in millimetres")?;
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
