//! The tool drawing inks are theme tokens (#1120): each built-in column
//! keeps the colour the draw site wrote as a literal before, a theme
//! file sets each ink by name, and a partly transparent fill keeps its
//! token's own colour.
//! Provenance: our own tests over our own token table and theme format.
//!
//! A module of the `theme_files` suite (#3187: no new test binary).

use camcraft_ui::egui_app::theme::{self, Tokens};
use camcraft_ui::egui_app::theme_files as tf;
use egui::Color32;

/// The tool drawing inks of a token table, in the order of `KEYS`.
fn inks(t: &Tokens) -> [Color32; 18] {
    [
        t.tool_ground,
        t.tool_axis,
        t.tool_grid,
        t.tool_flute,
        t.tool_shank,
        t.tool_body,
        t.tool_edge,
        t.tool_holder,
        t.tool_note,
        t.tool_dim,
        t.thumb_flute,
        t.bad_ink,
        t.warn_ink,
        t.mount_axis,
        t.mount_radial,
        t.mount_tip,
        t.mount_origin,
        t.mount_hint,
    ]
}

const KEYS: [&str; 18] = [
    "tool_ground",
    "tool_axis",
    "tool_grid",
    "tool_flute",
    "tool_shank",
    "tool_body",
    "tool_edge",
    "tool_holder",
    "tool_note",
    "tool_dim",
    "thumb_flute",
    "bad_ink",
    "warn_ink",
    "mount_axis",
    "mount_radial",
    "mount_tip",
    "mount_origin",
    "mount_hint",
];

fn parse(colors: &str) -> Tokens {
    let text = format!(
        r#"{{"format":"camcraft-theme/1","descriptor":{{"name":"t"}},"base":"dark","colors":{{{colors}}}}}"#
    );
    let theme = tf::parse_text(&text, "t.json").unwrap_or_else(|e| panic!("the theme reads: {e}"));
    assert!(theme.notes.is_empty(), "notes: {:?}", theme.notes);
    theme.tokens
}

#[test]
fn the_built_in_columns_keep_the_tool_drawing_colours() {
    // The literals and consts of the tool pages, the tool library views
    // and the tool body page before #1120. The previews draw on their own
    // dark ground, so both columns hold the same set.
    let before = [
        Color32::from_rgb(24, 26, 31),
        Color32::from_rgb(70, 74, 84),
        Color32::from_rgb(80, 86, 98),
        Color32::from_rgb(168, 178, 192),
        Color32::from_rgb(122, 130, 142),
        Color32::from_rgb(86, 92, 104),
        Color32::from_rgb(214, 220, 228),
        Color32::from_rgb(230, 160, 60),
        Color32::from_rgb(0x9E, 0xA8, 0xB8),
        Color32::from_rgb(0x7F, 0xB4, 0xD8),
        Color32::from_rgb(200, 160, 80),
        Color32::from_rgb(230, 90, 80),
        Color32::from_rgb(230, 170, 60),
        Color32::from_rgb(90, 200, 230),
        Color32::from_rgb(240, 180, 60),
        Color32::from_rgb(240, 150, 40),
        Color32::from_rgb(160, 160, 170),
        Color32::from_rgb(240, 200, 120),
    ];
    assert_eq!(inks(&Tokens::DARK), before, "dark column");
    assert_eq!(inks(&Tokens::LIGHT), before, "light column");
}

#[test]
fn each_tool_drawing_ink_is_a_theme_key() {
    for (at, name) in KEYS.iter().enumerate() {
        assert!(
            tf::COLOR_NAMES.contains(name),
            "{name} is a colour key of the theme format"
        );
        let t = parse(&format!(r##""{name}":"#0A0B0C""##));
        assert_eq!(
            inks(&t)[at],
            Color32::from_rgb(10, 11, 12),
            "{name} sets its own ink"
        );
    }
}

#[test]
fn a_holder_fill_keeps_the_colour_of_its_token() {
    // The built-in holder gives the fills the profile (alpha 90) and the
    // preview (alpha 70) drew before.
    let holder = Tokens::DARK.tool_holder;
    assert_eq!(
        theme::with_alpha(holder, 90),
        Color32::from_rgba_unmultiplied(230, 160, 60, 90)
    );
    assert_eq!(
        theme::with_alpha(holder, 70),
        Color32::from_rgba_unmultiplied(230, 160, 60, 70)
    );
    // A partly transparent token keeps its own colour: no second darkening.
    let half_red = parse(r##""tool_holder":"#FF000080""##).tool_holder;
    assert_eq!(
        theme::with_alpha(half_red, 70),
        Color32::from_rgba_unmultiplied(255, 0, 0, 70)
    );
}
