//! The ribbon's pure layout math: cell rects, captions, dividers and the
//! door dropdown rows, split from panes.rs (#4393); our own design notes.

use egui::{Rect, Vec2};

use camcraft_app::operation;

use crate::machine_modes;

use crate::egui_app::layout;
use crate::egui_app::theme::Tokens;

use super::cmd::{Cmd, Group};

// The cell box is tokens, not constants (M6.1 A): `Tokens::cell_w`/
// `cell_h` - icon over caption, ribbon.rs's cell: glyph + 2 + one line,
// so the PNG and the painted fallback occupy the same box; the 11.5 px
// label needs 35, and shell.rs's ribbon height follows the token
// (`theme::ribbon_h`). GAP/PAD_H stay literals: the ribbon's own
// rhythm, not a theme knob.
const GAP: f32 = 4.0;
const PAD_H: f32 = 8.0;

/// One laid-out ribbon cell: which command, where, and its id salt.
/// A flat index salt (was `(group, index)`) - the value only has to be
/// stable across frames for hover bookkeeping, and the flat list is
/// what the gesture tests can reach.
pub(in crate::egui_app) struct Cell {
    pub cmd: Cmd,
    pub rect: Rect,
    pub salt: usize,
}

/// One folded section's dropdown, pure geometry (M6.2): the More button
/// that owns it and its hidden kinds' row rects, all in GLOBAL
/// coordinates - so the layout math is testable (and the reach of every
/// kind provable) without a frame, and the draw only relocates it into
/// its Area. Rows are in registry order, the same order the section
/// drew before it folded.
pub(in crate::egui_app) struct Drop {
    pub section: operation::Section,
    pub button: Rect,
    pub rows: Vec<(Cmd, Rect)>,
}

/// The dropdown's own rhythm (ribbon constants law): wide enough for
/// the longest registry label, rows one theme row tall.
/// What one layout pass answers: cells to draw, group captions (mid x,
/// title), divider x's, and the doors with their dropdown rows.
pub(in crate::egui_app) type RibbonGeom = (Vec<Cell>, Vec<(f32, f32, &'static str)>, Vec<f32>, Vec<Drop>);

/// The machine door's dropdown rows, pure geometry (the section doors'
/// `Drop` law, worn by the switcher): the modes in table order, one
/// theme row tall each, `width` wide, hung under the door button in the
/// strip's global frame. The tests click the switch through exactly
/// these rects - one source of truth for where each mode lives.
pub(in crate::egui_app) fn machine_rows(
    button: Rect,
    bounds: Rect,
    row_h: f32,
    width: f32,
    g: &crate::egui_app::theme::DropGeom,
) -> Vec<(camcraft_app::machine::Kind, Rect)> {
    let left = g.fit_left(button.left(), bounds, width);
    machine_modes::MACHINE_MODES
        .iter()
        .enumerate()
        .map(|(k, (kind, _, _))| {
            (
                *kind,
                Rect::from_min_size(
                    egui::Pos2::new(
                        left,
                        button.bottom() + g.gap + k as f32 * row_h,
                    ),
                    Vec2::new(width, row_h),
                ),
            )
        })
        .collect()
}

/// The machine dropdown's width: as wide as its longest row (text is
/// never cut), and never narrower than the theme menu width.
pub(in crate::egui_app) fn machine_menu_width(longest_px: f32, pal: &Tokens) -> f32 {
    pal.dropdown().machine_width(longest_px)
}

/// The ribbon's layout math, pure: a plan (ribbon_plan's groups) and the
/// strip give back cell rects, group captions (centre x, title), the
/// divider x's between groups, and one `Drop` per folded section.
/// `ribbon` draws exactly this, and the tests click through it - one
/// source of truth for where each command lives. FLOOR_W/GAP/PAD_H are
/// ribbon.rs's; the cell box is the theme's (Tokens::cell_w/cell_h).
///
/// The fold law (the ribbon law, retired M6.2's width law): a
/// section's kinds live behind their labeled door AT EVERY WIDTH -
/// the bar's inline op cells are exactly the pinned ones, at most
/// MAX_PINS of them. A crowd that only folds when squeezed is still a
/// crowd, and the user asked for this one directly. Cells share the
/// strip's width - a flow, not imgui's fixed pitch - and anchors
/// never fold; if the anchors and doors cannot fit, the row overflows
/// on purpose (clamped at FLOOR_W) rather than rendering a narrower
/// lie. Every section's door owns its full list; the layout draws
/// the door where the run of cells would have stood.
pub(in crate::egui_app) fn ribbon_cells(strip: Rect, pal: &Tokens, plan: &[Group]) -> RibbonGeom {
    const FLOOR_W: f32 = 40.0;
    const BUILD_BADGE_W: f32 = 310.0;
    // Each group's inline run and folded tail: sections start fully
    // folded, anchors fully inline - the split IS the law, no loop
    // decides it.
    let groups: Vec<(&Group, Vec<Cmd>, Vec<Cmd>)> = plan
        .iter()
        .map(|g| match g.section {
            Some(_) => (g, Vec::new(), g.cells.clone()),
            None => (g, g.cells.clone(), Vec::new()),
        })
        .collect();
    let n_groups = groups.len();
    let n_cells: usize = groups
        .iter()
        .map(|(_, shown, hidden)| shown.len() + usize::from(!hidden.is_empty()))
        .sum();
    let overhead = 2.0 * PAD_H
        + (n_cells - n_groups) as f32 * GAP
        + (n_groups - 1) as f32 * (GAP + 2.0);
    // Reserve the largest build label that fits after the minimum cell row.
    // The three sizes carry full details, version plus source, or version.
    let minimum = overhead + n_cells as f32 * FLOOR_W;
    let badge_w = [BUILD_BADGE_W, 165.0, 75.0]
        .into_iter()
        .find(|w| strip.width() >= minimum + *w)
        .unwrap_or(0.0);
    let cell_w = ((strip.width() - overhead - badge_w) / n_cells.max(1) as f32)
        .clamp(FLOOR_W, pal.cell_w);
    // The cells start below the tab row and its gap, both theme tokens.
    let cell_top = strip.top() + pal.ribbon_tab_h + pal.ribbon_cell_top_gap;
    let mut cells = Vec::new();
    let mut captions = Vec::new();
    let mut dividers = Vec::new();
    let mut drops = Vec::new();
    let mut x = strip.left() + PAD_H;
    let mut salt = 0usize;
    for (gi, (group, shown, hidden)) in groups.iter().enumerate() {
        let start = x;
        let mut draw = |cmd: Cmd, x: &mut f32, cells: &mut Vec<Cell>| {
            cells.push(Cell {
                cmd,
                rect: Rect::from_min_size(
                    egui::Pos2::new(*x, cell_top),
                    Vec2::new(cell_w, pal.cell_h),
                ),
                salt: {
                    salt += 1;
                    salt - 1
                },
            });
            *x += cell_w + GAP;
        };
        for c in shown {
            draw(*c, &mut x, &mut cells);
        }
        // Only section groups ever carry a hidden tail (the fold law
        // starts them folded and folds nothing else), so the `section`
        // test is a type guard, not a second opinion.
        if let (Some(section), true) = (group.section, !hidden.is_empty()) {
            // The section's door stands at the end of its inline run,
            // and owns the dropdown of everything behind it.
            let button = Rect::from_min_size(
                egui::Pos2::new(x, cell_top),
                Vec2::new(cell_w, pal.cell_h),
            );
            let g = pal.dropdown();
            let width = g.fit_width(g.menu_width, strip.width());
            let left = g.fit_left(button.left(), strip, width);
            let row_h = pal.row_height;
            let rows = hidden
                .iter()
                .enumerate()
                .map(|(k, c)| {
                    (
                        *c,
                        Rect::from_min_size(
                            egui::Pos2::new(
                                left,
                                button.bottom() + g.gap + k as f32 * row_h,
                            ),
                            Vec2::new(width, row_h),
                        ),
                    )
                })
                .collect();
            drops.push(Drop {
                section,
                button,
                rows,
            });
            cells.push(Cell {
                cmd: Cmd::More(section),
                rect: button,
                salt: {
                    salt += 1;
                    salt - 1
                },
            });
            x += cell_w + GAP;
        }
        x -= GAP;
        captions.push(((start + x) * 0.5, (x - start).max(0.0), group.title));
        if gi + 1 < n_groups {
            x += GAP + 2.0;
            dividers.push(x - (GAP + 2.0) / 2.0 - 1.0);
        }
    }
    (cells, captions, dividers, drops)
}

/// The View door's rows under `button` in `strip` - the one geometry the
/// draw and the tests share.
pub(in crate::egui_app) fn view_rows_of(
    button: Rect,
    strip: Rect,
    pal: &Tokens,
) -> Vec<(layout::ViewRow, Rect)> {
    let g = pal.dropdown();
    let width = g.fit_width(g.menu_width, strip.width());
    layout::view_rows(button, strip, pal.row_height, width, g.gap)
}
