//! HEIDENHAIN Klartext (conversational, `.H`) reader, first slice: a 3-axis subset into the same `ParsedProgram` sections as the RS-274 reader.
//! Provenance: HEIDENHAIN iTNC 530 conversational user's manual 533 190-22 and the TNC cycle manuals, as summarized in docs/research/heidenhain/; CamCraft's own Klartext post output (assets/posts/heidenhain-klartext.cps). Decisions: docs/decisions/2227-klartext-reader.md.

mod blocks;
mod cycles;
mod exec;
mod path;
mod source;
mod words;

use camcraft_geometry::geometry::Point3;

use crate::domain::MAX_ABS_MM;
use crate::types::{GcodeError, ParsedProgram};

/// Caller configuration for a Klartext program.
#[derive(Clone, Debug, PartialEq)]
pub struct KlartextOptions {
    /// Tool position (work frame, mm) before the first block.
    pub initial_position: Point3,
    /// Largest start/end radius difference of a `C` arc, in mm.
    pub arc_radius_tolerance_mm: f64,
    /// Work-frame Z of the end of the tool-axis traverse range. `M140 MB MAX`
    /// retracts to it; without it `M140 MB MAX` is refused.
    pub tool_axis_max_z: Option<f64>,
}

impl Default for KlartextOptions {
    fn default() -> Self {
        Self {
            initial_position: Point3::ZERO,
            arc_radius_tolerance_mm: 0.02,
            tool_axis_max_z: None,
        }
    }
}

/// Parse a HEIDENHAIN Klartext program with default options.
///
/// # Errors
/// Returns a line-numbered [`GcodeError`]; a refusal names the block number and
/// the Klartext function that this reader does not read.
pub fn parse_klartext(program: &str) -> Result<ParsedProgram, GcodeError> {
    parse_klartext_with(program, &KlartextOptions::default())
}

/// Parse a HEIDENHAIN Klartext program with caller options.
///
/// # Errors
/// Returns a line-numbered [`GcodeError`] for bad options, syntax, an unsupported
/// function, an invalid arc or cycle, or a missing `BEGIN PGM` / `END PGM`.
pub fn parse_klartext_with(
    program: &str,
    options: &KlartextOptions,
) -> Result<ParsedProgram, GcodeError> {
    check_options(options)?;
    let (blocks, lines) = source::split_blocks(program)?;
    let mut m = exec::Machine::new(options);
    for b in &blocks {
        m.step(b)?;
    }
    m.finish(lines)
}

fn check_options(o: &KlartextOptions) -> Result<(), GcodeError> {
    let p = o.initial_position;
    let bad = |field: &str, v: String| GcodeError::InvalidOptions {
        field: field.to_string(),
        message: v,
    };
    if ![p.x, p.y, p.z]
        .iter()
        .all(|v| v.is_finite() && v.abs() <= MAX_ABS_MM)
    {
        return Err(bad(
            "initial_position",
            format!("({}, {}, {}) is not finite or out of range", p.x, p.y, p.z),
        ));
    }
    let t = o.arc_radius_tolerance_mm;
    if !(t.is_finite() && (0.0..=1.0e3).contains(&t)) {
        return Err(bad(
            "arc_radius_tolerance_mm",
            format!("({t}) must be finite and in [0, 1000] mm"),
        ));
    }
    if let Some(z) = o.tool_axis_max_z
        && !(z.is_finite() && z.abs() <= MAX_ABS_MM)
    {
        return Err(bad(
            "tool_axis_max_z",
            format!("({z}) is not finite or out of range"),
        ));
    }
    Ok(())
}
