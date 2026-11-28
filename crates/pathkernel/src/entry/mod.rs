//! Stage 6, Entry and exit (K5, #4178): each entry carries a proof, and its
//! checked constructor binds the descent to the proved place (law 5). The
//! stage tries the entries in the record's order - in air outside the stock
//! with an arc lead (#3746), in a pre-drilled hole, in a cleared column, and
//! into material by a helix, a ramp or a plunge only when the user chose an
//! entry into material - and refuses by name when none fits. An entry of
//! another kind than the asked one carries a named [`Fallback`] (law 4).
//! The exit is a lead-out in the plane where there is room, then the retract.
//!
//! Provenance: our own design record (`RECOMMENDED.md`, `## #4024`)
//! "Stage 6: Entry and exit"; helical, ramp and tangential arc lead-in and
//! lead-out entries as in public machining handbooks (Machinery's Handbook,
//! milling entry practice; RS-274 / ISO 6983 G17 arcs). No vendor material
//! consulted.

mod air;
mod column;
mod exit;
mod hole;
mod material;
mod proof;
mod room;
mod stage;

pub use exit::{
    BAD_EXIT, EXIT_OFF_CUT, Exit, LEAD_OUT_NAME, NO_END_DIRECTION, NO_LEAD_OUT_ROOM, RAPID_IN_MATERIAL, STRAIGHT_UP_NAME, UNPROVED_LEAD_OUT,
};
pub use proof::{
    Area, BAD_AREA, Entry, Hole, LEAD_GAP, LEAD_NOT_FLAT, LEAD_OFF_ROOM, LEAD_SHORT, Lead, NO_DESCENT, NO_HOLE, OFF_PROOF,
    PLUNGE_NOT_ALLOWED, Proof, WRONG_PROOF,
};
pub use room::{NO_DIRECTION, ROOM_TOL};
pub(crate) use room::legal_room;
pub use stage::{
    ABOVE, ArcChoice, BAD_ARC, BAD_HEIGHTS, ENTRY_GEOMETRY, ENTRY_GEOMETRY_INVALID, EntryStage, Heights, NO_MATERIAL_ENTRY, NO_SAFE_ENTRY, NOT_PLANAR,
};

use crate::frame::Positive;
use crate::plan::Cut;
use crate::region::KernelRegion;
use crate::verdict::{Fallback, Refusal, Stage, Verdict};
use crate::Region;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EntryKind {
    Air,
    Predrilled,
    ClearedColumn,
    Helix,
    Ramp,
    ArcLead,
    Plunge,
}

impl EntryKind {
    /// The name of the kind on the status line.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Air => "air entry",
            Self::Predrilled => "pre-drilled hole",
            Self::ClearedColumn => "cleared column",
            Self::Helix => "helix",
            Self::Ramp => "ramp",
            Self::ArcLead => "arc lead from the air",
            Self::Plunge => "plunge",
        }
    }

    /// True for the kinds that go down into material: a helix, a ramp or a
    /// plunge. The other kinds go down where no material stands.
    #[must_use]
    pub fn into_material(self) -> bool {
        matches!(self, Self::Helix | Self::Ramp | Self::Plunge)
    }

    /// The entries in air are one family: an arc lead is an air entry with
    /// a lead in the plane.
    fn family(self) -> Self {
        if self == Self::ArcLead { Self::Air } else { self }
    }
}

/// True when an entry of kind `used`, where the user asked for `asked`,
/// must carry a named fallback (law 4). An entry in air, a pre-drilled hole
/// or a cleared column where an entry into material was asked is not a
/// fallback: the asked kind applies only where the tool must enter material
/// (the record's order of tries). An air entry and an arc lead are one
/// family. Each other difference is a fallback.
#[must_use]
pub fn needs_fallback(asked: EntryKind, used: EntryKind) -> bool {
    asked.family() != used.family() && !(asked.into_material() && !used.into_material())
}

/// `entry` as the verdict of the stage: done, with a named fallback when
/// [`needs_fallback`] says so (`why` is the reason the asked kind did not
/// fit), and with `notes` for the status line.
pub fn named(entry: Entry, choice: &EntryChoice, why: &'static str, notes: Vec<crate::Note>) -> Result<Verdict<Entry>, Refusal> {
    let (asked, used) = (choice.asked(), entry.kind());
    let fallbacks = if needs_fallback(asked, used) { vec![Fallback::new(Stage::Entry, asked.name(), used.name(), why)?] } else { vec![] };
    Ok(Verdict::Done { out: entry, fallbacks, notes })
}

/// The user's limits for an entry into material: the descent angle of a
/// helix or a ramp (degrees above the plane, 0 to 90 exclusive) and the
/// helix radius (mm; none = half the tool radius).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaterialChoice {
    angle: f64,
    helix_radius: Option<Positive>,
}

/// Refusal reason: a descent angle out of range, or a helix radius that is
/// not a positive finite number.
pub const BAD_MATERIAL_CHOICE: &str = "an entry into material needs a descent angle above 0 and below 90 degrees, and a positive helix radius if any";

/// The descent angle of a helix or a ramp when the user gave none (degrees):
/// the default ramp angle of the old planners (`links.rs`, 3 degrees).
pub const DEFAULT_DESCENT_ANGLE: f64 = 3.0;

impl MaterialChoice {
    pub fn new(angle_deg: f64, helix_radius: Option<f64>) -> Result<Self, Refusal> {
        let bad = || Refusal::new(Stage::Recipe, BAD_MATERIAL_CHOICE);
        if !(angle_deg.is_finite() && angle_deg > 0.0 && angle_deg < 90.0) {
            return Err(bad());
        }
        let helix_radius = helix_radius.map(|r| Positive::new(Stage::Recipe, r).map_err(|_| bad())).transpose()?;
        Ok(Self { angle: angle_deg, helix_radius })
    }
    /// The descent angle in degrees, above 0 and below 90.
    #[must_use]
    pub fn angle(&self) -> f64 {
        self.angle
    }
    #[must_use]
    pub fn helix_radius(&self) -> Option<f64> {
        self.helix_radius.map(Positive::get)
    }
}

impl Default for MaterialChoice {
    fn default() -> Self {
        Self { angle: DEFAULT_DESCENT_ANGLE, helix_radius: None }
    }
}

/// The entry that the user asked for, the approach distance in air, the
/// pre-drilled holes, and the limits of an entry into material.
#[derive(Clone, Debug, PartialEq)]
pub struct EntryChoice {
    asked: EntryKind,
    approach: f64,
    holes: Vec<Hole>,
    material: MaterialChoice,
}

/// Refusal reason: an approach distance below zero or not finite.
pub const BAD_CHOICE: &str = "the entry needs a finite approach distance of 0 or more";

impl EntryChoice {
    pub fn new(asked: EntryKind, approach: f64, holes: Vec<Hole>) -> Result<Self, Refusal> {
        if !approach.is_finite() || approach < 0.0 {
            return Err(Refusal::new(Stage::Recipe, BAD_CHOICE));
        }
        Ok(Self { asked, approach, holes, material: MaterialChoice::default() })
    }
    /// The same choice with the user's limits for an entry into material.
    #[must_use]
    pub fn with_material(mut self, m: MaterialChoice) -> Self {
        self.material = m;
        self
    }
    #[must_use]
    pub fn asked(&self) -> EntryKind {
        self.asked
    }
    #[must_use]
    pub fn approach(&self) -> f64 {
        self.approach
    }
    #[must_use]
    pub fn holes(&self) -> &[Hole] {
        &self.holes
    }
    #[must_use]
    pub fn material(&self) -> MaterialChoice {
        self.material
    }
}

/// An entry and exit planner. `cleared` is the area of the plane where no
/// stock stands down to the depth of the cut (the caller reads it from its
/// stock reader); the entry uses it for a cleared column, the exit for the
/// room of its lead-out.
///
/// `entry` returns a [`Verdict`] so that an entry of another kind than the
/// asked one names its [`Fallback`] (law 4, #4178): [`named`] builds it. A
/// planner that finds no safe entry returns `Err` with a named reason; it
/// never returns [`Verdict::Refused`] or [`Verdict::Incomplete`].
pub trait EntryPlanner {
    fn entry(&self, first: &Cut, r: &KernelRegion, cleared: &Region, e: &EntryChoice) -> Result<Verdict<Entry>, Refusal>;
    fn exit(&self, last: &Cut, r: &KernelRegion, cleared: &Region) -> Result<Exit, Refusal>;
}
