//! The gallery's two exhaustive token listings (#4394): every colour and
//! every geometry field of `Tokens`, by its file name. Both functions
//! destructure the table without a `..` arm, so a new token that the
//! gallery does not show is a build error - the M6.1 A drift guard,
//! restated for the reviewers' page.
//! Provenance: our own module over our own token table; no HSMWorks material.

use egui::Color32;

use crate::theme::Tokens;

/// Every colour token, in table order, by its `camcraft-theme/1` name.
#[must_use]
#[rustfmt::skip]
pub fn colours(t: &Tokens) -> Vec<(&'static str, Color32)> {
    let Tokens {
        ink, ink_alt, well, text, text_dim, text_faint, border, accent,
        accent_hover, field_bg, field_hover, field_active, selection_bg,
        selection_stroke, viewport, ribbon_ink, handle_shade, handle_text,
        pick_rim, cube_edge, cube_label, cube_glyph_on, cube_glyph_off,
        viewport_hint, marker_stem, marker_ring, marker_dot, axis_x, axis_y,
        axis_z, viewport_hover, overlay_panel, overlay_edge, overlay_text,
        overlay_track, drop_veil, empty_hint, pick_halo, plate_hole,
        plate_dowel, gcode_line,
        tool_ground, tool_axis, tool_grid, tool_flute, tool_shank, tool_body, tool_edge, tool_holder, tool_note,
        tool_dim, thumb_flute, bad_ink, warn_ink, mount_axis, mount_radial, mount_tip, mount_origin, mount_hint,
        radius_widget: _, radius_window: _, radius_menu: _,
        item_spacing_x: _, item_spacing_y: _, button_padding_x: _,
        button_padding_y: _, control_height: _, row_height: _, cell_h: _,
        cell_w: _, glyph_px: _, font_label: _, font_caption: _, font_row: _,
        font_row_suffix: _, ribbon_tab_h: _, ribbon_cell_top_gap: _,
        ribbon_caption_gap: _, ribbon_caption_row_h: _, ribbon_bottom_gap: _,
        window_margin: _, status_h: _, ribbon_tab_pad_x: _, ribbon_tab_gap: _,
        ribbon_tab_stroke: _, font_label_min: _, op_tab_h: _, op_tab_pad_x: _,
        op_tab_gap: _, op_tab_bottom_gap: _, op_tab_icon_px: _,
        op_tab_icon_gap: _, op_tab_radius: _, font_op_tab: _,
        tree_glyph_px: _, tree_inset_x: _, tree_indent: _,
        tree_eye_reserve: _, tree_pad_x: _, tree_pad_y: _,
        tree_suffix_gap: _, tree_header_boost: _, highlight_stroke_px: _,
        menu_width: _, recent_width: _, menu_gap: _, menu_pad_x: _,
    } = *t;
    vec![
        ("ink", ink), ("ink_alt", ink_alt), ("well", well), ("text", text),
        ("text_dim", text_dim), ("text_faint", text_faint), ("border", border),
        ("accent", accent), ("accent_hover", accent_hover), ("field_bg", field_bg),
        ("field_hover", field_hover), ("field_active", field_active),
        ("selection_bg", selection_bg), ("selection_stroke", selection_stroke),
        ("viewport", viewport), ("ribbon_ink", ribbon_ink),
        ("handle_shade", handle_shade), ("handle_text", handle_text),
        ("pick_rim", pick_rim), ("cube_edge", cube_edge), ("cube_label", cube_label),
        ("cube_glyph_on", cube_glyph_on), ("cube_glyph_off", cube_glyph_off),
        ("viewport_hint", viewport_hint), ("marker_stem", marker_stem),
        ("marker_ring", marker_ring), ("marker_dot", marker_dot), ("axis_x", axis_x),
        ("axis_y", axis_y), ("axis_z", axis_z), ("viewport_hover", viewport_hover),
        ("overlay_panel", overlay_panel), ("overlay_edge", overlay_edge),
        ("overlay_text", overlay_text), ("overlay_track", overlay_track),
        ("drop_veil", drop_veil), ("empty_hint", empty_hint), ("pick_halo", pick_halo),
        ("plate_hole", plate_hole), ("plate_dowel", plate_dowel), ("gcode_line", gcode_line),
        ("tool_ground", tool_ground), ("tool_axis", tool_axis), ("tool_grid", tool_grid), ("tool_flute", tool_flute), ("tool_shank", tool_shank), ("tool_body", tool_body), ("tool_edge", tool_edge), ("tool_holder", tool_holder), ("tool_note", tool_note), ("tool_dim", tool_dim), ("thumb_flute", thumb_flute), ("bad_ink", bad_ink), ("warn_ink", warn_ink), ("mount_axis", mount_axis), ("mount_radial", mount_radial), ("mount_tip", mount_tip), ("mount_origin", mount_origin), ("mount_hint", mount_hint),
    ]
}

/// Every geometry token, in table order, by its `camcraft-theme/1` name
/// (the three radii and the op-tab radius widen from `u8`).
#[must_use]
#[rustfmt::skip]
pub fn geometry(t: &Tokens) -> Vec<(&'static str, f32)> {
    let Tokens {
        ink: _, ink_alt: _, well: _, text: _, text_dim: _, text_faint: _,
        border: _, accent: _, accent_hover: _, field_bg: _, field_hover: _,
        field_active: _, selection_bg: _, selection_stroke: _, viewport: _,
        ribbon_ink: _, handle_shade: _, handle_text: _, pick_rim: _,
        cube_edge: _, cube_label: _, cube_glyph_on: _, cube_glyph_off: _,
        viewport_hint: _, marker_stem: _, marker_ring: _, marker_dot: _,
        axis_x: _, axis_y: _, axis_z: _, viewport_hover: _, overlay_panel: _,
        overlay_edge: _, overlay_text: _, overlay_track: _, drop_veil: _,
        empty_hint: _, pick_halo: _, plate_hole: _, plate_dowel: _,
        gcode_line: _,
        tool_ground: _, tool_axis: _, tool_grid: _, tool_flute: _, tool_shank: _, tool_body: _, tool_edge: _, tool_holder: _, tool_note: _,
        tool_dim: _, thumb_flute: _, bad_ink: _, warn_ink: _, mount_axis: _, mount_radial: _, mount_tip: _, mount_origin: _, mount_hint: _,
        radius_widget, radius_window, radius_menu, item_spacing_x,
        item_spacing_y, button_padding_x, button_padding_y, control_height,
        row_height, cell_h, cell_w, glyph_px, font_label, font_caption,
        font_row, font_row_suffix, ribbon_tab_h, ribbon_cell_top_gap,
        ribbon_caption_gap, ribbon_caption_row_h, ribbon_bottom_gap,
        window_margin, status_h, ribbon_tab_pad_x, ribbon_tab_gap,
        ribbon_tab_stroke, font_label_min, op_tab_h, op_tab_pad_x, op_tab_gap,
        op_tab_bottom_gap, op_tab_icon_px, op_tab_icon_gap, op_tab_radius,
        font_op_tab, tree_glyph_px, tree_inset_x, tree_indent,
        tree_eye_reserve, tree_pad_x, tree_pad_y, tree_suffix_gap,
        tree_header_boost, highlight_stroke_px, menu_width, recent_width,
        menu_gap, menu_pad_x,
    } = *t;
    vec![
        ("radius_widget", f32::from(radius_widget)), ("radius_window", f32::from(radius_window)),
        ("radius_menu", f32::from(radius_menu)), ("item_spacing_x", item_spacing_x),
        ("item_spacing_y", item_spacing_y), ("button_padding_x", button_padding_x),
        ("button_padding_y", button_padding_y), ("control_height", control_height),
        ("row_height", row_height), ("cell_h", cell_h), ("cell_w", cell_w),
        ("glyph_px", glyph_px), ("font_label", font_label), ("font_caption", font_caption),
        ("font_row", font_row), ("font_row_suffix", font_row_suffix),
        ("ribbon_tab_h", ribbon_tab_h), ("ribbon_cell_top_gap", ribbon_cell_top_gap),
        ("ribbon_caption_gap", ribbon_caption_gap), ("ribbon_caption_row_h", ribbon_caption_row_h),
        ("ribbon_bottom_gap", ribbon_bottom_gap), ("window_margin", window_margin),
        ("status_h", status_h), ("ribbon_tab_pad_x", ribbon_tab_pad_x),
        ("ribbon_tab_gap", ribbon_tab_gap), ("ribbon_tab_stroke", ribbon_tab_stroke),
        ("font_label_min", font_label_min), ("op_tab_h", op_tab_h),
        ("op_tab_pad_x", op_tab_pad_x), ("op_tab_gap", op_tab_gap),
        ("op_tab_bottom_gap", op_tab_bottom_gap), ("op_tab_icon_px", op_tab_icon_px),
        ("op_tab_icon_gap", op_tab_icon_gap), ("op_tab_radius", f32::from(op_tab_radius)),
        ("font_op_tab", font_op_tab), ("tree_glyph_px", tree_glyph_px),
        ("tree_inset_x", tree_inset_x), ("tree_indent", tree_indent),
        ("tree_eye_reserve", tree_eye_reserve), ("tree_pad_x", tree_pad_x),
        ("tree_pad_y", tree_pad_y), ("tree_suffix_gap", tree_suffix_gap),
        ("tree_header_boost", tree_header_boost), ("highlight_stroke_px", highlight_stroke_px),
        ("menu_width", menu_width), ("recent_width", recent_width),
        ("menu_gap", menu_gap), ("menu_pad_x", menu_pad_x),
    ]
}
