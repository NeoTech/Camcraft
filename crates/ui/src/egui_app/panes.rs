//! The egui shell's pane layer, graduated from the spike's panels.rs -
//! every pane a faithful re-implementation of its imgui original (the
//! retired ribbon.rs/browser.rs): same strings, same command shapes, same
//! enabled rules. The strips are painted (super::painted) because that is
//! the law the spike proved; the property pages moved to `pages` with M-c,
//! leaving this module the keyboard layer and the two painted panes. Since
//! M-g the bundled Tabler glyphs ride both strips as egui textures
//! (super::icons), with the painted vector marks as their no-GPU twins -
//! the retired imgui icons module folded into this stack, not replaced.
//!
//! Layering holds (AGENTS.md): world access is `camcraft_app`'s public API
//! only, and every change goes through `state.pending` / the command
//! methods, never an ad-hoc field poke. The whole-pane law
//! (docs/spike-egui-go-no-go.md) forbids porting a pane by halves.
//!
//! Since M-f the surfaces here stopped forking their own command logic: the
//! ribbon cells, browser rows and keyboard shortcuts all fire through the
//! shared `crate::actions` layer - the same one the imgui ribbon/browser/menu
//! call - so each effect lives exactly once across both shells. `Cmd::Paths`
//! stays a direct field flip (view state, mirroring imgui ribbon.rs) and the
//! browser pushes its `Select` alongside `actions::target` (the tree law).

//!
//! #4393 split the ribbon and the status strip into `panes/` along the
//! domain seams (the 400-line law); this file keeps the keyboard layer and
//! re-exports the old names, so callers keep the `panes::` paths.

use egui::{Key, Ui};

use crate::actions;
use crate::state::State;

mod cmd;
mod gates;
mod mode_tabs;
mod ribbon_layout;
mod ribbon_strip;
mod run;
mod section_drop;
mod status_bar;

pub use ribbon_strip::ribbon;
pub use status_bar::status;
pub(super) use cmd::Cmd;
pub(super) use ribbon_layout::Cell;
#[cfg(test)]
pub(super) use cmd::{ribbon_plan, text_of, tip_of};
#[cfg(test)]
pub(super) use gates::{GateRoute, browser_refusal, enabled, follow_gate, gate_route, icon_of, toggled};
#[cfg(test)]
pub(super) use ribbon_layout::{Drop, machine_menu_width, machine_rows, ribbon_cells, view_rows_of};
#[cfg(test)]
pub(super) use run::toggle_pin;

/// The operation kinds in panel order - the app's kind registry labels
/// (M5.0.2), one list for the whole surface. Shared with pages.rs: the
/// Type combo there and the ribbon tips here quote the same table.
pub(crate) const TYPES: [&str; camcraft_app::operation::KIND_COUNT] =
    camcraft_app::operation::ALL_LABELS;

// ---------------------------------------------------------------- shortcuts

/// The shell's keyboard layer: history keys always, destructive and file
/// keys only while no text field owns focus (imgui's io.WantTextInput
/// guard - a dialog must never spawn mid-type). Every arm fires through
/// `crate::actions`, the same entry the ribbon uses.
pub fn shortcuts(ui: &mut Ui, state: &mut State) {
    let typing = ui.memory(|m| m.focused().is_some());
    let (modifiers, n, o, s, p, q, z, y, del, f, c, v) = ui.input(|i| {
        (
            i.modifiers,
            i.key_pressed(Key::N),
            i.key_pressed(Key::O),
            i.key_pressed(Key::S),
            i.key_pressed(Key::P),
            i.key_pressed(Key::Q),
            i.key_pressed(Key::Z),
            i.key_pressed(Key::Y),
            i.key_pressed(Key::Delete),
            i.key_pressed(Key::F),
            i.key_pressed(Key::C),
            i.key_pressed(Key::V),
        )
    });
    let cmd = modifiers.command;
    if cmd && z && !modifiers.shift && !modifiers.alt {
        actions::undo(state);
    }
    if cmd && y {
        // Ctrl+Y and Ctrl+Shift+Y both redo (imgui law).
        actions::redo(state);
    }
    if cmd && !typing {
        if c && let Some(name) = state.selected_operation() {
            actions::copy_operation(state, &name);
        }
        if v {
            actions::paste_operation(state);
        }
        // The File tips' keys (ribbon.rs strings): N new, O a model,
        // Shift+O a project, S save, Shift+S save as.
        if n {
            actions::new_document(state);
        }
        if o && !modifiers.shift {
            actions::open_model_dialog(state);
        }
        if o && modifiers.shift {
            actions::open_project(state);
        }
        if s {
            if modifiers.shift {
                actions::save_project_as(state);
            } else {
                actions::save_project(state);
            }
        }
        if p {
            // M1.7b: posting joins the file lane - the same dialog law
            // as Save (the gate's refusals arrive through the queue arm).
            actions::post_program(state);
        }
        if q {
            // The window close button (CloseRequested) always quit; since
            // M-g it has a keyboard sibling. imgui's only quit surface was
            // its File menu, which retired with the imgui shell - Ctrl+Q
            // keeps that capability on the shared law (Quit -> request_quit
            // -> exit_requested -> el.exit()).
            actions::quit(state);
        }
    }
    if del && !typing {
        actions::delete_selected(state);
    }
    if f && !typing && !cmd && !modifiers.alt {
        actions::frame_all(state);
    }
    if p && !typing && !cmd && !modifiers.alt {
        // #434: bare P flips perspective/ortho - a viewport-layer key
        // like F (Ctrl+P is the post lane above).
        actions::toggle_projection(state);
    }
}

// ------------------------------------------------------------------ browser

// #343: the browser tree moved to its own modules (the 400-line law) -
// rows in `browser_tree`, the pane and its click law in `browser_pane`,
// the operation menu in `tree_menu`. Re-exported so callers keep the path.
pub(crate) use super::browser_tree::kind_name;
#[cfg(test)]
pub(crate) use super::browser_tree::{Leaf, browser_rows, model_row_marks};
pub use super::browser_pane::browser;
