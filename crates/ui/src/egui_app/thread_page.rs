//! The Thread operation page's pitch and hand rows (#894): the thread's
//! own pitch - left to a multi-form mill's, or stated for a single-form
//! mill - and its hand, by name. The rows edit the page draft; Apply
//! rides `EditOperationParams`, which refuses a pitch the mounted mill
//! cannot cut, so the edit is gated and undoable.
//!
//! Provenance: ISO metric thread terms; our own page law. No vendor
//! material consulted.

use egui::Ui;

use camcraft_app::cl::ThreadHand;
use camcraft_app::units::Unit;

use super::fields::{check_row, hint, num, prop_row};

/// The Passes tab's thread rows: whether the pitch is stated, the pitch,
/// and the hand.
pub(super) fn thread_rows(ui: &mut Ui, u: Unit, pitch: &mut Option<f64>, hand: &mut ThreadHand) {
    let mut stated = pitch.is_some();
    check_row(ui, "State the pitch", &mut stated);
    match (stated, pitch.as_mut()) {
        (true, Some(p)) => {
            prop_row(ui, "Thread pitch", |ui| num(ui, u, p));
        }
        (true, None) => *pitch = Some(1.0),
        (false, _) => *pitch = None,
    }
    hint(ui, "unstated reads a multi-form thread mill's own pitch; a single-form mill needs the pitch stated, inside its range");
    // The row lays right to left: Left first reads "Right hand  Left hand".
    prop_row(ui, "Hand", |ui| {
        ui.radio_value(hand, ThreadHand::Left, "Left hand");
        ui.radio_value(hand, ThreadHand::Right, "Right hand");
    });
    hint(ui, "climb milling winds a right-hand thread up from the bottom of the hole, a left-hand one down from the top");
}
