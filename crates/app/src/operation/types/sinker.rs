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
pub use crate::sinker_undersize::SinkerPass;

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
    /// Which pass of the cavity this burn is (#543): a single electrode
    /// (#541), or the rough or finish of an undersized pair.
    pub pass: SinkerPass,
    /// The finished cavity's X and Y size, mm (#543): what a rough or
    /// finish burn is judged against. Unread for a single burn; a rough
    /// or finish without it is refused by name.
    pub cavity_x: f64,
    pub cavity_y: f64,
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
            && self.cavity_sane()
            && self.orbit.as_ref().is_none_or(|o| o.check().is_ok())
    }

    /// The finished cavity's numbers: unread (any finite, non-negative
    /// value) for a single burn; a rough or finish must pass
    /// [`Self::pass_refusal`].
    fn cavity_sane(&self) -> bool {
        let nneg = |v: f64| v.is_finite() && v >= 0.0;
        nneg(self.cavity_x) && nneg(self.cavity_y) && self.pass_refusal().is_none()
    }

    /// The rough / finish pass's structural refusal by name (#543, no
    /// world), so it answers at command creation, not first at generate:
    /// the finished cavity's X and Y must both be stated (> 0), and the
    /// pass burns without an orbit - the undersize law sizes the burn as
    /// electrode plus two spark gaps and does not yet count an orbit's
    /// radius, which would widen it past that (#1355). `None` for a single burn
    /// or a sound pass.
    #[must_use]
    pub fn pass_refusal(&self) -> Option<String> {
        if self.pass == SinkerPass::Single {
            return None;
        }
        let pass = self.pass.label();
        let stated = |v: f64| v.is_finite() && v > 0.0;
        if !stated(self.cavity_x) || !stated(self.cavity_y) {
            return Some(format!(
                "a sinker {pass} is judged against the finished cavity - state its X and Y size (both above 0)"
            ));
        }
        if self.orbit.is_some() {
            return Some(format!(
                "a sinker {pass} burns without an orbit: the undersize law does not count an orbit's radius yet (#1355) - burn the {pass} as a plunge"
            ));
        }
        None
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
        if self.burn.pass != SinkerPass::Single {
            return Some(
                "a Sinker Side Burn sizes its electrode by its own undersize - its burn is a single pass (rough and finish are Sinker Burn passes)"
                    .into(),
            );
        }
        crate::sinker_side::side_refusal(&self.side())
    }

    /// Structural sanity (no world): the burn's, and no [`Self::refusal`].
    #[must_use]
    pub fn is_sane(&self) -> bool {
        self.burn.is_sane() && self.refusal().is_none()
    }
}
