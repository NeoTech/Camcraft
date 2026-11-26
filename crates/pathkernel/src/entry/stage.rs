//! The entry and exit planner of the 2D kinds (K5): the order of tries of
//! the record, the named fallback, the refusals, and the exit.
//! Provenance: our own design record `docs/decisions/4024-path-kernel-design.md`
//! "Stage 6: Entry and exit"; the entry order of #3743 and #3746.

use super::room::Ctx;
use super::{Entry, EntryChoice, EntryKind, EntryPlanner, Exit, named};
use super::{air, column, exit, hole, material};
use crate::frame::Positive;
use crate::plan::Cut;
use crate::region::KernelRegion;
use crate::verdict::{Note, Refusal, Stage, Verdict};
use crate::{KERNEL_TOL, Region};

/// Refusal reason: no entry outside material fits, and the user chose no
/// entry into material.
pub const NO_SAFE_ENTRY: &str =
    "no entry in air, in a pre-drilled hole or in a cleared column fits, and no helix or ramp entry is selected";
/// Refusal reason: the selected entry into material does not fit.
pub const NO_MATERIAL_ENTRY: &str =
    "no entry in air, in a pre-drilled hole or in a cleared column fits, and no helix, ramp or plunge fits in the cutter-centre area";
/// Refusal reason: the entry geometry cannot be computed.
pub const ENTRY_GEOMETRY: &str = "the entry geometry cannot be computed: a value is out of range";
/// Refusal reason: a region that is not planar.
pub const NOT_PLANAR: &str = "the entry stage plans only a planar region";
/// Refusal reason: bad heights.
pub const BAD_HEIGHTS: &str = "the heights need a finite material top, a clearance of 0 or more, and a retract at or above the top";
/// Refusal reason: a bad arc lead choice.
pub const BAD_ARC: &str = "the arc lead needs a positive radius and a sweep above 0 and up to 180 degrees";

/// The fallback reason when the cut starts above the material top.
const ABOVE: &str = "the cut starts above the material top";

/// The heights of one level along the plane normal, in the depth units of
/// the plan (0 = the level): the material top, the clearance above it where
/// a descent starts, and the retract of the exit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Heights {
    top: f64,
    clear: f64,
    retract: f64,
}

impl Heights {
    pub fn new(top: f64, clear: f64, retract: f64) -> Result<Self, Refusal> {
        let ok = [top, clear, retract].iter().all(|v| v.is_finite()) && clear >= 0.0 && retract >= top;
        if !ok {
            return Err(Refusal::new(Stage::Recipe, BAD_HEIGHTS));
        }
        Ok(Self { top, clear, retract })
    }
    #[must_use]
    pub fn top(&self) -> f64 {
        self.top
    }
    #[must_use]
    pub fn clear(&self) -> f64 {
        self.clear
    }
    #[must_use]
    pub fn retract(&self) -> f64 {
        self.retract
    }
    /// The depth where a descent starts: the top plus the clearance.
    #[must_use]
    pub fn start(&self) -> f64 {
        self.top + self.clear
    }
}

/// The arc of a lead-in and a lead-out: its radius (mm) and its sweep
/// (radians, above 0 and up to half a turn).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ArcChoice {
    radius: Positive,
    sweep: f64,
}

impl ArcChoice {
    pub fn new(radius: f64, sweep_deg: f64) -> Result<Self, Refusal> {
        let bad = || Refusal::new(Stage::Recipe, BAD_ARC);
        let radius = Positive::new(Stage::Recipe, radius).map_err(|_| bad())?;
        if !(sweep_deg.is_finite() && sweep_deg > 0.0 && sweep_deg <= 180.0) {
            return Err(bad());
        }
        Ok(Self { radius, sweep: sweep_deg.to_radians() })
    }
    #[must_use]
    pub fn radius(&self) -> f64 {
        self.radius.get()
    }
    /// The sweep in radians.
    #[must_use]
    pub fn sweep(&self) -> f64 {
        self.sweep
    }
}

/// The entry and exit planner: the heights of the level, the arc of the
/// leads, and the chord tolerance of the arcs and the helix.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EntryStage {
    heights: Heights,
    arc: ArcChoice,
    chord_tol: Positive,
}

impl EntryStage {
    pub fn new(heights: Heights, arc: ArcChoice, chord_tol: f64) -> Result<Self, Refusal> {
        Ok(Self { heights, arc, chord_tol: Positive::new(Stage::Recipe, chord_tol)? })
    }
    #[must_use]
    pub fn heights(&self) -> Heights {
        self.heights
    }
    #[must_use]
    pub fn arc(&self) -> ArcChoice {
        self.arc
    }
    #[must_use]
    pub fn chord_tol(&self) -> f64 {
        self.chord_tol.get()
    }
}

/// The reason why the asked kind did not fit, for its fallback.
fn missed(asked: EntryKind) -> &'static str {
    match asked {
        EntryKind::Predrilled => "no pre-drilled hole serves the start of the cut",
        EntryKind::ClearedColumn => "no cleared column serves the start of the cut",
        EntryKind::Helix => "the helix does not fit in the cutter-centre area at the start of the cut",
        EntryKind::Ramp => "the ramp does not fit along the first cut",
        EntryKind::Plunge => "the tool cannot plunge",
        _ => "no entry in air with a lead fits at the start of the cut",
    }
}

/// The kinds to try, in order: the asked kind first when it is an entry
/// outside material, then the record's order (air, hole, column), then the
/// asked entry into material and the other one of helix and ramp. A plunge
/// is tried only when it is the asked kind: never as a fallback.
fn order(asked: EntryKind) -> Vec<EntryKind> {
    let asked = if asked == EntryKind::Air { EntryKind::ArcLead } else { asked };
    let mut out = Vec::with_capacity(6);
    if !asked.into_material() {
        out.push(asked);
    }
    for k in [EntryKind::ArcLead, EntryKind::Predrilled, EntryKind::ClearedColumn] {
        if !out.contains(&k) {
            out.push(k);
        }
    }
    if asked.into_material() {
        out.push(asked);
        for k in [EntryKind::Helix, EntryKind::Ramp] {
            if !out.contains(&k) {
                out.push(k);
            }
        }
    }
    out
}

fn attempt(k: EntryKind, ctx: &mut Ctx<'_>) -> Result<Option<Entry>, Refusal> {
    match k {
        EntryKind::ArcLead | EntryKind::Air => air::enter(ctx),
        EntryKind::Predrilled => hole::enter(ctx),
        EntryKind::ClearedColumn => column::enter(ctx),
        EntryKind::Helix => material::helix(ctx),
        EntryKind::Ramp => material::ramp(ctx),
        EntryKind::Plunge => material::plunge(ctx),
    }
}

impl EntryPlanner for EntryStage {
    fn entry(&self, first: &Cut, r: &KernelRegion, cleared: &Region, e: &EntryChoice) -> Result<Verdict<Entry>, Refusal> {
        let mut ctx = Ctx::new(self, first, r, cleared, e)?;
        if ctx.depth() >= self.heights.top() - KERNEL_TOL {
            let at = ctx.start.at();
            let proof = super::Proof::InAir(super::room::disc(at, super::ROOM_TOL)?);
            let entry = Entry::new(EntryKind::Air, vec![ctx.above(at)?], vec![ctx.start], proof, e, ctx.tool())?;
            return named(entry, e, ABOVE, Vec::new());
        }
        for k in order(e.asked()) {
            if let Some(entry) = attempt(k, &mut ctx)? {
                let notes: Vec<Note> = std::mem::take(&mut ctx.notes);
                return named(entry, e, missed(e.asked()), notes);
            }
            ctx.notes.clear();
        }
        let at = ctx.start.at();
        Err(if e.asked().into_material() {
            Refusal::new(Stage::Entry, NO_MATERIAL_ENTRY).at(at).fix("use a smaller helix radius or a larger ramp angle, or pre-drill a hole")
        } else {
            Refusal::new(Stage::Entry, NO_SAFE_ENTRY).at(at).fix("pre-drill a hole at the start of the cut, or select a helix or ramp entry")
        })
    }

    fn exit(&self, last: &Cut, r: &KernelRegion, cleared: &Region) -> Result<Exit, Refusal> {
        exit::plan(self, last, r, cleared)
    }
}
