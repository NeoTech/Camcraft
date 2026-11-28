//! #3892 (U1 of #3663): the named layout checks of the tool library
//! window. `page_layout.rs` is full, so the window's tests live here
//! (#1953: one named layout test for each page and window).
//! Gate map: covers ui::egui_app::tool_library_dialog, ui::actions, ui::state.
//! Provenance: our own layout law over the egui_kittest AccessKit tree; no
//! HSMWorks material involved.

use super::page_layout_windows::assert_window_layout;
use crate::egui_app::tool_library_dialog::recipes_actions::{ADD, COPY, SUGGEST};
use crate::egui_app::tool_library_dialog::recipes_tab::TAB;
use crate::egui_app::tool_library_dialog::{HOLDERS_PAGE, TOOLS_PAGE, categories, frame, holder_view, holders, preview};
use super::*;
use egui_kittest::kittest::NodeT;

type H = Harness<'static, ShellUi>;

fn library_window(h: &H) -> egui::Rect {
    h.query_all_by_role(egui::accesskit::Role::Window)
        .find(|n| n.accesskit_node().label().as_deref() == Some("Tool library"))
        .map(|n| n.rect())
        .unwrap_or_else(|| panic!("the Tool library window is not open"))
}

/// The tool library is its own window: (#3897) the Cutting tools | Holders
/// switch first, then (#4335) the one header row of the list - the source
/// tabs, the search box, New tool…, Import… and Export… - then "N of M
/// tools", the column heads and the rows with the categories on their
/// left. It floats beside the dock and leaves the Properties pane free, so
/// both can be used at once. #4334: the header row holds the title, the
/// tabs and the close button; the footer (the hint, then Close at the
/// right) stands under the rows.
#[test]
fn layout_window_tool_library() {
    use crate::egui_app::tool_library_dialog::list::{self, RECIPES};
    use crate::egui_app::tool_library_dialog::list_header::{CATALOG, DOCUMENT, EXPORT, IMPORT, NEW, SEARCH};
    let _g = snapshot_guard();
    let mut h = shell_harness();
    h.run_steps(2);
    h.get_by_role_and_label(egui::accesskit::Role::Button, "Tools").click();
    h.run_steps(3);
    let n = h.state().state.tool_names().len();
    let count = list::count_text(n, n);
    assert_window_layout(&h, "Tool library", &[TOOLS_PAGE, DOCUMENT, &count, RECIPES, frame::HINT_TOOLS]);
    let win = library_window(&h);
    // #3897: the switch heads the window; the categories stand left of the rows.
    let node = h
        .query_all_by_role(egui::accesskit::Role::Window)
        .find(|n| n.accesskit_node().label().as_deref() == Some("Tool library"))
        .unwrap_or_else(|| panic!("the Tool library window is not open"));
    let rect = |l: &str| node.query_all_by_label(l).next().unwrap_or_else(|| panic!("no {l:?} in the window")).rect();
    let (tools, holders) = (rect(TOOLS_PAGE), rect(HOLDERS_PAGE));
    assert!((tools.top() - holders.top()).abs() < 2.0 && tools.right() <= holders.left() + 1.0, "Cutting tools {tools:?} | Holders {holders:?}");
    let all = rect(categories::ALL);
    assert!(all.right() < rect(RECIPES).left(), "the categories {all:?} stand left of the rows");
    // #4335: one header row, from left to right.
    let row = [DOCUMENT, crate::egui_app::tool_library_dialog::components::SOURCE, CATALOG, SEARCH, NEW, IMPORT, EXPORT].map(rect);
    for pair in row.windows(2) {
        assert!(pair[0].right() <= pair[1].left() + 1.0, "the header row runs left to right: {pair:?}");
        assert!((pair[0].center().y - pair[1].center().y).abs() < 4.0, "one header row: {pair:?}");
    }
    assert!(win.expand(1.0).contains_rect(row[6]), "Export {:?} stays in the window {win:?}", row[6]);
    assert!(row[0].bottom() <= rect(&count).top() + 1.0 && rect(&count).bottom() <= rect(RECIPES).top() + 1.0, "the count stands between the header row and the column heads");
    // #4335: the column heads, from left to right.
    let heads: Vec<egui::Rect> = list::heads(h.state().state.units()).iter().filter(|t| !t.is_empty()).map(|t| rect(t)).collect();
    for pair in heads.windows(2) {
        assert!(pair[0].right() <= pair[1].left() + 1.0 && (pair[0].center().y - pair[1].center().y).abs() < 4.0, "the heads run left to right: {pair:?}");
    }
    // #4334: the header: the title, the tabs, the close button on one row.
    let title = node
        .query_all_by_label(crate::egui_app::tool_library_dialog::TITLE)
        .find(|n| n.accesskit_node().role() != egui::accesskit::Role::Window)
        .unwrap_or_else(|| panic!("the header names the window"))
        .rect();
    let x = rect(frame::CLOSE_WINDOW);
    assert!(title.right() <= tools.left() + 1.0 && (title.center().y - tools.center().y).abs() < 4.0, "the title {title:?} heads the tabs {tools:?}");
    assert!(x.left() > holders.right() && (x.center().y - holders.center().y).abs() < 4.0, "the close button {x:?} ends the header row");
    assert!(title.bottom() <= rect(NEW).top() + 1.0, "the header {title:?} stands above the page");
    // #4334: the footer under the rows: the hint at the left, Close at the right.
    let (hint, close) = (rect(frame::HINT_TOOLS), rect(frame::CLOSE));
    assert!(close.top() > rect(RECIPES).bottom(), "Close {close:?} stands under the rows");
    assert!(hint.right() < close.left() && (hint.center().y - close.center().y).abs() < 4.0, "the hint {hint:?} and Close {close:?} share the footer row");
    assert!(win.expand(1.0).contains_rect(close), "Close {close:?} stays in the window {win:?}");
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
/// left; the editor stands right of it: (#4334) the header with the way
/// back and the name, the Tool and Details tabs, the rows, then the footer
/// with Duplicate, Cancel and Apply (at the right). #4336: the Tool tab's
/// rows and groups in the mock-up's order. #3897: the preview (its Side /
/// 3D toggle and the Dimensions and Holder toggles) stands right of the
/// rows, under the heading. #3896: the Recipes tab stands between Tool and Details, and
/// its Suggest row stands under the tabs. The window still leaves the
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
    // #4334: the way back heads the window; the name follows it.
    let heading = frame::BACK.to_owned();
    let title = node
        .query_all_by_label(&name)
        .map(|n| n.rect())
        .min_by(|a, b| a.top().total_cmp(&b.top()))
        .unwrap_or_else(|| panic!("the window does not name {name}"));
    let back = rect(frame::BACK);
    assert!(title.left() > back.right() && (title.center().y - back.center().y).abs() < 4.0, "the name {title:?} follows the way back {back:?}");
    let (apply, cancel, dup) = (rect(frame::APPLY), rect(frame::CANCEL), rect(frame::DUPLICATE));
    assert!(dup.right() <= cancel.left() + 1.0 && cancel.right() <= apply.left() + 1.0, "Duplicate, Cancel, Apply from left to right");
    assert!((dup.center().y - apply.center().y).abs() < 4.0 && (cancel.center().y - apply.center().y).abs() < 4.0, "the three share the footer row");
    assert!(win.expand(1.0).contains_rect(apply), "Apply {apply:?} stays in the window {win:?}");
    let mut last: Option<(&str, f32)> = None;
    for a in [heading.as_str(), "Details", "Diameter", "Apply"] {
        let r = rect(a);
        assert!(win.expand(1.0).contains_rect(r), "{a:?} {r:?} leaves the window {win:?}");
        if let Some((prev, y)) = last {
            assert!(r.top() > y, "{a:?} (top {}) must be under {prev:?} (top {y})", r.top());
        }
        last = Some((a, r.top()));
    }
    // #3895: the assembly group follows the geometry rows in the Tool tab
    // (it may stand below the fold of the scroll area).
    let group = rect(crate::egui_app::tool_library_dialog::assembly_group::HEADING);
    assert!(group.top() > rect("Diameter").top(), "the assembly group {group:?} stands under the geometry rows");
    assert!(group.left() >= win.left() - 1.0 && group.right() <= win.right() + 1.0, "the group {group:?} keeps to the window's width");
    // #3930: the folded Body section follows the assembly group.
    let body = rect(crate::egui_app::tool_library_dialog::assembly_group::BODY);
    assert!(body.top() > group.top(), "the Body section {body:?} stands under the assembly group {group:?}");
    assert!(node.query_all_by_label("Import body…").next().is_none(), "the Body section starts folded");
    // #4336: Side (the default) comes before 3D, then the check boxes.
    // The Tool tab has a "Holder" and a "Flutes" row too: the toggles are
    // the check boxes of those names.
    let toggle = |l: &str| node.get_by_role_and_label(egui::accesskit::Role::CheckBox, l).rect();
    let (view, dims, holder) = (rect("Side"), toggle(preview::DIMS), toggle(preview::HOLDER));
    assert!(view.top() > rect(&heading).top(), "the preview {view:?} stands under the heading");
    assert!(view.left() > rect("Diameter").right(), "the preview {view:?} stands right of the rows");
    let three = rect("3D");
    assert!(three.left() > view.left() && dims.left() > three.left() && holder.left() > dims.left(), "Side, 3D, Dimensions, Holder on one row");
    // #4336: the Tool tab reads Name, Type, Tool number, then the CUTTER
    // group, the SHANK group and the assembly group, top to bottom.
    use crate::egui_app::tool_library_dialog::tool_tab::{CUTTER, NAME, SHANK};
    let mut prev: Option<(&str, f32)> = None;
    for l in [NAME, "Type", "Tool number", CUTTER, "Diameter", "Flute length", "Corner radius", "Flutes", "Helix angle", "Tool material", SHANK, "Shank diameter"] {
        let r = node
            .query_all_by_label(l)
            .map(|n| n.rect())
            .min_by(|a, b| a.left().total_cmp(&b.left()))
            .unwrap_or_else(|| panic!("the Tool tab has no {l:?}"));
        assert!(r.right() < view.left(), "{l:?} {r:?} stands in the rows, left of the preview");
        if let Some((p, y)) = prev {
            assert!(r.top() > y, "{l:?} (top {}) stands under {p:?} (top {y})", r.top());
        }
        prev = Some((l, r.top()));
    }
    // #4370: the Name row is a text field (Apply renames the tool).
    let name_field = node.query_all_by_role_and_label(egui::accesskit::Role::TextInput, NAME).next().map(|n| n.rect());
    assert!(name_field.is_some_and(|r| r.right() < view.left()), "the Name row has a text field in the rows: {name_field:?}");
    assert!(rect(crate::egui_app::tool_library_dialog::assembly_group::HEADING).top() > rect("Shank diameter").top(), "the assembly group follows the SHANK group");
    assert!(win.expand(1.0).contains_rect(holder), "the toggles {holder:?} stay in the window {win:?}");
    // #3908: Flutes follows Holder on its row, or wraps under it.
    let flutes = toggle(preview::FLUTES);
    let beside = flutes.left() > holder.left() && (flutes.center().y - holder.center().y).abs() < 2.0;
    assert!(beside || flutes.top() >= holder.bottom() - 1.0, "Flutes {flutes:?} follows Holder {holder:?}");
    assert!(win.expand(1.0).contains_rect(flutes), "the Flutes toggle {flutes:?} stays in the window {win:?}");
    let rail = rect("All tools");
    // #4334: the name shows twice - in the header and as the rail's row.
    let row = node
        .query_all_by_label(&name)
        .map(|n| n.rect())
        .max_by(|a, b| a.top().total_cmp(&b.top()))
        .unwrap_or_else(|| panic!("the rail has no row {name}"));
    assert!(rail.right() < rect("Diameter").left() && row.top() > rect(&heading).bottom(), "the rail {rail:?} stands left of the editor, under the header");
    assert!(node.query_all_by_label(crate::egui_app::tool_library_dialog::list_header::IMPORT).next().is_none(), "the header row folds away with the list");
    let Some(props) = super::layout::leaf_rect(&h.state().dock, &Tab::Properties) else {
        panic!("the Properties leaf is not laid out");
    };
    assert!(!win.intersects(props), "the window {win:?} stands beside the Properties pane {props:?}");
    let (recipes, details) = (rect(TAB), rect("Details"));
    assert!(
        (recipes.top() - details.top()).abs() < 2.0 && recipes.right() <= details.left() + 1.0,
        "the Recipes tab {recipes:?} stands left of Details {details:?} on the tab row"
    );
    super::tool_library_editor::click_window_button(&mut h, TAB);
    let node = h
        .query_all_by_role(egui::accesskit::Role::Window)
        .find(|n| n.accesskit_node().label().as_deref() == Some("Tool library"))
        .unwrap_or_else(|| panic!("the Tool library window is not open"));
    let rect = |label: &str| {
        node.query_all_by_label(label)
            .next()
            .unwrap_or_else(|| panic!("the Recipes tab has no widget labelled {label:?}"))
            .rect()
    };
    // #4337 (U11): the explanation, then the table's columns, then the
    // three buttons on one row, then the units note - all above Apply.
    let line = |part: &str| {
        node.query_by_label_contains(part)
            .unwrap_or_else(|| panic!("the Recipes tab has no line with {part:?}"))
            .rect()
    };
    let (tab, explain, apply) = (rect(TAB), line("One recipe for each work material."), rect("Apply"));
    let (material, rpm, stepdown) = (rect("Material"), rect("Spindle rpm"), rect("Stepdown"));
    let (add, suggest, copy, units) = (rect(ADD), rect(SUGGEST), rect(COPY), line("Units: feed"));
    assert!(tab.top() < explain.top() && explain.bottom() <= material.top() + 1.0, "the explanation {explain:?} heads the table {material:?}");
    assert!(material.left() < rpm.left() && rpm.left() < stepdown.left(), "Material, Spindle rpm, Stepdown read left to right");
    assert!(add.top() > material.bottom(), "the buttons {add:?} follow the table");
    // #4453: the surface speed and chip load inputs stand between the
    // table and the buttons, the surface speed left of the chip load.
    let vc_fz = |what: &str| {
        node.query_all_by_label_contains(what)
            .find(|n| n.accesskit_node().role() == egui::accesskit::Role::SpinButton)
            .unwrap_or_else(|| panic!("the Recipes tab has no {what:?} input"))
            .rect()
    };
    let (vc, fz) = (vc_fz(crate::egui_app::tool_library_dialog::recipes_speeds::VC), vc_fz(crate::egui_app::tool_library_dialog::recipes_speeds::FZ));
    assert!(vc.top() > rpm.bottom() && vc.bottom() <= add.top() + 1.0, "the surface speed {vc:?} stands under the table, above the buttons");
    assert!((vc.top() - fz.top()).abs() < 2.0 && vc.right() <= fz.left(), "the surface speed {vc:?} then the chip load {fz:?} on one row");
    assert!((add.top() - suggest.top()).abs() < 2.0 && add.right() <= suggest.left() + 1.0, "Add {add:?} then Suggest {suggest:?} on one row");
    assert!(copy.top() >= suggest.top() - 2.0, "Copy {copy:?} follows Suggest, on its row or under it");
    assert!(units.top() > copy.top() && units.bottom() < apply.top(), "the units note {units:?} follows the buttons, above Apply");
    for b in [add, suggest, copy] {
        assert!(win.expand(1.0).contains_rect(b), "the button {b:?} stays in the window {win:?}");
    }
    assert!(node.query_all_by_label("Diameter").next().is_none(), "the geometry rows stay on the Tool tab");
}

/// #3897 (U6): the Holders page. The taper column stands left of the
/// holder rows; #4338 (U12): one header row - the search, then New holder,
/// Import and Export at its right - heads the rows. A holder
/// opens in its view: the way back and the name first, the facts on the
/// left, the drawing's Dimensions toggle right of them.
#[test]
fn layout_window_holders() {
    let _g = snapshot_guard();
    let mut h = shell_harness();
    h.run_steps(2);
    h.get_by_role_and_label(egui::accesskit::Role::Button, "Tools").click();
    h.run_steps(2);
    super::tool_library_editor::click_window_button(&mut h, HOLDERS_PAGE);
    assert_window_layout(&h, "Tool library", &[HOLDERS_PAGE, holders::SEARCH, "Used by"]);
    let win = library_window(&h);
    let node = super::tool_library_editor::library_window(&h);
    let rect = |l: &str| node.query_all_by_label(l).next().unwrap_or_else(|| panic!("no {l:?} in the window")).rect();
    let (all, name) = (rect(holders::ALL), rect("Name"));
    assert!(all.right() < name.left(), "the taper column {all:?} stands left of the rows {name:?}");
    let (search, new, export) = (rect(holders::SEARCH), rect(holders::NEW), rect(holders::EXPORT));
    assert!(search.right() <= new.left() && (search.center().y - new.center().y).abs() < 8.0, "New holder {new:?} stands right of the search {search:?}, in one row");
    assert!(new.right() <= export.left() && search.bottom() < name.top(), "Export ends the row, and the row heads the table");
    assert!(win.expand(1.0).contains_rect(export), "the header row stays in the window");
    assert!(win.expand(1.0).contains_rect(rect("Used by")), "the last head stays in the window");
    let Some(props) = super::layout::leaf_rect(&h.state().dock, &Tab::Properties) else {
        panic!("the Properties leaf is not laid out");
    };
    assert!(!win.intersects(props), "the window {win:?} stands beside the Properties pane {props:?}");
    super::tool_library_holders_ui::filter_to_holder(&mut h);
    super::tool_library_editor::click_window_button(&mut h, "BT40 ER32");
    let node = super::tool_library_editor::library_window(&h);
    let rect = |l: &str| node.query_all_by_label(l).next().unwrap_or_else(|| panic!("no {l:?} in the holder view")).rect();
    let (back, taper, dims) = (rect(holder_view::BACK), rect("Spindle taper"), rect(holder_view::DIMS));
    assert!(back.top() < taper.top(), "the way back {back:?} heads the view");
    assert!(dims.left() > taper.right(), "the drawing's toggle {dims:?} stands right of the facts {taper:?}");
    assert!(win.expand(1.0).contains_rect(dims), "the toggle stays in the window {win:?}");
}

/// #4338 (U12): the holder form of a user holder. The way back and the
/// name head it; the fields stand left of the drawing, the taper above
/// the clamp above the shape; the footer has Apply, Cancel and Duplicate
/// below the fields, in the window.
#[test]
fn layout_window_holder_form() {
    use crate::egui_app::tool_library_dialog::holder_form;
    let _g = snapshot_guard();
    let mut h = shell_harness();
    h.run_steps(2);
    h.get_by_role_and_label(egui::accesskit::Role::Button, "Tools").click();
    h.run_steps(2);
    super::tool_library_editor::click_window_button(&mut h, HOLDERS_PAGE);
    super::tool_library_editor::click_window_button(&mut h, holders::NEW);
    let win = library_window(&h);
    let node = super::tool_library_editor::library_window(&h);
    let rect = |l: &str| node.query_all_by_label(l).next().unwrap_or_else(|| panic!("no {l:?} in the holder form")).rect();
    let (back, taper, clamp, nose) = (rect(frame::BACK), rect(holder_form::TAPER), rect(holder_form::CLAMP), rect(holder_form::NOSE));
    assert!(back.top() < taper.top() && taper.top() < clamp.top() && clamp.top() < nose.top(), "back, taper, clamp, shape from the top");
    let scale = rect("to scale · mm");
    assert!(scale.left() > nose.right(), "the drawing {scale:?} stands right of the fields {nose:?}");
    let (apply, cancel, dup) = (rect(frame::APPLY), rect(frame::CANCEL), rect(frame::DUPLICATE));
    assert!(dup.right() <= cancel.left() && cancel.right() <= apply.left(), "Duplicate, Cancel, Apply from the left");
    assert!(apply.top() > nose.bottom(), "the footer stands below the fields");
    for r in [apply, scale, nose] {
        assert!(win.expand(1.0).contains_rect(r), "{r:?} stays in the window {win:?}");
    }
}

/// #3898 (U7): the "My library" source. The component groups stand left
/// of the rows; (#4335) the list's search box and the buttons of a picked
/// row head the rows; the document's component table follows them.
#[test]
fn layout_window_my_library() {
    use crate::egui_app::tool_library_dialog::components;
    let _g = snapshot_guard();
    let mut h = shell_harness();
    h.run_steps(2);
    h.get_by_role_and_label(egui::accesskit::Role::Button, "Tools").click();
    h.run_steps(2);
    super::tool_library_editor::click_window_button(&mut h, components::SOURCE);
    use crate::egui_app::tool_library_dialog::list_header::SEARCH;
    assert_window_layout(&h, "Tool library", &[components::SOURCE, components::ADD, components::DOCUMENT]);
    let win = library_window(&h);
    let node = super::tool_library_editor::library_window(&h);
    let rect = |l: &str| node.query_all_by_label(l).next().unwrap_or_else(|| panic!("no {l:?} in the window")).rect();
    let (all, filter, add, doc) = (rect(components::ALL), rect(SEARCH), rect(components::ADD), rect(components::DOCUMENT));
    assert!(all.right() < filter.left(), "the group column {all:?} stands left of the rows {filter:?}");
    assert!(filter.top() < add.top(), "the filter heads the buttons");
    assert!(add.top() < doc.top(), "the document's table follows the library rows");
    assert!(win.expand(1.0).contains_rect(add), "the buttons stay in the window {win:?}");
}
