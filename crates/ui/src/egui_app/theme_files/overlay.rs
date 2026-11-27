//! The overlay of a parsed theme file on its base table (moved unchanged out
//! of `theme_files.rs` for the 400-line law, #2021).
//! Provenance: our own format; every name and clamp is ours.

use std::collections::HashMap;

use egui::Color32;

use super::names;
use crate::egui_app::theme::{Tokens, clamp};

/// Overlay the parsed maps on the base table. The destructuring is the
/// M6.1 A exhaustiveness law, restated for files: a token added to
/// `Tokens` must be named here (pattern AND lookup) or the build
/// fails - the format can never be able to set less than the whole
/// table. Geometry lands clamped, so every reader (draw sites
/// included) sees shell-legal numbers.
pub(super) fn apply_overlay(
    base: Tokens,
    colors: &HashMap<String, Color32>,
    geom: &HashMap<String, f64>,
) -> Tokens {
    let Tokens {
        ink, ink_alt, well, text, text_dim, text_faint, border, accent,
        accent_hover, field_bg, field_hover, field_active, selection_bg,
        selection_stroke, viewport, radius_widget, radius_window, radius_menu,
        item_spacing_x, item_spacing_y, button_padding_x, button_padding_y,
        control_height, row_height, cell_h, cell_w, glyph_px, font_label,
        font_caption, font_row, font_row_suffix, ribbon_ink, handle_shade, handle_text, pick_rim, ribbon_tab_h,
        ribbon_cell_top_gap, ribbon_caption_gap, ribbon_caption_row_h,
        ribbon_bottom_gap, window_margin, status_h, ribbon_tab_pad_x,
        ribbon_tab_gap, ribbon_tab_stroke,
        font_label_min, op_tab_h, op_tab_pad_x, op_tab_gap,
        op_tab_bottom_gap, op_tab_icon_px, op_tab_icon_gap, op_tab_radius,
        font_op_tab, tree_glyph_px, tree_inset_x, tree_indent,
        tree_eye_reserve, tree_pad_x, tree_pad_y, tree_suffix_gap,
        tree_header_boost, highlight_stroke_px,
        menu_width, recent_width, menu_gap, menu_pad_x,
        overlay_panel, overlay_edge, overlay_text, overlay_track, drop_veil, empty_hint, pick_halo, plate_hole, plate_dowel, gcode_line,
        // The viewport furniture reads `ground` below, not `base`.
        cube_edge: _, cube_label: _, cube_glyph_on: _, cube_glyph_off: _, viewport_hint: _, marker_stem: _, marker_ring: _, marker_dot: _, axis_x: _, axis_y: _, axis_z: _, viewport_hover: _,
    } = base;
    let c = |name: &str, keep: Color32| colors.get(name).copied().unwrap_or(keep);
    let f = |name: &str, keep: f32, win: (f32, f32)| {
        geom.get(name).map(|v| clamp::range(*v as f32, win)).unwrap_or(keep)
    };
    let rad = |name: &str, keep: u8, cap: f32| {
        geom.get(name)
            .map(|v| clamp::range(*v as f32, (0.0, cap)).round() as u8)
            .unwrap_or(keep)
    };
    let font_label = f(names::FONT_LABEL, font_label, clamp::FONT);
    // #1120: the cube, marker and axis inks belong to the viewport
    // ground, not to the base. A file that turns its ground light (or
    // dark) gets the built-in set of that ground for each key it leaves
    // out - the furniture followed the ground before it was a token.
    let viewport = c(names::VIEWPORT, viewport);
    let light = crate::egui_app::ground_ink::light_ground;
    let ground = match (light(viewport), light(base.viewport)) {
        (now, was) if now == was => base,
        (true, _) => Tokens::LIGHT,
        (false, _) => Tokens::DARK,
    };
    let font_label_min = f(names::FONT_LABEL_MIN, font_label_min, clamp::FONT).min(font_label);
    Tokens {
        ink: c(names::INK, ink),
        ink_alt: c(names::INK_ALT, ink_alt),
        well: c(names::WELL, well),
        text: c(names::TEXT, text),
        text_dim: c(names::TEXT_DIM, text_dim),
        text_faint: c(names::TEXT_FAINT, text_faint),
        border: c(names::BORDER, border),
        accent: c(names::ACCENT, accent),
        accent_hover: c(names::ACCENT_HOVER, accent_hover),
        field_bg: c(names::FIELD_BG, field_bg),
        field_hover: c(names::FIELD_HOVER, field_hover),
        field_active: c(names::FIELD_ACTIVE, field_active),
        selection_bg: c(names::SELECTION_BG, selection_bg),
        selection_stroke: c(names::SELECTION_STROKE, selection_stroke),
        viewport,
        ribbon_ink: c(names::RIBBON_INK, ribbon_ink),
        handle_shade: c(names::HANDLE_SHADE, handle_shade),
        handle_text: c(names::HANDLE_TEXT, handle_text),
        pick_rim: c(names::PICK_RIM, pick_rim),
        cube_edge: c(names::CUBE_EDGE, ground.cube_edge),
        cube_label: c(names::CUBE_LABEL, ground.cube_label),
        cube_glyph_on: c(names::CUBE_GLYPH_ON, ground.cube_glyph_on),
        cube_glyph_off: c(names::CUBE_GLYPH_OFF, ground.cube_glyph_off),
        viewport_hint: c(names::VIEWPORT_HINT, ground.viewport_hint),
        marker_stem: c(names::MARKER_STEM, ground.marker_stem),
        marker_ring: c(names::MARKER_RING, ground.marker_ring),
        marker_dot: c(names::MARKER_DOT, ground.marker_dot),
        axis_x: c(names::AXIS_X, ground.axis_x),
        axis_y: c(names::AXIS_Y, ground.axis_y),
        axis_z: c(names::AXIS_Z, ground.axis_z),
        viewport_hover: c(names::VIEWPORT_HOVER, ground.viewport_hover),
        overlay_panel: c(names::OVERLAY_PANEL, overlay_panel),
        overlay_edge: c(names::OVERLAY_EDGE, overlay_edge),
        overlay_text: c(names::OVERLAY_TEXT, overlay_text),
        overlay_track: c(names::OVERLAY_TRACK, overlay_track),
        drop_veil: c(names::DROP_VEIL, drop_veil),
        empty_hint: c(names::EMPTY_HINT, empty_hint),
        pick_halo: c(names::PICK_HALO, pick_halo),
        plate_hole: c(names::PLATE_HOLE, plate_hole),
        plate_dowel: c(names::PLATE_DOWEL, plate_dowel),
        gcode_line: c(names::GCODE_LINE, gcode_line),
        radius_widget: rad(names::RADIUS_WIDGET, radius_widget, clamp::RADIUS as f32),
        radius_window: rad(names::RADIUS_WINDOW, radius_window, f32::from(clamp::RADIUS_WINDOW)),
        radius_menu: rad(names::RADIUS_MENU, radius_menu, f32::from(clamp::RADIUS_WINDOW)),
        item_spacing_x: f(names::ITEM_SPACING_X, item_spacing_x, clamp::SPACING),
        item_spacing_y: f(names::ITEM_SPACING_Y, item_spacing_y, clamp::SPACING),
        button_padding_x: f(names::BUTTON_PADDING_X, button_padding_x, clamp::SPACING),
        button_padding_y: f(names::BUTTON_PADDING_Y, button_padding_y, clamp::SPACING),
        control_height: f(names::CONTROL_HEIGHT, control_height, clamp::CONTROL_H),
        row_height: f(names::ROW_HEIGHT, row_height, clamp::CONTROL_H),
        cell_h: f(names::CELL_H, cell_h, clamp::CELL),
        cell_w: f(names::CELL_W, cell_w, clamp::CELL),
        glyph_px: f(names::GLYPH_PX, glyph_px, clamp::GLYPH),
        font_label,
        font_label_min,
        font_caption: f(names::FONT_CAPTION, font_caption, clamp::FONT),
        font_row: f(names::FONT_ROW, font_row, clamp::FONT),
        font_row_suffix: f(names::FONT_ROW_SUFFIX, font_row_suffix, clamp::FONT),
        ribbon_tab_h: f(names::RIBBON_TAB_H, ribbon_tab_h, clamp::RIBBON_ROW),
        ribbon_cell_top_gap: f(names::RIBBON_CELL_TOP_GAP, ribbon_cell_top_gap, clamp::RIBBON_GAP),
        ribbon_caption_gap: f(names::RIBBON_CAPTION_GAP, ribbon_caption_gap, clamp::RIBBON_GAP),
        ribbon_caption_row_h: f(names::RIBBON_CAPTION_ROW_H, ribbon_caption_row_h, clamp::RIBBON_ROW),
        ribbon_bottom_gap: f(names::RIBBON_BOTTOM_GAP, ribbon_bottom_gap, clamp::RIBBON_GAP),
        window_margin: f(names::WINDOW_MARGIN, window_margin, clamp::WINDOW_MARGIN),
        status_h: f(names::STATUS_H, status_h, clamp::STATUS_H),
        ribbon_tab_pad_x: f(names::RIBBON_TAB_PAD_X, ribbon_tab_pad_x, clamp::RIBBON_TAB_PAD_X),
        ribbon_tab_gap: f(names::RIBBON_TAB_GAP, ribbon_tab_gap, clamp::RIBBON_GAP),
        ribbon_tab_stroke: f(names::RIBBON_TAB_STROKE, ribbon_tab_stroke, clamp::RIBBON_STROKE),
        op_tab_h: f(names::OP_TAB_H, op_tab_h, clamp::CONTROL_H),
        op_tab_pad_x: f(names::OP_TAB_PAD_X, op_tab_pad_x, clamp::SPACING),
        op_tab_gap: f(names::OP_TAB_GAP, op_tab_gap, clamp::SPACING),
        op_tab_bottom_gap: f(names::OP_TAB_BOTTOM_GAP, op_tab_bottom_gap, clamp::RIBBON_GAP),
        op_tab_icon_px: f(names::OP_TAB_ICON_PX, op_tab_icon_px, clamp::GLYPH),
        op_tab_icon_gap: f(names::OP_TAB_ICON_GAP, op_tab_icon_gap, clamp::SPACING),
        op_tab_radius: rad(names::OP_TAB_RADIUS, op_tab_radius, clamp::RADIUS as f32),
        font_op_tab: f(names::FONT_OP_TAB, font_op_tab, clamp::FONT),
        tree_glyph_px: f(names::TREE_GLYPH_PX, tree_glyph_px, clamp::GLYPH),
        tree_inset_x: f(names::TREE_INSET_X, tree_inset_x, clamp::SPACING),
        tree_indent: f(names::TREE_INDENT, tree_indent, clamp::SPACING),
        tree_eye_reserve: f(names::TREE_EYE_RESERVE, tree_eye_reserve, clamp::EYE_RESERVE),
        tree_pad_x: f(names::TREE_PAD_X, tree_pad_x, clamp::SPACING),
        tree_pad_y: f(names::TREE_PAD_Y, tree_pad_y, clamp::SPACING),
        tree_suffix_gap: f(names::TREE_SUFFIX_GAP, tree_suffix_gap, clamp::SPACING),
        tree_header_boost: f(names::TREE_HEADER_BOOST, tree_header_boost, clamp::HEADER_BOOST),
        highlight_stroke_px: f(names::HIGHLIGHT_STROKE_PX, highlight_stroke_px, clamp::STROKE),
        menu_width: f(names::MENU_WIDTH, menu_width, clamp::MENU_W),
        recent_width: f(names::RECENT_WIDTH, recent_width, clamp::RECENT_W),
        menu_gap: f(names::MENU_GAP, menu_gap, clamp::MENU_GAP),
        menu_pad_x: f(names::MENU_PAD_X, menu_pad_x, clamp::MENU_PAD_X),
    }
}
