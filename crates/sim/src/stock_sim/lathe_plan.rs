//! What a turning op's plan carries into the field (#19; moved from
//! app's `stock_sim::lathe`, #2268, so the playback in this crate
//! holds it): the footprint and the spindle line. Provenance: our own
//! lathe clause; nothing from vendor binaries.

use super::{LatheAxis, LatheProfile};

/// What a turning op's plan carries into the field: the footprint
/// its moves sweep and the spindle line it revolves about.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LathePlan {
    pub profile: LatheProfile,
    pub axis: LatheAxis,
}

impl LathePlan {
    /// The footprint's finest cutting feature, the radius the display
    /// cell law (`auto_cell`) sizes the picture by: the nose, or half
    /// a blade's band.
    #[must_use]
    pub fn feature(&self) -> f64 {
        match self.profile {
            LatheProfile::Nose { radius } => radius,
            LatheProfile::Blade { width, .. } | LatheProfile::Thread { width, .. } | LatheProfile::Form { width, .. } => 0.5 * width,
        }
    }
}
