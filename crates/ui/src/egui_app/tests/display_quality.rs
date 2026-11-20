//! #2958: the Display menu's "Line width" and "Anti-aliasing" entries in its
//! Graphics group. A pick is a per-user view setting: it goes out to the
//! view file at once, survives a restart, queues no command, and Ctrl+Z
//! does not take it back. Another view setting never resets it. An
//! anti-aliasing count that the graphics device does not offer is a
//! disabled entry, and a pick of it is refused by name and not kept.
//! Provenance: our own tests over our own shell; no HSMWorks material.
//!
//! Gate map (M4.4): covers ui::egui_app::display_menu (the quality rows),
//! ui::state (state_view), app::view through the State handoff.

use std::path::PathBuf;

use camcraft_app::view::{self, Antialias, FILE_NAME, LineWidth};
use egui_kittest::kittest::NodeT;

use super::*;
use crate::egui_app::display_menu as menu;

fn temp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("camcraft-quality-ui-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("make the test folder");
    std::fs::canonicalize(&d).expect("canonical folder")
}

/// A shell that writes `file`, with the Display menu open through a real
/// press on the viewport's Display button.
fn open_menu(file: &std::path::Path, offered: &[u32]) -> Harness<'static, ShellUi> {
    let mut h = shell_harness();
    h.state_mut().state.view_store = Some(file.to_path_buf());
    h.state_mut().state.samples_offered = offered.to_vec();
    h.run();
    let p = hud(h.state()).nav_rect(viewcube::Nav::Display).center();
    click_viewport(&mut h, p);
    assert!(h.state().state.display_menu_open, "a press on Display opens the menu");
    h
}

fn disabled(h: &Harness<'static, ShellUi>, label: &str) -> bool {
    h.get_by_label(label).accesskit_node().is_disabled()
}

#[test]
fn the_defaults_are_one_and_a_half_points_and_four_samples() {
    let state = State::new();
    assert_eq!((state.line_width, state.antialias), (LineWidth::Normal, Antialias::X4));
    assert_eq!(state.samples_offered, vec![1, 4], "headless keeps the WebGPU guaranteed counts");
}

#[test]
fn the_graphics_group_offers_the_line_widths_and_the_antialiasing() {
    let h = open_menu(&temp_dir("labels").join(FILE_NAME), &[1, 4]);
    for label in [menu::LINE_WIDTH, menu::ANTIALIAS, "1 pt", "1.5 pt", "2 pt", "3 pt", "Off", "2x", "4x"] {
        assert!(h.query_by_label(label).is_some(), "entry {label:?}");
    }
    assert!(h.query_by_label("Line width").is_some());
}

#[test]
fn a_line_width_pick_is_kept_survives_a_restart_and_undo_leaves_it() {
    let file = temp_dir("width").join(FILE_NAME);
    let mut h = open_menu(&file, &[1, 4]);
    // The seed document is already unsaved; the pick must not change that.
    let dirty = h.state().state.history.is_dirty();
    h.get_by_label("2 pt").click();
    h.run_steps(2);
    assert_eq!(h.state().state.line_width, LineWidth::Bold);
    assert!(h.state().state.display_menu_open, "a choice keeps the menu open, as the other entries do");
    assert!(h.state().state.pending.is_empty(), "a view setting queues no command");
    assert_eq!(view::load(&file).0.line_width, LineWidth::Bold, "the pick goes out to the file at once");
    assert_eq!(h.state().state.history.is_dirty(), dirty, "a view setting is not an unsaved change");
    h.state_mut().state.pending.push_back(CommandKind::Undo);
    h.run_steps(3);
    assert_eq!(h.state().state.line_width, LineWidth::Bold, "undo leaves the width");
    // The gizmo-name switch writes the same file: it keeps the width.
    h.state_mut().state.set_gizmo_names(false);
    assert_eq!(view::load(&file).0.line_width, LineWidth::Bold, "another view setting never resets it");
    let mut restarted = State::new();
    restarted.load_view_from(&file);
    assert_eq!(restarted.line_width, LineWidth::Bold, "the width survives a restart");
}

#[test]
fn an_offered_antialias_pick_is_kept_and_an_unoffered_one_is_disabled() {
    let file = temp_dir("aa").join(FILE_NAME);
    let mut h = open_menu(&file, &[1, 4]);
    assert!(disabled(&h, "2x"), "2x is not offered on WebGPU's guaranteed counts");
    assert!(!disabled(&h, "Off") && !disabled(&h, "4x"));
    h.get_by_label("Off").click();
    h.run_steps(2);
    assert_eq!(h.state().state.antialias, Antialias::Off);
    assert_eq!(view::load(&file).0.antialias, Antialias::Off, "kept at once");
    // A device that offers 2x enables the entry.
    let h = open_menu(&file, &[1, 2, 4]);
    assert!(!disabled(&h, "2x"), "an offered 2x is live");
}

#[test]
fn a_pick_of_an_unoffered_count_is_refused_by_name_and_not_kept() {
    let file = temp_dir("refuse").join(FILE_NAME);
    let mut state = State::new();
    state.view_store = Some(file.clone());
    state.select_antialias(Antialias::X2);
    assert_eq!(state.antialias, Antialias::X4, "the refused pick is not worn");
    assert!(state.status.contains("anti-aliasing 2x not selected"), "refused by name: {}", state.status);
    assert!(!file.exists(), "nothing is written for a refusal");
    state.samples_offered = vec![1, 2, 4];
    state.select_antialias(Antialias::X2);
    assert_eq!(state.antialias, Antialias::X2, "an offered count is kept");
    assert_eq!(view::load(&file).0.antialias, Antialias::X2);
}
