//! The ribbon strip: the tab band, the painted cells, captions, dividers,
//! the door dropdowns and the build badge, split from panes.rs (#4393); our own design notes.

use egui::{Rect, Sense, Ui};

use crate::build_info;
use crate::state::State;

use crate::egui_app::icons::IconSet;
use crate::egui_app::layout;
use crate::egui_app::painted;
use crate::egui_app::theme::Tokens;

use super::cmd::{Cmd, ribbon_plan, text_of, tip_of};
use super::gates::{enabled, follow_gate, icon_of, toggled};
use super::mode_tabs::{machine_drop, ribbon_tabs};
use super::ribbon_layout::{ribbon_cells, view_rows_of};
use super::run::{run_cmd, toggle_pin};
use super::section_drop::section_drop;

/// The ribbon strip: painted cells over a fixed-height top panel. Group
/// captions and hairline dividers match imgui's layout math, the cells
/// themselves are painted::cell - all from `ribbon_cells`'s geometry.
/// Since M-g each cell carries its bundled glyph (the retired imgui
/// ribbon's Item.icon table), or the painted fallback headless.
pub fn ribbon(
    ui: &mut Ui,
    state: &mut State,
    pal: &Tokens,
    icons: Option<&IconSet>,
) {
    let rect = Rect::from_min_size(ui.cursor().min, ui.available_size());
    painted::strip_bg(ui.painter(), rect, pal);
    let base = ui.id().with("ribbon");
    // The workspace tabs sit above the cells: the
    // active machine mode is one click away, and the switch is the same
    // undoable `LoadBuiltinMachine` the Machine door queues.
    let band = Rect::from_min_max(
        rect.min,
        egui::Pos2::new(rect.right(), rect.top() + pal.ribbon_tab_h),
    );
    let tabs_right = ribbon_tabs(ui, state, pal, rect, band);
    let donate_left = crate::egui_app::donate::draw(ui, state, pal, icons, band);
    let home_left = crate::egui_app::home::button(ui, state, pal, icons, band, donate_left - pal.item_spacing_x);
    let settings_left = crate::egui_app::settings_drop::button(ui, state, pal, band, home_left - pal.item_spacing_x);
    crate::egui_app::update_notice::draw(
        ui,
        state,
        pal,
        band,
        tabs_right + pal.item_spacing_x,
        settings_left - pal.item_spacing_x,
    );
    let body = Rect::from_min_max(egui::Pos2::new(rect.left(), band.bottom()), rect.max);
    // The plan is read fresh every frame from the two facts it is a
    // function of - the document's machine and its pins (M6.2). No
    // cached shape: a machine mount or an unpin cannot leave a stale
    // group standing.
    let machine = camcraft_app::machine::kind_of(&state.world);
    let plan = ribbon_plan(machine, &camcraft_app::ribbon::pinned_of(&state.world));
    let (cells, captions, dividers, drops) = ribbon_cells(rect, pal, &plan);
    // Self-heal first: a folded-away door (window widened, pin moved a
    // cell inline, document swapped) must not leave `ribbon_more`
    // naming a button that no longer exists.
    if let Some(s) = state.ribbon_more
        && !cells.iter().any(|cell| cell.cmd == Cmd::More(s))
    {
        state.ribbon_more = None;
    }
    let before = state.ribbon_more;
    for cell in &cells {
        let c = cell.cmd;
        let resp = painted::cell(
            ui,
            base.with(cell.salt),
            cell.rect,
            &text_of(c, state),
            pal,
            enabled(c, state),
            toggled(c, state),
            icon_of(c, state).and_then(|i| icons.and_then(|s| s.texture(i))),
        );
        // The pin gesture lives at the cell, not in a command: the
        // right-click that answers it names itself in every live tip
        // (tip_of), and its law - layout rides the document, undo
        // replays cuts - is ROADMAP M6.2's, recorded in RECOMMENDED.md.
        // Read before `on_hover_text` moves the response into the
        // tooltip wiring.
        let fired = resp.clicked() && enabled(c, state);
        // A dark Create cell that gets clicked is not a dead pixel:
        // the refusal speaks (gate_note), the same answer the
        // dropdown rows give. Hover already taught it; the click
        // proves it.
        let gated = resp.clicked() && !fired && matches!(c, Cmd::Create(_));
        let pin = resp.secondary_clicked();
        let tip = tip_of(c, state);
        ui.ctx()
            .accesskit_node_builder(resp.id, |b| b.set_description(tip.as_str()));
        resp.on_hover_text(tip);
        if fired {
            run_cmd(c, state);
        }
        if gated {
            follow_gate(c, state);
        }
        if let (Cmd::Create(i), true) = (c, pin) {
            toggle_pin(state, i);
        }
    }
    // Group captions (#948): under each group's cells when they fit the
    // group's span. A single-door group (Milling 2D, Drilling ...) is one
    // narrow cell that already names its section, so its caption is
    // skipped rather than run under the neighbour's.
    let caption_top = body.top() + pal.ribbon_cell_top_gap + pal.cell_h + pal.ribbon_caption_gap;
    for (mid, width, title) in captions {
        painted::caption(ui, mid, width, title, body, pal, caption_top);
    }
    for dx in dividers {
        painted::divider(ui.painter(), dx, body, pal);
    }
    section_drop(ui, state, pal, base, &drops, before);
    machine_drop(ui, state, pal, base, &cells, rect);
    // The View door's dropdown (layout.rs): the panel toggles and the
    // layout reset, same popup law as the doors above.
    if state.ribbon_view_open {
        match cells.iter().find(|c| c.cmd == Cmd::View) {
            None => {
                debug_assert!(false, "ribbon: view door missing from the bar");
                diag_warn!("ribbon: closing a view list whose button is gone");
                state.ribbon_view_open = false;
            }
            Some(cell) => {
                let rows = view_rows_of(cell.rect, rect, pal);
                layout::view_drop(ui, state, pal, base.with("view-drop"), cell.rect, &rows);
            }
        }
    }
    // The Theme door's dropdown (#441): theme_drop.rs, same popup law.
    if state.ribbon_theme_open {
        match cells.iter().find(|c| c.cmd == Cmd::Theme) {
            None => {
                debug_assert!(false, "ribbon: theme door missing from the bar");
                diag_warn!("ribbon: closing a theme list whose button is gone");
                state.ribbon_theme_open = false;
            }
            Some(cell) => {
                let rows = crate::egui_app::theme_drop::rows_of(state, cell.rect, body, pal);
                crate::egui_app::theme_drop::show(ui, state, pal, base.with("theme-drop"), cell.rect, &rows);
            }
        }
    }
    // The recent-files lists under Open and Model (#662): recent_drop.rs.
    crate::egui_app::recent_drop::draw(ui, state, pal, base.with("recent-drop"), &cells, body);
    // The source/build label is the rightmost toolbar item. At small window
    // widths it contracts to version plus source, then version alone; the
    // full text remains available on hover.
    let badge_left = cells.last().map_or(body.left(), |cell| cell.rect.right()) + 8.0;
    let badge_rect = Rect::from_min_max(
        egui::Pos2::new(badge_left, body.top()),
        egui::Pos2::new(body.right() - 8.0, body.bottom()),
    );
    let lines = if badge_rect.width() >= 285.0 {
        Some(build_info::toolbar_lines(false))
    } else if badge_rect.width() >= 145.0 {
        Some(build_info::toolbar_lines(true))
    } else if badge_rect.width() >= 50.0 {
        Some((build_info::toolbar_version(), String::new()))
    } else {
        None
    };
    if let Some((top, bottom)) = lines {
        let (full_top, full_bottom) = build_info::toolbar_lines(false);
        ui.interact(badge_rect, base.with("build-info"), Sense::hover())
            .on_hover_text(format!("{full_top}\n{full_bottom}"));
        let painter = ui.painter().with_clip_rect(badge_rect);
        let y = badge_rect.center().y;
        painter.text(
            egui::Pos2::new(badge_rect.right(), y - 8.0),
            egui::Align2::RIGHT_CENTER,
            top,
            egui::FontId::proportional(pal.font_row),
            pal.text_faint,
        );
        if !bottom.is_empty() {
            painter.text(
                egui::Pos2::new(badge_rect.right(), y + 8.0),
                egui::Align2::RIGHT_CENTER,
                bottom,
                egui::FontId::proportional(pal.font_row),
                pal.text_faint,
            );
        }
    }
    ui.allocate_rect(rect, Sense::hover());
}
