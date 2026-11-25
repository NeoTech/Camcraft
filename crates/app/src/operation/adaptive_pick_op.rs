//! The 2D Adaptive's start and exit points at generation (#3327): the
//! operation's picks, refused by name with the door's own words, or laid
//! over the generation's copy of the approach so that the HSM plan reads
//! them (`machining2d::adaptive_picks`).
//!
//! Provenance: our own parameter plumbing over the general machining
//! practice of a chosen entry and exit position; no vendor material.

use crate::machining2d::{GenError, PocketParams};
use crate::path_picks::PickRole;

use super::types::OpParams;

/// Refuse the picks of `params` by name, or lay them over `pp`. Without
/// a pick `pp` is not changed, so every earlier op plans the same path.
pub(super) fn lay(params: &OpParams, pp: &mut PocketParams) -> Result<(), GenError> {
    let picks = params.path_picks();
    if picks.is_empty() {
        return Ok(());
    }
    if let Some(why) = params.pick_refusal() {
        return Err(GenError::Invalid(why));
    }
    pp.links.approach.start_point = picks.first(PickRole::Start).map(|k| k.xy);
    pp.links.approach.exit_point = picks.first(PickRole::Exit).map(|k| k.xy);
    Ok(())
}
