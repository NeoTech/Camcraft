//! The program of `camcraft-sim check` (#2586): the NC file parsed into
//! sections in the work frame, each move with its NC line, and each move
//! whose position this run cannot know marked to skip.
//! Provenance: RS-274/NGC (NISTIR 6556) and ISO 6983 for the G28, G30
//! and G53 words and the work offsets; the parser is `camcraft-gcode`.

use std::collections::BTreeMap;

use camcraft_gcode::{parse_with, Dialect, ParseOptions, ReferencePositions};
use camcraft_sim::cl::Move;
use camcraft_sim::geometry::Point3;

/// One section of the program: one tool, one work offset.
#[derive(Clone, Debug)]
pub struct Section {
    pub tool: Option<u32>,
    /// 1 = G54 .. 6 = G59.
    pub offset: u8,
    /// The machine position of the offset (`--wcs`), or `None`.
    pub origin: Option<[f64; 3]>,
    /// The moves in the frame of the section's work offset, in mm.
    pub moves: Vec<Move>,
    /// The 1-based NC line of each move.
    pub lines: Vec<usize>,
    /// A move whose start or end this run cannot know: a G28, G30 or
    /// G53 move in an offset with no `--wcs`, and the move after it.
    pub skip: Vec<bool>,
}

/// The parsed program.
#[derive(Clone, Debug)]
pub struct Program {
    pub sections: Vec<Section>,
    /// The unit words that the program states (`G20`, `G21`).
    pub inch: bool,
    pub mm: bool,
    /// The NC line of the first move, whose start is not known.
    pub first_line: Option<usize>,
}

impl Program {
    /// The lines of the moves that are skipped, in order.
    #[must_use]
    pub fn skipped_lines(&self) -> Vec<usize> {
        let mut out: Vec<usize> = self
            .sections
            .iter()
            .flat_map(|s| s.skip.iter().zip(&s.lines).filter(|(k, _)| **k).map(|(_, l)| *l))
            .collect();
        out.dedup();
        out
    }
}

/// The G words of one NC line, comments removed.
fn g_words(line: &str) -> Vec<f64> {
    let mut text = String::new();
    let mut depth = 0usize;
    for c in line.chars() {
        match c {
            ';' if depth == 0 => break,
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            c if depth == 0 => text.push(c.to_ascii_uppercase()),
            _ => {}
        }
    }
    let mut out = Vec::new();
    let mut rest = text.as_str();
    while let Some(at) = rest.find('G') {
        let tail = &rest[at + 1..];
        let end = tail.find(|c: char| !(c.is_ascii_digit() || c == '.' || c == ' ')).unwrap_or(tail.len());
        let word: String = tail[..end].chars().filter(|c| !c.is_whitespace()).collect();
        if let Ok(v) = word.parse::<f64>() {
            out.push(v);
        }
        rest = &tail[end..];
    }
    out
}

/// Parse `text` with `dialect`. `wcs` maps the offsets that the run
/// knows to their machine positions. G28 and G30 return to machine
/// zero, the reference point of a control that states no other.
pub fn read(text: &str, dialect: Dialect, wcs: &BTreeMap<u8, [f64; 3]>) -> Result<Program, String> {
    let mut references = ReferencePositions {
        g28: Some(Point3::ZERO),
        g30: Some(Point3::ZERO),
        ..ReferencePositions::default()
    };
    // An offset with no `--wcs` gets a stand-in origin so that the parse
    // completes; each move that depends on it is marked to skip below.
    for (i, slot) in references.wcs_origins.iter_mut().enumerate() {
        let n = u8::try_from(i + 1).unwrap_or(u8::MAX);
        let at = wcs.get(&n).copied().unwrap_or([0.0; 3]);
        *slot = Some(Point3::new(at[0], at[1], at[2]));
    }
    let parsed = parse_with(text, &ParseOptions { dialect, references }).map_err(|e| e.to_string())?;
    let source: Vec<&str> = text.lines().collect();
    let words_at = |line: usize| source.get(line.wrapping_sub(1)).map(|l| g_words(l)).unwrap_or_default();
    let mut sections = Vec::with_capacity(parsed.sections.len());
    // The end of the last move is a point that this run cannot know.
    let mut end_unknown = false;
    let mut last_offset: Option<u8> = None;
    for s in parsed.sections {
        let offset = s.work_offset.number();
        let origin = wcs.get(&offset).copied();
        // A change of offset after motion moves the cursor by the two
        // origins; with one of them unknown, the next start is unknown.
        if last_offset.is_some_and(|o| o != offset && (origin.is_none() || !wcs.contains_key(&o))) {
            end_unknown = true;
        }
        last_offset = Some(offset);
        let mut skip = Vec::with_capacity(s.toolpath.moves.len());
        for line in &s.move_lines {
            let reference = words_at(*line).iter().any(|g| [28.0, 30.0, 53.0].contains(g));
            let here = origin.is_none() && reference;
            skip.push(here || end_unknown);
            end_unknown = here;
        }
        sections.push(Section {
            tool: s.tool_number,
            offset,
            origin,
            moves: s.toolpath.moves,
            lines: s.move_lines,
            skip,
        });
    }
    // The first move starts where the machine stands, which the program
    // does not say: only its end is judged (the parser's start is a guess).
    let mut first_line = None;
    if let Some(mv) = sections.iter_mut().find_map(|s| s.moves.first_mut().map(|m| (m, s.lines.first().copied()))) {
        mv.0.from = mv.0.to;
        mv.0.arc = None;
        first_line = mv.1;
    }
    let all: Vec<f64> = source.iter().flat_map(|l| g_words(l)).collect();
    Ok(Program { sections, inch: all.contains(&20.0), mm: all.contains(&21.0), first_line })
}
