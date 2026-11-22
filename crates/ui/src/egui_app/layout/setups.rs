//! Saved panel setups (#2339): the dock's whole arrangement as one JSON
//! value - which panels are open, where each is attached (a split side or
//! a floating window with its rect), and how (its leaf's size, its tab
//! order, its active tab and its collapsed state). The data is the dock's
//! own serde form, so nothing the user can arrange by hand is lost; this
//! module adds the file format's version tag (in `app::prefs`), the
//! validation on the way in, and the start-of-run load.
//!
//! A corrupt or older saved arrangement never reaches the dock: it is
//! refused by name on the status line and the default three-pane layout
//! stands. Provenance: our own code, specced from the egui_dock 0.21 docs
//! and vendored sources; no HSMWorks material involved.

use egui_dock::{DockState, SurfaceIndex};

use crate::state::{PanelSetupAsk, State};

use super::build_dock;
use crate::egui_app::shell::Tab;

/// The arrangement inside this frame's dock, as the file keeps it. A node
/// that has never been laid out carries an infinite rect (and a fresh
/// split an infinite fraction); JSON spells either one `null`, which
/// cannot be read back. They are saved as a placeholder instead - the
/// dock computes the real rect on the next frame - so a save of a fresh
/// float or a split always reads back.
pub(crate) fn encode(dock: &DockState<Tab>) -> Result<serde_json::Value, String> {
    let mut copy = dock.clone();
    for (_, node) in copy.iter_all_nodes_mut() {
        match node {
            egui_dock::Node::Leaf(leaf) => {
                leaf.rect = known(leaf.rect);
                leaf.viewport = known(leaf.viewport);
            }
            egui_dock::Node::Vertical(split) | egui_dock::Node::Horizontal(split) => {
                split.rect = known(split.rect);
                if !split.fraction.is_finite() {
                    split.fraction = 0.5;
                }
            }
            egui_dock::Node::Empty => {}
        }
    }
    serde_json::to_value(&copy).map_err(|e| camcraft_i18n::tf("the panel arrangement is not writable ({e})", &[format!("{}", e)]))
}

/// A rect JSON can read back: an unlaid-out dock's infinite one becomes a
/// placeholder size.
fn known(rect: egui::Rect) -> egui::Rect {
    let finite = rect.min.x.is_finite()
        && rect.min.y.is_finite()
        && rect.max.x.is_finite()
        && rect.max.y.is_finite();
    if finite {
        rect
    } else {
        egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(320.0, 240.0))
    }
}

/// Read a saved arrangement back. A shape this build does not know, or a
/// dock that breaks the permanence law, is refused by name.
pub(crate) fn decode(value: &serde_json::Value) -> Result<DockState<Tab>, String> {
    let dock: DockState<Tab> = serde_json::from_value(value.clone())
        .map_err(|e| camcraft_i18n::tf("not a panel arrangement ({e})", &[format!("{}", e)]))?;
    validate(&dock)?;
    Ok(dock)
}

/// The permanence law holds after a load too: exactly one viewport, docked
/// in the main area, and a main area that a reopened panel can split
/// against (egui_dock's `split` asserts on a rootless surface).
fn validate(dock: &DockState<Tab>) -> Result<(), String> {
    let mut viewports = 0_usize;
    for (path, tab) in dock.iter_all_tabs() {
        if matches!(tab, Tab::Viewport) {
            viewports += 1;
            if path.surface != SurfaceIndex::main() {
                return Err(String::from("the viewport was saved in a floating window"));
            }
        }
    }
    if viewports != 1 {
        return Err(camcraft_i18n::tf("the arrangement has {viewports} viewports instead of one", &[format!("{}", viewports)]));
    }
    let rooted = dock
        .get_surface(SurfaceIndex::main())
        .and_then(egui_dock::Surface::node_tree)
        .and_then(egui_dock::Tree::root_node)
        .is_some_and(|n| n.is_leaf() || n.is_parent());
    if !rooted {
        return Err(String::from("the main area is empty"));
    }
    Ok(())
}

/// The dock the window starts with: the last setup the user saved or
/// loaded, when auto-load is on and it decodes; otherwise the default
/// three-pane layout, with the refusal on the status line.
pub(crate) fn start_dock(state: &mut State) -> DockState<Tab> {
    let default = build_dock();
    if !state.panel_setups.auto_load {
        return default;
    }
    let Some(name) = state.panel_setups.last.clone() else {
        return default;
    };
    let Some(saved) = state.panel_setups.find(&name).map(|s| s.dock.clone()) else {
        return default;
    };
    match decode(&saved) {
        Ok(dock) => dock,
        Err(why) => {
            let line =
                camcraft_i18n::tf("panel setup '{name}' not restored: {why} - the default layout is in use", &[format!("{}", name), format!("{}", why)]);
            diag_warn!("{line}");
            state.status = line;
            default
        }
    }
}

/// Carry out one ask from the Panel setups window (#2339). The dock
/// belongs to the shell, so the window only files the ask and this reads
/// it, writes it, or swaps it. A load that fails puts the default layout
/// back and says why - the saved file stays untouched.
pub(crate) fn apply_ask(state: &mut State, dock: &mut DockState<Tab>, ask: PanelSetupAsk) {
    match ask {
        PanelSetupAsk::Save(raw) => {
            state.status = match camcraft_app::prefs::setup_name(&raw) {
                Err(why) => camcraft_i18n::tf("panel setup not saved: {why}", &[format!("{}", why)]),
                Ok(name) => match encode(dock) {
                    Err(why) => camcraft_i18n::tf("panel setup not saved: {why}", &[format!("{}", why)]),
                    Ok(json) => match state.save_panel_setup(name, json) {
                        Ok(()) => camcraft_i18n::tf("panel setup '{name}' saved", &[format!("{}", name)]),
                        Err(why) => camcraft_i18n::tf("panel setup not saved: {why}", &[format!("{}", why)]),
                    },
                },
            };
        }
        PanelSetupAsk::Load(name) => {
            let saved = state.panel_setups.find(&name).map(|s| s.dock.clone());
            state.status = match saved {
                Some(json) => match decode(&json) {
                    Ok(loaded) => {
                        *dock = loaded;
                        state.remember_setup(&name);
                        camcraft_i18n::tf("panel setup '{name}' loaded", &[format!("{}", name)])
                    }
                    Err(why) => {
                        *dock = build_dock();
                        camcraft_i18n::tf("panel setup '{name}' not loaded: {why} - the default layout is in use", &[format!("{}", name), format!("{}", why)])
                    }
                },
                None => {
                    *dock = build_dock();
                    camcraft_i18n::tf("panel setup '{name}' is not saved - the default layout is in use", &[format!("{}", name)])
                }
            };
        }
        PanelSetupAsk::Delete(name) => {
            state.status = if state.delete_panel_setup(&name) {
                camcraft_i18n::tf("panel setup '{name}' deleted", &[format!("{}", name)])
            } else {
                camcraft_i18n::tf("panel setup '{name}' is not saved", &[format!("{}", name)])
            };
        }
    }
}
