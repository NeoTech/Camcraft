//! #3892 (U1 of #3663): the named layout checks of the tool library
//! window. `page_layout.rs` is full, so the window's tests live here
//! (#1953: one named layout test for each page and window).
//! Gate map: covers ui::egui_app::tool_library_dialog, ui::actions, ui::state.
//! Provenance: our own layout law over the egui_kittest AccessKit tree; no
//! HSMWorks material involved.

use super::page_layout_windows::assert_window_layout;
use super::*;
use egui_kittest::kittest::NodeT;

type H = Harness<'static, ShellUi>;

fn library_window(h: &H) -> egui::Rect {
    h.query_all_by_role(egui::accesskit::Role::Window)
        .find(|n| n.accesskit_node().label().as_deref() == Some("Tool library"))
        .map(|n| n.rect())
        .unwrap_or_else(|| panic!("the Tool library window is not open"))
}

/// The tool library is its own window: the file buttons first, then the
/// source row, then the Document list. It floats beside the dock and
/// leaves the Properties pane free, so both can be used at once.
#[test]
fn layout_window_tool_library() {
    let _g = snapshot_guard();
    let mut h = shell_harness();
    h.run_steps(2);
    h.get_by_role_and_label(egui::accesskit::Role::Button, "Tools").click();
    h.run_steps(3);
    assert_window_layout(
        &h,
        "Tool library",
        &["New tool…", "Import tool holders", "Document", "Recipes"],
    );
    let win = library_window(&h);
    let Some(props) = super::layout::leaf_rect(&h.state().dock, &Tab::Properties) else {
        panic!("the Properties leaf is not laid out");
    };
    assert!(
        !win.intersects(props),
        "the window {win:?} stands beside the Properties pane {props:?}, not over it"
    );
}

/// A closed window (its ✕ clears `state.tool_library`) opens again from
/// the ribbon; the dock never gets a Tool library tab.
#[test]
fn the_tool_library_window_closes_and_opens_again() {
    let mut h = shell_harness();
    h.run_steps(2);
    h.get_by_role_and_label(egui::accesskit::Role::Button, "Tools").click();
    h.run_steps(3);
    let _ = library_window(&h);
    assert!(!crate::egui_app::layout::is_open(&h.state().dock, &Tab::Tools), "no dock tab");
    h.state_mut().state.tool_library = None;
    h.run_steps(2);
    assert!(
        !h.query_all_by_role(egui::accesskit::Role::Window)
            .any(|n| n.accesskit_node().label().as_deref() == Some("Tool library")),
        "a closed window is gone"
    );
    h.get_by_role_and_label(egui::accesskit::Role::Button, "Tools").click();
    h.run_steps(3);
    let _ = library_window(&h);
}

/// #3893: a tool open in the window. The list folds into a rail on the
/// left; the editor stands right of it: the heading, the preview's 3D /
/// Side toggle (#3894), the Tool and Details tabs, the rows, then Apply at
/// the foot. The window still leaves the
/// Properties pane free. Queries stay inside the window, because a tool
/// can be open in Properties too.
#[test]
fn layout_window_tool_editor() {
    let _g = snapshot_guard();
    let mut h = shell_harness();
    h.run_steps(2);
    h.get_by_role_and_label(egui::accesskit::Role::Button, "Tools").click();
    h.run_steps(2);
    let Some(name) = h.state().state.tool_names().into_iter().next() else {
        panic!("the seeded document has a tool");
    };
    h.state_mut().state.open_tool_in_library(&name);
    h.run_steps(3);
    let win = library_window(&h);
    let screen = h.ctx.content_rect();
    assert!(screen.contains_rect(win), "the window {win:?} stays on the screen {screen:?}");
    let node = h
        .query_all_by_role(egui::accesskit::Role::Window)
        .find(|n| n.accesskit_node().label().as_deref() == Some("Tool library"))
        .unwrap_or_else(|| panic!("the Tool library window is not open"));
    let rect = |label: &str| {
        node.query_all_by_label(label)
            .next()
            .unwrap_or_else(|| panic!("the editor has no widget labelled {label:?}"))
            .rect()
    };
    let heading = format!("Tool: {name}");
    let mut last: Option<(&str, f32)> = None;
    for a in [heading.as_str(), "3D", "Details", "Diameter", "Apply"] {
        let r = rect(a);
        assert!(win.expand(1.0).contains_rect(r), "{a:?} {r:?} leaves the window {win:?}");
        if let Some((prev, y)) = last {
            assert!(r.top() > y, "{a:?} (top {}) must be under {prev:?} (top {y})", r.top());
        }
        last = Some((a, r.top()));
    }
    let rail = rect("All tools");
    let row = rect(&name);
    assert!(rail.right() < rect("Diameter").left() && row.right() < rect(&heading).left(), "the rail {rail:?} stands left of the editor");
    assert!(node.query_all_by_label("Import library…").next().is_none(), "the file row folds away with the list");
    let Some(props) = super::layout::leaf_rect(&h.state().dock, &Tab::Properties) else {
        panic!("the Properties leaf is not laid out");
    };
    assert!(!win.intersects(props), "the window {win:?} stands beside the Properties pane {props:?}");
}
