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
