//! #4334 (U8 of #3663): the frame of the tool library window - one header
//! (the way back, the name, the Cutting tools | Holders tabs, the close
//! button) and one footer on every page (the hint; Close on a list page;
//! Duplicate, Cancel and Apply on the tool editor). The tests assert the
//! labels, the roles and the enabled state, and what each button does.
//! Gate map: covers ui::egui_app::tool_library_dialog, ui::state.
//! Provenance: our own structural tests over the AccessKit tree and the
//! command history; no HSMWorks material involved.

use egui::accesskit::Role;
use egui_kittest::kittest::NodeT;

use super::tool_library_editor::{H, click_window_button, draft, library_window, open_in_window, tool};
use super::*;
use crate::egui_app::tool_library_dialog::frame::{self, Face};
use crate::egui_app::tool_library_dialog::{HOLDERS_PAGE, TITLE, TOOLS_PAGE, holder_view};
use crate::state::{LibraryPage, ToolSource};

/// The window's button `label`, if it shows one, with its enabled state.
fn button(h: &H, label: &str) -> Option<bool> {
    library_window(h)
        .query_all_by_role_and_label(Role::Button, label)
        .next()
        .map(|n| !n.accesskit_node().is_disabled())
}

fn has(h: &H, label: &str) -> bool {
    library_window(h).query_all_by_label(label).next().is_some()
}

fn first_tool(h: &H) -> String {
    h.state().state.tool_names().into_iter().next().unwrap_or_else(|| panic!("the seeded document has a tool"))
}

/// A list page: the title and the two tabs head it; the footer has the
/// hint and an enabled Close, and none of the editor's buttons. Close
/// closes the window and queues nothing.
#[test]
fn the_tool_list_has_the_header_and_the_close_footer() {
    let mut h = shell_harness();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Tools").click();
    h.run_steps(3);
    assert!(has(&h, TITLE), "the header names the window");
    assert!(has(&h, TOOLS_PAGE) && has(&h, HOLDERS_PAGE), "the header has the two tabs");
    assert_eq!(button(&h, frame::CLOSE_WINDOW), Some(true), "the header has its close button");
    assert_eq!(button(&h, frame::BACK), None, "a list page has no way back");
    assert!(has(&h, frame::HINT_TOOLS), "the footer gives the hint");
    assert_eq!(button(&h, frame::CLOSE), Some(true), "the footer has Close");
    for label in [frame::APPLY, frame::CANCEL, frame::DUPLICATE] {
        assert_eq!(button(&h, label), None, "a list page has no {label:?}");
    }
    let done = h.state().state.history.done_len();
    click_window_button(&mut h, frame::CLOSE);
    assert!(h.state().state.tool_library.is_none(), "Close closes the window");
    assert_eq!(h.state().state.history.done_len(), done, "Close is view state: no command");
}

/// The tool editor: the way back and the tool's name head the window; the
/// footer has Duplicate (disabled, with the reason), Cancel and Apply, and
/// no Close.
#[test]
fn the_tool_editor_has_the_back_button_and_the_apply_footer() {
    let mut h = shell_harness();
    h.run_steps(2);
    let name = first_tool(&h);
    open_in_window(&mut h, &name);
    assert_eq!(button(&h, frame::BACK), Some(true), "the header has the way back");
    assert!(has(&h, &name), "the header names the tool");
    assert!(has(&h, TOOLS_PAGE) && has(&h, HOLDERS_PAGE), "the tabs stay in the header");
    assert!(has(&h, frame::HINT_APPLY), "the footer says that Apply is one undo step");
    assert_eq!(button(&h, frame::APPLY), Some(true), "Apply is enabled");
    assert_eq!(button(&h, frame::CANCEL), Some(true), "Cancel is enabled");
    assert_eq!(button(&h, frame::DUPLICATE), Some(false), "Duplicate is shown, disabled");
    assert_eq!(button(&h, frame::CLOSE), None, "an editor page has no Close in its footer");
    assert_eq!(button(&h, frame::CLOSE_WINDOW), Some(true), "the header still closes the window");
}

/// Cancel drops the unapplied draft and goes back to the list: no command
/// lands and the tool keeps its numbers. Apply stays one undo step.
#[test]
fn cancel_drops_the_draft_and_apply_stays_one_undo_step() {
    let mut h = shell_harness();
    h.run_steps(2);
    let name = first_tool(&h);
    open_in_window(&mut h, &name);
    let before = tool(&h, &name);
    let done = h.state().state.history.done_len();
    draft(&mut h).diameter = before.diameter + 0.5;
    h.run_steps(2);
    click_window_button(&mut h, frame::CANCEL);
    assert_eq!(h.state().state.tool_library.as_ref().and_then(|v| v.open_tool()), None, "Cancel goes back to the list");
    assert_eq!(h.state().state.history.done_len(), done, "Cancel queues no command");
    assert!((tool(&h, &name).diameter - before.diameter).abs() < 1e-12, "the tool keeps its diameter");
    open_in_window(&mut h, &name);
    draft(&mut h).diameter = before.diameter + 0.5;
    h.run_steps(2);
    click_window_button(&mut h, frame::APPLY);
    h.run_steps(2);
    assert_eq!(h.state().state.history.done_len(), done + 1, "Apply is one undo step");
    assert!((tool(&h, &name).diameter - (before.diameter + 0.5)).abs() < 1e-9, "Apply lands the draft");
    h.state_mut().state.pending.push_back(crate::state::CommandKind::Undo);
    h.run_steps(3);
    assert!((tool(&h, &name).diameter - before.diameter).abs() < 1e-12, "one undo restores the tool");
}

/// The header's way back leaves the tool editor and the holder view for
/// their lists; the header's close button closes the window from an editor.
#[test]
fn back_leaves_each_editor_and_the_header_closes_the_window() {
    let mut h = shell_harness();
    h.run_steps(2);
    let name = first_tool(&h);
    open_in_window(&mut h, &name);
    click_window_button(&mut h, frame::BACK);
    assert_eq!(h.state().state.tool_library.as_ref().and_then(|v| v.open_tool()), None, "back closes the tool editor");
    click_window_button(&mut h, HOLDERS_PAGE);
    super::tool_library_holders_ui::filter_to_holder(&mut h);
    click_window_button(&mut h, super::tool_library_mockup::HOLDER);
    assert!(has(&h, frame::HINT_HOLDER), "the holder view gives its hint");
    assert_eq!(button(&h, frame::CLOSE), Some(true), "the read-only holder view keeps Close");
    click_window_button(&mut h, holder_view::BACK);
    let open = h.state().state.tool_library.as_ref().map(|v| v.holders.open.clone());
    assert_eq!(open, Some(None), "back closes the holder view");
    open_in_window(&mut h, &name);
    click_window_button(&mut h, frame::CLOSE_WINDOW);
    assert!(h.state().state.tool_library.is_none(), "the header's button closes the window");
}

/// Each face has its hint, and only an editor (or the holder view) offers
/// the way back.
#[test]
fn each_face_has_its_hint() {
    let faces = [
        (Face::List(LibraryPage::Tools, ToolSource::Document), frame::HINT_TOOLS, false),
        (Face::List(LibraryPage::Tools, ToolSource::Catalog), frame::HINT_CATALOG, false),
        (Face::List(LibraryPage::Tools, ToolSource::Library), frame::HINT_LIBRARY, false),
        (Face::List(LibraryPage::Holders, ToolSource::Document), frame::HINT_HOLDERS, false),
        (Face::Holder("BT40 ER32".into()), frame::HINT_HOLDER, true),
        (
            Face::Tool { name: "EM".into(), kind: camcraft_app::tools::ToolKind::EndMill, number: 1 },
            frame::HINT_APPLY,
            true,
        ),
        (Face::Component("holder".into()), frame::HINT_COMPONENT, true),
    ];
    for (face, hint, back) in faces {
        assert_eq!(frame::hint(&face), hint, "{face:?}");
        assert_eq!(face.has_back(), back, "{face:?}");
    }
}

/// The header and the footer are named frame pieces: each frame of the open
/// window records one `frame_cost` span for each (crates/ui/AGENTS.md
/// "Frame cost").
#[test]
fn the_header_and_the_footer_name_their_frame_cost() {
    let mut h = shell_harness();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Tools").click();
    h.run_steps(3);
    let ((), costs) = crate::frame_cost::record(|| {
        for _ in 0..4 {
            h.step();
        }
    });
    for name in [frame::HEADER_SPAN, frame::FOOTER_SPAN] {
        let calls = costs.iter().find(|c| c.name == name).map_or(0, |c| c.count);
        assert!(calls >= 4, "each frame records {name} ({calls} calls in 4 frames): {costs:?}");
    }
}
