//! The ribbon's one command path and the pin gesture, split from panes.rs
//! (#4393, the 400-line law); our own design notes.

use camcraft_app::operation;

use crate::actions;
use crate::state::{Sim, State, UiTarget};

use super::cmd::{Cmd, MAX_PINS};

/// The ribbon's one command path: every cell fires the shared actions
/// layer (`crate::actions`), so the effect lives there exactly once and
/// menu, tree and shortcut cannot fork a variant of it (actions.rs's
/// rule). `Cmd::Paths` stays a direct field flip - view state, not a
/// command, exactly imgui's run_toggle_paths (ribbon.rs:98).
pub(super) fn run_cmd(c: Cmd, state: &mut State) {
    match c {
        Cmd::New => actions::new_document(state),
        // #662: with recent files on hand the button opens its list
        // (recent_drop.rs); with none it is the dialog, as before.
        Cmd::Model => crate::egui_app::recent_drop::press(state, camcraft_app::recent::RecentKind::Model),
        Cmd::Project => crate::egui_app::recent_drop::press(state, camcraft_app::recent::RecentKind::Project),
        Cmd::Save => actions::save_project(state),
        Cmd::SaveAs => actions::save_project_as(state),
        Cmd::Post => actions::post_program(state),
        Cmd::Setups => actions::target(state, UiTarget::Setups),
        Cmd::Stock => actions::target(state, UiTarget::Stock),
        Cmd::Origin => actions::target(state, UiTarget::Wcs),
        Cmd::Tools => actions::target(state, UiTarget::Tool(String::new())),
        Cmd::Create(i) => actions::add_operation(state, i),
        Cmd::Undo => actions::undo(state),
        Cmd::Redo => actions::redo(state),
        Cmd::Run => actions::generate_selected(state),
        Cmd::Delete => actions::delete_selected(state),
        Cmd::Frame => actions::frame_all(state),
        Cmd::Paths => state.toggle_all_toolpaths(),
        Cmd::Simulate => {
            // View state, like Paths: the cell arms or disarms the
            // transport. The playhead inside is immediate-mode by
            // AGENTS.md's explicit exemption - no command to route.
            state.sim = if state.sim.is_some() { None } else { Some(Sim::new()) };
        }
        Cmd::Deviate => {
            // View state like Simulate, with one difference the actions
            // layer holds: arming RUNS the program sim (a world read,
            // never a world write), so the effect lives in actions.
            actions::toggle_deviate(state);
        }
        // Immediate-mode view state (the Paths law): which door stands
        // open is a frame fact, not a document fact - nothing to undo,
        // and clicking the open door again closes it. One dropdown at
        // a time: opening either closes the other.
        Cmd::More(s) => {
            state.ribbon_more = (state.ribbon_more != Some(s)).then_some(s);
            state.ribbon_recent_open = None;
            state.ribbon_mode_open = false;
            state.ribbon_view_open = false;
            state.ribbon_theme_open = false;
            state.ribbon_settings_open = false;
        }
        Cmd::Machine => {
            state.ribbon_mode_open = !state.ribbon_mode_open;
            if state.ribbon_mode_open {
                state.ribbon_recent_open = None;
                state.ribbon_more = None;
                state.ribbon_view_open = false;
                state.ribbon_theme_open = false;
                state.ribbon_settings_open = false;
            }
        }
        // Same door law; the dock itself is the shell's (layout.rs).
        Cmd::View => {
            state.ribbon_view_open = !state.ribbon_view_open;
            if state.ribbon_view_open {
                state.ribbon_recent_open = None;
                state.ribbon_more = None;
                state.ribbon_mode_open = false;
                state.ribbon_theme_open = false;
                state.ribbon_settings_open = false;
            }
        }
        Cmd::Theme => {
            state.ribbon_theme_open = !state.ribbon_theme_open;
            if state.ribbon_theme_open {
                state.ribbon_recent_open = None;
                state.ribbon_more = None;
                state.ribbon_mode_open = false;
                state.ribbon_view_open = false;
                state.ribbon_settings_open = false;
            }
        }
        Cmd::Copy => {
            if let Some(name) = state.selected_operation() {
                actions::copy_operation(state, &name);
            }
        }
        Cmd::Paste => actions::paste_operation(state),
        // The window close button's keyboard and menu siblings (M-g):
        // Quit -> request_quit -> exit_requested -> el.exit().
        Cmd::Quit => actions::quit(state),
        Cmd::Projection => actions::toggle_projection(state),
    }
}

/// The menu bar's and the keyboard's path (#4484): the ribbon's
/// `run_cmd`, except that Model and Open go straight to their file
/// dialogs. The ribbon's recent-files list hangs under its own cell, so
/// a menu row or a key press opens the dialog, as the keys always did.
pub(super) fn run_direct(c: Cmd, state: &mut State) {
    match c {
        Cmd::Model => actions::open_model_dialog(state),
        Cmd::Project => actions::open_project(state),
        other => run_cmd(other, state),
    }
}

/// The pin gesture (M6.2): toggle one kind's presence in the document's
/// pinned list. The pins are saved with the project, so the toggle is a
/// `SetRibbonPins` command on the history (issue #143): Ctrl+Z puts the
/// previous pins back and the document counts as changed. The status
/// line says what happened, because a silent toggle is a half-answer.
pub(in crate::egui_app) fn toggle_pin(state: &mut State, i: usize) {
    let Some(k) = operation::KindInfo::ALL.get(i) else {
        debug_assert!(false, "toggle_pin({i}) past the kind table");
        diag_warn!("ribbon: unknown op kind {i}");
        return;
    };
    let mut pins = camcraft_app::ribbon::pinned_of(&state.world);
    let verb = match pins.iter().position(|p| *p == i) {
        Some(at) => {
            pins.remove(at);
            "unpinned"
        }
        // The bar's op budget: a fifth pin is refused by name, not
        // silently dropped (Prime Directive - the gesture asked for
        // something the law declines, so the law says so).
        None if pins.len() >= MAX_PINS => {
            state.status = format!(
                "the ribbon keeps {MAX_PINS} pins - unpin one first (right-click a pinned cell)"
            );
            return;
        }
        None => {
            pins.push(i);
            "pinned"
        }
    };
    match camcraft_app::commands::SetRibbonPins::new(&state.world, &pins) {
        Ok(command) => {
            state.history.execute(&state.world, command);
            state.status = format!("{verb} {} on the ribbon", k.caption);
        }
        // Unreachable from the toggle (it always changes the list), but a
        // refusal is named, never swallowed.
        Err(err) => state.status = format!("ribbon pin not changed: {err}"),
    }
}
