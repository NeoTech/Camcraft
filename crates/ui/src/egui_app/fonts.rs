//! The interface fonts (#853): English keeps egui's own set, a translated
//! interface swaps in a face that carries its script.
//!
//! Provenance: our own font wiring over egui's `FontDefinitions` (the
//! vendored egui 0.36 docs); the font itself is Noto Sans SC, SIL OFL 1.1,
//! licence beside it in `assets/ui/fonts/OFL.txt`.
//!
//! The law this module keeps: **a language that cannot draw its own words is
//! not supported**. The source language therefore installs nothing, so the
//! English window is byte-identical to before this module existed (and every
//! existing screenshot test still passes). A translation installs its font
//! first in `Proportional`, so Latin, digits and hanzi inside one label share
//! one face and one line height - the mixed-face baseline jump is the one
//! thing that would look like "the layout broke".

use egui::{Context, FontData, FontDefinitions, FontFamily};
use std::sync::Arc;

/// The bundled Simplified Chinese face's name in `FontDefinitions`.
pub const SC: &str = "noto-sans-sc";

/// Noto Sans SC, pinned to Regular 400 and cut to GB2312 plus punctuation by
/// `scripts/fonts/subset_sc.py` (8 090 code points, 2.3 MB).
const SC_BYTES: &[u8] = include_bytes!("../../../../assets/ui/fonts/NotoSansSC-Regular-Subset.ttf");

/// The bundled face, for the tests that prove it covers the catalogue.
#[cfg(test)]
#[must_use]
pub fn sc_bytes() -> &'static [u8] {
    SC_BYTES
}

/// Install the fonts of the language in force, once per context.
pub fn install(ctx: &Context) {
    let id = egui::Id::new("camcraft.fonts.installed");
    if ctx.data(|d| d.get_temp::<bool>(id)) == Some(true) {
        return;
    }
    ctx.data_mut(|d| d.insert_temp(id, true));
    install_for(ctx, camcraft_i18n::lang());
}

/// Install the fonts of `lang`: nothing for the source language, the bundled
/// face for a translation that ships one.
pub fn install_for(ctx: &Context, lang: camcraft_i18n::Lang) {
    let mut defs = FontDefinitions::default();
    if lang.is_source() {
        ctx.set_fonts(defs);
        return;
    }
    defs.font_data
        .insert(SC.to_owned(), Arc::new(FontData::from_static(SC_BYTES)));
    // Proportional: first, so a label mixes no two faces.
    defs.families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, SC.to_owned());
    // Monospace: last, so the code-font columns keep Hack's metrics and a
    // hanzi that reaches them still has a glyph instead of a box.
    defs.families
        .entry(FontFamily::Monospace)
        .or_default()
        .push(SC.to_owned());
    ctx.set_fonts(defs);
}

#[cfg(test)]
#[path = "fonts_tests.rs"]
mod tests;
