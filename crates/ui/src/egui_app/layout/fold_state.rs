//! The safety state on a fold chip (#4483, the #4245 E2 rule): a folded
//! pane still says, as text with a glyph, what the user must act on. The
//! Browser chip counts the operations that need work; the Properties chip
//! names the state of the open operation in the words decided on #4245
//! ("Up to date", "Needs regenerate"). Both read the same live sources as
//! the tree and the operation page (`status_of`, `tool_problem_of`).
//! Provenance: our own code; no HSMWorks material involved.

use camcraft_app::model;
use camcraft_app::operation::{self, OpStatus};
use camcraft_ui_kit::fold::{ChipState, ChipTone};

use crate::egui_app::shell::Tab;
use crate::state::{State, UiTarget};

fn chip(tone: ChipTone, text: impl Into<String>, detail: impl Into<String>) -> ChipState {
    ChipState { tone, text: text.into(), detail: detail.into() }
}

/// The Browser chip: the counts of blocked, stale and never-generated
/// operations and the warnings of the toolpaths (#4503; a stale or blocked
/// operation counts its old warnings too, as its tree row prints them),
/// or "Up to date" when none needs work. `None` with no operation in the
/// document.
pub(crate) fn browser_state(state: &State) -> Option<ChipState> {
    let w = &state.world;
    let (mut blocked, mut stale, mut never) = (0_usize, 0_usize, 0_usize);
    let (mut warned, mut total) = (0_usize, 0_usize);
    for (_, e) in model::named_entities(w) {
        if !model::is_operation(w, &e) {
            continue;
        }
        total += 1;
        // The tree row's own count: the warnings of the cached toolpath.
        warned += model::with_toolpath(w, &e, |tp| tp.warnings.len()).unwrap_or(0);
        if operation::tool_gate::tool_problem_of(w, &e).is_some() {
            blocked += 1;
            continue;
        }
        match operation::status_of(w, &e) {
            OpStatus::Stale => stale += 1,
            OpStatus::Never => never += 1,
            OpStatus::Ok => {}
        }
    }
    if total == 0 {
        return None;
    }
    let warn_word = if warned == 1 { "warning" } else { "warnings" };
    let parts: Vec<String> = [(blocked, "blocked"), (stale, "stale"), (never, "not generated"), (warned, warn_word)]
        .iter()
        .filter(|(n, _)| *n > 0)
        .map(|(n, word)| format!("{n} {word}"))
        .collect();
    Some(if parts.is_empty() {
        chip(ChipTone::Ok, "Up to date", "")
    } else {
        chip(ChipTone::Attention, parts.join(" \u{b7} "), "")
    })
}

/// The Properties chip: the state of the open operation. `None` when the
/// pane shows no operation.
pub(crate) fn properties_state(state: &State) -> Option<ChipState> {
    let UiTarget::Operation(name) = &state.ui_target else { return None };
    let w = &state.world;
    let e = model::find_by_name(w, name)?;
    if let Some(why) = operation::tool_gate::tool_problem_of(w, &e) {
        return Some(chip(ChipTone::Attention, "Blocked", why));
    }
    Some(match operation::status_of(w, &e) {
        OpStatus::Ok => chip(ChipTone::Ok, "Up to date", ""),
        OpStatus::Stale => chip(
            ChipTone::Attention,
            "Needs regenerate",
            operation::stale_reason(w, &e).unwrap_or_default(),
        ),
        OpStatus::Never => chip(ChipTone::Attention, "Not generated", ""),
    })
}

/// The state line for the chip of a folded pane that shows `tab`.
pub(crate) fn of(state: &State, tab: &Tab) -> Option<ChipState> {
    match tab {
        Tab::Browser => browser_state(state),
        Tab::Properties => properties_state(state),
        _ => None,
    }
}
