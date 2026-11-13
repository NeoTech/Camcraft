//! The family door at post (#530): an op made on one machine and then
//! remounted onto steel that cannot run it (an EDM, a lathe under a
//! milling op) is refused by name - never posted in a dialect its
//! machine does not speak. The same law as the create-time
//! `family_gate`, read again over the program the post is about to emit.
//!
//! Provenance: our own rule (the umbrella #290's design notes D1-D3:
//! an EDM runs no milling or turning op); split out of `posting` so the
//! posting module does not grow.

use flecs_ecs::prelude::*;

use crate::model;
use crate::operation::OpParams;

/// Refuse the first program op whose family its setup's machine does
/// not admit, naming the op, its kind and the machine kind - or whose
/// kind mounts no library tool and has no post section of its own
/// (#532/#533: only the Wire Contour's, `posting_wire`, is built).
///
/// # Errors
/// `"<op>: a <kind> operation cannot run on its setup's <machine> machine - ..."`,
/// or `"<op>: a <kind> operation has no post section yet - nothing is posted"`.
pub(crate) fn family_at_post(world: &World, ops: &[(String, Entity)]) -> Result<(), String> {
    for (name, e) in ops {
        if let Some(op) = model::operation_of(world, e) {
            let machine = crate::machine::kind_of_setup(world, op.setup.as_deref());
            if !op.params.kind().info().admits(machine) {
                return Err(format!(
                    "{name}: a {} operation cannot run on its setup's {} machine - mount a machine that runs it, or suppress the operation",
                    op.params.kind(),
                    machine.label()
                ));
            }
            // #532/#533: an op that mounts no tool posts only as its own
            // kind's section - the Wire Contour's (`posting_wire`), whose
            // condition word only a wire EDM post voices; any other
            // tool-less kind is refused by name, never posted (and before
            // any tool is read: it has none).
            if !op.params.kind().info().mounts_tool() && !matches!(op.params, OpParams::WireContour(_)) {
                return Err(format!(
                    "{name}: a {} operation has no post section yet - nothing is posted",
                    op.params.kind()
                ));
            }
        }
    }
    Ok(())
}

/// The EDM door at post (review of #1131): no sinker EDM dialect ships
/// yet (#542), so an op whose setup's machine is a sinker EDM is refused
/// by name before any file is written - never handed to a milling post
/// that would address the EDM control with a tool change, a length
/// offset and the electrode tip as a Z target. A wire EDM passes this
/// door since its dialect shipped (#533, `wire-iso.cps`): its own
/// refusals come from `post_serves` (no cutting post is offered for a
/// wire machine), the engine's `wireEdm` door (no cutting script posts a
/// wire path) and the wire post itself (each missing `edm.codes.<x>`
/// refused by name). Shared by every writer: the one-file post, the
/// per-mount and the per-op writers, each beside its lathe door.
///
/// # Errors
/// `"<setup>: no post for a sinker EDM machine yet (#542) - ..."`.
pub(crate) fn edm_at_post<'a>(world: &World, names: impl IntoIterator<Item = &'a str>) -> Result<(), String> {
    use crate::machine::Kind;
    for name in names {
        let Some(op) = model::find_by_name(world, name).and_then(|e| model::operation_of(world, &e)) else {
            continue;
        };
        let machine = crate::machine::kind_of_setup(world, op.setup.as_deref());
        let issue = match machine {
            Kind::SinkerEdm => "#542",
            // #533: the wire dialect ships; its refusals live in the post.
            Kind::WireEdm | Kind::Mill | Kind::Lathe | Kind::MillTurn | Kind::Plasma => continue,
        };
        let setup = crate::setup::effective(world, op.setup.as_deref()).unwrap_or_else(|| "the setup".to_owned());
        return Err(format!(
            "{setup}: no post for a {} machine yet ({issue}) - {name} is not posted and no file is written; \
             a sinker EDM dialect ships with {issue}",
            machine.label()
        ));
    }
    Ok(())
}
