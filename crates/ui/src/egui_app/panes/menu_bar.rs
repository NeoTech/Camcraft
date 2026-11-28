//! The in-app menu bar (#4484): File, Edit, Setup, Operation, Inspect and
//! View, read from the same command table as the ribbon; our own design
//! notes (desktop menu-bar convention), no third-party text.

use egui::{Rect, Ui};

use camcraft_app::machine::Kind;
use camcraft_app::operation;

use crate::state::State;

use crate::egui_app::painted;
use crate::egui_app::theme::Tokens;

use super::cmd::{Cmd, text_of, tip_of};
use super::gates::{disabled_reason, enabled};
use super::run::run_direct;
use super::table::{self, Menu, TABLE};

/// The menu bar's height: one control row and its margin, from the theme.
#[must_use]
pub fn menu_bar_h(t: &Tokens) -> f32 {
    (t.control_height + 4.0).max(t.font_row * 1.3 + 6.0)
}

/// The menu bar over the ribbon. Each row runs `run_direct`, the
/// keyboard's path into `crate::actions`, so the menu adds no state
/// change of its own (AGENTS.md "Everything Is a Command").
pub fn menu_bar(ui: &mut Ui, state: &mut State, pal: &Tokens) {
    let rect = Rect::from_min_size(ui.cursor().min, ui.available_size());
    painted::strip_bg(ui.painter(), rect, pal);
    let machine = camcraft_app::machine::kind_of(&state.world);
    egui::MenuBar::new().ui(ui, |ui| {
        ui.add_space(pal.item_spacing_x);
        for m in Menu::ALL {
            let r = ui.menu_button(m.title(), |ui| menu_body(ui, state, m, machine));
            // The bar's titles share words with other buttons (the browser's
            // View door, the Setup page): the AccessKit name says "menu".
            let name = format!("{} menu", m.title());
            ui.ctx().accesskit_node_builder(r.response.id, |b| b.set_label(name));
        }
    });
}

/// One open menu: the Operation menu's section submenus first, then the
/// table's rows for the menu, in table order.
fn menu_body(ui: &mut Ui, state: &mut State, m: Menu, machine: Kind) {
    if m == Menu::Operation {
        for (s, kinds) in sections(machine) {
            ui.menu_button(s.label(), |ui| {
                for i in kinds {
                    item(ui, state, Cmd::Create(i));
                }
            });
        }
        ui.separator();
    }
    for r in TABLE.iter().filter(|r| r.menu == Some(m)) {
        item(ui, state, r.cmd);
    }
}

/// The catalog sections the machine can run, each with every kind it
/// lists (the pinned ones too: the menu shows the whole catalog).
fn sections(machine: Kind) -> Vec<(operation::Section, Vec<usize>)> {
    operation::shown_for(machine)
        .into_iter()
        .map(|s| (s, s.kinds_for(machine)))
        .filter(|(_, kinds)| !kinds.is_empty())
        .collect()
}

/// One menu row: the ribbon's label, the table's shortcut hint, the
/// ribbon's enabled rule, and the reason on hover when it is disabled.
fn item(ui: &mut Ui, state: &mut State, c: Cmd) {
    let live = enabled(c, state);
    let tip = tip_of(c, state);
    let hover = match disabled_reason(c, state) {
        Some(why) => why.to_owned(),
        None => tip.split('\t').next().unwrap_or_default().to_owned(),
    };
    let mut button = egui::Button::new(text_of(c, state));
    if let Some(k) = table::key_hint(c) {
        button = button.shortcut_text(k);
    }
    let resp = ui
        .add_enabled(live, button)
        .on_hover_text(&hover)
        .on_disabled_hover_text(&hover);
    if resp.clicked() {
        run_direct(c, state);
    }
}

/// Every command the menu bar lists for a machine: the section doors and
/// their kinds, then the table's menu rows. Pure, for the table tests.
#[cfg(test)]
pub(in crate::egui_app) fn menu_cmds(machine: Kind) -> Vec<Cmd> {
    let mut out = Vec::new();
    for (s, kinds) in sections(machine) {
        out.push(Cmd::More(s));
        out.extend(kinds.into_iter().map(Cmd::Create));
    }
    out.extend(TABLE.iter().filter(|r| r.menu.is_some()).map(|r| r.cmd));
    out
}

/// Every command the ribbon shows for a machine and its pins: the cells,
/// the section doors and the kinds behind them.
#[cfg(test)]
pub(in crate::egui_app) fn ribbon_cmds(machine: Kind, pinned: &[usize]) -> Vec<Cmd> {
    let mut out = Vec::new();
    for g in super::cmd::ribbon_plan(machine, pinned) {
        if let Some(s) = g.section {
            out.push(Cmd::More(s));
        }
        out.extend(g.cells);
    }
    out
}
