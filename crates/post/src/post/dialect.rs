//! What a post script and a section say about WHICH spindle turns
//! (#1190): the two facts a caller needs before it hands a section to
//! a turning dialect, each stated once.
//!
//! - [`speaks_turning`]: the script's own manifest. Every shipped post
//!   declares `capabilities`, an integer mask (1 = milling, 2 =
//!   turning). A turning script opens a plain section with the WORK
//!   spindle (`G97 S M03`), so the app's program gate must know it is
//!   about to post through one. A script that declares no usable mask
//!   (no `capabilities`, or not a finite non-negative number) is
//!   UNDECLARED ([`declared_turning`] answers `None`): nothing is guessed
//!   about a dialect it does not state, and the app decides from the
//!   machine what an undeclared script is held to (#1199).
//! - [`states_live_tool`]: the section's own statement. A path whose
//!   moves state spindle C angles is live-tool work; the stream's
//!   `PARAM LiveTool` is written from this one test (`dump`), and the
//!   app reads the same test, never a second copy.
//!
//! Provenance: our own contract (the manifest mask is the `.cps`
//! reference contract every script in `assets/posts` already states;
//! the C statement is issue #33's). No vendor material consulted.

use super::js::Script;
use super::PostError;
use crate::cl::Toolpath;

/// The `capabilities` bit a turning dialect declares.
const TURNING: f64 = 2.0;

/// Whether the script's manifest declares the turning dialect
/// (`capabilities` with the turning bit set). An undeclared script
/// reads as not turning here; a caller to whom that matters asks
/// [`declared_turning`] instead.
///
/// # Errors
/// A script that does not load is a clean refusal, as everywhere.
pub fn speaks_turning(cps: &str) -> Result<bool, PostError> {
    Ok(declared_turning(cps)?.unwrap_or(false))
}

/// What the script's manifest states about the turning dialect (#1199):
/// `Some(true)` / `Some(false)` for a declared mask with / without the
/// turning bit, `None` when the script states no usable mask - no
/// `capabilities` at all, a non-number, a negative or a non-finite one.
///
/// # Errors
/// A script that does not load is a clean refusal, as everywhere.
pub fn declared_turning(cps: &str) -> Result<Option<bool>, PostError> {
    let mut s = Script::load(cps)?;
    let Some(mask) = s.number_global("capabilities")? else {
        return Ok(None);
    };
    if !mask.is_finite() || mask < 0.0 {
        return Ok(None);
    }
    // The mask is a small whole number; read its turning bit without a
    // float-to-integer cast that could wrap.
    Ok(Some(((mask / TURNING).floor() % 2.0) >= 1.0))
}

/// Whether a path is C-axis live-tool work: some move states the
/// spindle's C angle (issue #33). The stream's `PARAM LiveTool` and
/// every reader that asks "does a live tool cut this section?" share
/// this one test.
#[must_use]
pub fn states_live_tool(path: &Toolpath) -> bool {
    path.moves.iter().any(|m| m.rotary.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cl::{Move, MoveKind};
    use crate::geometry::Point3;

    #[test]
    fn the_turning_bit_is_read_from_the_manifest_mask() {
        for (text, want) in [
            ("capabilities = 2;", true),
            ("capabilities = 3;", true),
            ("capabilities = 1;", false),
            ("capabilities = 0;", false),
            ("capabilities = 4;", false),
            ("capabilities = -2;", false),
            ("capabilities = \"2\";", false),
            ("capabilities = 0 / 0;", false),
            ("var other = 2;", false),
        ] {
            assert_eq!(speaks_turning(text).expect("loads"), want, "{text}");
        }
        assert!(speaks_turning("capabilities = ;").is_err(), "a script that does not parse refuses");
    }

    #[test]
    fn an_undeclared_mask_is_none_never_a_guess() {
        for (text, want) in [
            ("capabilities = 2;", Some(true)),
            ("capabilities = 3;", Some(true)),
            ("capabilities = 1;", Some(false)),
            ("capabilities = 0;", Some(false)),
            ("capabilities = 4;", Some(false)),
            ("var other = 2;", None),
            ("", None),
            ("capabilities = \"2\";", None),
            ("capabilities = -2;", None),
            ("capabilities = 0 / 0;", None),
            ("capabilities = 1 / 0;", None),
        ] {
            assert_eq!(declared_turning(text).expect("loads"), want, "{text}");
        }
        assert!(declared_turning("capabilities = ;").is_err(), "a script that does not parse refuses");
    }

    #[test]
    fn a_path_is_live_tool_work_only_when_a_move_states_c() {
        let m = Move::new(MoveKind::Feed, Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0), Some(100.0));
        assert!(!states_live_tool(&Toolpath::new(vec![m])));
        let mut c = m;
        c.rotary = Some(0.0);
        assert!(states_live_tool(&Toolpath::new(vec![m, c])));
        assert!(!states_live_tool(&Toolpath::new(Vec::new())));
    }
}
