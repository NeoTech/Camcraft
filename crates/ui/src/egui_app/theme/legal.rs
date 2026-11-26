//! The token table's exhaustiveness guard and the geometry sanity
//! window of the two built-in columns (moved unchanged out of `theme.rs`
//! for the 400-line law, #1120).
//! Provenance: our own module; no HSMWorks material.

use super::{Tokens, clamp};

/// The exhaustiveness guard, compiled: every field is named in this
/// arm, so the table can never gain a member that no code path reads.
/// Geometry gets its sanity window here (the same clamps the draw sites
/// re-apply to file-supplied values); colours have no range to check.
/// Both built-in columns wear the check (M6.1 C).
const fn geom_legal(t: &Tokens) -> bool {
    let Tokens {
        ink: _, ink_alt: _, well: _, text: _, text_dim: _, text_faint: _,
        border: _, accent: _, accent_hover: _, field_bg: _, field_hover: _,
        field_active: _, selection_bg: _, selection_stroke: _, viewport: _,
        ribbon_ink: _, handle_shade: _, handle_text: _, pick_rim: _,
        cube_edge: _, cube_label: _, cube_glyph_on: _, cube_glyph_off: _, viewport_hint: _, marker_stem: _, marker_ring: _, marker_dot: _, axis_x: _, axis_y: _, axis_z: _, viewport_hover: _,
        overlay_panel: _, overlay_edge: _, overlay_text: _, overlay_track: _, drop_veil: _, empty_hint: _, pick_halo: _, plate_hole: _, plate_dowel: _, gcode_line: _,
        radius_widget, radius_window, radius_menu,
        item_spacing_x, item_spacing_y, button_padding_x, button_padding_y,
        control_height, row_height, cell_h, cell_w, glyph_px,
        font_label, font_caption, font_row, font_row_suffix,
        ribbon_tab_h, ribbon_cell_top_gap, ribbon_caption_gap,
        ribbon_caption_row_h, ribbon_bottom_gap, window_margin, status_h,
        ribbon_tab_pad_x, ribbon_tab_gap, ribbon_tab_stroke,
        font_label_min, op_tab_h, op_tab_pad_x, op_tab_gap, op_tab_bottom_gap,
        op_tab_icon_px, op_tab_icon_gap, op_tab_radius, font_op_tab,
        tree_glyph_px, tree_inset_x, tree_indent, tree_eye_reserve,
        tree_pad_x, tree_pad_y, tree_suffix_gap, tree_header_boost,
        highlight_stroke_px,
        menu_width, recent_width, menu_gap, menu_pad_x,
    } = *t;
    radius_widget <= clamp::RADIUS
        && radius_window <= clamp::RADIUS_WINDOW
        && radius_menu <= clamp::RADIUS_WINDOW
        && item_spacing_x >= 0.0
        && item_spacing_y >= 0.0
        && button_padding_x >= 0.0
        && button_padding_y >= 0.0
        && control_height >= clamp::CONTROL_H.0
        && control_height <= clamp::CONTROL_H.1
        && row_height >= clamp::CONTROL_H.0
        && row_height <= clamp::CONTROL_H.1
        && cell_h >= clamp::CELL.0
        && cell_h <= clamp::CELL.1
        && cell_w >= clamp::CELL.0
        && cell_w <= clamp::CELL.1
        && glyph_px >= clamp::GLYPH.0
        && glyph_px <= clamp::GLYPH.1
        && font_label >= clamp::FONT.0
        && font_label <= clamp::FONT.1
        && font_caption >= clamp::FONT.0
        && font_caption <= clamp::FONT.1
        && font_row >= clamp::FONT.0
        && font_row <= clamp::FONT.1
        && font_row_suffix >= clamp::FONT.0
        && font_row_suffix <= clamp::FONT.1
        && ribbon_tab_h >= clamp::RIBBON_ROW.0
        && ribbon_tab_h <= clamp::RIBBON_ROW.1
        && ribbon_caption_row_h >= clamp::RIBBON_ROW.0
        && ribbon_caption_row_h <= clamp::RIBBON_ROW.1
        && ribbon_cell_top_gap >= clamp::RIBBON_GAP.0
        && ribbon_cell_top_gap <= clamp::RIBBON_GAP.1
        && ribbon_caption_gap >= clamp::RIBBON_GAP.0
        && ribbon_caption_gap <= clamp::RIBBON_GAP.1
        && ribbon_bottom_gap >= clamp::RIBBON_GAP.0
        && ribbon_bottom_gap <= clamp::RIBBON_GAP.1
        && window_margin >= clamp::WINDOW_MARGIN.0
        && window_margin <= clamp::WINDOW_MARGIN.1
        && status_h >= clamp::STATUS_H.0
        && status_h <= clamp::STATUS_H.1
        && ribbon_tab_pad_x >= clamp::RIBBON_TAB_PAD_X.0
        && ribbon_tab_pad_x <= clamp::RIBBON_TAB_PAD_X.1
        && ribbon_tab_gap >= clamp::RIBBON_GAP.0
        && ribbon_tab_gap <= clamp::RIBBON_GAP.1
        && ribbon_tab_stroke >= clamp::RIBBON_STROKE.0
        && ribbon_tab_stroke <= clamp::RIBBON_STROKE.1
        && font_label_min >= clamp::FONT.0
        && font_label_min <= clamp::FONT.1
        && font_label_min <= font_label
        && op_tab_h >= clamp::CONTROL_H.0
        && op_tab_h <= clamp::CONTROL_H.1
        && op_tab_pad_x >= clamp::SPACING.0
        && op_tab_pad_x <= clamp::SPACING.1
        && op_tab_gap >= clamp::SPACING.0
        && op_tab_gap <= clamp::SPACING.1
        && op_tab_bottom_gap >= clamp::RIBBON_GAP.0
        && op_tab_bottom_gap <= clamp::RIBBON_GAP.1
        && op_tab_icon_px >= clamp::GLYPH.0
        && op_tab_icon_px <= clamp::GLYPH.1
        && op_tab_icon_gap >= clamp::SPACING.0
        && op_tab_icon_gap <= clamp::SPACING.1
        && op_tab_radius <= clamp::RADIUS
        && font_op_tab >= clamp::FONT.0
        && font_op_tab <= clamp::FONT.1
        && tree_glyph_px >= clamp::GLYPH.0
        && tree_glyph_px <= clamp::GLYPH.1
        && tree_inset_x >= clamp::SPACING.0
        && tree_inset_x <= clamp::SPACING.1
        && tree_indent >= clamp::SPACING.0
        && tree_indent <= clamp::SPACING.1
        && tree_eye_reserve >= clamp::EYE_RESERVE.0
        && tree_eye_reserve <= clamp::EYE_RESERVE.1
        && tree_pad_x >= clamp::SPACING.0
        && tree_pad_x <= clamp::SPACING.1
        && tree_pad_y >= clamp::SPACING.0
        && tree_pad_y <= clamp::SPACING.1
        && tree_suffix_gap >= clamp::SPACING.0
        && tree_suffix_gap <= clamp::SPACING.1
        && tree_header_boost >= clamp::HEADER_BOOST.0
        && tree_header_boost <= clamp::HEADER_BOOST.1
        && highlight_stroke_px >= clamp::STROKE.0
        && highlight_stroke_px <= clamp::STROKE.1
        && menu_width >= clamp::MENU_W.0 && menu_width <= clamp::MENU_W.1
        && recent_width >= clamp::RECENT_W.0 && recent_width <= clamp::RECENT_W.1
        && menu_gap >= clamp::MENU_GAP.0 && menu_gap <= clamp::MENU_GAP.1
        && menu_pad_x >= clamp::MENU_PAD_X.0 && menu_pad_x <= clamp::MENU_PAD_X.1
}

const _: () = assert!(geom_legal(&Tokens::DARK));
const _: () = assert!(geom_legal(&Tokens::LIGHT));
