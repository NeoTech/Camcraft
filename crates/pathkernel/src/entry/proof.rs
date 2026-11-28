//! The proof of an entry and the checked entry (law 5): each entry has a
//! proof that fits its kind, its descent stays in the place that the proof
//! names, its lead in the plane stays in the room that the lead names, and
//! a plunge needs the user's choice and a tool that can plunge.
//! [`Entry::led`] is the single gate: every entry of the crate is built
//! through it.
//! Provenance: our own design record (`RECOMMENDED.md`, `## #4024`)
//! "Stage 6: Entry and exit"; #4055 review finding 4.

use super::{EntryChoice, EntryKind};
use crate::frame::{PlanePoint, Positive, Spot, ToolShape};
use crate::verdict::{Refusal, Stage};
use crate::{KERNEL_TOL, Point2, Region, in_area, region_finite};

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

/// A pre-drilled hole: a finite centre and a positive radius. The caller
/// gives only holes that reach the depth of the cut.
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

/// A lead in the plane at the depth of the cut: two or more points at one
/// depth, and no stretch of it outside the room that it names (the tool
/// centre's legal room for the lead-in, or the room where the tool cuts
/// nothing for a lead-out). A stretch on the boundary of the room is
/// decided by the half-open rule of `Region::contains`: prove a lead
/// against a room grown by the tolerance ([`super::ROOM_TOL`]).
#[derive(Clone, Debug, PartialEq)]
pub struct Lead {
    pts: Vec<PlanePoint>,
}

/// Refusal reason: a lead with fewer than two points.
pub const LEAD_SHORT: &str = "a lead needs two or more points";
/// Refusal reason: a lead whose points are not at one depth.
pub const LEAD_NOT_FLAT: &str = "a lead stays in the plane: all its points are at one depth";
/// Refusal reason: a lead with a stretch outside its room.
pub const LEAD_OFF_ROOM: &str = "each lead move must stay in the room that the lead names";

impl Lead {
    pub fn new(pts: Vec<PlanePoint>, room: &Area) -> Result<Self, Refusal> {
        let refuse = |r| Refusal::new(Stage::Entry, r);
        let Some(first) = pts.first().copied() else {
            return Err(refuse(LEAD_SHORT));
        };
        if pts.len() < 2 {
            return Err(refuse(LEAD_SHORT));
        }
        if pts.iter().any(|p| (p.depth() - first.depth()).abs() > KERNEL_TOL) {
            return Err(refuse(LEAD_NOT_FLAT).at(first.at()));
        }
        let xy: Vec<Point2> = pts.iter().map(PlanePoint::at).collect();
        match room.region().holds_path(&xy) {
            Ok(true) => Ok(Self { pts }),
            _ => Err(refuse(LEAD_OFF_ROOM).at(first.at())),
        }
    }
    #[must_use]
    pub fn pts(&self) -> &[PlanePoint] {
        &self.pts
    }
}

/// One entry: its kind, the approach moves above the material, the descent
/// moves to the cut depth, an optional lead in the plane onto the cut, and
/// the proof of the descent.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    kind: EntryKind,
    approach: Vec<PlanePoint>,
    descent: Vec<PlanePoint>,
    lead: Option<Lead>,
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
/// Refusal reason: a lead that does not start where the descent ends.
pub const LEAD_GAP: &str = "the lead must start where the descent ends";

impl Entry {
    /// The checked entry with no lead: [`Entry::led`] with `None`.
    pub fn new(
        kind: EntryKind,
        approach: Vec<PlanePoint>,
        descent: Vec<PlanePoint>,
        proof: Proof,
        choice: &EntryChoice,
        tool: ToolShape,
    ) -> Result<Self, Refusal> {
        Self::led(kind, approach, descent, None, proof, choice, tool)
    }

    /// The checked entry. The approach moves are above the material and are
    /// not bound; each descent move is bound to the place of the proof:
    /// inside the proved area within [`KERNEL_TOL`], or, for a hole, no
    /// farther from its centre than the hole radius less the tool radius (0
    /// at least). A lead (checked by [`Lead::new`]) starts where the descent
    /// ends, in place and depth.
    pub fn led(
        kind: EntryKind,
        approach: Vec<PlanePoint>,
        descent: Vec<PlanePoint>,
        lead: Option<Lead>,
        proof: Proof,
        choice: &EntryChoice,
        tool: ToolShape,
    ) -> Result<Self, Refusal> {
        let refuse = |r| Err(Refusal::new(Stage::Entry, r));
        let Some(end) = descent.last().copied() else {
            return refuse(NO_DESCENT);
        };
        let place = match (kind, &proof) {
            (EntryKind::Air | EntryKind::ArcLead, Proof::InAir(a))
            | (EntryKind::ClearedColumn, Proof::InColumn(a))
            | (EntryKind::Helix | EntryKind::Ramp | EntryKind::Plunge, Proof::IntoMaterial { area: a, .. }) => Place::Area(a.region()),
            (EntryKind::Predrilled, Proof::InHole(h)) => {
                if !choice.holes().iter().any(|c| c.near(h)) {
                    return refuse(NO_HOLE);
                }
                Place::Disk(h.center(), (h.radius() - tool.radius()).max(0.0) + KERNEL_TOL)
            }
            _ => return refuse(WRONG_PROOF),
        };
        if let Some(p) = descent.iter().find(|p| !place.holds(p.at())) {
            return Err(Refusal::new(Stage::Entry, OFF_PROOF).at(p.at()));
        }
        if kind == EntryKind::Plunge && (choice.asked() != EntryKind::Plunge || !tool.can_plunge()) {
            return refuse(PLUNGE_NOT_ALLOWED);
        }
        if let Some(l) = &lead
            && !l.pts.first().is_some_and(|p| p.near(&end))
        {
            return Err(Refusal::new(Stage::Entry, LEAD_GAP).at(end.at()));
        }
        Ok(Self { kind, approach, descent, lead, proof })
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
    pub fn lead(&self) -> Option<&Lead> {
        self.lead.as_ref()
    }
    #[must_use]
    pub fn proof(&self) -> &Proof {
        &self.proof
    }
}
