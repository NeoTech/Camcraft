//! The open section door's dropdown under the ribbon, split from panes.rs
//! (#4393, the 400-line law); our own design notes.

use egui::{Rect, Sense, Ui, Vec2};

use camcraft_app::operation;

use crate::state::State;

use crate::egui_app::painted;
use crate::egui_app::theme::Tokens;

use super::TYPES;
use super::cmd::{Cmd, label_of, tip_of};
use super::gates::{enabled, follow_gate};
use super::ribbon_layout::Drop;
use super::run::{run_cmd, toggle_pin};

/// The open section door's dropdown, drawn by `ribbon` after the cells.
/// `before` is the open door at the start of the frame.
pub(super) fn section_drop(
    ui: &mut Ui,
    state: &mut State,
    pal: &Tokens,
    base: egui::Id,
    drops: &[Drop],
    before: Option<operation::Section>,
) {
    // The open door's dropdown: one Area at foreground order (egui's
    // popup layer), anchored under its button, its rows the section's
    // hidden kinds in registry
    // order. Geometry arrives global (ribbon_cells); the Area-local
    // rect is the global one minus the panel origin - one pair of
    // numbers, no second layout.
    if let Some(s) = state.ribbon_more {
        // The self-heal above proved a door is on the ribbon; finding
        // none means that proof broke, so the state drops itself -
        // named, printed, never a silent unwrap.
        match drops.iter().find(|d| d.section == s) {
            None => {
                debug_assert!(false, "ribbon: open door has no dropdown");
                diag_warn!("ribbon: closing a dropdown whose button is gone");
                state.ribbon_more = None;
            }
            Some(drop) => {
                // Every rect here is global: ribbon_cells computed the
                // button and row rects in the strip's global frame, and
                // egui paints and hit-tests this ui in global coords
                // too. The Area adds only the layer - Foreground order,
                // so a press on a row belongs to the row, never to the
                // cell it happens to cover.
                let g = pal.dropdown();
                let top = drop.rows.first().map_or(drop.button.left_bottom(), |(_, r)| r.min);
                let width = drop.rows.first().map_or(0.0, |(_, r)| r.width());
                let panel = Rect::from_min_size(
                    top, Vec2::new(width, drop.rows.len() as f32 * pal.row_height),
                );
                let mut hit: Option<Cmd> = None;
                let mut gated_row: Option<Cmd> = None;
                let mut pinned_row: Option<usize> = None;
                let rows = &drop.rows;
                egui::Area::new(base.with(("drop", drop.section.label())))
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
                        for (k, (cmd, r)) in rows.iter().enumerate() {
                            let resp = ui.interact(*r, base.with(("drop-row", k)), Sense::click());
                            if resp.hovered() {
                                p.rect_filled(*r, 0.0, pal.field_hover);
                            }
                            let en = enabled(*cmd, state);
                            let label = match *cmd {
                                Cmd::Create(i) => TYPES.get(i).copied().unwrap_or("op"),
                                other => label_of(other),
                            };
                            painted::menu_item(&resp, label, en, None);
                            // The kind's own glyph at the row's left edge,
                            // in the row's ink - the door lists operations
                            // the way the tree shows them. Headless frames
                            // (no icon set) keep the theme's row padding;
                            // the icon's inset, size and label clearance
                            // come from the same menu metrics (#2065).
                            let ink = if en { pal.text } else { pal.text_faint };
                            let mut text_x = g.pad_x;
                            if let Cmd::Create(i) = *cmd
                                && let (Some(set), Some(info)) = (
                                    crate::egui_app::icons::IconSet::from_ctx(ui.ctx()),
                                    camcraft_app::operation::KindInfo::ALL.get(i),
                                )
                                && let Some(tex) = set.texture(crate::egui_app::op_icons::op_icon(info.ty))
                            {
                                text_x = g.icon_text_x;
                                let icon_rect = Rect::from_center_size(
                                    egui::Pos2::new(r.left() + g.clip_inset_x + g.icon_px * 0.5, r.center().y),
                                    Vec2::splat(g.icon_px),
                                );
                                p.image(
                                    tex.id(),
                                    icon_rect,
                                    egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)),
                                    ink,
                                );
                            }
                            p.text(
                                r.left_center() + Vec2::new(text_x, 0.0),
                                egui::Align2::LEFT_CENTER,
                                label,
                                egui::FontId::proportional(pal.font_row),
                                ink,
                            );
                            // Read the click flags before
                            // `on_hover_text` moves the response -
                            // the cell law again, one line lower. The
                            // hint on a row is the bar's tip for the
                            // same command: what it creates, or why
                            // it cannot yet.
                            let clicked = resp.clicked();
                            let secondary = resp.secondary_clicked();
                            resp.on_hover_text(tip_of(*cmd, state));
                            if secondary
                                && let Cmd::Create(i) = *cmd
                            {
                                pinned_row = Some(i);
                            }
                            if clicked {
                                if en {
                                    hit = Some(*cmd);
                                } else {
                                    gated_row = Some(*cmd);
                                }
                            }
                        }
                    });
                if let Some(i) = pinned_row {
                    toggle_pin(state, i);
                }
                if let Some(c) = hit {
                    run_cmd(c, state);
                    state.ribbon_more = None;
                } else if let Some(c) = gated_row {
                    // The dark row answered: the refusal named on the
                    // status line, the door closed on the way out. A
                    // click into silence is what made the folded
                    // ribbon read as dead - this branch is why it
                    // never is.
                    state.ribbon_more = None;
                    follow_gate(c, state);
                } else if state.ribbon_more == before {
                    // Primary press anywhere outside the door and its
                    // rows closes it (egui's popup law, hand-drawn
                    // because the panel is geometry, not a Widget). A
                    // press that already toggled a door this frame
                    // (`state.ribbon_more != before`) is the new
                    // door's own business.
                    let outside = ui.input(|i| {
                        i.pointer.button_pressed(egui::PointerButton::Primary)
                            && i.pointer.interact_pos().is_none_or(|p| {
                                !drop.button.expand(g.gap).contains(p)
                                    && !drop.rows.iter().any(|(_, r)| r.contains(p))
                            })
                    });
                    if outside {
                        state.ribbon_more = None;
                    }
                }
            }
        }
    }
}
