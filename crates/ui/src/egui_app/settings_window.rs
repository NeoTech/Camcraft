//! The Settings window (#4493, design on #3025): one in-app window for the
//! per-user application settings, in four sections - Appearance, Graphics,
//! Graphics device (read-only) and Updates. Each choice applies at once and
//! goes to the per-user file (`prefs.json`, `view.json`), outside the
//! project file and the undo history (#877). Each section that holds a
//! setting has "Reset to defaults". The ribbon's Settings button and the
//! menu bar's "Settings…" row open it.
//!
//! Provenance: our own window, written from the section list of issues
//! #3025 and #4493 and common desktop settings-dialog practice; our own
//! wording. No vendor material.

use camcraft_app::commands::{color_vision_of, theme_of};
use camcraft_app::schema::{ColorVision, ThemeRef};
use camcraft_app::view::{Antialias, LineWidth};
use egui::{ComboBox, Ui};

use super::layout;
use crate::state::{CommandKind, State};

/// The window title and its AccessKit name.
pub(crate) const TITLE: &str = "Settings";
/// The reset button of each section that holds a setting.
pub(crate) const RESET: &str = "Reset to defaults";
/// The Appearance rows.
pub(crate) const THEME: &str = "Colour theme";
pub(crate) const COLOR_VISION: &str = "Colour vision";
/// The Graphics rows (#2958; they were in the Display menu, #2818).
pub(crate) const LINE_WIDTH: &str = "Line width";
pub(crate) const ANTIALIAS: &str = "Anti-aliasing";
/// The Graphics device rows.
pub(crate) const ADAPTER: &str = "Adapter";
pub(crate) const BACKEND: &str = "Backend";
pub(crate) const SAMPLES: &str = "Sample counts";
/// The Updates row (#2428; it was the Settings dropdown's one row).
pub(crate) const CHECK_FOR_UPDATES: &str = "Check for updates";

/// One section of the window, in list order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Section {
    #[default]
    Appearance,
    Graphics,
    Device,
    Updates,
}

impl Section {
    pub(crate) const ALL: [Section; 4] = [Section::Appearance, Section::Graphics, Section::Device, Section::Updates];

    /// The section's name in the list.
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Section::Appearance => "Appearance",
            Section::Graphics => "Graphics",
            Section::Device => "Graphics device",
            Section::Updates => "Updates",
        }
    }
}

/// Open the window (view state, not a command). The ribbon's dropdowns
/// close, so no list stays open over it.
pub(crate) fn open(state: &mut State) {
    state.settings_open = true;
    state.ribbon_more = None;
    state.ribbon_mode_open = false;
    state.ribbon_view_open = false;
    state.ribbon_theme_open = false;
    state.ribbon_recent_open = None;
}

/// The entry label of a line width.
#[must_use]
pub(crate) fn line_label(w: LineWidth) -> String {
    format!("{} pt", w.word())
}

/// Why an anti-aliasing choice cannot be picked on this device, if it cannot.
pub(crate) fn antialias_refusal(state: &State, a: Antialias) -> Option<String> {
    (!state.samples_offered.contains(&a.samples()))
        .then(|| format!("This graphics device does not offer {} anti-aliasing.", a.word()))
}

/// The adapter and the backend from the startup GPU line (`gpu: adapter
/// "<name>" via <backend>[ — ...]`, `gpu::GpuInfo::log_line`). `None` when
/// no graphics device is in use (a headless run) or the line has another
/// form.
#[must_use]
pub(crate) fn device_of(gpu_line: Option<&str>) -> Option<(String, String)> {
    let rest = gpu_line?.strip_prefix("gpu: adapter \"")?;
    let (adapter, rest) = rest.split_once("\" via ")?;
    let backend = rest.split(" — ").next().unwrap_or(rest).trim();
    Some((adapter.to_owned(), backend.to_owned()))
}

/// The sample counts that the device offers, as one line ("1, 4").
#[must_use]
pub(crate) fn samples_line(offered: &[u32]) -> String {
    if offered.is_empty() {
        return "None".to_owned();
    }
    offered.iter().map(u32::to_string).collect::<Vec<_>>().join(", ")
}

/// Draw the window while it is open. The title bar's close button closes it.
pub(crate) fn window(ctx: &egui::Context, state: &mut State) {
    if !state.settings_open {
        return;
    }
    let mut open = true;
    egui::Window::new(TITLE)
        .id(egui::Id::new("settings-window"))
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_size([520.0, 300.0])
        .show(ctx, |ui| {
            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(130.0);
                    for s in Section::ALL {
                        if ui.selectable_label(state.settings_section == s, s.label()).clicked() {
                            state.settings_section = s;
                        }
                    }
                });
                ui.separator();
                ui.vertical(|ui| match state.settings_section {
                    Section::Appearance => appearance(ui, state),
                    Section::Graphics => graphics(ui, state),
                    Section::Device => device(ui, state),
                    Section::Updates => updates(ui, state),
                });
            });
        });
    if !open {
        state.settings_open = false;
    }
}

/// The reset button at the foot of a section.
fn reset_button(ui: &mut Ui, help: &str) -> bool {
    ui.add_space(layout::GROUP_GAP);
    ui.button(RESET).on_hover_text(help).clicked()
}

/// Theme and colour vision. A pick queues the same `SelectTheme` /
/// `SelectColorVision` that the Theme door queues (theme_drop.rs).
fn appearance(ui: &mut Ui, state: &mut State) {
    let theme = theme_of(&state.world);
    let vision = color_vision_of(&state.world);
    let mut chose: Option<CommandKind> = None;
    let options = state.theme_options();
    let selected = state.theme_selection_text();
    layout::prop_row_hint(ui, THEME, "The colours of the whole application.", |ui| {
        let r = ComboBox::from_id_salt("settings-theme").selected_text(selected).show_ui(ui, |ui| {
            for (label, cmd) in &options {
                let current = matches!(cmd, CommandKind::SelectTheme(r) if *r == theme);
                let why = if state.browser_build { crate::browser_files::refusal(cmd, false) } else { None };
                let r = ui.add_enabled(why.is_none(), egui::Button::selectable(current, label.as_str()));
                let r = match why {
                    Some(why) => r.on_disabled_hover_text(why),
                    None => r,
                };
                if r.clicked() && !current {
                    chose = Some(cmd.clone());
                }
            }
        });
        layout::name_by_row(ui, r.response);
    });
    layout::prop_row_hint(ui, COLOR_VISION, "Remap the colours for a colour-vision deficiency.", |ui| {
        let r = ComboBox::from_id_salt("settings-vision").selected_text(vision.label()).show_ui(ui, |ui| {
            for mode in ColorVision::ALL {
                if ui.selectable_label(mode == vision, mode.label()).clicked() && mode != vision {
                    chose = Some(CommandKind::SelectColorVision(mode));
                }
            }
        });
        layout::name_by_row(ui, r.response);
    });
    if let Some(cmd) = chose {
        state.pending.push_back(cmd);
    }
    if reset_button(ui, "Use the Dark theme and no colour-vision remap.") {
        let d = camcraft_app::prefs::ViewPrefs::default();
        if theme != d.theme {
            state.pending.push_back(CommandKind::SelectTheme(d.theme));
        }
        if vision != d.color_vision {
            state.pending.push_back(CommandKind::SelectColorVision(d.color_vision));
        }
    }
}

/// #2958: the overlay line width and the anti-aliasing. Each pick is kept
/// at once by its `State` method (which also refuses an unoffered count).
fn graphics(ui: &mut Ui, state: &mut State) {
    layout::prop_row_hint(ui, LINE_WIDTH, "The width of toolpaths, height planes, outlines and axes.", |ui| {
        for w in LineWidth::ALL.into_iter().rev() {
            if ui.radio(state.line_width == w, line_label(w)).clicked() && state.line_width != w {
                state.select_line_width(w);
            }
        }
    });
    layout::prop_row_hint(
        ui,
        ANTIALIAS,
        "Smooths the edges of models and stock. 4x uses the most graphics memory.",
        |ui| {
            for a in Antialias::ALL.into_iter().rev() {
                let why = antialias_refusal(state, a);
                let r = ui.add_enabled(why.is_none(), egui::RadioButton::new(state.antialias == a, a.word()));
                let r = match &why {
                    Some(why) => r.on_disabled_hover_text(why),
                    None => r,
                };
                if r.clicked() && state.antialias != a {
                    state.select_antialias(a);
                }
            }
        },
    );
    if reset_button(ui, "Use a 1.5 pt line width and 4x anti-aliasing.") {
        if state.line_width != LineWidth::default() {
            state.select_line_width(LineWidth::default());
        }
        if state.antialias != Antialias::default() {
            state.select_antialias(Antialias::default());
        }
    }
}

/// The graphics device the viewport runs on: read-only.
fn device(ui: &mut Ui, state: &State) {
    let (adapter, backend) = device_of(state.gpu_line.as_deref())
        .unwrap_or_else(|| ("Not known".to_owned(), "Not known".to_owned()));
    layout::prop_row(ui, ADAPTER, |ui| ui.label(adapter));
    layout::prop_row(ui, BACKEND, |ui| ui.label(backend));
    layout::prop_row_hint(ui, SAMPLES, "The anti-aliasing sample counts that this device offers.", |ui| {
        ui.label(samples_line(&state.samples_offered))
    });
    layout::hint(ui, "Read-only: the device is chosen when CamCraft starts.");
}

/// The startup update check (#2428).
fn updates(ui: &mut Ui, state: &mut State) {
    let mut on = state.check_for_updates;
    let r = layout::check_row(ui, CHECK_FOR_UPDATES, &mut on)
        .on_hover_text("Check GitHub for new releases once per day on startup");
    if r.changed() {
        set_check_for_updates(state, on);
    }
    if reset_button(ui, "Check for updates on startup.") && !state.check_for_updates {
        set_check_for_updates(state, true);
    }
}

/// Keep the update-check choice; off also hides a shown update notice.
fn set_check_for_updates(state: &mut State, on: bool) {
    state.check_for_updates = on;
    if !on {
        state.update_available = None;
    }
    state.persist_prefs();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_of_reads_the_adapter_and_the_backend() {
        assert_eq!(
            device_of(Some("gpu: adapter \"Test GPU\" via Metal")),
            Some(("Test GPU".to_owned(), "Metal".to_owned()))
        );
        assert_eq!(
            device_of(Some("gpu: adapter \"Soft\" via GL — gl path: conservative feature set")),
            Some(("Soft".to_owned(), "GL".to_owned()))
        );
        assert_eq!(device_of(None), None);
        assert_eq!(device_of(Some("something else")), None);
    }

    #[test]
    fn samples_line_joins_the_counts() {
        assert_eq!(samples_line(&[1, 4]), "1, 4");
        assert_eq!(samples_line(&[]), "None");
    }
}
