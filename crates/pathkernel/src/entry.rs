//! Stage 6, Entry and exit (K5): each entry carries a proof, and the
//! constructor checks law 5 - each entry has a proof that fits its kind,
//! its descent stays in the place that the proof names, and a plunge needs
//! the user's choice and a tool that can plunge. K5 adds `entry()` and `exit()`.
//! Provenance: our own design record `docs/decisions/4024-path-kernel-design.md`
//! "Stage 6: Entry and exit"; helical, ramp and arc lead-in entries as in
//! any machining handbook (Machinery's Handbook, milling entry practice).

use crate::frame::{PlanePoint, Positive, Spot, ToolShape};
use crate::plan::Cut;
use crate::region::KernelRegion;
use crate::verdict::{Refusal, Stage};
use crate::{KERNEL_TOL, Point2, Region, in_area, region_finite};

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

/// A proved area: finite, not empty, with a positive area.
#[derive(Clone, Debug, PartialEq)]
pub struct Area(Region);

/// Refusal reason: an area that is empty or not finite.
pub const BAD_AREA: &str = "a proved area must be finite and not empty";

impl Area {
    pub fn new(r: Region) -> Result<Self, Refusal> {
        let a = r.area();
        if r.is_empty() || !region_finite(&r) || !a.is_finite() || a <= 0.0 {
            return Err(Refusal::new(Stage::Entry, BAD_AREA));
        }
        Ok(Self(r))
    }
    #[must_use]
    pub fn region(&self) -> &Region {
        &self.0
    }
}

/// A pre-drilled hole: a finite centre and a positive radius.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hole {
    center: Spot,
    radius: Positive,
}

impl Hole {
    pub fn new(center: Point2, radius: f64) -> Result<Self, Refusal> {
        Ok(Self { center: Spot::new(Stage::Entry, center)?, radius: Positive::new(Stage::Entry, radius)? })
    }
    #[must_use]
    pub fn center(&self) -> Point2 {
        self.center.get()
    }
    #[must_use]
    pub fn radius(&self) -> f64 {
        self.radius.get()
    }
    fn near(&self, o: &Self) -> bool {
        crate::near(self.center(), o.center()) && (self.radius() - o.radius()).abs() <= KERNEL_TOL
    }
}

/// The entry that the user asked for, the approach distance in air, and the
/// pre-drilled holes.
#[derive(Clone, Debug, PartialEq)]
pub struct EntryChoice {
    asked: EntryKind,
    approach: f64,
    holes: Vec<Hole>,
}

/// Refusal reason: an approach distance below zero or not finite.
pub const BAD_CHOICE: &str = "the entry needs a finite approach distance of 0 or more";

impl EntryChoice {
    pub fn new(asked: EntryKind, approach: f64, holes: Vec<Hole>) -> Result<Self, Refusal> {
        if !approach.is_finite() || approach < 0.0 {
            return Err(Refusal::new(Stage::Recipe, BAD_CHOICE));
        }
        Ok(Self { asked, approach, holes })
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
}

/// Why an entry is safe. Each proof names the place it proves, and the
/// descent of the entry must stay in that place.
#[derive(Clone, Debug, PartialEq)]
pub enum Proof {
    /// The descent is in air (outside the stock) in this area.
    InAir(Area),
    /// The descent is in this pre-drilled hole.
    InHole(Hole),
    /// The descent is in this column of cleared material.
    InColumn(Area),
    /// The descent goes into material in this area, at this ramp angle
    /// (degrees) or helix pitch (mm).
    IntoMaterial { area: Area, angle_or_pitch: Positive },
}

/// The place that a proof binds the descent to.
enum Place<'a> {
    Area(&'a Region),
    Disk(Point2, f64),
}

impl Place<'_> {
    fn holds(&self, p: Point2) -> bool {
        match self {
            Self::Area(r) => in_area(r, p),
            Self::Disk(c, reach) => (p.x - c.x).hypot(p.y - c.y) <= *reach,
        }
    }
}

/// One entry: its kind, the approach moves above the material, the descent
/// moves to the cut depth, and the proof of the descent.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    kind: EntryKind,
    approach: Vec<PlanePoint>,
    descent: Vec<PlanePoint>,
    proof: Proof,
}

/// Refusal reason: the proof does not fit the entry kind.
pub const WRONG_PROOF: &str = "the proof does not fit the entry kind";
/// Refusal reason: a pre-drilled entry at no known hole.
pub const NO_HOLE: &str = "a pre-drilled entry must be at one of the pre-drilled holes";
/// Refusal reason: a descent move outside the place that the proof names.
pub const OFF_PROOF: &str = "each descent move must stay in the place that the proof names";
/// Refusal reason: a plunge that the user did not ask for, or a tool that cannot plunge.
pub const PLUNGE_NOT_ALLOWED: &str = "a plunge into material needs the user's choice and a tool that can plunge";
/// Refusal reason: an entry with no descent move.
pub const NO_DESCENT: &str = "an entry needs one or more descent moves";

impl Entry {
    /// The checked entry. The approach moves are above the material and are
    /// not bound; each descent move is bound to the place of the proof: inside
    /// the proved area within [`KERNEL_TOL`], or, for a hole, no farther from
    /// its centre than the hole radius less the tool radius (0 at least).
    pub fn new(
        kind: EntryKind,
        approach: Vec<PlanePoint>,
        descent: Vec<PlanePoint>,
        proof: Proof,
        choice: &EntryChoice,
        tool: ToolShape,
    ) -> Result<Self, Refusal> {
        let refuse = |r| Err(Refusal::new(Stage::Entry, r));
        if descent.is_empty() {
            return refuse(NO_DESCENT);
        }
        let place = match (kind, &proof) {
            (EntryKind::Air | EntryKind::ArcLead, Proof::InAir(a))
            | (EntryKind::ClearedColumn, Proof::InColumn(a))
            | (EntryKind::Helix | EntryKind::Ramp | EntryKind::Plunge, Proof::IntoMaterial { area: a, .. }) => Place::Area(a.region()),
            (EntryKind::Predrilled, Proof::InHole(h)) => {
                if !choice.holes.iter().any(|c| c.near(h)) {
                    return refuse(NO_HOLE);
                }
                Place::Disk(h.center(), (h.radius() - tool.radius()).max(0.0) + KERNEL_TOL)
            }
            _ => return refuse(WRONG_PROOF),
        };
        if let Some(p) = descent.iter().find(|p| !place.holds(p.at())) {
            return Err(Refusal::new(Stage::Entry, OFF_PROOF).at(p.at()));
        }
        if kind == EntryKind::Plunge && (choice.asked != EntryKind::Plunge || !tool.can_plunge()) {
            return refuse(PLUNGE_NOT_ALLOWED);
        }
        Ok(Self { kind, approach, descent, proof })
    }
    #[must_use]
    pub fn kind(&self) -> EntryKind {
        self.kind
    }
    #[must_use]
    pub fn approach(&self) -> &[PlanePoint] {
        &self.approach
    }
    #[must_use]
    pub fn descent(&self) -> &[PlanePoint] {
        &self.descent
    }
    #[must_use]
    pub fn proof(&self) -> &Proof {
        &self.proof
    }
}

/// The exit of a cut: an optional lead-out, then the retract height above
/// the plane.
#[derive(Clone, Debug, PartialEq)]
pub struct Exit {
    lead_out: Option<Vec<PlanePoint>>,
    retract: f64,
}

/// Refusal reason: an exit with a negative retract, or an empty lead-out.
pub const BAD_EXIT: &str = "the exit needs a finite retract of 0 or more and a lead-out of one or more points if any";

impl Exit {
    pub fn new(lead_out: Option<Vec<PlanePoint>>, retract: f64) -> Result<Self, Refusal> {
        let lead_ok = lead_out.as_ref().is_none_or(|l| !l.is_empty());
        if !retract.is_finite() || retract < 0.0 || !lead_ok {
            return Err(Refusal::new(Stage::Exit, BAD_EXIT));
        }
        Ok(Self { lead_out, retract })
    }
    #[must_use]
    pub fn lead_out(&self) -> Option<&[PlanePoint]> {
        self.lead_out.as_deref()
    }
    #[must_use]
    pub fn retract(&self) -> f64 {
        self.retract
    }
}

/// An entry and exit planner.
pub trait EntryPlanner {
    fn entry(&self, first: &Cut, r: &KernelRegion, cleared: &Region, e: &EntryChoice) -> Result<Entry, Refusal>;
    fn exit(&self, last: &Cut, r: &KernelRegion, cleared: &Region) -> Result<Exit, Refusal>;
}
