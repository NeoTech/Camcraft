//! Help and the nonmodal start-page tip (#3030).
//! Provenance: our own UI over the shared MCP tips and per-user prefs.

use camcraft_app::mcp::tips::{self, Tip};
use egui::{Button, Rect, RichText, Ui};

use super::{browser_open, layout, theme::Tokens};
use crate::state::State;

/// The top-band Help door. Popup state belongs to egui, not the document.
pub(super) fn help_button(
    ui: &mut Ui,
    state: &mut State,
    pal: &Tokens,
    band: Rect,
    right: f32,
) -> f32 {
    let text = RichText::new("Help").size(pal.font_label).color(pal.text);
    let width = ui
        .painter()
        .layout_no_wrap(
            "Help".into(),
            egui::FontId::proportional(pal.font_label),
            pal.text,
        )
        .size()
        .x
        + 2.0 * pal.button_padding_x
        + 8.0;
    let rect = Rect::from_min_max(
        egui::pos2(right - width, band.top() + 2.0),
        egui::pos2(right, band.bottom() - 2.0),
    );
    let response = ui.put(rect, Button::new(text));
    egui::Popup::menu(&response).show(|ui| {
        if layout::button(ui, "Tips").clicked() {
            state.tips_open = true;
            ui.close();
        }
    });
    rect.left()
}

fn entry(ui: &mut Ui, state: &State, tip: &Tip) {
    let os = if ui.ctx().os().is_mac() {
        "macos"
    } else {
        "linux"
    };
    let tip = tip.for_os(os);
    ui.strong(&tip.title);
    layout::list_row(ui, &tip.text);
    if let Some(url) = &tip.guide_link {
        let id = egui::Id::new(("tip-guide", &tip.id));
        if layout::button(ui, format!("Show me: {}", tip.title))
            .on_hover_text(url)
            .clicked()
        {
            browser_open::open(ui, state.browser_build, "tip-guide", id, url);
        }
        match browser_open::take_result(ui.ctx(), id) {
            Some(Err(why)) => {
                ui.ctx().data_mut(|d| d.insert_temp(id.with("error"), why));
            }
            Some(Ok(())) => {
                ui.ctx().data_mut(|d| d.remove::<String>(id.with("error")));
            }
            None => {}
        }
        if let Some(why) = ui.ctx().data(|d| d.get_temp::<String>(id.with("error"))) {
            layout::list_row(ui, format!("Guide not opened: {why}"));
        }
    }
}

/// A user-requested, closable window; never a modal or an automatic popup.
pub(super) fn window(ctx: &egui::Context, state: &mut State) {
    if !state.tips_open {
        return;
    }
    let _span = crate::frame_cost::span("ui/tips");
    let mut open = state.tips_open;
    egui::Window::new("Tips")
        .open(&mut open)
        .default_width(500.0)
        .vscroll(true)
        .show(ctx, |ui| {
            match tips::all() {
                Ok(tips) => {
                    for tip in tips {
                        ui.push_id(&tip.id, |ui| entry(ui, state, tip));
                        ui.separator();
                    }
                }
                Err(why) => {
                    layout::list_row(ui, why);
                }
            }
            layout::list_row(
                ui,
                "This preference hides only the start-screen tip. Help > Tips stays available.",
            );
            preference(ui, state);
        });
    state.tips_open = open;
}

fn preference(ui: &mut Ui, state: &mut State) {
    let mut hide = !state.show_start_tip;
    if layout::check_row(ui, "Don't show again", &mut hide).changed() {
        state.show_start_tip = !hide;
        state.persist_prefs();
    }
}

/// Inline after the start page's normal actions, never over the workspace.
pub(super) fn start_tip(ui: &mut Ui, state: &mut State) {
    if !state.show_start_tip
        || state.gen_jobs.busy()
        || state.import_in_progress()
        || state.ai_chat.is_running()
        || crate::play_precalc::holds_playhead(state)
    {
        return;
    }
    let _span = crate::frame_cost::span("ui/start-tip");
    ui.separator();
    ui.heading("Tip of the day");
    match tips::all() {
        Ok(tips) if !tips.is_empty() => {
            // One stable selection per UTC day, without random state.
            #[cfg(not(target_arch = "wasm32"))]
            let day = camcraft_app::update_check::current_unix_secs().unwrap_or(0) / 86_400;
            // Browser builds have no system clock syscall; keep the first tip.
            #[cfg(target_arch = "wasm32")]
            let day = 0;
            let index = (day % tips.len() as u64) as usize;
            entry(ui, state, &tips[index]);
        }
        Ok(_) => {}
        Err(why) => {
            layout::list_row(ui, why);
        }
    }
    preference(ui, state);
}
