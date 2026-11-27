//! #4455, its "Done means": a copy of a built-in holder (an ER chuck, a
//! Weldon chuck and a shrink-fit chuck, each on a flange) opens in the
//! holder form with the largest diameter of its outline - the drawing's
//! value - and the body steel as its own field; Apply with no edit keeps
//! the outline byte for byte.
//! Gate map: covers ui::egui_app::tool_library_dialog, ui::state.
//! Provenance: our own structural tests over the AccessKit tree and the
//! command history; no HSMWorks material involved.

use camcraft_app::component_geometry::ComponentGeometry;
use camcraft_app::tool_assembly as ta;
use egui::accesskit::Role;

use super::tool_library_components::{done, lib_dir, shell, shows};
use super::tool_library_editor::{H, click_window_button};
use super::*;
use crate::egui_app::tool_library_dialog::{HOLDERS_PAGE, frame, holder_form, holders};

/// Twice the largest radius of the outline.
fn outline_d(g: &ComponentGeometry) -> f64 {
    g.body().map_or(0.0, |b| b.profile().points().iter().map(|p| p.r).fold(0.0, f64::max)) * 2.0
}

/// Copies built-in holder `name`; gives the copy's id.
fn copy(h: &mut H, name: &str) -> String {
    h.state_mut().state.open_holder_in_library(Some(name));
    h.run_steps(2);
    let n0 = done(h);
    click_window_button(h, holders::COPY);
    assert_eq!(done(h), n0 + 1, "{name}: the copy is one undo step");
    let id = h.state().state.tool_library.as_ref().and_then(|v| v.component_edit.as_ref()).map(|d| d.id.clone());
    id.unwrap_or_else(|| panic!("{name}: the copy opens in the editor"))
}

#[test]
fn a_copied_holder_shows_the_largest_diameter_of_its_outline_and_applies_as_it_is() {
    let dir = lib_dir("4455-largest");
    let mut h = shell(&dir);
    h.get_by_role_and_label(Role::Button, "Tools").click();
    h.run_steps(2);
    click_window_button(&mut h, HOLDERS_PAGE);
    // (built-in, its body steel): an ER32 nut Ø50, a Weldon body, the
    // DIN 69882-8 Ø42 body of a shrink chuck that its flange envelope covers.
    for (name, body) in [("BT40 ER32", Some(50.0)), ("BT40 Weldon 12 mm", None), ("BT40 Shrink 20 mm", Some(42.0))] {
        let id = copy(&mut h, name);
        assert!(shows(&h, holder_form::BODY) && shows(&h, holder_form::LARGEST), "{name}: the form opens on the copy");
        let shape = h.state().state.tool_library.as_ref().and_then(|v| v.component_edit.as_ref()).and_then(|d| d.shape);
        let shape = shape.unwrap_or_else(|| panic!("{name}: the copy is a form shape"));
        let g0 = ta::component(&h.state().state.world, &id).map(|r| r.geometry).unwrap_or_else(|| panic!("{name}: the row"));
        let largest = holder_form::largest(&shape).unwrap_or_else(|| panic!("{name}: the values draw"));
        assert!((largest - outline_d(&g0)).abs() < 1e-9, "{name}: the form's largest diameter {largest} is the outline's {}", outline_d(&g0));
        assert!(shows(&h, &holder_form::largest_text(largest)), "{name}: the form shows {largest}");
        if let Some(body) = body {
            assert!((shape.largest_d - body).abs() < 1e-9, "{name}: the body field is the steel, not {}", shape.largest_d);
        }
        if name == "BT40 ER32" {
            assert_eq!(holder_form::largest_text(largest), "63.00", "the BT40 flange, as the drawing states it");
        }
        // Apply with no edit: no undo step, the outline byte for byte.
        assert_eq!(holder_form::refusal(&mut h.state_mut().state), None, "{name}: nothing to refuse");
        let n0 = done(&h);
        click_window_button(&mut h, frame::APPLY);
        assert_eq!(done(&h), n0, "{name}: an Apply with no edit is no undo step");
        let g1 = ta::component(&h.state().state.world, &id).map(|r| r.geometry);
        assert_eq!(g1, Some(g0), "{name}: the outline stays byte for byte");
        click_window_button(&mut h, frame::BACK);
    }
}
