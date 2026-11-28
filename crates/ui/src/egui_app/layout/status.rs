//! The page status and actions of the property-page form law (#1122,
//! split from `form.rs` for the 400-line law and as the home of the
//! page frame that the UI plan #4245 adds): the one-line coloured
//! status banner that expands on click ([`status_banner`]) and the
//! bottom action bar ([`action_bar`]). Immediate-mode layout only: a
//! click is returned to the page, which queues its command.
//!
//! Provenance: our own layout code over egui's layouts (the vendored
//! egui 0.36 docs). No vendor material, strings or layouts copied.

use egui::{Align, Button, Color32, Label, Layout, RichText, Ui, Vec2};

/// How a status banner reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Ok,
    Warn,
    Error,
    Quiet,
}

/// The one-line coloured status at the top of a page: `line` in its
/// tone's ink; with `details`, a click expands them as a compact list
/// (view state, never a command).
pub fn status_banner(ui: &mut Ui, id: &str, line: &str, ink: Color32, details: &[String]) {
    let _ = status_banner_located(ui, id, line, ink, details, &[]);
}

/// [`status_banner`] whose detail lines can be links (#261): a detail
/// `i` with `links[i]` true reads as a link (hover says so) and a click
/// on it returns `Some(i)` - the caller decides what it shows. A
/// missing `links` entry is plain text.
pub fn status_banner_located(ui: &mut Ui, id: &str, line: &str, ink: Color32, details: &[String], links: &[bool]) -> Option<usize> {
    let key = ui.make_persistent_id(("camcraft.banner", id));
    let mut open: bool = ui.ctx().data(|d| d.get_temp(key)).unwrap_or(false);
    let arrow = if details.is_empty() {
        ""
    } else if open {
        "v "
    } else {
        "> "
    };
    let text = RichText::new(format!("{arrow}{line}")).color(ink).strong();
    let r = ui.add(Label::new(text).sense(egui::Sense::click()).truncate());
    if !details.is_empty() && r.clicked() {
        open = !open;
        ui.ctx().data_mut(|d| d.insert_temp(key, open));
    }
    if !details.is_empty() {
        r.on_hover_text(if open { "hide the details" } else { "show the details" });
    }
    let mut hit = None;
    if open {
        ui.indent(key, |ui| {
            for (i, d) in details.iter().enumerate() {
                if links.get(i).copied().unwrap_or(false) {
                    let r = ui.add(Label::new(RichText::new(d).small().underline()).sense(egui::Sense::click()).wrap());
                    if r.on_hover_text("show this move in the viewport").clicked() {
                        hit = Some(i);
                    }
                } else {
                    ui.add(Label::new(RichText::new(d).small()).wrap());
                }
            }
        });
    }
    hit
}

/// An action bar button's role.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionKind {
    Primary,
    Secondary,
    Destructive,
}

/// One action bar button.
pub struct Action<'a> {
    pub label: &'a str,
    pub kind: ActionKind,
    pub enabled: bool,
    /// Hover text while enabled.
    pub hint: Option<&'a str>,
    /// Why it is disabled (hover text while disabled).
    pub why_not: Option<&'a str>,
}

/// The inks the bar paints with (theme tokens and the colour-vision
/// palette - never literals).
pub struct BarInks {
    pub accent: Color32,
    pub on_accent: Color32,
    pub danger: Color32,
}

/// The bottom action bar: the primary action filled with the accent,
/// the secondaries plain, the destructive ones pushed to the far right
/// in the danger ink. Returns the index of the clicked action.
pub fn action_bar(ui: &mut Ui, actions: &[Action<'_>], inks: &BarInks) -> Option<usize> {
    let mut clicked = None;
    ui.separator();
    let draw = |ui: &mut Ui, i: usize, a: &Action<'_>, clicked: &mut Option<usize>| {
        let btn = match a.kind {
            ActionKind::Primary => Button::new(RichText::new(a.label).strong().color(inks.on_accent)).fill(inks.accent),
            ActionKind::Secondary => Button::new(a.label),
            ActionKind::Destructive => {
                Button::new(RichText::new(a.label).color(inks.danger)).stroke(egui::Stroke::new(1.0, inks.danger))
            }
        };
        let mut r = ui.add_enabled(a.enabled, btn.min_size(Vec2::new(0.0, 24.0)));
        if let Some(h) = a.hint {
            r = r.on_hover_text(h);
        }
        if let Some(w) = a.why_not {
            r = r.on_disabled_hover_text(w);
        }
        if r.clicked() {
            *clicked = Some(i);
        }
    };
    // The working verbs wrap on a narrow pane; the destructive ones sit
    // apart on their own row, at the right.
    ui.horizontal_wrapped(|ui| {
        for (i, a) in actions.iter().enumerate().filter(|(_, a)| a.kind != ActionKind::Destructive) {
            draw(ui, i, a, &mut clicked);
        }
    });
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        for (i, a) in actions.iter().enumerate().filter(|(_, a)| a.kind == ActionKind::Destructive) {
            draw(ui, i, a, &mut clicked);
        }
    });
    clicked
}
