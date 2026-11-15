//! #1994: the ribbon's three rows each have their own height. The tab row
//! clears its text, the caption row clears the button labels above it and
//! the bottom edge of the ribbon, and every ribbon glyph is white on the
//! dark ground (dark ink on the light workbench ground, as before #1994's
//! tint commit). Structural checks only: tokens and painted text positions.
//! Provenance: our own tests over our own ribbon.

use super::structural::painted;
use super::*;
use crate::egui_app::painted::glyph_ink;
use crate::egui_app::theme::{
    CAPTION_GAP, CAPTION_ROW_H, RIBBON_BOTTOM_GAP, RIBBON_CELL_TOP_GAP, RIBBON_TAB_H, Tokens,
    ribbon_h,
};

/// Text height of one line: the egui line box is about 1.2 to 1.3 times
/// the font size, so 1.3 is the estimate here.
fn line_h(px: f32) -> f32 {
    px * 1.3
}

#[test]
fn ribbon_rows_have_their_own_heights() {
    let pal = Tokens::DARK;
    // The tab row holds one line of tab text with clear space above and below.
    assert!(RIBBON_TAB_H >= line_h(pal.font_label) + 2.0 * 4.0, "tab row {RIBBON_TAB_H} px is too short");
    // The caption row holds one caption line (MIN_LABEL_PX) with clear space.
    assert!(CAPTION_ROW_H >= line_h(10.0) + 2.0 * 2.0, "caption row {CAPTION_ROW_H} px is too short");
    // The caption starts below the cell box and clear of the button label,
    // which starts at the glyph and 2 px down in the cell.
    let label_bottom = pal.glyph_px + 2.0 + line_h(pal.font_label);
    assert!(pal.cell_h + CAPTION_GAP >= label_bottom + 2.0, "caption touches the button label");
    // The rows stack inside the strip with the bottom gap left over.
    let rows_end = RIBBON_TAB_H + RIBBON_CELL_TOP_GAP + pal.cell_h + CAPTION_GAP + CAPTION_ROW_H;
    assert!(ribbon_h(&pal) >= rows_end + RIBBON_BOTTOM_GAP, "caption row meets the ribbon's bottom edge");
    assert!((ribbon_h(&pal) - (rows_end + RIBBON_BOTTOM_GAP)).abs() < f32::EPSILON, "no stray space in the ribbon");
}

#[test]
fn ribbon_glyphs_are_white_on_the_dark_ground_and_dark_on_the_light() {
    assert_eq!(glyph_ink(&Tokens::DARK, None), egui::Color32::WHITE, "a plain glyph is white on the dark ribbon");
    assert_eq!(
        glyph_ink(&Tokens::DARK, Some(Tokens::DARK.accent)),
        egui::Color32::WHITE,
        "a glyph on the accent fill is white",
    );
    // The light workbench keeps a dark ink: white would vanish on its near-white strip.
    assert_eq!(glyph_ink(&Tokens::LIGHT, None), egui::Color32::BLACK, "light theme keeps dark ink");
}

#[test]
fn ribbon_text_rows_stack_top_to_bottom() {
    let mut h = shell_harness();
    h.run_steps(3);
    let texts = painted(&h);
    let find = |want: &str| texts.iter().find(|(t, _)| t == want).map(|(_, p)| *p);
    let tab_caption = crate::machine_modes::MACHINE_MODES[0].1.to_uppercase();
    let tab = find(&tab_caption).expect("the first machine tab paints its caption");
    let new = find("New").expect("the New cell paints its label");
    let caption = find("FILE").expect("the FILE caption paints");
    let pal = Tokens::DARK;
    // The tab text ends before the button labels start.
    assert!(new.y - tab.y >= line_h(pal.font_label), "tab text runs into the button labels");
    // The caption row starts clear of the button label above it.
    assert!(caption.y - new.y >= line_h(pal.font_label) + 2.0, "caption touches the button label");
}
