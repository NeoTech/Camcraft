//! #4370: the tool editor's Name row is a text field. Apply lands a typed
//! name as one `RenameTool`: the tool and each operation that uses it
//! take the new name, the editor follows the tool, and one undo restores
//! the old name (the editor follows it back). A refused name changes
//! nothing and keeps the typed text; Cancel drops it; a typed name makes
//! the draft dirty, so Duplicate waits for Apply. A rename of another
//! tool (with its undo and redo) leaves the open editor on its tool.
//! Gate map: covers ui::egui_app::tool_library_dialog, ui::state.
//! Provenance: our own structural tests over the AccessKit tree and the
//! command history; no HSMWorks material involved.

use egui::accesskit::Role;
use egui_kittest::kittest::NodeT;

use super::tool_library_editor::{H, click_window_button, library_window, open_in_window, tool};
use super::*;
use crate::egui_app::tool_library_dialog::{frame, tool_tab};

fn first_tool(h: &H) -> String {
    h.state().state.tool_names().into_iter().next().unwrap_or_else(|| panic!("the seeded document has a tool"))
}

fn open_tool(h: &H) -> Option<String> {
    h.state().state.tool_library.as_ref().and_then(|v| v.open_tool()).map(str::to_owned)
}

/// The operations of the document that name `tool`.
fn users(h: &H, tool: &str) -> Vec<String> {
    let w = &h.state().state.world;
    model::named_entities(w)
        .into_iter()
        .filter(|(_, e)| model::operation_of(w, e).is_some_and(|op| op.tool == tool))
        .map(|(n, _)| n)
        .collect()
}

fn duplicate_enabled(h: &H) -> bool {
    !library_window(h).get_by_role_and_label(Role::Button, frame::DUPLICATE).accesskit_node().is_disabled()
}

/// The Name row is a text input in the window that shows the tool's name;
/// typing into it changes only the window's draft.
#[test]
fn the_name_row_is_a_text_field_and_typing_is_a_draft() {
    let mut h = shell_harness();
    h.run_steps(2);
    let name = first_tool(&h);
    open_in_window(&mut h, &name);
    let field = library_window(&h).get_by_role_and_label(Role::TextInput, tool_tab::NAME);
    assert_eq!(field.accesskit_node().value().as_deref(), Some(name.as_str()), "the field shows the name");
    let done = h.state().state.history.done_len();
    library_window(&h).get_by_role_and_label(Role::TextInput, tool_tab::NAME).click();
    h.run_steps(2);
    library_window(&h).get_by_role_and_label(Role::TextInput, tool_tab::NAME).type_text("X");
    h.run_steps(2);
    let typed = h.state().state.library_name_text(&name);
    assert!(typed != name && typed.contains('X'), "the typed text is the draft: {typed}");
    assert_eq!(h.state().state.history.done_len(), done, "typing queues no command");
    assert!(h.state().state.tool_names().contains(&name), "the tool keeps its name until Apply");
    assert!(h.state().state.library_draft_changed(), "a typed name makes the draft dirty");
    assert!(!duplicate_enabled(&h), "Duplicate waits for Apply");
}

/// Apply renames the tool and its operations as one undo step; the
/// editor follows the tool, and one undo brings the old name back.
#[test]
fn apply_renames_the_tool_and_its_operations_in_one_undo_step() {
    let mut h = shell_harness();
    h.run_steps(2);
    let name = first_tool(&h);
    let before = tool(&h, &name);
    let ops = users(&h, &name);
    open_in_window(&mut h, &name);
    let new = format!("{name} renamed");
    let done = h.state().state.history.done_len();
    h.state_mut().state.set_library_name_text(&name, format!(" {new} "));
    h.run_steps(2);
    click_window_button(&mut h, frame::APPLY);
    h.run_steps(2);
    assert_eq!(h.state().state.history.done_len(), done + 1, "a rename alone is one undo step");
    assert!(!h.state().state.tool_names().contains(&name), "the old name is gone");
    assert_eq!(tool(&h, &new), before, "the tool keeps its definition");
    assert_eq!(users(&h, &new), ops, "each operation follows the new name");
    assert_eq!(open_tool(&h).as_deref(), Some(new.as_str()), "the editor follows the tool");
    assert!(!h.state().state.library_draft_changed(), "after Apply the draft is the tool");
    assert!(duplicate_enabled(&h), "after Apply Duplicate is enabled");

    h.state_mut().state.pending.push_back(crate::state::CommandKind::Undo);
    h.run_steps(3);
    assert_eq!(tool(&h, &name), before, "one undo restores the old name");
    assert_eq!(users(&h, &name), ops, "and every operation's reference");
    assert_eq!(open_tool(&h).as_deref(), Some(name.as_str()), "the editor follows the tool back");
}

/// A rename of another tool (an agent's `rename_tool`, its undo and its
/// redo) leaves the open editor on its own tool: a view follows only the
/// tool that it shows.
#[test]
fn a_rename_of_another_tool_leaves_the_open_editor_alone() {
    let mut h = shell_harness();
    h.run_steps(2);
    let name = first_tool(&h);
    open_in_window(&mut h, &name);
    click_window_button(&mut h, frame::DUPLICATE);
    h.run_steps(2);
    let copy = format!("{name} (2)");
    assert_eq!(open_tool(&h).as_deref(), Some(copy.as_str()), "Duplicate opens the copy and stays on it");
    open_in_window(&mut h, &name);
    let other = format!("{name} other");
    h.state_mut().state.pending.push_back(crate::state::CommandKind::RenameTool { from: copy.clone(), to: other.clone() });
    h.run_steps(2);
    assert!(h.state().state.tool_names().contains(&other), "the copy is renamed");
    assert_eq!(open_tool(&h).as_deref(), Some(name.as_str()), "the editor stays on its tool");
    for (step, kind) in [("undo", crate::state::CommandKind::Undo), ("redo", crate::state::CommandKind::Redo)] {
        h.state_mut().state.pending.push_back(kind);
        h.run_steps(3);
        assert_eq!(open_tool(&h).as_deref(), Some(name.as_str()), "after {step} the editor stays on its tool");
    }
    assert!(h.state().state.tool_names().contains(&other), "redo lands the rename again");
}

/// An empty name is refused by name: nothing lands, the typed text stays
/// for correction. Cancel then drops it.
#[test]
fn an_empty_name_is_refused_and_cancel_drops_the_typed_name() {
    let mut h = shell_harness();
    h.run_steps(2);
    let name = first_tool(&h);
    open_in_window(&mut h, &name);
    let done = h.state().state.history.done_len();
    h.state_mut().state.set_library_name_text(&name, "  ".into());
    h.run_steps(2);
    click_window_button(&mut h, frame::APPLY);
    h.run_steps(2);
    assert_eq!(h.state().state.history.done_len(), done, "a refused rename lands nothing");
    let status = h.state().state.status.clone();
    assert!(status.contains("tool not renamed") && status.contains("must not be empty"), "{status}");
    assert!(h.state().state.tool_names().contains(&name), "the tool keeps its name");
    assert_eq!(h.state().state.library_name_text(&name), "  ", "the typed text stays for correction");
    click_window_button(&mut h, frame::CANCEL);
    open_in_window(&mut h, &name);
    assert_eq!(h.state().state.library_name_text(&name), name, "Cancel dropped the typed name");
}
