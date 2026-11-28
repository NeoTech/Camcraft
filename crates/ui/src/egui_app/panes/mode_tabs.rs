//! The machine-mode tabs above the ribbon cells and the machine door's
//! dropdown, split from panes.rs (#4393); our own design notes.

use egui::{Rect, Sense, Ui, Vec2};

use crate::machine_modes;
use crate::state::State;

use crate::egui_app::painted;
use crate::egui_app::theme::Tokens;

use super::cmd::Cmd;
use super::ribbon_layout::{Cell, machine_menu_width, machine_rows};

/// The workspace tab strip above the ribbon cells: one
/// tab per machine mode in `machine_modes::MACHINE_MODES`, the running
/// kind's tab joined to the strip below and underlined in the accent,
/// the rest quiet until hover. A click is the Machine door's own
/// undoable `LoadBuiltinMachine` - a second door onto one switch,
/// never a second switch. A document whose machine has no mode row
/// (an imported wire-EDM machine) leaves every tab inactive; the mode
/// switch still works, and the status line names the mounted kind.
pub(super) fn ribbon_tabs(ui: &mut Ui, state: &mut State, pal: &Tokens, strip: Rect, band: Rect) -> f32 {
    ui.painter().rect_filled(band, 0.0, pal.ink);
    ui.painter().line_segment(
        [
            egui::Pos2::new(band.left(), band.bottom() - 0.5),
            egui::Pos2::new(band.right(), band.bottom() - 0.5),
        ],
        egui::Stroke::new(1.0, pal.border),
    );
    let current = camcraft_app::machine::kind_of(&state.world);
    let font = egui::FontId::proportional(pal.font_label);
    let mut x = strip.left() + pal.item_spacing_x;
    for (kind, caption, machine) in machine_modes::MACHINE_MODES {
        let active = kind == current;
        let ink = if active { pal.text } else { pal.text_dim };
        let galley = ui
            .painter()
            .layout_no_wrap(caption.to_uppercase(), font.clone(), ink);
        let w = galley.size().x + pal.ribbon_tab_pad_x * 2.0;
        let r = Rect::from_min_max(
            egui::Pos2::new(x, band.top()),
            egui::Pos2::new(x + w, band.bottom()),
        );
        let resp = ui.interact(r, ui.id().with(("ribbon-tab", caption)), Sense::click());
        let painter = ui.painter();
        if active {
            painter.rect_filled(
                Rect::from_min_max(r.min, egui::Pos2::new(r.right(), r.bottom() - 1.0)),
                0.0,
                pal.strip_fill(),
            );
            let pad = (pal.ribbon_tab_pad_x * 0.5).min(w * 0.25);
            painter.line_segment(
                [
                    egui::Pos2::new(r.left() + pad, r.bottom() - 1.0),
                    egui::Pos2::new(r.right() - pad, r.bottom() - 1.0),
                ],
                egui::Stroke::new(pal.ribbon_tab_stroke, pal.accent),
            );
        } else if resp.clicked() || resp.is_pointer_button_down_on() {
            painter.rect_filled(r, pal.radius_widget, pal.accent);
        } else if resp.hovered() {
            painter.rect_filled(r, pal.radius_widget, pal.field_hover);
        }
        painter.galley(
            egui::Pos2::new(
                r.center().x - galley.size().x * 0.5,
                r.center().y - galley.size().y * 0.5,
            ),
            galley,
            ink,
        );
        let clicked = resp.clicked();
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, caption));
        ui.ctx().accesskit_node_builder(resp.id, |b| {
            b.set_role(egui::accesskit::Role::Tab);
            b.set_selected(active);
        });
        resp.on_hover_text(format!("{caption} workspace - mounts {machine} (undoable)"));
        if clicked {
            machine_modes::set_machine_mode(state, kind);
        }
        x += w + pal.ribbon_tab_gap;
    }
    x
}

/// The machine door's open dropdown, drawn by `ribbon` after the cells.
pub(super) fn machine_drop(
    ui: &mut Ui,
    state: &mut State,
    pal: &Tokens,
    base: egui::Id,
    cells: &[Cell],
    rect: Rect,
) {
    // The machine door's dropdown (the ribbon law): the workspaces
    // of the mode table, the current one marked and inert, the others
    // mounting their kind's library reference - through the queue,
    // so the switch rides undo. Same hand-drawn popup law as the
    // section doors: foreground order, outside press closes it.
    if state.ribbon_mode_open {
        match cells.iter().find(|c| c.cmd == Cmd::Machine) {
            None => {
                // The anchor law says the door stands on the bar every
                // frame; finding none means that broke - named, not
                // silent, and the state drops itself.
                debug_assert!(false, "ribbon: machine door missing from the bar");
                diag_warn!("ribbon: closing a mode list whose button is gone");
                state.ribbon_mode_open = false;
            }
            Some(cell) => {
                let button = cell.rect;
                // The menu is as wide as its longest row, the current
                // row's "(current)" tail included.
                let longest = ui.ctx().fonts_mut(|f| {
                    machine_modes::MACHINE_MODES
                        .iter()
                        .map(|(k, _, _)| {
                            let text = format!("{}  (current)", machine_modes::mode_label(*k));
                            f.layout_no_wrap(text, egui::FontId::proportional(pal.font_row), pal.text).size().x
                        })
                        .fold(0.0_f32, f32::max)
                });
                let g = pal.dropdown();
                let width = g.fit_width(machine_menu_width(longest, pal), rect.width());
                let rows = machine_rows(button, rect, pal.row_height, width, &g);
                let machine = camcraft_app::machine::kind_of(&state.world);
                let mut pick: Option<camcraft_app::machine::Kind> = None;
                let mut chose = false;
                let panel = Rect::from_min_size(
                    rows.first().map_or(button.left_bottom(), |(_, r)| r.min),
                    Vec2::new(width, rows.len() as f32 * pal.row_height),
                );
                egui::Area::new(base.with("machine-drop"))
                    .fixed_pos(panel.min)
                    .order(egui::Order::Foreground)
                    .movable(false)
                    .show(ui.ctx(), |ui| {
                        let p = ui.painter();
                        p.rect_filled(panel, pal.radius_menu, pal.ink);
                        p.rect_stroke(
                            panel,
                            pal.radius_menu,
                            egui::Stroke::new(1.0, pal.border),
                            egui::StrokeKind::Outside,
                        );
                        for (k, (kind, r)) in rows.iter().enumerate() {
                            let resp =
                                ui.interact(*r, base.with(("mode-row", k)), Sense::click());
                            if resp.hovered() {
                                p.rect_filled(*r, 0.0, pal.field_hover);
                            }
                            let current = *kind == machine;
                            let label = if current {
                                format!("{}  (current)", machine_modes::mode_label(*kind))
                            } else {
                                machine_modes::mode_label(*kind)
                            };
                            painted::menu_item(&resp, &label, !current, Some(current));
                            p.text(
                                r.left_center() + Vec2::new(g.pad_x, 0.0),
                                egui::Align2::LEFT_CENTER,
                                label,
                                egui::FontId::proportional(pal.font_row),
                                if current { pal.text_faint } else { pal.text },
                            );
                            // A press on the current row is a choice
                            // too - it closes the list, it just
                            // mounts nothing.
                            if resp.clicked() {
                                chose = true;
                                if !current {
                                    pick = Some(*kind);
                                }
                            }
                        }
                    });
                if chose {
                    if let Some(k) = pick {
                        machine_modes::set_machine_mode(state, k);
                    }
                    state.ribbon_mode_open = false;
                } else {
                    let outside = ui.input(|i| {
                        i.pointer.button_pressed(egui::PointerButton::Primary)
                            && i.pointer.interact_pos().is_none_or(|p| {
                                !button.expand(g.gap).contains(p)
                                    && !rows.iter().any(|(_, r)| r.contains(p))
                            })
                    });
                    if outside {
                        state.ribbon_mode_open = false;
                    }
                }
            }
        }
    }
}
