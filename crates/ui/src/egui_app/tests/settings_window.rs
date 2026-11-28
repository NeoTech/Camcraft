//! #4493: the Settings window. It opens from the ribbon's Settings button
//! and from the menu bar's "Settings…" row; each control applies at once
//! and writes the per-user file; "Reset to defaults" restores each section;
//! an old `prefs.json` / `view.json` shows the same values as before.
//! Gate map: covers ui::egui_app::settings_window, ui::egui_app::settings_drop,
//! ui::state_prefs, ui::state_view.
//! Provenance: our own tests over our own window; no vendor material.

use std::path::PathBuf;

use camcraft_app::commands::{color_vision_of, theme_of};
use camcraft_app::schema::{ColorVision, ThemeRef};
use camcraft_app::view::{self, Antialias, LineWidth};
use camcraft_app::prefs;
use egui::accesskit::{Role, Toggled};
use egui_kittest::kittest::NodeT;

use super::super::settings_drop::SETTINGS;
use super::super::settings_window::{self as settings, Section};
use super::*;

type H = Harness<'static, ShellUi>;

/// Put the house look back for the frames of the next test.
struct RestoreDark;

impl Drop for RestoreDark {
    fn drop(&mut self) {
        theme::set_current(&theme::Tokens::DARK);
    }
}

fn temp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("camcraft-settings-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("make the test folder");
    d
}

/// The control drawn on the same row as `label` (a check box of `check_row`).
pub(super) fn row_check<'a>(h: &'a H, label: &str) -> egui_kittest::Node<'a> {
    let lab = h.get_by_label(label).rect();
    h.query_all_by(move |n| n.role() == Role::CheckBox)
        .find(|n| {
            let c = n.rect().center().y;
            c >= lab.top() - 2.0 && c <= lab.bottom() + 2.0
        })
        .expect("a check box on the label's row")
}

fn window_open(h: &H) -> bool {
    h.query_all_by_role(Role::Window)
        .any(|n| n.accesskit_node().label().as_deref() == Some(settings::TITLE))
}

fn combo_value(h: &H, label: &str) -> Option<String> {
    h.get_by_role_and_label(Role::ComboBox, label).accesskit_node().value()
}

/// A shell with both per-user files in `dir`, the window open on `section`.
fn open_on(dir: &std::path::Path, section: Section) -> H {
    let mut h = shell_harness();
    h.state_mut().state.load_prefs_from(dir.join(prefs::FILE_NAME));
    h.state_mut().state.view_store = Some(dir.join(view::FILE_NAME));
    h.run();
    h.get_by_role_and_label(Role::Button, SETTINGS).click();
    h.run_steps(2);
    h.get_by_label(section.label()).click();
    h.run_steps(2);
    h
}

#[test]
fn the_settings_button_opens_and_closes_the_window() {
    let mut h = shell_harness();
    h.run();
    assert!(!window_open(&h), "closed on a fresh state");
    h.get_by_role_and_label(Role::Button, SETTINGS).click();
    h.run_steps(2);
    assert!(h.state().state.settings_open);
    assert!(window_open(&h), "the button opens the window");
    for s in Section::ALL {
        assert!(h.query_by_label(s.label()).is_some(), "section {:?}", s.label());
    }
    assert!(
        h.query_by_role_and_label(Role::MenuItem, settings::CHECK_FOR_UPDATES).is_none(),
        "the one-row dropdown is gone"
    );
    h.get_by_role_and_label(Role::Button, SETTINGS).click();
    h.run_steps(2);
    assert!(!window_open(&h), "a second press closes the window");
    assert!(h.state().state.pending.is_empty(), "opening the window is view state, never a command");
}

#[test]
fn the_menu_bar_settings_row_opens_the_window() {
    let mut h = shell_harness();
    h.run();
    h.get_by_role_and_label(Role::Button, "File menu").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Settings\u{2026}").click();
    h.run_steps(2);
    assert!(h.state().state.settings_open);
    assert!(window_open(&h), "File > Settings… opens the window");
}

#[test]
fn appearance_picks_apply_at_once_are_kept_and_reset() {
    let _g = snapshot_guard();
    let _restore = RestoreDark;
    let dir = temp_dir("look");
    let mut h = open_on(&dir, Section::Appearance);
    assert_eq!(combo_value(&h, settings::THEME).as_deref(), Some("Dark"));
    h.get_by_role_and_label(Role::ComboBox, settings::COLOR_VISION).click();
    h.run();
    h.get_by_label(ColorVision::Protan.label()).click();
    h.run_steps(3);
    assert_eq!(color_vision_of(&h.state().state.world), ColorVision::Protan);
    assert_eq!(prefs::load(&dir.join(prefs::FILE_NAME)).0.color_vision, ColorVision::Protan, "kept at once");
    h.get_by_role_and_label(Role::ComboBox, settings::THEME).click();
    h.run();
    h.get_by_label("Light").click();
    h.run_steps(3);
    let light = ThemeRef::Named("light".into());
    assert_eq!(theme_of(&h.state().state.world), light);
    assert_eq!(prefs::load(&dir.join(prefs::FILE_NAME)).0.theme, light, "kept at once");
    assert!(h.state().state.settings_open, "a pick keeps the window open");
    h.get_by_label(settings::RESET).click();
    h.run_steps(3);
    assert_eq!(theme_of(&h.state().state.world), ThemeRef::Dark);
    assert_eq!(color_vision_of(&h.state().state.world), ColorVision::Off);
    let saved = prefs::load(&dir.join(prefs::FILE_NAME)).0;
    assert_eq!((saved.theme, saved.color_vision), (ThemeRef::Dark, ColorVision::Off));
}

#[test]
fn graphics_reset_restores_the_line_width_and_the_antialiasing() {
    let dir = temp_dir("gfx");
    let file = dir.join(view::FILE_NAME);
    let mut h = open_on(&dir, Section::Graphics);
    h.get_by_label("3 pt").click();
    h.run_steps(2);
    h.get_by_label("Off").click();
    h.run_steps(2);
    assert_eq!((h.state().state.line_width, h.state().state.antialias), (LineWidth::Heavy, Antialias::Off));
    assert_eq!(view::load(&file).0.line_width, LineWidth::Heavy);
    h.get_by_label(settings::RESET).click();
    h.run_steps(2);
    assert_eq!((h.state().state.line_width, h.state().state.antialias), (LineWidth::Normal, Antialias::X4));
    let saved = view::load(&file).0;
    assert_eq!((saved.line_width, saved.antialias), (LineWidth::Normal, Antialias::X4), "the reset is kept");
}

#[test]
fn the_graphics_device_section_is_read_only() {
    let dir = temp_dir("device");
    let mut h = open_on(&dir, Section::Device);
    // Headless: no graphics device is in use.
    assert_eq!(h.query_all_by_label("Not known").count(), 2, "adapter and backend");
    assert!(h.query_by_label("1, 4").is_some(), "the headless sample counts");
    h.state_mut().state.gpu_line = Some("gpu: adapter \"Test GPU\" via Metal".into());
    h.state_mut().state.samples_offered = vec![1, 2, 4];
    h.run_steps(2);
    for label in [settings::ADAPTER, settings::BACKEND, settings::SAMPLES, "Test GPU", "Metal", "1, 2, 4"] {
        assert!(h.query_by_label(label).is_some(), "row {label:?}");
    }
    assert!(h.query_by_label(settings::RESET).is_none(), "nothing to reset");
}

#[test]
fn updates_reset_turns_the_check_on_again() {
    let dir = temp_dir("updates");
    let mut h = open_on(&dir, Section::Updates);
    h.get_by_label(settings::CHECK_FOR_UPDATES).click();
    h.run_steps(2);
    assert!(!h.state().state.check_for_updates);
    assert_eq!(row_check(&h, settings::CHECK_FOR_UPDATES).accesskit_node().toggled(), Some(Toggled::False));
    h.get_by_label(settings::RESET).click();
    h.run_steps(2);
    assert!(h.state().state.check_for_updates);
    assert!(prefs::load(&dir.join(prefs::FILE_NAME)).0.check_for_updates, "the reset is kept");
}

#[test]
fn old_prefs_and_view_files_load_with_the_same_values() {
    let _g = snapshot_guard();
    let _restore = RestoreDark;
    let dir = temp_dir("old");
    // Files as the build before #4493 wrote them: same formats, same keys.
    std::fs::write(
        dir.join(prefs::FILE_NAME),
        r#"{"format":"camcraft-prefs/1","colorVision":"protan","checkForUpdates":false}"#,
    )
    .expect("write prefs");
    std::fs::write(
        dir.join(view::FILE_NAME),
        r#"{"format":"camcraft-view/1","view":{"line_width":"2","antialias":"Off"}}"#,
    )
    .expect("write view");
    let mut h = shell_harness();
    h.state_mut().state.load_prefs_from(dir.join(prefs::FILE_NAME));
    h.state_mut().state.load_view_from(&dir.join(view::FILE_NAME));
    h.run();
    let st = &h.state().state;
    assert_eq!(color_vision_of(&st.world), ColorVision::Protan);
    assert!(!st.check_for_updates);
    assert_eq!((st.line_width, st.antialias), (LineWidth::Bold, Antialias::Off));
    h.get_by_role_and_label(Role::Button, SETTINGS).click();
    h.run_steps(2);
    assert_eq!(combo_value(&h, settings::COLOR_VISION).as_deref(), Some(ColorVision::Protan.label()));
    h.get_by_label(Section::Graphics.label()).click();
    h.run_steps(2);
    assert_eq!(h.get_by_label("2 pt").accesskit_node().toggled(), Some(Toggled::True));
    assert_eq!(h.get_by_label("Off").accesskit_node().toggled(), Some(Toggled::True));
    h.get_by_label(Section::Updates.label()).click();
    h.run_steps(2);
    assert_eq!(row_check(&h, settings::CHECK_FOR_UPDATES).accesskit_node().toggled(), Some(Toggled::False));
}
