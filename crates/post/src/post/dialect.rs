//! What a post script and a section say about WHICH spindle turns
//! (#1190): the two facts a caller needs before it hands a section to
//! a turning dialect, each stated once.
//!
//! - [`speaks_turning`]: the script's own manifest. Every shipped post
//!   declares `capabilities`, an integer mask (1 = milling, 2 =
//!   turning). A turning script opens a plain section with the WORK
//!   spindle (`G97 S M03`), so the app's program gate must know it is
//!   about to post through one. A script that declares no mask is read
//!   as not turning - nothing is guessed about a dialect it does not
//!   state.
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
/// (`capabilities` with the turning bit set).
///
/// # Errors
/// A script that does not load is a clean refusal, as everywhere.
pub fn speaks_turning(cps: &str) -> Result<bool, PostError> {
    let mut s = Script::load(cps)?;
    let mask = s.number_global("capabilities")?.unwrap_or(0.0);
    if !mask.is_finite() || mask < 0.0 {
        return Ok(false);
    }
    // The mask is a small whole number; read its turning bit without a
    // float-to-integer cast that could wrap.
    Ok(((mask / TURNING).floor() % 2.0) >= 1.0)
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
    fn a_path_is_live_tool_work_only_when_a_move_states_c() {
        let m = Move::new(MoveKind::Feed, Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0), Some(100.0));
        assert!(!states_live_tool(&Toolpath::new(vec![m])));
        let mut c = m;
        c.rotary = Some(0.0);
        assert!(states_live_tool(&Toolpath::new(vec![m, c])));
        assert!(!states_live_tool(&Toolpath::new(Vec::new())));
    }
}
