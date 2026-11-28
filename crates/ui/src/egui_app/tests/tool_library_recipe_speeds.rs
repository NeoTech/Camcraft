//! #4453: the Recipes tab's surface speed and chip load are inputs. A
//! chip load edit sets the row's feed, a surface speed edit sets its rpm,
//! the derived value follows the stored one, and the edits land through
//! the editor's one Apply as one undo step. A missing flute count or a
//! zero diameter disables the input.
//! Gate map: covers ui::egui_app::tool_library_dialog, ui::state.
//! Provenance: our own structural tests over the AccessKit tree and the
//! command history; no HSMWorks material involved.

use super::tool_library_editor::{H, click_window_button, draft, library_window, open_in_window, tool};
use super::tool_library_recipes::mill;
use super::*;
use crate::egui_app::tool_library_dialog::recipes_speeds::{FZ, VC};
use crate::egui_app::tool_library_dialog::recipes_tab::TAB;
use camcraft_app::tool_cutting_rows::CuttingRow;
use camcraft_app::tools::WorkMaterial;
use egui::accesskit::Role;
use egui_kittest::kittest::NodeT;

fn field_name(what: &str) -> String {
    format!("{what} ({})", WorkMaterial::Brass.label())
}

/// The value of the Brass row's `what` field.
fn value(h: &H, what: &str) -> f64 {
    library_window(h).get_by_role_and_label(Role::SpinButton, &field_name(what)).accesskit_node().numeric_value().unwrap_or(f64::NAN)
}

fn disabled(h: &H, what: &str) -> bool {
    library_window(h).get_by_role_and_label(Role::SpinButton, &field_name(what)).accesskit_node().is_disabled()
}

/// Drag the Brass row's `what` field 40 px to the right. A press on a
/// disabled field drags the window, which repaints: step, do not `run`.
fn drag(h: &mut H, what: &str) {
    let at = library_window(h).get_by_role_and_label(Role::SpinButton, &field_name(what)).rect().center();
    h.drag_at(at);
    h.run_steps(1);
    h.hover_at(at + egui::Vec2::new(40.0, 0.0));
    h.run_steps(1);
    h.drop_at(at + egui::Vec2::new(40.0, 0.0));
    h.run_steps(2);
}

fn brass(h: &mut H) -> CuttingRow {
    match draft(h).cutting.iter().find(|r| r.material == Some(WorkMaterial::Brass)) {
        Some(r) => *r,
        None => panic!("the draft has the Brass row"),
    }
}

fn open(h: &mut H) -> String {
    h.run_steps(2);
    let name = mill(h, vec![CuttingRow::plain(Some(WorkMaterial::Brass), 8000.0, 1200.0)]);
    open_in_window(h, &name);
    click_window_button(h, TAB);
    name
}

/// A chip load edit sets the feed (F = fz x z x n) and keeps the rpm; a
/// surface speed edit sets the rpm (n = Vc x 1000 / (pi x D)) and keeps
/// the feed. Apply lands both as one undo step; one undo restores the tool.
#[test]
fn an_fz_or_vc_edit_sets_the_stored_feed_or_rpm_and_is_one_undo_step() {
    let mut h = shell_harness();
    let name = open(&mut h);
    let before = tool(&h, &name);
    let done = h.state().state.history.done_len();
    let z = f64::from(draft(&mut h).flutes.unwrap_or_default());
    let d = draft(&mut h).diameter;
    assert!(z > 0.0 && d > 0.0, "the mill has flutes and a diameter");

    let fz0 = value(&h, FZ);
    drag(&mut h, FZ);
    let fz1 = value(&h, FZ);
    let row = brass(&mut h);
    assert!((fz1 - fz0).abs() > 1e-6, "the drag changes the chip load: {fz0} -> {fz1}");
    assert_eq!(row.rpm, 8000.0, "a chip load edit keeps the rpm");
    assert!((row.feed - fz1 * z * 8000.0).abs() < 1e-6, "F = fz x z x n: {} for fz {fz1}", row.feed);

    let vc0 = value(&h, VC);
    drag(&mut h, VC);
    let vc1 = value(&h, VC);
    let row = brass(&mut h);
    assert!((vc1 - vc0).abs() > 1e-6, "the drag changes the surface speed: {vc0} -> {vc1}");
    if h.state().state.units() == camcraft_app::units::Unit::Millimeters {
        let want = vc1 * 1000.0 / (std::f64::consts::PI * d);
        assert!((row.rpm - want).abs() < 1e-6, "n = Vc x 1000 / (pi x D): {} for Vc {vc1}", row.rpm);
    }
    assert!((row.feed - fz1 * z * 8000.0).abs() < 1e-6, "a surface speed edit keeps the feed");
    assert_eq!(h.state().state.history.done_len(), done, "the edits queue no command");
    assert_eq!(tool(&h, &name), before, "the edits write the draft, not the document");

    click_window_button(&mut h, "Apply");
    assert_eq!(h.state().state.history.done_len(), done + 1, "one Apply is one undo step");
    let landed = tool(&h, &name).cutting.into_iter().find(|r| r.material == Some(WorkMaterial::Brass));
    assert_eq!(landed.map(|r| (r.rpm, r.feed)), Some((row.rpm, row.feed)), "Apply lands the row: {}", h.state().state.status);
    crate::actions::undo(&mut h.state_mut().state);
    h.run_steps(2);
    assert_eq!(tool(&h, &name), before, "one undo restores the tool");
}

/// A tool with no flute count refuses the chip load input; a zero
/// diameter refuses the surface speed input. The stored numbers stay.
#[test]
fn a_missing_flute_count_or_a_zero_diameter_refuses_the_input() {
    let mut h = shell_harness();
    open(&mut h);
    assert!(!disabled(&h, FZ) && !disabled(&h, VC), "both inputs take a value");

    draft(&mut h).flutes = None;
    h.run_steps(2);
    assert!(disabled(&h, FZ), "no flute count: the chip load input is refused");
    assert!(!disabled(&h, VC), "the surface speed needs no flute count");
    let row = brass(&mut h);
    drag(&mut h, FZ);
    assert_eq!(brass(&mut h), row, "a refused input changes nothing");

    draft(&mut h).diameter = 0.0;
    h.run_steps(2);
    assert!(disabled(&h, VC), "zero diameter: the surface speed input is refused");
    drag(&mut h, VC);
    assert_eq!(brass(&mut h), row, "a refused input changes nothing");
}
