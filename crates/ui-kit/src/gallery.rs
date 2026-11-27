//! The design-system gallery (#4394): one page that shows every token of
//! a theme - colour swatches, geometry numbers, the readability pairs the
//! gate checks, and a few stock widgets in the theme's visuals - for the
//! dark and the light column side by side. Reviewers compare the kit with
//! the mockups here; later PRs add their components. The page is plain
//! data in, pixels out: it reads no document and keeps no state.
//! The look follows the #4245 design rules: a flat frame with a hairline,
//! no shadow, a small radius, section headers from type with a thin rule.
//! Provenance: our own module over our own token table; no HSMWorks material.

use egui::{Color32, CornerRadius, Frame, Grid, Margin, RichText, Sense, Stroke, Ui, vec2};

use crate::contrast;
use crate::theme::Tokens;

mod rows;
pub use rows::{colours, geometry};

/// The colour as `#RRGGBB`, or `#RRGGBBAA` when it is not opaque - the
/// spelling a `camcraft-theme/1` file uses.
#[must_use]
pub fn hex(c: Color32) -> String {
    let [r, g, b, a] = c.to_srgba_unmultiplied();
    if a == u8::MAX { format!("#{r:02X}{g:02X}{b:02X}") } else { format!("#{r:02X}{g:02X}{b:02X}{a:02X}") }
}

/// The reading pairs of the readability gate (`contrast::readable`), each
/// with its ratio and its floor, in the order the gate checks them.
#[must_use]
pub fn reading_pairs(t: &Tokens) -> Vec<(String, f64, f64)> {
    let mut out = Vec::new();
    for (ground, bg) in [("ink", t.ink), ("ink_alt", t.ink_alt), ("well", t.well), ("field_bg", t.field_bg)] {
        out.push((format!("text on {ground}"), contrast::ratio(t.text, bg), contrast::BODY));
    }
    for (ground, bg) in [("ink", t.ink), ("ink_alt", t.ink_alt)] {
        out.push((format!("text_dim on {ground}"), contrast::ratio(t.text_dim, bg), contrast::BODY));
    }
    for (name, fill) in [("accent", t.accent), ("accent_hover", t.accent_hover)] {
        out.push((format!("ink on {name}"), contrast::ratio(contrast::on_fill(fill), fill), contrast::BODY));
    }
    out.push(("ribbon_ink on ink_alt".to_owned(), contrast::ratio(t.ribbon_ink, t.strip_fill()), contrast::BODY));
    out.push(("selection_stroke on ink_alt".to_owned(), contrast::ratio(t.selection_stroke, t.ink_alt), contrast::UI));
    out
}

/// Both built-in columns, side by side: dark left, light right.
pub fn show(ui: &mut Ui) {
    ui.columns(2, |cols| {
        if let [left, right] = cols {
            show_theme(left, "Dark", &Tokens::DARK);
            show_theme(right, "Light", &Tokens::LIGHT);
        }
    });
}

/// One theme's page, drawn in that theme's own visuals and spacing.
pub fn show_theme(ui: &mut Ui, title: &str, t: &Tokens) {
    ui.scope(|ui| {
        let mut style = t.style(ui.style());
        style.visuals = t.visuals();
        ui.set_style(style);
        Frame::NONE
            .fill(t.ink)
            .stroke(Stroke::new(1.0, t.border))
            .corner_radius(CornerRadius::same(t.radius_widget))
            .inner_margin(Margin::same(8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new(title).size(t.font_row + t.tree_header_boost).color(t.text).strong());
                section(ui, t, "Widgets");
                widgets(ui, t);
                section(ui, t, "Readability");
                readability(ui, t, title);
                section(ui, t, "Colours");
                colour_grid(ui, t, title);
                section(ui, t, "Geometry");
                geometry_grid(ui, t, title);
            });
    });
}

/// A section header from type: a small dim caption over a thin rule.
fn section(ui: &mut Ui, t: &Tokens, name: &str) {
    ui.add_space(t.item_spacing_y * 2.0);
    ui.label(RichText::new(name).size(t.font_caption).color(t.text_dim).strong());
    ui.add(egui::Separator::default().spacing(2.0));
}

/// Stock widgets in the theme's visuals: the states a reviewer checks.
fn widgets(ui: &mut Ui, t: &Tokens) {
    ui.horizontal(|ui| {
        let _ = ui.button("Generate");
        ui.add_enabled(false, egui::Button::new("Post"));
        let _ = ui.selectable_label(true, "Selected");
        let _ = ui.selectable_label(false, "Plain");
    });
    ui.horizontal(|ui| {
        let mut on = true;
        let _ = ui.checkbox(&mut on, "Checked");
        let mut text = String::from("12.500");
        ui.add(egui::TextEdit::singleline(&mut text).desired_width(t.cell_w));
        ui.label(RichText::new("disabled ink").color(t.text_faint));
    });
}

fn readability(ui: &mut Ui, t: &Tokens, title: &str) {
    Grid::new(("kit_gallery_reading", title)).num_columns(3).show(ui, |ui| {
        for (pair, ratio, floor) in reading_pairs(t) {
            ui.label(pair);
            ui.label(RichText::new(format!("{ratio:.1}:1")).monospace());
            let verdict = if ratio >= floor { format!("ok (floor {floor}:1)") } else { format!("UNDER {floor}:1") };
            ui.label(RichText::new(verdict).color(t.text_dim));
            ui.end_row();
        }
    });
}

fn colour_grid(ui: &mut Ui, t: &Tokens, title: &str) {
    Grid::new(("kit_gallery_colours", title)).num_columns(3).show(ui, |ui| {
        for (name, c) in colours(t) {
            swatch(ui, t, c);
            ui.label(name);
            ui.label(RichText::new(hex(c)).monospace().color(t.text_dim));
            ui.end_row();
        }
    });
}

fn geometry_grid(ui: &mut Ui, t: &Tokens, title: &str) {
    Grid::new(("kit_gallery_geometry", title)).num_columns(2).show(ui, |ui| {
        for (name, v) in geometry(t) {
            ui.label(name);
            ui.label(RichText::new(format!("{v}")).monospace().color(t.text_dim));
            ui.end_row();
        }
    });
}

/// A flat colour chip with a hairline, so a colour equal to the ground
/// still shows its edge.
fn swatch(ui: &mut Ui, t: &Tokens, c: Color32) {
    let side = t.font_row + 2.0;
    let (rect, _) = ui.allocate_exact_size(vec2(side * 2.0, side), Sense::hover());
    let r = CornerRadius::same(t.radius_widget.min(2));
    ui.painter().rect_filled(rect, r, c);
    ui.painter().rect_stroke(rect, r, Stroke::new(1.0, t.border), egui::StrokeKind::Inside);
}
