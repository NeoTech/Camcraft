//! The gallery page of the UI kit (#4394): it lists every token of the
//! table, it agrees with the theme-file vocabulary, its reading pairs
//! agree with the readability gate, and both themes lay out in a plain
//! egui context (no renderer, no window) without a panic.
//! Provenance: our own tests over our own token table.
//!
//! Gate map (M4.4): covers ui_kit::gallery, ui_kit::theme, ui_kit::theme_files.

use std::collections::BTreeSet;

use camcraft_ui_kit::gallery;
use camcraft_ui_kit::theme::Tokens;
use camcraft_ui_kit::theme_files::{COLOR_NAMES, GEOM_NAMES};
use egui::Color32;

#[test]
fn the_gallery_lists_each_token_once_by_its_file_name() {
    for t in [Tokens::DARK, Tokens::LIGHT] {
        let colours: Vec<&str> = gallery::colours(&t).into_iter().map(|(n, _)| n).collect();
        let geometry: Vec<&str> = gallery::geometry(&t).into_iter().map(|(n, _)| n).collect();
        let unique: BTreeSet<&str> = colours.iter().chain(&geometry).copied().collect();
        assert_eq!(unique.len(), colours.len() + geometry.len(), "a row name is not unique");
        // Every name a theme file may set is a row of the page.
        for name in COLOR_NAMES {
            assert!(colours.contains(name), "colour token {name} is missing from the gallery");
        }
        for name in GEOM_NAMES {
            assert!(geometry.contains(name), "geometry token {name} is missing from the gallery");
        }
    }
    assert_eq!(gallery::colours(&Tokens::DARK)[0], ("ink", Tokens::DARK.ink));
    assert_eq!(gallery::geometry(&Tokens::LIGHT)[0], ("radius_widget", f32::from(Tokens::LIGHT.radius_widget)));
}

#[test]
fn hex_spells_colours_the_way_a_theme_file_does() {
    assert_eq!(gallery::hex(Color32::from_rgb(0x23, 0x26, 0x2B)), "#23262B");
    assert_eq!(gallery::hex(Color32::from_rgba_unmultiplied(0x10, 0x20, 0x30, 0x80)), "#10203080");
}

#[test]
fn the_reading_pairs_clear_their_floors_on_both_builtins() {
    for t in [Tokens::DARK, Tokens::LIGHT] {
        let pairs = gallery::reading_pairs(&t);
        assert_eq!(pairs.len(), 10, "the ten pairs of contrast::readable");
        for (pair, ratio, floor) in pairs {
            assert!(ratio >= floor, "{pair} reads {ratio:.2}:1, under {floor}:1");
        }
    }
}

#[test]
fn both_themes_lay_out_in_a_headless_context() {
    let ctx = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(960.0, 900.0))),
        ..Default::default()
    };
    let mut out = ctx.run_ui(input, |ui| {
        egui::ScrollArea::vertical().show(ui, gallery::show);
    });
    assert!(!out.shapes.is_empty(), "the gallery paints");
    // No renderer takes the font atlas here; egui asks that an unused
    // texture delta is cleared on purpose before it drops.
    out.textures_delta.clear();
}
