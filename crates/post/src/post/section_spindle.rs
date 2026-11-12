//! Which spindle a path's section drives (#1174): the one reading the
//! stream ([`super::dump`]'s `PARAM LiveTool`) and the program gate's
//! workholding rule (`app::setup_workholding`) both take, so the two
//! can never disagree about whether the work turns at the section's S.
//!
//! - A path stating a C angle on any move is C-axis live-tool work
//!   (issue #33): the stream says `PARAM LiveTool`, and a lathe post
//!   gives the section's S to the live spindle with the work on C.
//! - A path stating a leaning tool axis on a non-C move commands A/B
//!   words; the lathe posts refuse those sections ("posed move").
//! - Anything else a lathe post opens on the WORK spindle
//!   (`G97 S M03` / `G96`), whatever the operation's kind.
//!
//! Provenance: our own stream grammar (issue #33, M5.1 stage A); a
//! lathe's main spindle turns the work and a live-tool spindle the
//! cutter (any turning handbook's mill-turn chapter). No vendor
//! material consulted.

use crate::cl::{Move, Toolpath};

/// What a lathe post does with a path's section, by its moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectionSpindle {
    /// No C, no lean: the work spindle turns at the section's S.
    Work,
    /// C stated: live-tool work, the work positioned and fed on C.
    LiveTool,
    /// A leaning axis on a non-C move: A/B words a lathe post refuses.
    Posed,
}

/// A non-C move stating a leaning tool axis - one that commands A/B.
#[must_use]
pub fn leans(m: &Move) -> bool {
    m.rotary.is_none() && m.axis.is_some_and(|a| !crate::cl::is_vertical(&a))
}

/// The section spindle of `t` (C first: a C move commands no A/B).
#[must_use]
pub fn of(t: &Toolpath) -> SectionSpindle {
    if t.moves.iter().any(|m| m.rotary.is_some()) {
        SectionSpindle::LiveTool
    } else if t.moves.iter().any(leans) {
        SectionSpindle::Posed
    } else {
        SectionSpindle::Work
    }
}

/// Which spindle a post script gives a [`SectionSpindle::Work`]
/// section's S to (#1174), from the dialect its manifest declares
/// (`capabilities`: bit 2 = turning, bit 1 = milling). A turning post
/// opens such a section `G97 S M03` / `G96` on the WORK spindle (both
/// shipped lathe posts); a milling post's S drives the TOOL spindle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkVoice {
    /// A turning dialect: a Work section turns the work at its S.
    WorkSpindle,
    /// A milling dialect only: the S word drives the tool.
    ToolSpindle,
    /// No dialect stated (a user's own script without the manifest
    /// line), or one this reading cannot settle: the caller takes the
    /// stricter reading (#1199 decides undeclared scripts for good).
    Undeclared,
}

/// The [`WorkVoice`] of post script `cps`.
#[must_use]
pub fn work_voice(cps: &str) -> WorkVoice {
    match manifest_mask(cps) {
        Some(m) if m & 2 != 0 => WorkVoice::WorkSpindle,
        Some(m) if m & 1 != 0 => WorkVoice::ToolSpindle,
        _ => WorkVoice::Undeclared,
    }
}

/// The script's top-level `capabilities = <integer>;` mask: a line at
/// column 0 (a manifest line, never one inside a function body), its
/// `//` comment set aside. Two lines that disagree, or a value that is
/// not a plain integer, read as `None` - never guessed.
fn manifest_mask(cps: &str) -> Option<u32> {
    let mut found: Option<u32> = None;
    for line in cps.lines() {
        let code = line.split("//").next().unwrap_or("");
        let Some(rest) = code.strip_prefix("capabilities") else { continue };
        let Some(value) = rest.trim_start().strip_prefix('=') else { continue };
        let value = value.trim().trim_end_matches(';').trim();
        let m = value.parse::<u32>().ok()?;
        if found.is_some_and(|f| f != m) {
            return None;
        }
        found = Some(m);
    }
    found
}

#[cfg(test)]
mod tests {
    use super::{work_voice, WorkVoice};

    #[test]
    fn the_manifest_mask_names_the_spindle_a_work_section_drives() {
        assert_eq!(work_voice("capabilities = 2; // turning\n"), WorkVoice::WorkSpindle);
        assert_eq!(work_voice("capabilities = 3;\n"), WorkVoice::WorkSpindle, "turning wins: the stricter reading");
        assert_eq!(work_voice("capabilities = 1; // milling\n"), WorkVoice::ToolSpindle);
        assert_eq!(work_voice("function onOpen() {}\n"), WorkVoice::Undeclared);
        assert_eq!(work_voice("// capabilities = 1;\n"), WorkVoice::Undeclared, "a comment declares nothing");
        assert_eq!(work_voice("function f() {\n  capabilities = 1;\n}\n"), WorkVoice::Undeclared, "only a top-level line");
        assert_eq!(work_voice("capabilities = 1;\ncapabilities = 2;\n"), WorkVoice::Undeclared, "disagreeing lines");
        assert_eq!(work_voice("capabilities = CAP;\n"), WorkVoice::Undeclared, "not a plain integer");
        assert_eq!(work_voice(include_str!("../../../../assets/posts/fanuc-lathe.cps")), WorkVoice::WorkSpindle);
        assert_eq!(work_voice(include_str!("../../../../assets/posts/tormach-lathe.cps")), WorkVoice::WorkSpindle);
        assert_eq!(work_voice(include_str!("../../../../assets/posts/fanuc.cps")), WorkVoice::ToolSpindle);
    }
}
