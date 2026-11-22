//! The "Pass starts at" row (#3316): one Passes-tab row for every kind
//! that has a pass order. It edits the page draft; Apply sends the
//! undoable `EditOperationParams`. Provenance: our own page code; no
//! vendor material consulted.

use egui::Ui;

use camcraft_app::machining2d::PassStart;

use super::fields::*;

/// The row's label - one spelling for every kind.
pub(crate) const PASS_START_LABEL: &str = "Pass starts at";

/// The consequence line under the row when the 2D Adaptive method is
/// Trochoidal and the row is off Auto (#3355): its loops have no ring
/// order, so the generator refuses the operation by name.
pub(crate) const TROCH_PASS_START_REFUSED: &str =
    "Inside and Outside are not available with the Trochoidal method (its loops have no ring order): the operation is refused";

/// What the three values do: the row's hover text.
const PASS_START_HINT: &str = "Auto: the order this operation always cut. Inside: start at the part or island wall and work out. Outside: start at the outer edge and work in. Each ring follows the edge it starts from, one stepover at a time. With an Outside containment the outer edge is the stock edge, so each level starts in air. Where the first ring runs along a wall, or the area is narrower than the tool plus one stepover, a cut takes the full tool width and the toolpath shows a warning";

/// What the values mean on a Bore (#3360): a hole has no inner wall.
pub(crate) const BORE_PASS_START_HINT: &str = "Bore: a hole has no inner wall, so Inside cuts from the centre out, as Auto does. Outside cuts the wall ring first and then works in to the centre";

/// Why Face has no pass order (#3356): the line under its disabled row.
pub(crate) const FACE_NO_PASS_ORDER: &str =
    "Not used on Face: its rows run edge to edge across the face, so there is no inside or outside order";

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

/// The row for a kind with no pass order (#3356): the same label and
/// combo, held at Auto and disabled, with `why` in view under it.
pub(super) fn no_pass_order_row(ui: &mut Ui, tag: &str, why: &str) {
    let names = pass_start_names();
    let mut idx = 0;
    ui.add_enabled_ui(false, |ui| {
        prop_row(ui, PASS_START_LABEL, |ui| combo(ui, &format!("{tag}.pass_start"), &mut idx, &names));
    });
    consequence(ui, why);
}
