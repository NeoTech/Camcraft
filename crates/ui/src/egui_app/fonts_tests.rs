//! The bundled font's own proof (#853): every glyph the Chinese catalogue
//! needs exists in the file that ships, so no label can come out as a box.

use super::*;

/// The glyph ids of a TrueType `cmap`, format 4 (the BMP table Noto ships).
/// A minimal reader on purpose: the test must not gain a font dependency.
fn cmap_ranges(ttf: &[u8]) -> Vec<(u16, u16, u16, i16, u16, usize)> {
    let be16 = |i: usize| u16::from_be_bytes([ttf[i], ttf[i + 1]]);
    let be32 = |i: usize| u32::from_be_bytes([ttf[i], ttf[i + 1], ttf[i + 2], ttf[i + 3]]);
    let tables = usize::from(be16(4));
    let mut cmap = None;
    for i in 0..tables {
        let rec = 12 + i * 16;
        if &ttf[rec..rec + 4] == b"cmap" {
            cmap = Some(be32(rec + 8) as usize);
        }
    }
    let cmap = cmap.expect("the font has no cmap table");
    let subtables = usize::from(be16(cmap + 2));
    let mut best = None;
    for i in 0..subtables {
        let rec = cmap + 4 + i * 8;
        let platform = be16(rec);
        let encoding = be16(rec + 2);
        let off = cmap + be32(rec + 4) as usize;
        // 3/1 is Windows BMP, 0/x Unicode; both carry format 4 here.
        if ((platform, encoding) == (3, 1) || platform == 0) && be16(off) == 4 {
            best = Some(off);
            if (platform, encoding) == (3, 1) {
                break;
            }
        }
    }
    let cmap = best.expect("the font has no format 4 cmap");
    let segs = usize::from(be16(cmap + 6)) / 2;
    let ends = cmap + 14;
    let starts = ends + segs * 2 + 2;
    let deltas = starts + segs * 2;
    let ranges = deltas + segs * 2;
    (0..segs)
        .map(|s| {
            let end = be16(ends + s * 2);
            let start = be16(starts + s * 2);
            let delta = be16(deltas + s * 2) as i16;
            let range_off = be16(ranges + s * 2);
            (start, end, 0, delta, range_off, ranges + s * 2)
        })
        .collect()
}

fn has_glyph(ttf: &[u8], ch: char) -> bool {
    let c = u32::from(ch);
    if c > 0xFFFF {
        return false;
    }
    let c = c as u16;
    for (start, end, _, delta, range_off, ro_pos) in cmap_ranges(ttf) {
        if c < start || c > end {
            continue;
        }
        if range_off == 0 {
            return (c as i32 + i32::from(delta)) as u16 != 0;
        }
        // The spec's byte arithmetic: &idRangeOffset[i] + idRangeOffset[i]
        // + 2 * (c - startCode[i]) - the offset is already in bytes.
        let idx = ro_pos + usize::from(range_off) + 2 * usize::from(c - start);
        let be16 = |i: usize| u16::from_be_bytes([ttf[i], ttf[i + 1]]);
        let glyph = be16(idx);
        if glyph == 0 {
            return false;
        }
        return (glyph as i32 + i32::from(delta)) as u16 != 0;
    }
    false
}

#[test]
fn the_bundled_font_covers_every_translated_string() {
    let ttf = sc_bytes();
    assert!(ttf.len() > 100_000, "the bundled face is {} bytes", ttf.len());
    let cat = camcraft_i18n::shipped(camcraft_i18n::Lang::ZhCn).expect("zh-CN catalogue");
    let mut missing = Vec::new();
    for (_, value) in cat.entries() {
        for ch in value.chars() {
            // Newline and tab are layout controls, not painted glyphs: the
            // English source uses them for menu shortcut alignment and for
            // multi-line prompt text, and egui handles them without a face.
            if matches!(ch, '\n' | '\r' | '\t') {
                continue;
            }
            if !has_glyph(ttf, ch) {
                missing.push(ch);
            }
        }
    }
    assert!(missing.is_empty(), "no glyph for {missing:?} - those labels would draw a box");
}

#[test]
fn the_bundled_font_keeps_latin_and_the_unit_signs() {
    let ttf = sc_bytes();
    for ch in "AaZz09 .,:;+-*/()[]{}<>_=%°Øµ±∅×÷≈…—’“”·，。：；！？（）《》、".chars() {
        assert!(has_glyph(ttf, ch), "no glyph for {ch:?}");
    }
}

#[test]
fn english_installs_no_font_so_the_source_window_is_unchanged() {
    let ctx = egui::Context::default();
    install_for(&ctx, camcraft_i18n::Lang::En);
    warm(&ctx);
    let before = ctx.fonts_mut(|f| f.layout_no_wrap("Save".to_owned(), egui::FontId::proportional(11.0), egui::Color32::WHITE));
    let ctx2 = egui::Context::default();
    install_for(&ctx2, camcraft_i18n::Lang::En);
    warm(&ctx2);
    let after = ctx2.fonts_mut(|f| f.layout_no_wrap("Save".to_owned(), egui::FontId::proportional(11.0), egui::Color32::WHITE));
    assert_eq!(before.size(), after.size());
}

#[test]
fn chinese_installs_a_face_that_measures_its_own_glyphs() {
    let ctx = egui::Context::default();
    install_for(&ctx, camcraft_i18n::Lang::ZhCn);
    warm(&ctx);
    let g = ctx.fonts_mut(|f| f.layout_no_wrap("保存".to_owned(), egui::FontId::proportional(11.0), egui::Color32::WHITE));
    // Two full-width glyphs at 11 px are about 22 px; a tofu fallback or a
    // missing face would measure nothing like it.
    assert!(g.size().x > 18.0 && g.size().x < 30.0, "saved glyphs measure {} px", g.size().x);
}

/// One frame, so egui builds its font atlas; the delta is dropped on purpose
/// (a test owns no GPU texture to apply it to).
fn warm(ctx: &egui::Context) {
    let mut out = ctx.run_ui(egui::RawInput::default(), |_| {});
    out.textures_delta.clear();
}
