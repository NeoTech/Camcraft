//! Laser profile cutting (#4081): 2D through-cuts of closed and open
//! contours with a laser beam - the kerf offset, inner contours before
//! the contour that frees them, straight or arc leads with the pierce
//! in the waste, tabs left as uncut gaps, and the tight-hole rule.
//!
//! Provenance (public sources, behaviour only, our own words and code):
//! - General laser-cutting shop practice: the kerf is the burnt width
//!   of the focused beam, the cut path stands kerf / 2 off the part on
//!   the waste side, a pierce is made in the scrap and led in, holes
//!   are cut before the outline that frees the part, and tabs
//!   (micro-joints) hold a part in the sheet. The same rules as the
//!   abrasive waterjet, so this cut rides that generator
//!   ([`crate::waterjet`]) under its own voice.
//! - RS-274 / ISO 6983 for G41/G42 and for linear-only controls.
//!
//! The power is a fraction of the source's maximum (the Grbl laser
//! mode reads it as the S word against `$30`); this module only checks
//! and states it. Nothing here guesses: each refusal names its value.

use crate::cl::Toolpath;
use crate::geo2d::{Loop, Point2, Region};
use crate::machining2d::GenError;
use crate::sheet::Voice;
use crate::waterjet::{waterjet_profile, WaterjetParams};

pub use crate::waterjet::{JetTabs, LeadShape, OpenSide};

/// Everything the generator reads, resolved to plain numbers (mm,
/// mm/min, seconds, percent).
#[derive(Clone, Debug, PartialEq)]
pub struct LaserParams {
    /// The kerf width (the laser head tool's diameter, or the op's
    /// override).
    pub kerf: f64,
    /// Post G41/G42 instead of the offset path the computer walked.
    pub control_comp: bool,
    /// Run outer contours counter-clockwise and holes clockwise.
    pub reversed: bool,
    pub lead_shape: LeadShape,
    /// Lead-in / lead-out length, mm (0 = none).
    pub lead_in: f64,
    pub lead_out: f64,
    /// The kerf side of every open run.
    pub open_side: OpenSide,
    pub tabs: Option<JetTabs>,
    /// The beam power while it cuts, percent of the source's maximum:
    /// more than 0, at most 100.
    pub power: f64,
    /// Seconds the beam dwells to pierce.
    pub pierce_time: f64,
    /// Send every arc as chords at this tolerance (mm); `None` keeps
    /// the arcs.
    pub linearize: Option<f64>,
    pub feed: f64,
    /// The beam-off travel plane, and the plane the cut rides (the
    /// sheet top).
    pub clearance_z: f64,
    pub cut_z: f64,
    /// How much coarser than the importer's promise the model is, mm
    /// (#1053).
    pub model_excess: f64,
}

impl LaserParams {
    /// The waterjet generator's numbers under the laser's voice: no
    /// abrasive, the same geometry laws.
    #[must_use]
    pub fn as_sheet(&self) -> WaterjetParams {
        WaterjetParams {
            kerf: self.kerf,
            control_comp: self.control_comp,
            reversed: self.reversed,
            lead_shape: self.lead_shape,
            lead_in: self.lead_in,
            lead_out: self.lead_out,
            open_side: self.open_side,
            tabs: self.tabs.clone(),
            pierce_time: self.pierce_time,
            abrasive_delay: 0.0,
            linearize: self.linearize,
            feed: self.feed,
            clearance_z: self.clearance_z,
            cut_z: self.cut_z,
            model_excess: self.model_excess,
            voice: Voice::LASER,
        }
    }

    /// The door: the power by name, then every number the shared
    /// generator checks, in the laser's voice.
    pub fn door(&self) -> Result<(), GenError> {
        if !(self.power.is_finite() && self.power > 0.0 && self.power <= 100.0) {
            return Err(Voice::LASER.invalid(format!(
                "the power must be more than 0 and at most 100 % of the source's maximum (it is {})",
                self.power
            )));
        }
        self.as_sheet().door()
    }
}

/// Cut `loops` (closed contours) and `chains` (open runs) with the
/// beam. `part` is the material the kerf must never reach; an empty
/// region skips that law (said in the notes).
pub fn laser_profile(loops: &[Loop], chains: &[Vec<Point2>], part: &Region, p: &LaserParams) -> Result<Toolpath, GenError> {
    p.door()?;
    let mut path = waterjet_profile(loops, chains, part, &p.as_sheet())?;
    path.notes.push(format!(
        "laser: requested power {:.1} % of the source's maximum at {:.0} mm/min; no power record is posted yet",
        p.power, p.feed
    ));
    Ok(path)
}

#[cfg(test)]
mod tests;
