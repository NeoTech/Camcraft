//! L5 (#4027): an entry into material needs a proof; else the kernel
//! refuses with a reason. The kernel's `Entry` already binds each descent
//! to the place that its proof names. This check reads each descent on the
//! check's own stock grid, as the stock is at that step of the plan: a
//! descent onto material with a proof other than `IntoMaterial` (air, a
//! hole, a cleared column) is a breach.

use camcraft_pathkernel::entry::{Area, Entry, EntryChoice, EntryKind, EntryPlanner, Exit, Proof};
use camcraft_pathkernel::plan::Cut;
use camcraft_pathkernel::region::KernelRegion;
use camcraft_pathkernel::{PlanePoint, Refusal, Region, Verdict};

use super::grid::walk;
use super::kit::{Case, Law, LawVerdict, Measure, Planners, Run, assert_broken, assert_held, judge, planar, refused_held, run};
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
        Err(e) => return refused_held(Law::L5, e),
    };
    let Ok(tool_r) = planar(&r.region).map(|p| p.tool().radius()) else {
        return judge(Law::L5, vec![Measure::new(f64::INFINITY, 0.0, "the region is not planar", None)]);
    };
    let Some(w) = walk(&r.plan, &c.stock, tool_r) else {
        return judge(Law::L5, vec![Measure::new(f64::INFINITY, 0.0, NO_STOCK, None)]);
    };
    let m = w
        .entries
        .iter()
        .map(|e| {
            let v = if e.into_material { 0.0 } else { e.fill };
            let place = format!("entry {}: material share {:.3} under the tool, proof into material: {}", e.index, e.fill, e.into_material);
            Measure::new(v, ENTRY_FILL, place, Some(e.at))
        })
        .collect();
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
        Err(Refusal::new(camcraft_pathkernel::Stage::Entry, NO_LEGAL_ENTRY))
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
fn l5_holds_for_a_proved_helix_and_for_a_named_refusal() {
    assert_held(&l5(&good()));
    let v = l5(&Planners { entry: Box::new(NoEntry), ..good() });
    assert_held(&v);
    assert!(v.worst.place.contains(NO_LEGAL_ENTRY), "{v}");
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
