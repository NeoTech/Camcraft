//! The notes of a toolpath that the laws read: the named lifts of the link
//! stage and the fallbacks (law L4 of #4023).
//! Provenance: our own design record (#4024, L4 and "Link rule"); the note
//! texts are the ones that `links.rs` and the entry and pass-start stages
//! of `camcraft-motion` write. The lift count is the same parse as
//! `named_lifts` of `pass_start_levels/ring_links.rs`.

use camcraft_motion::cl::Toolpath;
use camcraft_motion::machining2d::{is_entry_stand_in, rings_fallbacks};

/// The lifts that the link notes name ("link: ... lifted ... N time(s)").
pub(super) fn named_lifts(path: &Toolpath) -> usize {
    path.notes
        .iter()
        .filter(|n| n.starts_with("link: "))
        .filter_map(|n| {
            n.split(" time(s)")
                .next()?
                .rsplit(' ')
                .next()?
                .parse::<usize>()
                .ok()
        })
        .fold(0usize, usize::saturating_add)
}

/// The words with which a note that has no classifier of its own says
/// that the planner used a different plan than the one asked.
const FALLBACK_WORDS: &[&str] = &[" instead", "fell back", "falls back", "fallback", "stand-in"];

/// True if `note` says that a different plan was used: the planner's own
/// classifiers (a stand-in entry, the Offset rings in place of the
/// Adaptive plan) first, then the words above. A lift note is not a
/// fallback: it is a link with a reason.
pub fn is_fallback(note: &str) -> bool {
    if note.starts_with("link: ") {
        return false;
    }
    if is_entry_stand_in(note) || rings_fallbacks(&[note.to_string()]) > 0 {
        return true;
    }
    let low = note.to_ascii_lowercase();
    FALLBACK_WORDS.iter().any(|w| low.contains(w))
}
