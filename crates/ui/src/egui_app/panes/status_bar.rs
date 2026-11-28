//! The status strip at the bottom of the shell, split from panes.rs
//! (#4393, the 400-line law); our own design notes.

use egui::{Label, Rect, RichText, Sense, Ui};

use camcraft_app::model;
use camcraft_app::operation;

use crate::state::{CommandKind, State};

use crate::egui_app::icons::IconSet;
use crate::egui_app::painted;
use crate::egui_app::theme::Tokens;

/// The status strip (bottom panel): the shell's one-line feedback, dimmer,
/// with the chrome the painted ground gives it. Since M-d this is the
/// file-op outcome line too ("saved …", "open failed: …" - State::run
/// writes them all), while command refusals read twice by design: here for
/// everyone, and on their own pane through `error_for` (M-c). Startup is a
/// plain empty document; an untouched status reads "ready".
pub fn status(ui: &mut Ui, state: &mut State, pal: &Tokens, icons: Option<&IconSet>) {
    let rect = Rect::from_min_size(ui.cursor().min, ui.available_size());
    painted::strip_bg(ui.painter(), rect, pal);
    // Owned so the button below can reborrow `state` mutably while the
    // label still shows the text (disjoint fields - cheap either way).
    let text = if state.status.is_empty() {
        "ready".to_owned()
    } else {
        state.status.clone()
    };
    // M3.1 C: the batch fix rides the bar whenever a revision has left
    // operations Stale - the tree marks them, the page says why, this is
    // the one click that brings the job current.
    let stale = model::named_entities(&state.world)
        .into_iter()
        .filter(|(_, e)| {
            model::is_operation(&state.world, e)
                && operation::status_of(&state.world, e) == operation::OpStatus::Stale
        })
        .count();
    let ops_n = model::named_entities(&state.world)
        .into_iter()
        .filter(|(_, e)| model::is_operation(&state.world, e))
        .count();
    // #192: a refusal on the status line must not read like any other
    // status - error ink, bold, a "!" marker, on an error-tinted band.
    let alert = crate::egui_app::status_alert::is_alert(state);
    let line = if alert {
        let ink = crate::egui_app::colorvision::text_inks(ui, &state.world).error;
        ui.painter().rect_filled(rect, 0.0, ink.gamma_multiply(0.16));
        RichText::new(format!("! {text}")).color(ink).strong()
    } else {
        // #215: routine status in the body text ink, not egui's "weak"
        // grey - that read as near-invisible on the dark strip.
        RichText::new(text.as_str()).color(ui.visuals().text_color())
    };
    ui.horizontal(|ui| {
        // The right-hand widgets are laid out first so a long status line
        // can never push them off the strip (#157: the Debug info door
        // must stay reachable); the line takes what is left, truncated,
        // with the full text on hover and in "Copy message".
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // #157: the Debug info door, rightmost - the glyph when the
            // icon set is up, its name when it is not (headless frames
            // and a failed decode degrade to text, never to nothing).
            if crate::egui_app::debug_info::button(ui, icons).clicked() {
                // #2262: a dock tab like the other panels; the shell lands it.
                state.layout_request = Some(crate::state::LayoutRequest::ToggleDebugInfo);
            }
            // #1696: the Report door, next to it - opens the window that
            // fills GitHub's new-issue form in (the forms' promised
            // in-app "Report an Issue").
            if crate::egui_app::report::button(ui, icons).clicked() {
                state.report_open = !state.report_open;
            }
            // #183: the live MCP server says it is listening.
            if let Some(note) = crate::mcp_host::badge(state) {
                ui.label(RichText::new(note).color(pal.text_faint).small());
            }
            if stale > 0 {
                let button = ui.button(format!("Regenerate all stale ({stale})"));
                if button.clicked() {
                    state.pending.push_back(CommandKind::RegenerateAll);
                }
            }
            // #2262: placed before the status line, which fills what is
            // left; behind it the label ran past the line onto the door.
            if ops_n > 0 {
                ui.label(RichText::new(crate::egui_app::browser_tree::count(ops_n, "Operation")).color(pal.text_faint).small());
            }
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.add(Label::new(line).selectable(true).truncate())
                    .on_hover_text(text.as_str())
                    .context_menu(|ui| {
                        if ui.button("Copy message").clicked() {
                            ui.ctx().copy_text(text.clone());
                            ui.close();
                        }
                    });
            });
        });
    });
    ui.allocate_rect(rect, Sense::hover());
}
