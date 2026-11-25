//! Stage 6, Entry and exit (K5): each entry carries a proof, and the
//! constructor checks law 5 - an entry into material needs a proof that
//! names it, and a plunge needs the user's choice and a tool that can
//! plunge. K5 adds `entry()` and `exit()`.
//! Provenance: our own design record `docs/decisions/4024-path-kernel-design.md`
//! "Stage 6: Entry and exit"; helical, ramp and arc lead-in entries as in
//! any machining handbook (Machinery's Handbook, milling entry practice).

use crate::frame::{PlanePoint, ToolShape};
use crate::plan::Cut;
use crate::region::KernelRegion;
use crate::verdict::{Refusal, Stage};
use crate::{KERNEL_TOL, Point2, Region, finite};

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

/// The entry that the user asked for, the approach distance in air, and the
/// pre-drilled holes.
#[derive(Clone, Debug, PartialEq)]
pub struct EntryChoice {
    asked: EntryKind,
    approach: f64,
    holes: Vec<Point2>,
}

/// Refusal reason: an approach distance below zero, or a point that is not finite.
pub const BAD_CHOICE: &str = "the entry needs a finite approach of 0 or more and finite hole places";

impl EntryChoice {
    pub fn new(asked: EntryKind, approach: f64, holes: Vec<Point2>) -> Result<Self, Refusal> {
        if !approach.is_finite() || approach < 0.0 || !holes.iter().all(|h| finite(&[h.x, h.y])) {
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
    pub fn holes(&self) -> &[Point2] {
        &self.holes
    }
}

/// Why an entry is safe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Proof {
    InAir,
    InHole(Point2),
    InColumn,
    /// The entry goes into material at this ramp angle (degrees) or helix pitch (mm).
    IntoMaterial { angle_or_pitch: f64 },
}

/// One entry: its kind, its moves and its proof.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    kind: EntryKind,
    moves: Vec<PlanePoint>,
    proof: Proof,
}

/// Refusal reason: the proof does not fit the entry kind.
pub const WRONG_PROOF: &str = "the proof does not fit the entry kind";
/// Refusal reason: a pre-drilled entry at no known hole.
pub const NO_HOLE: &str = "a pre-drilled entry must be at one of the pre-drilled holes";
/// Refusal reason: a plunge that the user did not ask for, or a tool that cannot plunge.
pub const PLUNGE_NOT_ALLOWED: &str = "a plunge into material needs the user's choice and a tool that can plunge";
/// Refusal reason: an entry with no move or a move that is not finite.
pub const BAD_MOVES: &str = "an entry needs one or more finite moves";

impl Entry {
    pub fn new(kind: EntryKind, moves: Vec<PlanePoint>, proof: Proof, choice: &EntryChoice, tool: ToolShape) -> Result<Self, Refusal> {
        let refuse = |r| Err(Refusal::new(Stage::Entry, r));
        if moves.is_empty() || !moves.iter().all(PlanePoint::is_finite) {
            return refuse(BAD_MOVES);
        }
        let fits = match (kind, proof) {
            (EntryKind::Air | EntryKind::ArcLead, Proof::InAir) | (EntryKind::ClearedColumn, Proof::InColumn) => true,
            (EntryKind::Predrilled, Proof::InHole(p)) => {
                let known = choice.holes.iter().any(|h| (h.x - p.x).abs() <= KERNEL_TOL && (h.y - p.y).abs() <= KERNEL_TOL);
                if !known {
                    return refuse(NO_HOLE);
                }
                true
            }
            (EntryKind::Helix | EntryKind::Ramp | EntryKind::Plunge, Proof::IntoMaterial { angle_or_pitch }) => {
                angle_or_pitch.is_finite() && angle_or_pitch > 0.0
            }
            _ => false,
        };
        if !fits {
            return refuse(WRONG_PROOF);
        }
        if kind == EntryKind::Plunge && (choice.asked != EntryKind::Plunge || !tool.can_plunge()) {
            return refuse(PLUNGE_NOT_ALLOWED);
        }
        Ok(Self { kind, moves, proof })
    }
    #[must_use]
    pub fn kind(&self) -> EntryKind {
        self.kind
    }
    #[must_use]
    pub fn moves(&self) -> &[PlanePoint] {
        &self.moves
    }
    #[must_use]
    pub fn proof(&self) -> Proof {
        self.proof
    }
}

/// The exit of a cut: an optional lead-out, then the retract height above
/// the plane.
#[derive(Clone, Debug, PartialEq)]
pub struct Exit {
    lead_out: Option<Vec<PlanePoint>>,
    retract: f64,
}

/// Refusal reason: an exit with a negative retract, or a lead-out point that is not finite.
pub const BAD_EXIT: &str = "the exit needs a finite retract of 0 or more and finite lead-out points";

impl Exit {
    pub fn new(lead_out: Option<Vec<PlanePoint>>, retract: f64) -> Result<Self, Refusal> {
        let lead_ok = lead_out.as_ref().is_none_or(|l| !l.is_empty() && l.iter().all(PlanePoint::is_finite));
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

#[cfg(test)]
mod tests {
    use super::*;

    fn at() -> Vec<PlanePoint> {
        vec![PlanePoint::flat(Point2::new(0.0, 0.0))]
    }

    fn tool(can_plunge: bool) -> ToolShape {
        ToolShape::disc(3.0, 0.0, can_plunge, 15.0).unwrap()
    }

    #[test]
    fn each_entry_kind_needs_its_own_proof() {
        let c = EntryChoice::new(EntryKind::Helix, 2.0, vec![]).unwrap();
        let t = tool(true);
        assert!(Entry::new(EntryKind::Air, at(), Proof::InAir, &c, t).is_ok());
        assert!(Entry::new(EntryKind::ArcLead, at(), Proof::InAir, &c, t).is_ok());
        assert!(Entry::new(EntryKind::ClearedColumn, at(), Proof::InColumn, &c, t).is_ok());
        assert!(Entry::new(EntryKind::Helix, at(), Proof::IntoMaterial { angle_or_pitch: 1.0 }, &c, t).is_ok());
        assert_eq!(Entry::new(EntryKind::Helix, at(), Proof::InAir, &c, t).unwrap_err().reason, WRONG_PROOF);
        assert_eq!(Entry::new(EntryKind::Air, at(), Proof::InColumn, &c, t).unwrap_err().reason, WRONG_PROOF);
        assert!(Entry::new(EntryKind::Ramp, at(), Proof::IntoMaterial { angle_or_pitch: 0.0 }, &c, t).is_err());
        assert_eq!(Entry::new(EntryKind::Air, vec![], Proof::InAir, &c, t).unwrap_err().reason, BAD_MOVES);
    }

    #[test]
    fn a_predrilled_entry_is_at_a_known_hole() {
        let h = Point2::new(5.0, 5.0);
        let c = EntryChoice::new(EntryKind::Helix, 2.0, vec![h]).unwrap();
        assert!(Entry::new(EntryKind::Predrilled, at(), Proof::InHole(h), &c, tool(false)).is_ok());
        let e = Entry::new(EntryKind::Predrilled, at(), Proof::InHole(Point2::new(6.0, 5.0)), &c, tool(false)).unwrap_err();
        assert_eq!(e.reason, NO_HOLE);
    }

    #[test]
    fn a_plunge_needs_the_users_choice_and_a_plunging_tool() {
        let proof = Proof::IntoMaterial { angle_or_pitch: 90.0 };
        let asked = EntryChoice::new(EntryKind::Plunge, 0.0, vec![]).unwrap();
        let other = EntryChoice::new(EntryKind::Ramp, 0.0, vec![]).unwrap();
        assert!(Entry::new(EntryKind::Plunge, at(), proof, &asked, tool(true)).is_ok());
        assert_eq!(Entry::new(EntryKind::Plunge, at(), proof, &asked, tool(false)).unwrap_err().reason, PLUNGE_NOT_ALLOWED);
        assert_eq!(Entry::new(EntryKind::Plunge, at(), proof, &other, tool(true)).unwrap_err().reason, PLUNGE_NOT_ALLOWED);
    }

    #[test]
    fn choices_and_exits_are_checked() {
        assert_eq!(EntryChoice::new(EntryKind::Air, -1.0, vec![]).unwrap_err().reason, BAD_CHOICE);
        assert!(EntryChoice::new(EntryKind::Air, 1.0, vec![Point2::new(f64::NAN, 0.0)]).is_err());
        assert!(Exit::new(None, 5.0).is_ok());
        assert_eq!(Exit::new(None, -1.0).unwrap_err().reason, BAD_EXIT);
        assert!(Exit::new(Some(vec![]), 1.0).is_err());
        assert_eq!(Exit::new(Some(at()), 1.0).unwrap().lead_out().map(<[_]>::len), Some(1));
    }
}
