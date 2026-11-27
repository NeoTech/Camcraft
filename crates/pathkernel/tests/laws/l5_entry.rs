//! L5 (#4027): an entry into material needs a proof; else the kernel
//! refuses with a reason. The kernel's `Entry` already binds each descent
//! to the place that its proof names. This check reads each descent on the
//! check's own stock grid, as the stock is at that step of the plan: a
//! descent onto material with a proof other than `IntoMaterial` (air, a
//! hole, a cleared column) is a breach.
//!
//! #4406: an `IntoMaterial` proof is read too, not passed at 0 by
//! construction. A descent onto material under it must move in the plane
//! (a helix or a ramp: the descent travels one grid cell or more), or be a
//! plunge that the user asked for. A descent onto material that does not
//! move, under another kind than an asked plunge, is a plunge with the
//! name of a ramp: a breach, measured by its fill. A run that a stage
//! refused is refused (not measured), never held.

use camcraft_pathkernel::entry::{Area, Entry, EntryChoice, EntryKind, EntryPlanner, Exit, Proof};
use camcraft_pathkernel::plan::Cut;
use camcraft_pathkernel::region::KernelRegion;
use camcraft_pathkernel::{PlanePoint, Positive, Refusal, Region, Stage, Verdict};

use super::grid::{CELL, walk};
use super::kit::{Case, Law, LawVerdict, Measure, Planners, Run, assert_broken, assert_held, assert_refused, judge, planar, refused, run};
use super::standins::{ProvedEntry, good, square_area, square_case, start_of};

/// A descent whose tool disc holds more material than this share is a
/// descent into material.
pub const ENTRY_FILL: f64 = 0.02;

/// Refusal reason of the check: the stock of the case is empty.
pub const NO_STOCK: &str = "the case has no stock to read";

/// L5 over one run of the case `c`, or over its refusal.
pub fn check_l5(run: &Result<Run, Refusal>, c: &Case) -> LawVerdict {
    let r = match run {
        Ok(r) => r,
        Err(e) => return refused(Law::L5, e),
    };
    let Ok(tool_r) = planar(&r.region).map(|p| p.tool().radius()) else {
        return judge(Law::L5, vec![Measure::new(f64::INFINITY, 0.0, "the region is not planar", None)]);
    };
    let Some(w) = walk(&r.plan, &c.stock, tool_r) else {
        return judge(Law::L5, vec![Measure::new(f64::INFINITY, 0.0, NO_STOCK, None)]);
    };
    let (cell, asked) = (tool_r * CELL, c.recipe.entry().asked());
    let m: Vec<Measure> = w
        .entries
        .iter()
        .map(|e| {
            let place = format!("entry {}: material share {:.3} under the tool, proof into material: {}", e.index, e.fill, e.into_material);
            if !e.into_material || e.fill <= ENTRY_FILL {
                return Measure::new(e.fill, ENTRY_FILL, place, Some(e.at));
            }
            let plunge = e.kind == EntryKind::Plunge && asked == EntryKind::Plunge;
            let moves = e.kind != EntryKind::Plunge && e.travel >= cell;
            let v = if plunge || moves { 0.0 } else { e.fill };
            let place = format!("{place}, {:?} descent travels {:.3} mm (one cell is {cell:.3})", e.kind, e.travel);
            Measure::new(v, ENTRY_FILL, place, Some(e.at))
        })
        .collect();
    if m.is_empty() {
        return judge(Law::L5, vec![Measure::unread("the plan", "the plan has no entry")]);
    }
    judge(Law::L5, m)
}

/// Breaks L5: claims air (or a cleared column) at each entry, wherever it is.
struct ClaimsFree {
    column: bool,
}

impl EntryPlanner for ClaimsFree {
    fn entry(&self, first: &Cut, r: &KernelRegion, _cleared: &Region, e: &EntryChoice) -> Result<Verdict<Entry>, Refusal> {
        let at = start_of(first);
        let tool = planar(r)?.tool();
        let area = Area::new(square_area(at, 2.0)?)?;
        let (kind, proof) =
            if self.column { (EntryKind::ClearedColumn, Proof::InColumn(area)) } else { (EntryKind::Air, Proof::InAir(area)) };
        Entry::new(kind, vec![], vec![PlanePoint::flat(at)?], proof, e, tool).map(Verdict::done)
    }
    fn exit(&self, last: &Cut, r: &KernelRegion, cleared: &Region) -> Result<Exit, Refusal> {
        ProvedEntry.exit(last, r, cleared)
    }
}

/// Refuses each entry with a reason (no legal entry).
struct NoEntry;

const NO_LEGAL_ENTRY: &str = "no legal entry: the helix does not fit and no hole is pre-drilled";

impl EntryPlanner for NoEntry {
    fn entry(&self, _f: &Cut, _r: &KernelRegion, _c: &Region, _e: &EntryChoice) -> Result<Verdict<Entry>, Refusal> {
        Err(Refusal::new(Stage::Entry, NO_LEGAL_ENTRY))
    }
    fn exit(&self, last: &Cut, r: &KernelRegion, cleared: &Region) -> Result<Exit, Refusal> {
        ProvedEntry.exit(last, r, cleared)
    }
}

/// Breaks L5 (#4406): an `IntoMaterial` proof named a helix, but the
/// descent does not move in the plane: a plunge into material that the user
/// did not ask for.
struct HiddenPlunge;

impl EntryPlanner for HiddenPlunge {
    fn entry(&self, first: &Cut, r: &KernelRegion, _cleared: &Region, e: &EntryChoice) -> Result<Verdict<Entry>, Refusal> {
        let at = start_of(first);
        let tool = planar(r)?.tool();
        let area = Area::new(square_area(at, 2.0)?)?;
        let proof = Proof::IntoMaterial { area, angle_or_pitch: Positive::new(Stage::Entry, 1.0)? };
        Entry::new(EntryKind::Helix, vec![PlanePoint::new(at, 5.0)?], vec![PlanePoint::flat(at)?], proof, e, tool).map(Verdict::done)
    }
    fn exit(&self, last: &Cut, r: &KernelRegion, cleared: &Region) -> Result<Exit, Refusal> {
        ProvedEntry.exit(last, r, cleared)
    }
}

fn l5(p: &Planners) -> LawVerdict {
    let c = square_case().unwrap();
    check_l5(&run(p, &c, Default::default()), &c)
}

#[test]
fn l5_holds_for_a_proved_helix() {
    let v = l5(&good());
    assert_held(&v);
    assert!(v.worst.place.contains("Helix descent travels"), "the helix into material is read: {v}");
}

#[test]
fn l5_is_refused_never_held_for_a_named_refusal() {
    // #4406: the refusal is the legal answer of the entry stage, and the
    // law is not measured on it; before, it read as held at 0 against 0.
    let v = l5(&Planners { entry: Box::new(NoEntry), ..good() });
    assert_refused(&v, Law::L5, NO_LEGAL_ENTRY);
    assert!(v.worst.place.contains(NO_LEGAL_ENTRY), "{v}");
}

#[test]
fn l5_breaks_for_a_plunge_into_material_named_a_helix() {
    let v = l5(&Planners { entry: Box::new(HiddenPlunge), ..good() });
    assert_broken(&v, Law::L5, "Helix descent travels 0.000");
    assert!(v.worst.value > 0.9, "a full disc of material: {v}");
}

#[test]
fn l5_breaks_for_an_air_proof_on_material() {
    let v = l5(&Planners { entry: Box::new(ClaimsFree { column: false }), ..good() });
    assert_broken(&v, Law::L5, "entry 0: material share");
    assert!(v.worst.value > 0.9, "{v}");
}

#[test]
fn l5_breaks_for_a_cleared_column_proof_on_material() {
    let v = l5(&Planners { entry: Box::new(ClaimsFree { column: true }), ..good() });
    assert_broken(&v, Law::L5, "entry 0");
}
