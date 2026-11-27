//! The theme's single token table (M6.1 A): every colour, radius, spacing
//! and size the shell shows is a named field here - widget path and
//! painted path read the SAME struct, so a retheme is one table and the
//! two render paths cannot drift. The law is adopted from a sibling
//! project's proven design at the user's direction (ROADMAP M6.1); the
//! values are ours - the dark column carries today's `Palette::DARK`
//! numbers verbatim so the snapshots stay meaningful through the change.
//!
//! The drift guard is structural, not a checklist: the const block below
//! destructures EVERY `Tokens` field without a `..` arm, so adding a
//! field without wiring it (or explicitly resting it in the painted
//! path) fails to build - the sibling repo's exhaustiveness pattern,
//! restated for a table that paints as well as widgets.
//! Provenance: our own module (graduated from the spike and the M6.1 A
//! rework), specced from the imgui theme we already ship, the egui
//! Visuals docs, and the sibling architecture at the user's direction;
//! no HSMWorks material.

use egui::{Color32, CornerRadius, Margin, Stroke, Style, Visuals, vec2};

use super::contrast;

mod live;
pub use live::{current, set_current};

pub mod clamp;
mod dropdown;
mod legal;
mod tables;
pub use dropdown::DropGeom;

/// One theme's whole truth: colours, then geometry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tokens {
    // -- colours: the old Palette, names unchanged (M6.1 B keys on them).
    /// Window/panel ground.
    pub ink: Color32,
    /// Ribbon / browser / header ground (one step above ink).
    pub ink_alt: Color32,
    /// Input fields and empty recesses (one step below ink).
    pub well: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub text_faint: Color32,
    pub border: Color32,
    /// The selection/hover accent (steel blue).
    pub accent: Color32,
    pub accent_hover: Color32,
    /// Widget-field ground, by interaction state.
    pub field_bg: Color32,
    pub field_hover: Color32,
    pub field_active: Color32,
    /// Text selection and combo-open highlight.
    pub selection_bg: Color32,
    pub selection_stroke: Color32,
    /// The viewport's 3D ground (the render pass clear colour).
    pub viewport: Color32,
    /// Ink of a plain ribbon glyph, read against `strip_fill` (#2021).
    pub ribbon_ink: Color32,
    /// Viewport handles (#1120): the shadow under a move or turn handle,
    /// its readout and the origin triads, and the readout text over the
    /// viewport.
    pub handle_shade: Color32,
    pub handle_text: Color32,
    /// Rim of a picked point's disc in the viewport (#1120).
    pub pick_rim: Color32,
    /// The viewport furniture (#1120): view cube, nav buttons, corner
    /// hint, playback marker, axis triad and the hover gold. Each table
    /// holds the set for its own viewport ground; a theme file that turns
    /// the ground light or dark gets the matching set (`theme_files`).
    pub cube_edge: Color32,
    pub cube_label: Color32,
    pub cube_glyph_on: Color32,
    pub cube_glyph_off: Color32,
    pub viewport_hint: Color32,
    pub marker_stem: Color32,
    pub marker_ring: Color32,
    pub marker_dot: Color32,
    pub axis_x: Color32,
    pub axis_y: Color32,
    pub axis_z: Color32,
    pub viewport_hover: Color32,
    /// The viewport overlays (#1120): the generate progress panel, the
    /// file drop hint, the empty-view text, the pick halo, the fixture
    /// plate marks and the selected G-code move. Both tables hold the
    /// values the draw sites wrote before.
    pub overlay_panel: Color32,
    pub overlay_edge: Color32,
    pub overlay_text: Color32,
    pub overlay_track: Color32,
    pub drop_veil: Color32,
    pub empty_hint: Color32,
    pub pick_halo: Color32,
    pub plate_hole: Color32,
    pub plate_dowel: Color32,
    pub gcode_line: Color32,
    // -- geometry: the Phase-6 complaint made themeable - ribbon size,
    // input fields and button size are tokens, not literals.
    /// Corner radius of widget rects and painted cells/rows.
    pub radius_widget: u8,
    /// Corner radius of egui windows.
    pub radius_window: u8,
    /// Corner radius of menus and combo dropdowns.
    pub radius_menu: u8,
    /// egui item spacing, x/y.
    pub item_spacing_x: f32,
    pub item_spacing_y: f32,
    /// egui button padding, x/y.
    pub button_padding_x: f32,
    pub button_padding_y: f32,
    /// Input-field/combo minimum height (egui's `interact_size.y`).
    pub control_height: f32,
    /// Browser tree row height (painted::tree).
    pub row_height: f32,
    /// Ribbon cell height (painted cell box; RIBBON_H follows it).
    pub cell_h: f32,
    /// Ribbon cell width cap (panes' flow clamp; floor is geometry).
    pub cell_w: f32,
    /// Ribbon glyph edge (the PNG and the painted fallback ride it).
    pub glyph_px: f32,
    /// Cell caption size under the ribbon glyph.
    pub font_label: f32,
    /// Group caption size.
    pub font_caption: f32,
    /// Browser row label size.
    pub font_row: f32,
    /// Browser row suffix size.
    pub font_row_suffix: f32,
    /// Ribbon rows (#2021): the tab row, then the gap to the cells.
    pub ribbon_tab_h: f32,
    pub ribbon_cell_top_gap: f32,
    /// Gap from the cell box to the caption row, the row, the bottom gap.
    pub ribbon_caption_gap: f32,
    pub ribbon_caption_row_h: f32,
    pub ribbon_bottom_gap: f32,
    /// Shell metrics (#2064): window inner margin, status strip height, and
    /// ribbon tab horizontal padding, gap and active underline thickness.
    pub window_margin: f32,
    pub status_h: f32,
    pub ribbon_tab_pad_x: f32,
    pub ribbon_tab_gap: f32,
    pub ribbon_tab_stroke: f32,
    /// Minimum point size when shrinking an overflowing ribbon cell label.
    pub font_label_min: f32,
    /// Operation tab strip (#2066): row height, inner horizontal pad, inter-tab gap, bottom gap.
    pub op_tab_h: f32,
    pub op_tab_pad_x: f32,
    pub op_tab_gap: f32,
    pub op_tab_bottom_gap: f32,
    /// Operation tab icon edge, icon-to-label gap, top corner radius and font size.
    pub op_tab_icon_px: f32,
    pub op_tab_icon_gap: f32,
    pub op_tab_radius: u8,
    pub font_op_tab: f32,
    /// Browser tree & pane (#2066): glyph edge, root left inset, per-depth indent.
    pub tree_glyph_px: f32,
    pub tree_inset_x: f32,
    pub tree_indent: f32,
    /// Eye reserve width, header/row pad x/y, label-to-suffix gap, header font boost.
    pub tree_eye_reserve: f32,
    pub tree_pad_x: f32,
    pub tree_pad_y: f32,
    pub tree_suffix_gap: f32,
    pub tree_header_boost: f32,
    /// Active rule / row highlight / drop-target stroke thickness (px).
    pub highlight_stroke_px: f32,
    /// Dropdown width, recent-list width, trigger gap and row inset in points (#2065).
    pub menu_width: f32,
    pub recent_width: f32,
    pub menu_gap: f32,
    pub menu_pad_x: f32,
}

/// The ribbon strip's height: the tab row, its gap, the cell box, the
/// caption gap, the caption row and the bottom gap, all from the theme
/// (88 at the built-in tables). A theme that grows a row grows the ribbon.
#[must_use]
pub const fn ribbon_h(t: &Tokens) -> f32 {
    t.ribbon_tab_h
        + t.ribbon_cell_top_gap
        + t.cell_h
        + t.ribbon_caption_gap
        + t.ribbon_caption_row_h
        + t.ribbon_bottom_gap
}

/// Bottom status strip height (px), tracking typography and controls (#2064).
#[must_use]
pub const fn status_h(t: &Tokens) -> f32 {
    let a = if t.control_height + 4.0 > t.status_h { t.control_height + 4.0 } else { t.status_h };
    if t.font_row * 1.3 + 4.0 > a { t.font_row * 1.3 + 4.0 } else { a }
}

impl Tokens {
    /// Validated popup widths and row spacing shared by every dropdown.
    #[must_use]
    pub fn dropdown(&self) -> DropGeom {
        DropGeom::from_tokens(self)
    }

    /// The panel ground behind the ribbon/browser strips (painted.rs).
    #[must_use]
    pub const fn strip_fill(&self) -> Color32 {
        self.ink_alt
    }

    /// Build the egui Visuals from the tokens: every default-egui colour
    /// and corner the shell shows is named here, so the widget path and
    /// the painted path cannot disagree about what the theme is.
    #[must_use]
    pub fn visuals(&self) -> Visuals {
        // The destructuring arm: every COLOUR is named here and nowhere
        // else; a new colour field without a wiring line is a build
        // error, not default-egui grey. (Geometry routes through
        // `style()`; the const guard below covers those names.)
        let Tokens {
            ink, ink_alt, well, text, text_dim, text_faint, border, accent,
            accent_hover, field_bg, field_hover, field_active, selection_bg,
            selection_stroke, viewport: _, ribbon_ink: _,
            handle_shade: _, handle_text: _, pick_rim: _,
            cube_edge: _, cube_label: _, cube_glyph_on: _, cube_glyph_off: _, viewport_hint: _, marker_stem: _, marker_ring: _, marker_dot: _, axis_x: _, axis_y: _, axis_z: _, viewport_hover: _,
            overlay_panel: _, overlay_edge: _, overlay_text: _, overlay_track: _, drop_veil: _, empty_hint: _, pick_halo: _, plate_hole: _, plate_dowel: _, gcode_line: _,
            radius_widget, radius_window, radius_menu,
            item_spacing_x: _, item_spacing_y: _,
            button_padding_x: _, button_padding_y: _,
            control_height: _, row_height: _, cell_h: _, cell_w: _,
            glyph_px: _, font_label: _, font_caption: _,
            font_row: _, font_row_suffix: _, ribbon_tab_h: _,
            ribbon_cell_top_gap: _, ribbon_caption_gap: _,
            ribbon_caption_row_h: _, ribbon_bottom_gap: _,
            window_margin: _, status_h: _, ribbon_tab_pad_x: _,
            ribbon_tab_gap: _, ribbon_tab_stroke: _,
            font_label_min: _, op_tab_h: _, op_tab_pad_x: _, op_tab_gap: _,
            op_tab_bottom_gap: _, op_tab_icon_px: _, op_tab_icon_gap: _,
            op_tab_radius: _, font_op_tab: _, tree_glyph_px: _, tree_inset_x: _,
            tree_indent: _, tree_eye_reserve: _, tree_pad_x: _, tree_pad_y: _,
            tree_suffix_gap: _, tree_header_boost: _, highlight_stroke_px: _,
            menu_width: _, recent_width: _, menu_gap: _, menu_pad_x: _,
        } = *self;
        let mut v = Visuals::dark();
        v.panel_fill = ink;
        v.window_fill = ink_alt;
        v.faint_bg_color = well;
        v.extreme_bg_color = well;
        v.code_bg_color = well;
        v.override_text_color = Some(text);
        v.weak_text_color = Some(text_dim);
        v.hyperlink_color = accent_hover;
        v.window_corner_radius = CornerRadius::same(radius_window);
        v.menu_corner_radius = CornerRadius::same(radius_menu);
        v.selection = egui::style::Selection {
            bg_fill: selection_bg,
            stroke: Stroke::new(v.selection.stroke.width, selection_stroke),
        };
        let stroke_1px = Stroke::new(1.0, border);
        let r = CornerRadius::same(radius_widget);
        v.widgets.noninteractive.bg_fill = ink_alt;
        v.widgets.noninteractive.weak_bg_fill = Color32::TRANSPARENT;
        v.widgets.noninteractive.bg_stroke = stroke_1px;
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, text);
        v.widgets.noninteractive.corner_radius = r;
        v.widgets.inactive.bg_fill = field_bg;
        v.widgets.inactive.weak_bg_fill = ink_alt;
        v.widgets.inactive.bg_stroke = stroke_1px;
        v.widgets.inactive.fg_stroke = Stroke::new(1.0, text);
        v.widgets.inactive.corner_radius = r;
        v.widgets.hovered.bg_fill = field_hover;
        // #698: light hovers field_hover, not the accent (egui's bare title
        // glyphs took white-on-accent ink), and its title bars wear ink_alt.
        let light = super::ground_ink::light_ground(ink);
        let hover = if light { field_hover } else { accent };
        v.widgets.hovered.weak_bg_fill = hover;
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, accent_hover);
        // Glyphs ride the hover fill (M6.1 D): WHITE on steel blue, black on pale.
        v.widgets.hovered.fg_stroke = Stroke::new(1.0, contrast::on_fill(hover));
        v.widgets.hovered.corner_radius = r;
        v.widgets.active.bg_fill = field_active;
        v.widgets.active.weak_bg_fill = accent_hover;
        v.widgets.active.bg_stroke = Stroke::new(1.0, accent_hover);
        // The active (pressed) fill is the HOVER accent, one step
        // lighter - and there white only reads 3.8:1 on dark's blue.
        // The derivation picks black (5.5:1): pressed chrome is
        // momentary, readable chrome is the point. The first tables
        // hand-typed white here; the arithmetic now overrules the
        // habit, which is exactly what M6.1 D exists to do.
        v.widgets.active.fg_stroke = Stroke::new(1.0, contrast::on_fill(accent_hover));
        v.widgets.active.corner_radius = r;
        v.widgets.open.bg_fill = field_bg;
        if light {
            (v.widgets.open.weak_bg_fill, v.widgets.open.fg_stroke) = (ink_alt, Stroke::new(1.0, text));
        }
        v.widgets.open.bg_stroke = stroke_1px;
        v.widgets.open.corner_radius = r;
        for exp in [
            &mut v.widgets.noninteractive.expansion,
            &mut v.widgets.inactive.expansion,
            &mut v.widgets.hovered.expansion,
            &mut v.widgets.active.expansion,
            &mut v.widgets.open.expansion,
        ] {
            *exp = 0.0; // flat chrome: hovered widgets must not bleed past their bounds
        }
        let _ = text_faint; // read by the painted path's disabled ink, not Visuals
        v
    }

    /// The style spacing the table owns: item gaps, button padding, the
    /// field height. Defaults equal egui's own numbers, so wiring the
    /// tokens in changes no pixels until a theme file overrides them.
    #[must_use]
    pub fn style(&self, base: &Style) -> Style {
        let mut s = base.clone();
        // Clamp at the seam (the M6.1 law's other half): the const guard
        // proves the built-in tables are in range; a file-supplied
        // overlay is checked here, where it can no longer hurt.
        s.spacing.item_spacing =
            vec2(clamp::range(self.item_spacing_x, clamp::SPACING), clamp::range(self.item_spacing_y, clamp::SPACING));
        s.spacing.button_padding =
            vec2(clamp::range(self.button_padding_x, clamp::SPACING), clamp::range(self.button_padding_y, clamp::SPACING));
        s.spacing.interact_size =
            vec2(s.spacing.interact_size.x, clamp::range(self.control_height, clamp::CONTROL_H));
        s.spacing.window_margin =
            Margin::same(clamp::range(self.window_margin, clamp::WINDOW_MARGIN).round() as i8);
        s
    }
}

/// Install a token table into a context: visuals and spacing together,
/// so no theme can half-apply (Prime Directive, chrome edition). BOTH of
/// egui's theme slots get the table (#506): egui follows the OS
/// appearance, and writing only the slot active at startup (dark, before
/// the first input names the system theme) left a light-mode Mac on
/// egui's stock light visuals while the painted tree drew this table's
/// ink on it. The document's theme is the only theme; the OS never picks.
pub fn apply_to(ctx: &egui::Context, t: &Tokens) {
    let visuals = t.visuals();
    for slot in [egui::Theme::Dark, egui::Theme::Light] {
        let mut style = t.style(&ctx.style_of(slot));
        style.visuals = visuals.clone();
        ctx.set_style_of(slot, style);
    }
}

/// Install the live theme. Called per Context (windowed); the switcher
/// re-calls it on change (M6.1 C).
pub fn apply(ctx: &egui::Context) {
    apply_to(ctx, &current());
}

#[cfg(test)]
mod live_tests;
