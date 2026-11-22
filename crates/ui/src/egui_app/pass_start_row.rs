//! The "Pass starts at" row (#3316): one Passes-tab row for every kind
//! that has a pass order. It edits the page draft; Apply sends the
//! undoable `EditOperationParams`. Provenance: our own page code; no
//! vendor material consulted.

use egui::Ui;

use camcraft_app::machining2d::PassStart;

use super::fields::*;

/// The row's label - one spelling for every kind.
pub(crate) const PASS_START_LABEL: &str = "Pass starts at";

/// What the three values do: the row's hover text.
const PASS_START_HINT: &str = "Auto: the order this operation always cut. Inside: start at the part or island wall and work out. Outside: start at the outer edge and work in. Inside and Outside step one stepover at a time from the start side. A start on a wall (Inside at the part, Outside in a closed pocket) cuts its first pass at full width; the toolpath notes say so. With an Outside containment the outer edge is the stock edge, so the tool starts in air";

/// The names of the combo, in its order (`PassStart::ALL`).
pub(crate) fn pass_start_names() -> Vec<&'static str> {
    PassStart::ALL.iter().map(|s| s.label()).collect()
}

/// The row itself; `tag` scopes the combo id per kind.
pub(super) fn pass_start_row(ui: &mut Ui, tag: &str, start: &mut PassStart) {
    let names = pass_start_names();
    let before = PassStart::ALL.iter().position(|s| s == start).unwrap_or(0);
    let mut idx = before;
    prop_row(ui, PASS_START_LABEL, |ui| combo(ui, &format!("{tag}.pass_start"), &mut idx, &names));
    if idx != before
        && let Some(s) = PassStart::ALL.get(idx)
    {
        *start = *s;
    }
    hint(ui, PASS_START_HINT);
}
