//! #4496, its "Done means": a user holder with a library name and a wider
//! body under the flange envelope gets a gauge length edit in the generic
//! component editor. The edit is refused by name and nothing changes: the
//! library body, found by the name, is never drawn over the wider steel.
//! A holder whose outline is the built-in holder's, with its body in
//! view, still takes the edit as one undo step.
//! Gate map: covers ui::egui_app::tool_library_dialog, ui::state.
//! Provenance: our own structural tests over the AccessKit tree and the
//! command history; no HSMWorks material involved.

use camcraft_app::commands::UpsertDocumentComponent;
use camcraft_app::component::{ADAPTIVE_PORT, ComponentKind, ToolComponent};
use camcraft_app::component_geometry::ComponentGeometry;
use camcraft_app::connection_code::ConnectionCode;
use camcraft_app::holder_library::{self, BT40, HolderShape, ShapeClamp};
use camcraft_app::tool_assembly as ta;
use egui::accesskit::Role;

use super::tool_library_components::{done, lib_dir, shell, undo};
use super::tool_library_editor::{H, click_window_button};
use super::*;
use crate::egui_app::tool_library_dialog::{HOLDERS_PAGE, component_edit};
use crate::state::ErrorScope;

/// Lands user holder `id` named `name` with `kind` and `geometry`, opens
/// it in the component editor and gives its gauge length.
fn open_holder(h: &mut H, id: &str, name: &str, kind: ComponentKind, geometry: ComponentGeometry) -> f64 {
    let c = ToolComponent::new(id, name, kind).expect("holder");
    let st = &mut h.state_mut().state;
    let cmd = UpsertDocumentComponent::new(&st.world, c, geometry).expect("upsert");
    st.history.execute(&st.world, cmd);
    st.doc_version += 1;
    h.run_steps(2);
    component_edit::open(&mut h.state_mut().state, id);
    h.run_steps(2);
    let d = h.state().state.tool_library.as_ref().and_then(|v| v.component_edit.clone());
    let d = d.unwrap_or_else(|| panic!("{id}: the editor opens"));
    assert_eq!(d.shape, None, "{id}: the generic editor, not the holder form");
    d.gauge.unwrap_or_else(|| panic!("{id}: the editor offers the gauge length"))
}

fn code(text: &str) -> ConnectionCode {
    ConnectionCode::parse(text).unwrap_or_else(|e| panic!("connection code {text}: {e}"))
}

fn set_gauge(h: &mut H, g: f64) {
    if let Some(d) = h.state_mut().state.tool_library.as_mut().and_then(|v| v.component_edit.as_mut()) {
        d.gauge = Some(g);
    }
    h.run_steps(2);
}

fn row_geometry(h: &H, id: &str) -> Option<ComponentGeometry> {
    ta::component(&h.state().state.world, id).map(|r| r.geometry)
}

#[test]
fn a_gauge_edit_never_draws_the_library_body_over_wider_steel() {
    let dir = lib_dir("4496-gauge");
    let mut h = shell(&dir);
    h.get_by_role_and_label(Role::Button, "Tools").click();
    h.run_steps(2);
    click_window_button(&mut h, HOLDERS_PAGE);
    let lib = holder_library::find("BT40 Shrink 20 mm").expect("BT40 Shrink 20 mm");
    // A Ø50 body under the BT40 flange envelope: the outline is the
    // library chuck's, the steel is wider than its Ø42 body.
    let id = "user:holder:BT40 Shrink 20 mm";
    let look_alike = HolderShape { taper: BT40, clamp: ShapeClamp::Shrink, bore: 20.0, gauge_len: lib.gauge_len, nose_d: 33.0, largest_d: 50.0 };
    let (kind, geometry) = (look_alike.kind().expect("codes"), look_alike.geometry(ADAPTIVE_PORT).expect("draws"));
    let g0 = open_holder(&mut h, id, "BT40 Shrink 20 mm", kind, geometry);
    assert!((g0 - lib.gauge_len).abs() < 1e-9, "the gauge length {g0}");
    let before = row_geometry(&h, id);
    let n0 = done(&h);
    set_gauge(&mut h, g0 + 20.0);
    click_window_button(&mut h, component_edit::APPLY);
    assert_eq!(done(&h), n0, "a refused edit is no undo step");
    assert_eq!(row_geometry(&h, id), before, "the outline stays as it was");
    let msg = h.state().state.error_for(ErrorScope::Tool).map(str::to_owned).unwrap_or_default();
    assert!(msg.contains("component not changed") && msg.contains("the flange envelope covers the body"), "refused by name: {msg}");

    // A standalone ER20 chuck copy shows its body: the edit lands as one
    // undo step, at the library outline of the new length.
    let er = holder_library::find("ER20 chuck").expect("ER20 chuck");
    let id = "user:holder:ER20 chuck";
    let geometry = ComponentGeometry::holder(&er, ADAPTIVE_PORT, er.gauge_len).expect("draws");
    let kind = ComponentKind::AdaptiveItem { mcs: code("CYL 25"), csw: code("ER20") };
    let g0 = open_holder(&mut h, id, "ER20 chuck", kind, geometry.clone());
    assert!((g0 - er.gauge_len).abs() < 1e-9, "the gauge length {g0}");
    let n0 = done(&h);
    set_gauge(&mut h, er.gauge_len + 10.0);
    click_window_button(&mut h, component_edit::APPLY);
    assert_eq!(done(&h), n0 + 1, "the gauge edit is one undo step");
    let want = ComponentGeometry::holder(&er, ADAPTIVE_PORT, er.gauge_len + 10.0).expect("draws");
    assert_eq!(row_geometry(&h, id), Some(want), "the library outline at the new length");
    undo(&mut h);
    assert_eq!(row_geometry(&h, id), Some(geometry), "undo gives the old outline back");
}
