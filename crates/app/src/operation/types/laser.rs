//! The laser profile operation's stored parameters (#4081): what the op
//! keeps, as heights and plain numbers - resolved into the motion
//! crate's `laser::LaserParams` at generation.
//!
//! Provenance: our own parameter set over general laser-cutting
//! practice (kerf side, leads with the pierce in the scrap, tabs as
//! uncut gaps, pierce dwell, beam power as a fraction of the source's
//! maximum) and RS-274 / ISO 6983 for G41/G42 and linear-only controls.
//! The kerf side, lead and tab types are the waterjet's: the same rules.

use super::height::Height;
use super::waterjet::{JetLead, JetSide, JetTabsOp};
use crate::path_picks::PathPicks;

/// The laser profile's parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct LaserOp {
    /// The beam-off travel plane.
    pub clearance_z: Height,
    /// The sheet's top: the plane the cut rides.
    pub top_z: Height,
    /// The sheet's bottom: nothing steps down to it - a laser cut goes
    /// through in one pass.
    pub bottom_z: Height,
    /// The kerf width override, mm; `None` reads the laser head tool's
    /// diameter (its kerf width).
    pub kerf: Option<f64>,
    /// Post G41/G42 for the control to offset, instead of the computer's
    /// offset path.
    pub control_comp: bool,
    /// Outer contours counter-clockwise and holes clockwise.
    pub reversed: bool,
    pub lead: JetLead,
    /// Lead-in / lead-out length, mm; 0 = none.
    pub lead_in: f64,
    pub lead_out: f64,
    /// The kerf side of open runs.
    pub open_side: JetSide,
    /// `None` = no tabs.
    pub tabs: Option<JetTabsOp>,
    /// Cut feed, mm/min.
    pub feed: f64,
    /// The beam power while it cuts, percent of the source's maximum.
    pub power: f64,
    /// The pierce dwell, s.
    pub pierce_time: f64,
    /// Arcs sent as chords within this tolerance, mm; `None` keeps them.
    pub linearize: Option<f64>,
    /// The operation's start and exit picks.
    pub picks: PathPicks,
}

impl LaserOp {
    /// Structural sanity (no world): every number finite and in range.
    /// The motion crate's door repeats the refusals by name at
    /// generation; this is the command door's yes / no.
    #[must_use]
    pub fn is_sane(&self) -> bool {
        let nneg = |v: f64| v.is_finite() && v >= 0.0;
        let pos = |v: f64| v.is_finite() && v > 0.0;
        self.clearance_z.is_sane()
            && self.top_z.is_sane()
            && self.bottom_z.is_sane()
            && self.kerf.is_none_or(pos)
            && nneg(self.lead_in)
            && nneg(self.lead_out)
            && pos(self.feed)
            && pos(self.power)
            && self.power <= 100.0
            && nneg(self.pierce_time)
            && self.linearize.is_none_or(pos)
            && self.tabs.as_ref().is_none_or(|t| {
                t.count > 0
                    && t.count as usize == t.at.len()
                    && pos(t.width)
                    && t.at.iter().all(|p| p[0].is_finite() && p[1].is_finite())
            })
            && self.picks.refusal().is_none()
    }
}
