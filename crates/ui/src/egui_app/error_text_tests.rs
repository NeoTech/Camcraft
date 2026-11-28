//! The error line's copy affordance (moved out of `error_text.rs`, #1120:
//! a test colour is not a draw-site colour literal).
//! Provenance: our own tests.

use super::*;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

fn frame(msg: &'static str) -> Harness<'static> {
    let mut h = Harness::new_ui(move |ui| {
        error_text(ui, Color32::RED, msg);
    });
    h.run();
    h
}

#[test]
fn long_or_multiline_error_gets_a_copy_button() {
    let h = frame("post refused: move 3 gouges\nmove 9 collides with the vise");
    assert!(h.query_by_label(COPY_LABEL).is_some(), "multi-line error shows Copy");
    let long = "x".repeat(COPY_BUTTON_MIN_CHARS + 1);
    assert!(wants_button(&long));
}

#[test]
fn short_single_line_error_has_no_button_but_is_text() {
    let h = frame("tool not found");
    assert!(h.query_by_label(COPY_LABEL).is_none());
    assert!(h.query_by_label("tool not found").is_some(), "the message is a labelled node");
}

#[test]
fn right_click_offers_copy_message() {
    let mut h = frame("tool not found");
    h.get_by_label("tool not found").click_secondary();
    h.run();
    assert!(h.query_by_label(COPY_MENU_LABEL).is_some(), "context menu carries Copy message");
}
