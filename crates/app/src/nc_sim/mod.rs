//! External NC program simulation (#2218): a parsed RS-274 / ISO 6983
//! program (`camcraft-gcode`) cut from the setup's stock, judged against
//! the target model, the fixtures and the mounted machine, with each
//! finding on its 1-based NC line. Provenance: RS-274/NGC (NISTIR 6556) and
//! ISO 6983 program semantics; CamCraft's own stock-sim, verify and machine laws.
//!
//! The runner reads the document and changes nothing, so it is no undoable
//! command. Each refusal names its reason: a `T` number with no cutter
//! geometry, a setup with no stock, a work offset with no origin, a turned
//! WCS, a machine that is not a mill, a posed move.

mod checks;
mod frame;
mod tools;

pub use frame::{frame_for, NcFrame};
pub use tools::{map_tools, NcTool, NcToolMap, NcToolSource};

use flecs_ecs::prelude::World;

use camcraft_gcode::{GcodeError, ParseOptions, ParsedProgram, ParsedSection};
use camcraft_sim::fixture::FixtureBox;

use crate::cl::Move;
use crate::geometry::Triangle;
use crate::machine::Machine;
use crate::stock_sim::{Deviation, Planned, SimOp, DEV_TOL};
use crate::tool_library::LibraryTool;

/// The caller's inputs. Each `None` or empty field reads the document:
/// the active setup, its mounted machine, its fixtures and its parts.
#[derive(Clone, Debug, Default)]
pub struct NcSimOptions {
    /// The setup whose stock, WCS, fixtures and machine are read (`None`: the active one).
    pub setup: Option<String>,
    /// Explicit `T` number to tool name: a library row's name, else a document tool's.
    pub tool_map: Vec<(u32, String)>,
    /// Rows of a `.camtools` library (`tool_library::parse_tool_library`).
    pub library: Vec<LibraryTool>,
    /// World-millimetre origins of `G54`..`G59`, overriding the setup's WCS.
    pub wcs_origins: [Option<[f64; 3]>; 6],
    /// The target model's world facets (`Some(empty)`: no target).
    pub target: Option<Vec<Triangle>>,
    /// The fixture boxes (`Some(empty)`: no fixtures).
    pub fixtures: Option<Vec<FixtureBox>>,
    /// The machine, overriding the setup's mounted `.mch`.
    pub machine: Option<Machine>,
    /// The gouge and deviation tolerance, mm (default `DEV_TOL`).
    pub tol: Option<f64>,
}

/// Why an NC program was not simulated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NcSimError {
    /// The program text did not parse.
    Parse(String),
    /// A section cuts before any tool call, at this NC line.
    NoTool { line: usize },
    /// These `T` numbers have no cutter geometry, with one reason each.
    UnmappedTools { numbers: Vec<u32>, reasons: Vec<String> },
    /// Any other refusal, by name.
    Refused(String),
}

impl core::fmt::Display for NcSimError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            NcSimError::Parse(e) => write!(f, "the NC program does not parse: {e}"),
            NcSimError::NoTool { line } => write!(f, "line {line}: the program moves before any tool call (T word)"),
            NcSimError::UnmappedTools { numbers, reasons } => {
                let list: Vec<String> = numbers.iter().map(|n| format!("T{n}")).collect();
                write!(f, "unmapped tools {} - {}", list.join(", "), reasons.join("; "))
            }
            NcSimError::Refused(why) => f.write_str(why),
        }
    }
}

impl std::error::Error for NcSimError {}

/// The law a finding comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NcFindingKind {
    /// A cutting move in the target model.
    Gouge,
    /// A rapid move in the target model.
    RapidInPart,
    /// The shank or holder in the target model.
    Body,
    /// The tool, holder or spindle in a fixture or the table.
    Fixture,
    /// A machine limit: travel, feed, spindle speed, tool size.
    Machine,
    /// Any other path law.
    Path,
}

/// One finding, on its 1-based NC line.
#[derive(Clone, Debug, PartialEq)]
pub struct NcFinding {
    pub line: usize,
    pub section: usize,
    pub kind: NcFindingKind,
    pub message: String,
}

/// One simulated section.
#[derive(Clone, Debug, PartialEq)]
pub struct NcSection {
    pub index: usize,
    pub tool_number: u32,
    pub tool: String,
    pub moves: usize,
    pub first_line: usize,
}

/// The simulation result.
#[derive(Clone, Debug)]
pub struct NcSimReport {
    pub sections: Vec<NcSection>,
    /// The cut stock against the target model; `None` with no target.
    pub deviation: Option<Deviation>,
    pub tol: f64,
    /// Line-numbered findings, in line order.
    pub findings: Vec<NcFinding>,
    /// Program-wide machine findings (the tool changer's capacity).
    pub program_problems: Vec<String>,
    /// What was not judged, and why. Notes never block.
    pub notes: Vec<String>,
}

impl NcSimReport {
    /// No finding, and no gouge past the tolerance.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.findings.is_empty()
            && self.program_problems.is_empty()
            && self.deviation.as_ref().is_none_or(|d| d.max_gouge <= self.tol)
    }
}

/// One section in world millimetres, with its tool.
struct Section<'a> {
    src: &'a ParsedSection,
    tool: &'a NcTool,
    frame: NcFrame,
    moves: Vec<Move>,
}

fn first_line(s: &ParsedSection) -> usize {
    s.move_lines.first().copied().unwrap_or(0)
}

fn machine_of(world: &World, opts: &NcSimOptions) -> Result<Option<Machine>, NcSimError> {
    let m = opts.machine.clone().or_else(|| crate::machine::of_setup(world, opts.setup.as_deref()).1);
    if let Some(m) = &m {
        let kind = crate::machine::kind(Some(m));
        if !matches!(kind, crate::machine::Kind::Mill | crate::machine::Kind::MillTurn) {
            return Err(NcSimError::Refused(format!(
                "the machine '{}' is a {kind:?} - the NC simulation judges milling programs only",
                m.model
            )));
        }
    }
    Ok(m)
}

/// Map the program's `T` numbers to cutter geometry (no refusal for an
/// unmapped number: the map lists them).
///
/// # Errors
/// Returns a section that moves before any tool call, or the document's
/// tool-number refusal.
pub fn unmapped_tools(world: &World, program: &ParsedProgram, opts: &NcSimOptions) -> Result<NcToolMap, NcSimError> {
    let mut numbers = Vec::new();
    for s in program.sections.iter().filter(|s| !s.toolpath.moves.is_empty()) {
        let n = s.tool_number.ok_or(NcSimError::NoTool { line: first_line(s) })?;
        numbers.push(n);
    }
    map_tools(world, &numbers, opts).map_err(NcSimError::Refused)
}

fn sections<'a>(
    world: &World,
    program: &'a ParsedProgram,
    opts: &NcSimOptions,
    map: &'a NcToolMap,
    machine: Option<&Machine>,
) -> Result<Vec<Section<'a>>, NcSimError> {
    let mut out = Vec::new();
    for src in program.sections.iter().filter(|s| !s.toolpath.moves.is_empty()) {
        let line = first_line(src);
        if src.move_lines.len() != src.toolpath.moves.len() {
            return Err(NcSimError::Refused(format!("line {line}: the section's line map does not match its moves")));
        }
        let n = src.tool_number.ok_or(NcSimError::NoTool { line })?;
        let tool = map.get(n).ok_or_else(|| NcSimError::UnmappedTools { numbers: vec![n], reasons: Vec::new() })?;
        let frame = frame_for(world, opts, machine, src.work_offset.index())
            .map_err(|e| NcSimError::Refused(format!("line {line}: {e}")))?;
        let mut moves = src
            .toolpath
            .moves
            .iter()
            .zip(&src.move_lines)
            .map(|(m, l)| frame.move_to_world(m).map_err(|e| NcSimError::Refused(format!("line {l}: {e}"))))
            .collect::<Result<Vec<_>, _>>()?;
        // The program's first move starts from the parser's assumed
        // position (`Dialect::initial_position`), not from a place the
        // machine stood: the tool appears at its first commanded point.
        if out.is_empty()
            && let Some(m) = moves.first_mut()
            && m.arc.is_none()
        {
            m.from = m.to;
        }
        out.push(Section { src, tool, frame, moves });
    }
    if out.is_empty() {
        return Err(NcSimError::Refused("the NC program has no moves to simulate".into()));
    }
    Ok(out)
}

fn planned(sections: &[Section<'_>], setup: Option<&str>) -> Result<Vec<Planned>, NcSimError> {
    sections
        .iter()
        .map(|s| {
            let tools::Atom { radius, ball, cone, corner, drill } = tools::atom(s.tool).map_err(NcSimError::Refused)?;
            Ok(Planned {
                name: format!("T{} (line {})", s.tool.number, first_line(s.src)),
                moves: s.moves.clone(),
                radius,
                ball,
                cone,
                corner,
                setup: setup.map(str::to_owned),
                rotary: None,
                lathe: None,
                breaks: Vec::new(),
                walled: false,
                auto_edges: false,
                mark: None,
                electrode: None,
                drill,
                undercut: None,
                corner_reliefs: Vec::new(),
            })
        })
        .collect()
}

struct Prepared<'a> {
    sections: Vec<Section<'a>>,
    planned: Vec<Planned>,
    machine: Option<Machine>,
}

fn prepare<'a>(world: &World, program: &'a ParsedProgram, opts: &NcSimOptions, map: &'a NcToolMap) -> Result<Prepared<'a>, NcSimError> {
    if !map.unmapped.is_empty() {
        return Err(NcSimError::UnmappedTools { numbers: map.unmapped.clone(), reasons: map.reasons.clone() });
    }
    let machine = machine_of(world, opts)?;
    let sections = sections(world, program, opts, map, machine.as_ref())?;
    let planned = planned(&sections, opts.setup.as_deref())?;
    Ok(Prepared { sections, planned, machine })
}

fn tol_of(opts: &NcSimOptions) -> Result<f64, NcSimError> {
    let tol = opts.tol.unwrap_or(DEV_TOL);
    if tol.is_finite() && tol > 0.0 {
        Ok(tol)
    } else {
        Err(NcSimError::Refused(format!("the simulation tolerance must be a positive number, not {tol}")))
    }
}

fn target_of(world: &World, opts: &NcSimOptions) -> Vec<Triangle> {
    opts.target.clone().unwrap_or_else(|| crate::setup_align::parts_facets_in(world, opts.setup.as_deref()))
}

/// Parse `text` and simulate it ([`simulate_nc`]).
///
/// # Errors
/// Returns the parser's line-numbered error, or a refusal of [`simulate_nc`].
pub fn simulate_nc_text(world: &World, text: &str, parse: &ParseOptions, opts: &NcSimOptions) -> Result<NcSimReport, NcSimError> {
    let program = camcraft_gcode::parse_with(text, parse).map_err(|e: GcodeError| NcSimError::Parse(e.to_string()))?;
    simulate_nc(world, &program, opts)
}

/// Cut the program from the setup's stock, measure the deviation from the
/// target model, and judge the solid law, the fixture laws and the
/// machine limits on every move.
///
/// # Errors
/// Refuses unmapped tools, a setup with no stock, and the frame refusals
/// ([`frame_for`]), each by name. A refusal changes nothing.
pub fn simulate_nc(world: &World, program: &ParsedProgram, opts: &NcSimOptions) -> Result<NcSimReport, NcSimError> {
    let tol = tol_of(opts)?;
    let map = unmapped_tools(world, program, opts)?;
    let prep = prepare(world, program, opts, &map)?;
    let setup = opts.setup.as_deref();
    let refused = |e: String| NcSimError::Refused(e);

    let min_r = prep.planned.iter().map(|p| p.radius).fold(f64::INFINITY, f64::min);
    let mut dex = crate::stock_sim::stock_grid(world, setup, min_r).map_err(refused)?;
    let ops: Vec<SimOp<'_>> = prep
        .planned
        .iter()
        .map(|p| SimOp { moves: &p.moves, radius: p.radius, ball: p.ball, cone: p.cone, corner: p.corner })
        .collect();
    crate::stock_sim::simulate(&mut dex, &ops).map_err(|e| refused(e.to_string()))?;

    let mut notes = vec![format!(
        "line {}: the first move starts at its own end point - the position before the program is not known",
        prep.sections.first().map_or(0, |s| first_line(s.src))
    )];
    let target = target_of(world, opts);
    let deviation = if target.is_empty() {
        notes.push("no target model - the deviation and the solid law are not judged".to_string());
        None
    } else {
        Some(crate::stock_sim::deviate(&dex, &target, tol).map_err(|e| refused(e.to_string()))?)
    };
    let fixtures = opts.fixtures.clone().unwrap_or_else(|| crate::fixture::boxes_for(world, setup));
    let spindle = prep.machine.as_ref().and_then(|m| m.spindle_d.zip(m.spindle_len)).map(|(d, l)| (d * 0.5, l));

    let resolved: Vec<checks::Resolved<'_>> = prep
        .sections
        .iter()
        .map(|s| checks::Resolved {
            index: s.src.index,
            path: crate::cl::Toolpath::new(s.moves.clone()),
            tool: s.tool,
            lines: &s.src.move_lines,
            rpm: s.src.spindle_rpm,
            origin: s.frame.origin,
            flip: s.frame.flip,
        })
        .collect();
    let mut findings = checks::verify_sections(&resolved, &target, &fixtures, spindle, tol);
    let mut program_problems = Vec::new();
    match &prep.machine {
        Some(m) => {
            let (more, problems) = checks::machine_findings(m, &resolved);
            findings.extend(more);
            program_problems = problems;
        }
        None => notes.push("no machine mounted - travel, feed and spindle limits are not judged".to_string()),
    }
    findings.sort_by_key(|f| (f.line, f.section));

    let sections = prep
        .sections
        .iter()
        .map(|s| NcSection {
            index: s.src.index,
            tool_number: s.tool.number,
            tool: s.tool.name.clone(),
            moves: s.moves.len(),
            first_line: first_line(s.src),
        })
        .collect();
    Ok(NcSimReport { sections, deviation, tol, findings, program_problems, notes })
}

/// The playback seed of the program's sections at a display budget of
/// `texels` cells: the setup's stock, the sections in world millimetres
/// (one timeline entry each, named `T<n> (line <first line>)`), and the
/// target model when there is one.
///
/// # Errors
/// The refusals of [`simulate_nc`], and the field view's own refusals.
pub fn play_seed(world: &World, program: &ParsedProgram, opts: &NcSimOptions, texels: f64) -> Result<crate::stock_play::PlaySeed, NcSimError> {
    let tol = tol_of(opts)?;
    let map = unmapped_tools(world, program, opts)?;
    let prep = prepare(world, program, opts, &map)?;
    let target = target_of(world, opts);
    let part = (!target.is_empty()).then_some(target);
    crate::stock_play::PlaySeed::of_planned(world, prep.planned, opts.setup.clone(), part, tol, texels).map_err(NcSimError::Refused)
}
