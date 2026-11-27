//! The region-free verify laws and the machine limits over an external NC
//! program's world moves (#2218), each finding mapped to its 1-based NC
//! line. Provenance: CamCraft's own verify laws (`sim::verify::laws`,
//! `solid`) and machine check (`post::machine::check`); RS-274 line numbers.

use crate::cl::Toolpath;
use crate::geometry::Triangle;
use crate::machine::{Machine, MachineCheck, SourceLocation};
use crate::verify::{self, Cutter, SolidLaw, Warning};
use camcraft_sim::fixture::FixtureBox;
use camcraft_tools::holder_profile::HolderStack;

use super::tools::NcTool;
use super::{NcFinding, NcFindingKind};

/// Sample spacing of the solid law along a vertical-plane arc, mm (the
/// value the operation solid laws use).
const SOLID_STEP: f64 = 0.25;

/// One section, resolved: its world path, its tool, its frame and its NC lines.
pub(super) struct Resolved<'a> {
    pub index: usize,
    pub path: Toolpath,
    pub tool: &'a NcTool,
    pub lines: &'a [usize],
    pub rpm: Option<f64>,
    pub origin: [f64; 3],
    pub flip: [bool; 3],
}

/// The move index a warning points at, and its kind. Exhaustive on
/// purpose: a new warning kind fails to compile here until it says what it is.
fn kind_at(w: &Warning) -> (NcFindingKind, usize) {
    use NcFindingKind as K;
    match w {
        Warning::Crossing { moves } => (K::Path, moves[0]),
        Warning::Gouge { at } | Warning::StockChamferGouge { at } | Warning::StockDeburrGouge { at } => (K::Gouge, *at),
        Warning::RapidInPart { at } => (K::RapidInPart, *at),
        Warning::ShankHit { at, .. } | Warning::HolderHit { at, .. } | Warning::SweepHit { at, .. } => (K::Body, *at),
        Warning::FixtureHit { at, .. } | Warning::FirstTraverseUnderTable { at, .. } => (K::Fixture, *at),
        Warning::RapidInStock { at }
        | Warning::FirstTraverseLow { at, .. }
        | Warning::AvoidHit { at, .. }
        | Warning::NonFinite { at }
        | Warning::Envelope { at, .. }
        | Warning::RestUnjudged { at, .. }
        | Warning::Scallop { at, .. }
        | Warning::ProbeSearch { at, .. }
        | Warning::SinkerFinishOrder { at, .. }
        | Warning::DrillUndersize { at, .. }
        | Warning::UndercutLift { at }
        | Warning::StrategyFallback { at, .. }
        | Warning::FluteShort { at, .. }
        | Warning::PlungeTooDeep { at, .. }
        | Warning::EntryNotRecommended { at, .. }
        | Warning::EntryStandIn { at, .. }
        | Warning::PlungeIntoStock { at }
        | Warning::EngagementExceeded { at, .. } => (K::Path, *at),
    }
}

fn line_of(s: &Resolved<'_>, at: usize) -> usize {
    // The lines are checked 1-to-1 with the moves before any law runs, so
    // a missing entry cannot happen; 0 is the "no line" fallback.
    s.lines.get(at).copied().unwrap_or(0)
}

/// The solid law against the target and the two fixture laws, per section.
pub(super) fn verify_sections(
    sections: &[Resolved<'_>],
    target: &[Triangle],
    fixtures: &[FixtureBox],
    spindle: Option<(f64, f64)>,
    tol: f64,
) -> Vec<NcFinding> {
    let mut out = Vec::new();
    for s in sections {
        let geom = s.tool.tool.geom();
        let mut warns = Vec::new();
        if !target.is_empty() {
            let cutter = Cutter::judged(&geom, Some(Cutter::of(&s.tool.tool)));
            let law = SolidLaw { step: SOLID_STEP, allow: tol, hole_walls: false };
            warns.extend(verify::solid_hits(&s.path, target, &cutter, &law));
        }
        if !fixtures.is_empty() {
            let holder = HolderStack::cylinder(&geom);
            warns.extend(verify::bands_vs_fixtures_held(&s.path, &geom, &holder, fixtures, spindle, tol));
            warns.extend(verify::rapids_vs_fixtures_held(&s.path, &geom, &holder, fixtures, spindle));
        }
        for w in warns {
            let (kind, at) = kind_at(&w);
            out.push(NcFinding { line: line_of(s, at), section: s.index, kind, message: w.to_string() });
        }
    }
    out.sort_by_key(|f| (f.line, f.section));
    out
}

/// The machine's limits: the tool count, each tool's size and spindle
/// speed, and each move's feed and travel, named by NC line.
pub(super) fn machine_findings(machine: &Machine, sections: &[Resolved<'_>]) -> (Vec<NcFinding>, Vec<String>) {
    let mut program = MachineCheck::default();
    let mut names: Vec<&str> = sections.iter().map(|s| s.tool.name.as_str()).collect();
    names.dedup();
    program.check_tool_count(machine, &names);
    let mut out = Vec::new();
    for s in sections {
        let first = s.lines.first().copied().unwrap_or(0);
        let t = &s.tool.tool;
        let mut c = MachineCheck::default();
        c.check_tool_size(machine, Some(SourceLocation::NcLine(first)), &s.tool.name, t.flute_len + t.shank_len + t.holder_len, t.diameter);
        if let Some(rpm) = s.rpm {
            c.check_spindle(machine, Some(SourceLocation::NcLine(first)), &s.tool.name, rpm);
        }
        push(&mut out, s.index, first, c);
        for (i, mv) in s.path.moves.iter().enumerate() {
            let line = line_of(s, i);
            let mut c = MachineCheck::default();
            c.check_path(machine, Some(SourceLocation::NcLine(line)), &Toolpath::new(vec![*mv]), s.origin, s.flip, true);
            push(&mut out, s.index, line, c);
        }
    }
    let mut notes = program.problems;
    notes.extend(program.notes);
    (out, notes)
}

fn push(out: &mut Vec<NcFinding>, section: usize, line: usize, c: MachineCheck) {
    out.extend(c.problems.into_iter().map(|message| NcFinding { line, section, kind: NcFindingKind::Machine, message }));
}
