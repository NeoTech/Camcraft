//! #1953: one small, named layout test for each page and window, so a UI
//! refit PR that moves one page updates that page's test in the same PR,
//! and a failure names the page. Behaviour (labels, roles, enabled state,
//! commands) is asserted elsewhere and holds without these positions.
//!
//! Each page test asserts the coarse law only: the page's anchor widgets
//! are in the Properties leaf and stack top to bottom in the given order.
//! The pages that also have a detailed layout test keep it:
//! - Stock: `stock_size::the_stock_page_is_laid_out_in_groups_like_the_wcs_page`
//! - setup Machine / Tool list / Settings / Fixtures: `machine_page`,
//!   `tool_list_page`, `setup_settings_page`, `fixture_rows`
//! - tool editor window: `structural::tool_editor_structure`,
//!   `structural::tool_cutter_params_structure`
//! - machine editor window: `structural::wire_machine_editor_structure`
//! - G-code window: `gcode_viewer`
//! - dock panes: `layout`
//!
//! Provenance: our own page law over the egui_kittest AccessKit tree; no
//! HSMWorks material involved.

use egui_kittest::kittest::NodeT;

use super::structural::{at, at_where, painted};
use super::*;
use crate::state::SetupPage;

type H = Harness<'static, ShellUi>;

/// The seeded shell with one setup (`Setup1`) and `t` targeted.
fn page(t: UiTarget) -> H {
    let mut h = shell_harness();
    h.state_mut().state.pending.push_back(CommandKind::CreateSetup { name: None, datum_from: None });
    h.run_steps(2);
    crate::actions::target(&mut h.state_mut().state, t);
    h.run_steps(3);
    h
}

/// The page law: each anchor is drawn, lies across the Properties leaf's
/// width, starts inside its height, and the anchors stack top to bottom.
fn assert_page_layout(h: &H, page: &str, anchors: &[&str]) {
    let Some(leaf) = super::layout::leaf_rect(&h.state().dock, &Tab::Properties) else {
        panic!("{page} page: the Properties leaf is not laid out");
    };
    let mut last: Option<(&str, f32)> = None;
    for a in anchors {
        let Some(node) = h.query_by_label(a) else {
            panic!("{page} page: no widget labelled {a:?}");
        };
        let r = node.rect();
        assert!(
            r.left() >= leaf.left() - 1.0 && r.right() <= leaf.right() + 1.0,
            "{page} page: {a:?} {r:?} leaves the Properties leaf {leaf:?} across"
        );
        assert!(
            r.top() >= leaf.top() - 1.0 && r.top() < leaf.bottom(),
            "{page} page: {a:?} {r:?} starts outside the Properties leaf {leaf:?}"
        );
        if let Some((prev, y)) = last {
            assert!(r.top() > y, "{page} page: {a:?} (top {}) must be under {prev:?} (top {y})", r.top());
        }
        last = Some((a, r.top()));
    }
}

/// Where the seeded shell's parts sit: the ribbon above the dock, the
/// three leaves in Browser | Viewport | Properties order on one row, the
/// seed's rows in the browser, the cube in the viewport, the page in the
/// right leaf and the status strip at the bottom. Moved here from the
/// structural skeleton (#1953).
pub(super) fn assert_shell_layout(h: &H) {
    let texts = painted(h);
    let tab_y = at(&texts, "Browser").y;
    for t in ["New", "Open", "Save", "Post", "Stock", "Origin", "Undo", "Redo", "Simulate"] {
        assert!(at(&texts, t).y < tab_y, "shell: ribbon text {t:?} must sit above the dock tabs");
    }
    let ver = crate::build_info::toolbar_version();
    let build = texts.iter().find(|(t, _)| t.starts_with(&ver)).map(|(_, p)| *p);
    assert!(build.is_some_and(|b| b.y < tab_y), "shell: the build identity sits on the ribbon");
    let (b, v, p) = (at(&texts, "Browser"), at(&texts, "Viewport"), at(&texts, "Properties"));
    assert!(b.x < v.x && v.x < p.x, "shell: dock order must be Browser | Viewport | Properties");
    assert!((b.y - v.y).abs() < 1.0 && (v.y - p.y).abs() < 1.0, "shell: the tab titles share a row");
    let in_browser = |q: egui::Pos2| q.y > b.y && q.x < v.x;
    for row in ["Setups", "SETUP", "Stock", "Fixtures", "MODELS", "Block", "TOOLS"] {
        at_where(&texts, row, in_browser);
    }
    assert!(
        texts.iter().any(|(t, q)| t.starts_with("Contour1") && in_browser(*q)),
        "shell: the seed's operation row is in the browser"
    );
    for t in ["TOP", "RIGHT", "BACK"] {
        let q = at(&texts, t);
        assert!(q.x > v.x && q.x < p.x, "shell: view cube face {t:?} belongs to the viewport");
    }
    assert!(h.get_by_label("Operation: Contour1").rect().min.x > v.x, "shell: the page is the right leaf");
    // The status strip is the lowest painted row: under the dock, and the
    // Debug info door on it.
    let status = h.state().state.status.clone();
    let s = at(&texts, &status);
    let props = super::layout::leaf_rect(&h.state().dock, &Tab::Properties);
    let Some(props) = props else { panic!("shell: the Properties leaf is not laid out") };
    assert!(s.y >= props.bottom() - 1.0, "shell: the status line {s:?} sits under the dock {props:?}");
    let door = h.get_by_label("Debug info").rect();
    assert!(door.top() >= props.bottom() - 1.0, "shell: Debug info {door:?} is on the status strip");
}

#[test]
fn layout_shell() {
    let _g = snapshot_guard();
    let mut h = shell_harness();
    h.run_steps(3);
    assert_shell_layout(&h);
}

#[test]
fn layout_shell_light() {
    let _g = snapshot_guard();
    let mut h = shell_harness();
    theme_set(&h.state_mut().state.world, &ThemeRef::Named("light".to_owned()));
    h.run_steps(3);
    let laid = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| assert_shell_layout(&h)));
    theme_set(&h.state_mut().state.world, &ThemeRef::Dark);
    h.run_steps(1);
    if let Err(e) = laid {
        std::panic::resume_unwind(e);
    }
}

#[test]
fn layout_page_stock() {
    let _g = snapshot_guard();
    let h = page(UiTarget::Stock);
    assert_page_layout(&h, "Stock", &["Shape", "Size", "Position and rotation", "Fit to models", "Apply"]);
}

#[test]
fn layout_page_wcs() {
    let _g = snapshot_guard();
    let h = page(UiTarget::Wcs);
    assert_page_layout(&h, "WCS", &["Turn X+", "Turn X-", "Level", "Bottom", "Pick markers in viewport", "NW", "SW"]);
}

#[test]
fn layout_page_model() {
    let _g = snapshot_guard();
    let h = page(UiTarget::Model("Block".into()));
    assert_page_layout(&h, "Model", &["Model: Block", "STEP facet budget (next import)", "Turn X+", "Turn Y-", "Apply"]);
}

#[test]
fn layout_page_tool_library() {
    let _g = snapshot_guard();
    let h = page(UiTarget::Tool(String::new()));
    assert_page_layout(&h, "Tool library", &["New tool…", "Import library…", "Import tool holders"]);
}

#[test]
fn layout_page_tool() {
    let _g = snapshot_guard();
    let h = page(UiTarget::Tool("EM-0600-4FL".into()));
    // Selected tools render their settings directly in Properties; catalog
    // controls live in the Tool library tab (#2002).
    assert_page_layout(
        &h,
        "Tool",
        &["Tool: EM-0600-4FL", "Type", "Diameter", "Corner radius", "Flute length"],
    );
}

#[test]
fn layout_page_operation() {
    let _g = snapshot_guard();
    let h = page(UiTarget::Operation("Contour1".into()));
    assert_page_layout(
        &h,
        "Operation",
        &["Operation: Contour1", "Geometry", "Lead in / Lead out", "Models", "Edges", "Generate"],
    );
}

#[test]
fn layout_page_setups() {
    let _g = snapshot_guard();
    let h = page(UiTarget::Setups);
    assert_page_layout(&h, "Setups", &["Setup1", "Carry stock and WCS over", "Export setup sheets…"]);
}

#[test]
fn layout_page_setup_settings() {
    let _g = snapshot_guard();
    let h = page(UiTarget::SetupNode("Setup1".into(), SetupPage::Settings));
    assert_page_layout(&h, "Setup settings", &["Its own stock (the Stock node)", "+Y by Edge", "+X by Edge", "Place origin"]);
}

#[test]
fn layout_page_setup_machine() {
    let _g = snapshot_guard();
    let h = page(UiTarget::SetupNode("Setup1".into(), SetupPage::Machine));
    assert_page_layout(&h, "Setup machine", &["This setup has its own machine", "Machine library", "Load machine (.mch)"]);
}

#[test]
fn layout_page_setup_post() {
    let _g = snapshot_guard();
    let h = page(UiTarget::SetupNode("Setup1".into(), SetupPage::Post));
    assert_page_layout(
        &h,
        "Setup post",
        &["This setup has its own post", "Load post (.cps)", "Skip pauses and homes", "This setup has its own post options"],
    );
}

#[test]
fn layout_page_setup_fixtures() {
    let _g = snapshot_guard();
    let h = page(UiTarget::SetupNode("Setup1".into(), SetupPage::Fixtures));
    assert_page_layout(&h, "Setup fixtures", &["Import fixture…", "Add built-in vise…", "Add accessory…", "Add fixture plate…"]);
}

#[test]
fn layout_page_setup_tools() {
    let _g = snapshot_guard();
    let h = page(UiTarget::SetupNode("Setup1".into(), SetupPage::Tools));
    assert_page_layout(&h, "Setup tool list", &["Tools in this setup", "Export tool list…", "Export setup sheets…"]);
}

/// The Debug info window: inside the screen, its Copy button inside it.
#[test]
fn layout_window_debug_info() {
    let _g = snapshot_guard();
    let mut h = shell_harness();
    h.state_mut().state.debug_info_open = true;
    h.run_steps(3);
    let screen = h.ctx.content_rect();
    let Some(win) = h.query_all_by_role(egui::accesskit::Role::Window).find(|n| n.accesskit_node().label().as_deref() == Some("Debug info")) else {
        panic!("Debug info window: not open");
    };
    let wr = win.rect();
    assert!(screen.contains_rect(wr), "Debug info window: {wr:?} leaves the screen {screen:?}");
    let copy = win.get_by_label("Copy").rect();
    assert!(wr.contains_rect(copy), "Debug info window: Copy {copy:?} leaves the window {wr:?}");
}
