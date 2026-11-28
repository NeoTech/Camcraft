//! The contrast law of the theme (M6.1 D), in the UI kit crate (#4394):
//! both built-in columns clear their own readability gate, the follow law
//! is an identity on its base and a translation on a new accent, an
//! accent-only file derives its family and says so, and an unreadable
//! file refuses by name. Moved unchanged from `crates/ui/tests/theme_files.rs`
//! with the module; this binary builds egui and serde_json only.
//! Provenance: our own tests, specced from the ROADMAP's M6.1 notes and
//! WCAG 2.1's contrast definition.
//!
//! Gate map (M4.4): covers ui_kit::contrast, ui_kit::theme_files, ui_kit::theme.

use camcraft_ui_kit::contrast;
use camcraft_ui_kit::theme::Tokens;
use camcraft_ui_kit::theme_files as tf;
use egui::Color32;

/// M6.1 D, the arithmetic itself: both built-in columns must clear
/// the gate they enforce on files - a no-op file lands its base
/// exactly, so its base had better be legal - and `on_fill`'s choices
/// on the four built-in fills are the snapshot's memory.
#[test]
fn both_builtin_columns_clear_their_own_gate() {
    assert_eq!(contrast::readable(&Tokens::DARK), None, "dark must read");
    assert_eq!(contrast::readable(&Tokens::LIGHT), None, "light must read");
    assert!((contrast::ratio(Color32::BLACK, Color32::WHITE) - 21.0).abs() < 0.01, "the formula's ceiling");
    assert!((contrast::ratio(Color32::WHITE, Color32::WHITE) - 1.0).abs() < 0.001, "and its floor");
    assert_eq!(contrast::on_fill(Tokens::DARK.accent), Color32::WHITE);
    assert_eq!(contrast::on_fill(Tokens::LIGHT.accent), Color32::WHITE);
    // The finding the derivation made for us: the pressed fill (the
    // hover accent, one step lighter) never read 4.5:1 with white -
    // 3.8 dark, 4.3 light - while black clears both. The first
    // tables' hand-typed white there was a bug wearing a habit;
    // visuals() now asks the arithmetic, and pressed chrome reads.
    assert_eq!(contrast::on_fill(Tokens::DARK.accent_hover), Color32::BLACK);
    assert_eq!(contrast::on_fill(Tokens::LIGHT.accent_hover), Color32::BLACK);
}

/// The follow law is an identity on its own base (the built-ins stay
/// byte-for-byte the hand-tuned tables, snapshots intact) and a
/// translation on a new one.
#[test]
fn the_follow_law_round_trips_and_translates() {
    for base in [Tokens::DARK, Tokens::LIGHT] {
        assert_eq!(contrast::follow(base.accent, &base, base.accent_hover), base.accent_hover);
        assert_eq!(contrast::follow(base.accent, &base, base.field_active), base.field_active);
        assert_eq!(contrast::follow(base.accent, &base, base.selection_bg), base.selection_bg);
    }
    // Hover's step on both columns is a lift; onto a rust accent it
    // stays a lift. The number is the law's - pinned.
    let rust = Color32::from_rgb(0xC0, 0x50, 0x4D);
    let hover = contrast::follow(rust, &Tokens::DARK, Tokens::DARK.accent_hover);
    assert_eq!(hover, Color32::from_rgb(0xC5, 0x71, 0x82));
    assert!(contrast::luma(hover) > contrast::luma(rust), "hover lifts");
}

/// An accent-only file gets the whole family for free - and says so,
/// one note per state that moved. This is the ROADMAP's sentence
/// rendered: hover/active/selection generated from the accent, not
/// trusted from a hand.
#[test]
fn an_accent_only_file_derives_its_family_and_says_so() {
    let t = tf::parse_text(
        r##"{"format":"camcraft-theme/1","descriptor":{"name":"Rust"},"base":"dark",
            "colors":{"accent":"#C0504D"}}"##,
        "rust",
    )
    .expect("a red accent is legal - the derivation carries the family with it");
    assert_eq!(t.tokens.accent_hover, Color32::from_rgb(0xC5, 0x71, 0x82)); // lifts
    assert_eq!(t.tokens.selection_stroke, t.tokens.accent_hover, "the stroke rides the hover");
    assert_eq!(t.tokens.field_active, Color32::from_rgb(0xBC, 0x00, 0x00));
    assert_eq!(t.tokens.selection_bg, Color32::from_rgb(0xBD, 0x00, 0x00));
    assert_eq!(t.notes.len(), 4, "{:?}", t.notes); // hover, active, bg, stroke
    for kw in ["accent_hover", "field_active", "selection_bg", "selection_stroke"] {
        assert!(t.notes.iter().any(|n| n.contains(kw)), "{:?}", t.notes);
    }
    assert_eq!(contrast::readable(&t.tokens), None, "and the derived table still reads");
}

/// The gate's refusals, each named with its pair and its ratio: an
/// unreadable theme is refused, not rendered (Prime Directive, chrome
/// edition) - and never half-lands.
#[test]
fn unreadable_files_refuse_by_name() {
    for (key, value, want) in [
        ("text", "#404040", "text on ink"),
        ("text_dim", "#6E757E", "text_dim on ink"),
        ("selection_stroke", "#2E3136", "selection_stroke on ink_alt"),
    ] {
        let text = format!(
            r#"{{"format":"camcraft-theme/1","descriptor":{{"name":"B"}},"base":"dark","colors":{{"{key}":"{value}"}}}}"#
        );
        let err = tf::parse_text(&text, "gate").expect_err(want);
        assert!(err.contains(want), "{err}");
    }
}
