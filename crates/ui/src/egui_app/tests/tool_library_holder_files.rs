//! #4389: the Holders page's Import… and Export… buttons. They stand
//! enabled with a hover text; Export writes the document's user holders
//! to a holder file; Import adds its holders as user holders in one undo
//! step, a held id under a new `user:holder:` id; a file that is not
//! correct is refused by name on the Tool pane and the document does not
//! change.
//! Gate map: covers ui::egui_app::tool_library_dialog, ui::state (state_holder_files).
//! Provenance: our own structural and state tests; no HSMWorks material involved.

use egui::accesskit::Role;

use super::tool_library_editor::library_window;
use super::tool_library_holders_ui::open_holders;
use super::*;
use crate::egui_app::tool_library_dialog::holders;
use crate::state::{CommandKind, ErrorScope};
use camcraft_app::commands::UpsertDocumentComponent;
use camcraft_app::commands::holder_file::is_holder;
use camcraft_app::component::ToolComponent;
use camcraft_app::component_library;
use camcraft_app::schema::DocumentComponent;
use camcraft_app::tool_assembly as ta;

const ID: &str = "user:holder:Shop BT40";

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("camcraft-holder-files-ui-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("make the test folder");
    d
}

/// A state with an empty component library folder and one user holder
/// (a built-in holder's shape under a user id).
fn state_with_holder(dir: &std::path::Path) -> State {
    let mut state = State::new();
    state.component_lib.dir = Some(dir.to_path_buf());
    let lib = component_library::builtins().into_iter().find(|c| is_holder(&c.component)).expect("a built-in holder");
    let c = ToolComponent::new(ID, "Shop BT40", lib.component.kind().clone()).expect("component");
    let cmd = UpsertDocumentComponent::with_row(&state.world, DocumentComponent::new(c, lib.geometry)).expect("upsert");
    state.history.execute(&state.world, cmd);
    state
}

#[test]
fn import_and_export_stand_enabled_with_a_hover_text() {
    let mut h = shell_harness();
    h.run_steps(2);
    open_holders(&mut h);
    for label in [holders::IMPORT, holders::EXPORT] {
        let node = library_window(&h).get_by_role_and_label(Role::Button, label);
        assert!(!node.accesskit_node().is_disabled(), "{label} is enabled");
    }
    assert!(!holders::IMPORT_HINT.contains('#') && !holders::EXPORT_HINT.contains('#'), "the hover says what it does");
}

#[test]
fn export_then_import_adds_user_holders_in_one_undo_step() {
    let dir = temp_dir("round");
    let file = dir.join("holders.camcomponents");
    let mut state = state_with_holder(&dir);
    state.export_holders_to(&file);
    assert!(state.status.contains("holders written"), "{}", state.status);
    let text = std::fs::read_to_string(&file).expect("the file");
    assert!(text.contains(ID), "the user holder rides the file");

    let before = ta::components(&state.world).len();
    let done = state.history.done_len();
    state.import_holders_from(&file);
    assert!(state.status.contains("1 holder imported"), "{}", state.status);
    let copy = format!("{ID} (2)");
    assert!(ta::component(&state.world, &copy).is_some(), "a held id takes a new user id: {}", state.status);
    assert_eq!(ta::components(&state.world).len(), before + 1);
    assert_eq!(state.history.done_len(), done + 1, "one undo step");
    state.pending.push_back(CommandKind::Undo);
    state.drain_actions();
    assert_eq!(ta::components(&state.world).len(), before, "one undo removes the import");

    // Into a new document the holder keeps its id.
    let mut fresh = State::new();
    fresh.component_lib.dir = Some(dir.clone());
    fresh.import_holders_from(&file);
    assert!(ta::component(&fresh.world, ID).is_some(), "{}", fresh.status);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_file_that_is_not_correct_is_refused_on_the_tool_pane() {
    let dir = temp_dir("bad");
    let file = dir.join("bad.camcomponents");
    std::fs::write(&file, "{ \"format\": \"camcraft-components\", \"version\": 1, \"units\": \"mm\", \"components\": [ {").expect("write");
    let mut state = state_with_holder(&dir);
    let before = ta::components(&state.world);
    let done = state.history.done_len();
    state.import_holders_from(&file);
    let err = state.error_for(ErrorScope::Tool).map(str::to_owned).unwrap_or_default();
    assert!(err.starts_with("holders not imported: not a holder file"), "{err}");
    assert_eq!(ta::components(&state.world), before, "the document does not change");
    assert_eq!(state.history.done_len(), done, "no history entry");
    // A document with no user holder: Export is refused by name.
    let mut empty = State::new();
    empty.export_holders_to(&dir.join("none.camcomponents"));
    let err = empty.error_for(ErrorScope::Tool).map(str::to_owned).unwrap_or_default();
    assert!(err.contains("no user holder"), "{err}");
    let _ = std::fs::remove_dir_all(&dir);
}
