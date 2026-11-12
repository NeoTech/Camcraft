//! The post-serves-kind door (#1232): a section reaches a post only
//! when that post serves the kind of machine its setup runs on.
//!
//! Provenance: our own rule, read off [`super::post_serves`] (the
//! picker's listing law, M13.5) and made binding at post time. That a
//! milling program (XY plane, tool change, tool spindle) cannot drive a
//! two-axis lathe, nor a cutting program a torch or a wire, is general
//! machining knowledge from the machine-tool handbooks. No vendor
//! material consulted.
//!
//! The law: before any NC text is written, every mount the program
//! posts (whole program, one file per mount, one file per operation)
//! is asked whether its post serves its machine's kind. A shipped post
//! is judged by its registry name ([`super::post_serves`]); a script of
//! the user's own by the dialect its manifest declares (the
//! `capabilities` mask, [`crate::post::dialect::work_voice`]): a lathe
//! takes only a script that declares turning, a mill or a plasma table
//! refuses one that declares turning, and no script serves an EDM (no
//! EDM dialect ships, #533). A refusal names the setup, its operations,
//! the post and the machine's kind - nothing is posted.
//!
//! A user's script states the kinds it serves in its manifest
//! (`serves = ["plasma"];`, #1266): it then posts to exactly those
//! kinds. One that states none is accepted only where its dialect
//! proves it (turning serves a lathe and a mill-turn, a non-turning
//! one a mill and a mill-turn); a plasma table and an EDM are refused
//! for it, and the refusal says to declare `serves`.

use crate::machine::{Kind, Machine};
use crate::post::dialect::{declared_serves, work_voice, WorkVoice};

/// The registry name of a script text, when it is a shipped post's.
fn registry_name(cps: &str) -> Option<&'static str> {
    super::BUILTINS.iter().find(|(_, text)| *text == cps).map(|(name, _)| *name)
}

/// Whether the script `cps` serves a machine of `kind`.
fn serves(cps: &str, kind: Kind) -> Result<bool, String> {
    if let Some(name) = registry_name(cps) {
        return Ok(super::post_serves(name, kind));
    }
    if let Some(kinds) = declared_serves(cps).map_err(|e| format!("post engine: {e}"))? {
        return Ok(kinds.contains(&kind));
    }
    let turning = work_voice(cps).map_err(|e| format!("post engine: {e}"))? == WorkVoice::WorkSpindle;
    Ok(match kind {
        Kind::Lathe => turning,
        Kind::MillTurn => true,
        Kind::Mill => !turning,
        // The mask cannot tell a plasma script from a milling one, so an
        // undeclared script is not proven to serve a torch or a wire.
        Kind::Plasma | Kind::WireEdm | Kind::SinkerEdm => false,
    })
}

/// The refusal for one mount, or `None` when its post serves it:
/// `setups` (effective names, `""` for the implicit setup) and their
/// `ops` (program order) run on `machine` through the script `cps`;
/// `label` names a script that is not a shipped one (its path).
///
/// # Errors
/// A script that does not load, as everywhere.
pub(super) fn refusal(
    setups: &[String],
    ops: &[&str],
    machine: &(String, Option<Machine>),
    cps: &str,
    label: Option<&str>,
) -> Result<Option<String>, String> {
    let kind = crate::machine::kind(machine.1.as_ref());
    if serves(cps, kind)? {
        return Ok(None);
    }
    let who = if setups.iter().all(String::is_empty) { "the setup".to_owned() } else { setups.join(", ") };
    let post = match (registry_name(cps), label) {
        (Some(name), _) => format!("post {name}"),
        (None, Some(path)) => format!("post script {path}"),
        (None, None) => "the given post script".to_owned(),
    };
    Ok(Some(format!(
        "{who} ({}): {post} does not serve {}, a {} machine - it would write a program that machine cannot run; \
         give the setup a post that serves {}, or suppress its operations{}",
        ops.join(", "),
        crate::setup_mount::machine_label(machine),
        kind.label(),
        kind.label(),
        if registry_name(cps).is_some() {
            String::new()
        } else {
            format!(" (a script of your own states the kinds it serves: serves = [\"{}\"];)", kind.word())
        }
    )))
}
