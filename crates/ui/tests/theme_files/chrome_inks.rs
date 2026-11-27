//! The window chrome inks are theme tokens (#1120): the close button of
//! a floating window keeps its desktop red and white cross in both
//! built-in columns, a theme file sets each by name, and the colour
//! vocabulary still fits the colour map cap of a theme file.
//! Provenance: our own tests over our own token table and theme format.
//!
//! A module of the `theme_files` suite (#3187: no new test binary).

use camcraft_ui::egui_app::theme::Tokens;
use camcraft_ui::egui_app::theme_files as tf;
use egui::Color32;

#[test]
fn the_built_in_columns_keep_the_close_button_colours() {
    // `window_chrome::CLOSE_HOVER` and the white cross before #1120.
    for t in [Tokens::DARK, Tokens::LIGHT] {
        assert_eq!(t.close_hover, Color32::from_rgb(0xC4, 0x2B, 0x1C));
        assert_eq!(t.close_ink, Color32::WHITE);
    }
}

#[test]
fn each_close_button_ink_is_a_theme_key() {
    let text = r##"{"format":"camcraft-theme/1","descriptor":{"name":"c"},"base":"dark",
        "colors":{"close_hover":"#102030","close_ink":"#405060"}}"##;
    let theme = tf::parse_text(text, "c.json").unwrap_or_else(|e| panic!("the theme reads: {e}"));
    assert!(theme.notes.is_empty(), "notes: {:?}", theme.notes);
    assert_eq!(theme.tokens.close_hover, Color32::from_rgb(0x10, 0x20, 0x30));
    assert_eq!(theme.tokens.close_ink, Color32::from_rgb(0x40, 0x50, 0x60));
    assert_eq!(theme.tokens.field_hover, Tokens::DARK.field_hover, "the arrow's hover is unchanged");
}

#[test]
fn the_colour_vocabulary_fits_the_map_cap() {
    // A theme file can set every colour in one map: the cap must hold
    // the whole vocabulary.
    assert!(
        tf::COLOR_NAMES.len() <= tf::MAX_MAP_ENTRIES,
        "{} colour tokens must fit MAX_MAP_ENTRIES ({})",
        tf::COLOR_NAMES.len(),
        tf::MAX_MAP_ENTRIES,
    );
}
