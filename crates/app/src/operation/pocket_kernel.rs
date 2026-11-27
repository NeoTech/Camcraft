//! The 2D Pocket arm under the planner switch "new" (#3752): the options
//! that the first kernel recipe does not plan yet are refused by name
//! before the kernel runs, never dropped and never handed to the old
//! planner. The recipe itself is in `camcraft-motion`
//! (`machining2d::pocket_on_kernel`).
//! Provenance: our own design record `docs/decisions/4024-path-kernel-design.md`
//! ("The one switch": a kind that the new engine does not plan is refused
//! by name) and `docs/decisions/3752-kernel-pocket-recipe.md`.

use super::GenError;
use crate::machining2d::{CornerRelief, PocketStrategy, RestParams};

/// Refusal: rest machining of the 2D Pocket under the new planner.
pub const KERNEL_REST_NOT_YET: &str = "the new path planner does not plan rest machining of the 2D Pocket yet (#3753) - turn rest machining off, or unset CAMCRAFT_PLANNER";
/// Refusal: the "do not touch" areas under the new planner.
pub const KERNEL_AVOID_NOT_YET: &str =
    "the new path planner does not plan the 2D Pocket round \"do not touch\" areas yet (#3754) - clear them, or unset CAMCRAFT_PLANNER";
/// Refusal: the corner relief under the new planner.
pub const KERNEL_RELIEF_NOT_YET: &str = "the new path planner does not plan the corner relief of the 2D Pocket yet - turn it off, or unset CAMCRAFT_PLANNER";
/// Refusal: the Adaptive strategy under the new planner (as under the old one).
pub const KERNEL_ADAPTIVE: &str = crate::machining2d::KERNEL_POCKET_ADAPTIVE;

/// The refusal of a 2D Pocket that the kernel recipe does not plan, if any.
pub(super) fn out_of_scope(
    strategy: PocketStrategy,
    rest: Option<&RestParams>,
    avoids: bool,
    relief: &CornerRelief,
) -> Result<(), GenError> {
    let why = if strategy == PocketStrategy::Adaptive {
        Some(KERNEL_ADAPTIVE)
    } else if rest.is_some() {
        Some(KERNEL_REST_NOT_YET)
    } else if avoids {
        Some(KERNEL_AVOID_NOT_YET)
    } else if !relief.is_off() {
        Some(KERNEL_RELIEF_NOT_YET)
    } else {
        None
    };
    why.map_or(Ok(()), |w| Err(GenError::Invalid(w.to_owned())))
}
