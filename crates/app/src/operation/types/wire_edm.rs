//! The wire EDM contour's stored parameters (#532, part of #290): the
//! two planes of the workpiece, the machine condition the cut burns
//! with (by the control's own id), the stated punch / die side and the
//! start hole, and the skim passes after the rough cut (#538) - resolved into the motion crate's `wire_edm::WireParams`
//! at generation. The wire's diameter and the spark gap are NOT stored
//! here: the machine's condition table is their one source (#531).
//!
//! Provenance: our own parameter set over the umbrella's decisions
//! (#290 D2, D4, D8) and general wire EDM practice (see
//! `motion::wire_edm`); no vendor material consulted.

use super::height::Height;

pub use crate::wire_edm::WireSide;

/// The longest condition id an operation stores (a control's selector
/// is a short word; this only bounds a hand-edited file).
pub const CONDITION_ID_MAX: usize = 64;
/// The most skim passes an operation stores (#538): far past any
/// practice; it only bounds a hand-edited file.
pub const SKIMS_MAX: usize = 16;

/// One skim pass after the rough cut (#538): the machine condition it
/// burns with (its id; the gap and so the offset are that condition's)
/// and whether it walks the profile the other way round.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct WireSkim {
    /// The machine's EDM condition id; the empty word is "none chosen",
    /// refused at the command door by the skim's number.
    pub condition: String,
    /// Walk this pass clockwise (the rough pass runs counter-clockwise).
    pub reverse: bool,
}

fn sane_id(id: &str) -> bool {
    id.len() <= CONDITION_ID_MAX && !id.chars().any(char::is_control)
}

/// The wire contour's parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct WireContourOp {
    /// The workpiece's top: the plane the profile is read on (the upper
    /// programmed plane a taper will pose toward, #536).
    pub top_z: Height,
    /// The workpiece's bottom: the lower programmed plane the moves
    /// ride (#290 D2).
    pub bottom_z: Height,
    /// The machine's EDM condition this cut burns with, by its id; the
    /// empty word is "none chosen", refused at the command door.
    pub condition: String,
    /// Which side of the profile is the product - stated by the user,
    /// never inferred from the geometry (#290 D8).
    pub side: WireSide,
    /// The start hole / threading point, world XY mm; `None` is not
    /// stated, and a closed profile is refused at generation without it.
    pub start: Option<[f64; 2]>,
    /// The skim passes after the rough cut, in cut order (#538); empty
    /// is the rough pass alone.
    pub skims: Vec<WireSkim>,
}

impl WireContourOp {
    /// Structural sanity (no world): the heights, a finite start point
    /// and a condition id that is one short word. Whether the machine
    /// states that condition is the command door's question
    /// (`operation::wire_op::condition_of`).
    #[must_use]
    pub fn is_sane(&self) -> bool {
        self.top_z.is_sane()
            && self.bottom_z.is_sane()
            && self.start.is_none_or(|s| s.iter().all(|v| v.is_finite() && v.abs() < 1e9))
            && sane_id(&self.condition)
            && self.skims.len() <= SKIMS_MAX
            && self.skims.iter().all(|s| sane_id(&s.condition))
    }
}
