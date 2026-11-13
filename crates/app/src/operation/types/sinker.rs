//! The Sinker Burn operation's stored parameters (#541): what the op
//! keeps, as heights and plain numbers - resolved into the motion
//! crate's `sinker::SinkerParams` at generation.
//!
//! Provenance: our own parameter set over general die-sink EDM practice
//! (public machine-tool handbooks): an electrode positioned over the
//! cavity and fed down Z to a depth read from a datum, the spark gap
//! from the machine's condition (#531). No vendor material.

use super::height::Height;
pub use crate::sinker::SinkerDatum;
pub use crate::sinker_orbit::{Orbit as SinkerOrbit, OrbitKind as SinkerOrbitKind};

/// The Sinker Burn's parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct SinkerBurnOp {
    /// The approach / retract plane the electrode tip travels on.
    pub clearance_z: Height,
    /// The work's top (the `WorkTop` datum, and the electrode-length law).
    pub top_z: Height,
    /// The cavity floor datum: no burn passes it.
    pub floor_z: Height,
    pub datum: SinkerDatum,
    /// The depth from the datum, mm (from the cavity floor: the
    /// stand-off left above it; 0 burns exactly to it).
    pub depth: f64,
    /// The electrode face centre the burn is made at, mm.
    pub x: f64,
    pub y: f64,
    /// The servo feed down, mm/min.
    pub feed: f64,
    /// The machine's EDM condition id (#531): its spark gap places the
    /// tip. Required - an unstated condition is refused by name.
    pub condition: String,
    /// The orbit at the burn floor (#544), expanded into explicit moves;
    /// `None` burns a straight plunge. The pattern must be one the
    /// machine lists and the radius within its stated maximum - both
    /// read at generation, refused by name.
    pub orbit: Option<SinkerOrbit>,
}

impl SinkerBurnOp {
    /// Structural sanity (no world): every number finite and in range.
    /// The motion crate's door repeats the refusals by name.
    #[must_use]
    pub fn is_sane(&self) -> bool {
        self.clearance_z.is_sane()
            && self.top_z.is_sane()
            && self.floor_z.is_sane()
            && self.depth.is_finite()
            && self.depth >= 0.0
            && self.x.is_finite()
            && self.y.is_finite()
            && self.feed.is_finite()
            && self.feed > 0.0
            && self.orbit.as_ref().is_none_or(|o| o.check().is_ok())
    }
}

pub use crate::sinker_side::{SideBurn as SinkerSide, SideDirection as SinkerSideDirection};

/// The Sinker Side Burn's parameters (#545): its own burn (approach,
/// heights, datum, depth, point, feed and its OWN condition - no
/// orbit), the axis direction it moves along, the lateral amount it
/// moves the faced wall by, and its own electrode undersize per side.
#[derive(Clone, Debug, PartialEq)]
pub struct SinkerSideBurnOp {
    /// The plunge to depth the side move starts from (#541's fields).
    /// Its `orbit` stays `None`: a side burn moves along one axis.
    pub burn: SinkerBurnOp,
    pub direction: SinkerSideDirection,
    /// How far the faced wall moves out, mm.
    pub amount: f64,
    /// The electrode's undersize per side against the finished cavity,
    /// mm: with the condition's spark gap it bounds the amount.
    pub undersize: f64,
}

impl SinkerSideBurnOp {
    /// The side's numbers as the motion crate reads them.
    #[must_use]
    pub fn side(&self) -> SinkerSide {
        SinkerSide { direction: self.direction, amount: self.amount, undersize: self.undersize }
    }

    /// The structural refusal by name (no world): an orbit, an amount or
    /// undersize out of range, an amount past the undersize. `None`
    /// when sound.
    #[must_use]
    pub fn refusal(&self) -> Option<String> {
        if self.burn.orbit.is_some() {
            return Some("a Sinker Side Burn moves along one axis - it takes no orbit (orbit with a Sinker Burn)".into());
        }
        crate::sinker_side::side_refusal(&self.side())
    }

    /// Structural sanity (no world): the burn's, and no [`Self::refusal`].
    #[must_use]
    pub fn is_sane(&self) -> bool {
        self.burn.is_sane() && self.refusal().is_none()
    }
}
