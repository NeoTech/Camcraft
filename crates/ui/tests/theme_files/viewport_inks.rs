//! The viewport overlay inks are theme tokens (#1120): each built-in
//! column keeps the colour the draw site wrote as a literal before, so
//! the move changes no pixel, and a theme file can set each ink by name.
//! The view cube inks follow the viewport ground, as they did before.
//! Provenance: our own tests over our own token table and theme format.
//!
//! A module of the `theme_files` suite (#3187: no new test binary).

use camcraft_ui::egui_app::theme::Tokens;
use camcraft_ui::egui_app::theme_files as tf;
use egui::Color32;

/// The handle and pick inks of a token table.
fn inks(t: &Tokens) -> [Color32; 3] {
    [t.handle_shade, t.handle_text, t.pick_rim]
}

#[test]
fn the_built_in_columns_keep_the_handle_and_pick_colours() {
    // The literals of `move_handles.rs`, `turn_handles.rs` and
    // `align_pick_viz.rs` before #1120, in both columns: the handles and
    // the picks draw over the 3D view, whose ink does not follow the
    // chrome's column.
    let before = [
        Color32::from_black_alpha(170),
        Color32::WHITE,
        Color32::from_rgb(245, 240, 250),
    ];
    assert_eq!(inks(&Tokens::DARK), before, "dark column");
    assert_eq!(inks(&Tokens::LIGHT), before, "light column");
}

#[test]
fn a_theme_file_sets_each_handle_and_pick_ink_by_name() {
    let text = r##"{
        "format": "camcraft-theme/1",
        "descriptor": { "name": "inks" },
        "base": "dark",
        "colors": { "handle_shade": "#10203040", "handle_text": "#FFEE00", "pick_rim": "#00FF88" }
    }"##;
    let theme =
        tf::parse_text(text, "inks.json").unwrap_or_else(|e| panic!("the theme reads: {e}"));
    assert_eq!(
        inks(&theme.tokens),
        [
            Color32::from_rgba_unmultiplied(0x10, 0x20, 0x30, 0x40),
            Color32::from_rgb(0xFF, 0xEE, 0x00),
            Color32::from_rgb(0x00, 0xFF, 0x88),
        ]
    );
    assert!(
        theme.notes.is_empty(),
        "each key is known: {:?}",
        theme.notes
    );
    for name in ["handle_shade", "handle_text", "pick_rim"] {
        assert!(
            tf::COLOR_NAMES.contains(&name),
            "{name} is a colour key of the theme format"
        );
    }
}

/// The view cube, marker, axis and hover inks of a token table.
fn furniture(t: &Tokens) -> [Color32; 12] {
    [
        t.cube_edge,
        t.cube_label,
        t.cube_glyph_on,
        t.cube_glyph_off,
        t.viewport_hint,
        t.marker_stem,
        t.marker_ring,
        t.marker_dot,
        t.axis_x,
        t.axis_y,
        t.axis_z,
        t.viewport_hover,
    ]
}

const FURNITURE_KEYS: [&str; 12] = [
    "cube_edge",
    "cube_label",
    "cube_glyph_on",
    "cube_glyph_off",
    "viewport_hint",
    "marker_stem",
    "marker_ring",
    "marker_dot",
    "axis_x",
    "axis_y",
    "axis_z",
    "viewport_hover",
];

/// The gold of the view cube's hover dot and the machined facet's hover.
const GOLD: Color32 = Color32::from_rgb(255, 217, 102);

#[test]
fn the_built_in_columns_keep_the_view_cube_and_marker_colours() {
    // `ground_ink::cube_ink` before #1120: the dark set, then the light set.
    let dark = [
        Color32::from_white_alpha(170),
        Color32::from_white_alpha(230),
        Color32::WHITE,
        Color32::from_white_alpha(160),
        Color32::from_white_alpha(200),
        Color32::from_gray(225),
        Color32::from_gray(240),
        Color32::WHITE,
        Color32::from_rgb(245, 70, 70),
        Color32::from_rgb(60, 220, 80),
        Color32::from_rgb(90, 140, 255),
        GOLD,
    ];
    let light = [
        Color32::from_gray(90),
        Color32::from_gray(20),
        Color32::from_gray(10),
        Color32::from_gray(60),
        Color32::from_gray(60),
        Color32::from_gray(50),
        Color32::from_gray(40),
        Color32::from_gray(20),
        Color32::from_rgb(215, 20, 20),
        Color32::from_rgb(0, 140, 30),
        Color32::from_rgb(20, 70, 230),
        GOLD,
    ];
    assert_eq!(furniture(&Tokens::DARK), dark, "dark column");
    assert_eq!(furniture(&Tokens::LIGHT), light, "light column");
}

fn hex(c: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", c.r(), c.g(), c.b())
}

fn parse(base: &str, colors: &str) -> Tokens {
    let text = format!(
        r#"{{"format":"camcraft-theme/1","descriptor":{{"name":"g"}},"base":"{base}","colors":{{{colors}}}}}"#
    );
    let theme = tf::parse_text(&text, "g.json").unwrap_or_else(|e| panic!("the theme reads: {e}"));
    assert!(theme.notes.is_empty(), "notes: {:?}", theme.notes);
    theme.tokens
}

#[test]
fn the_view_cube_inks_follow_the_viewport_ground_of_a_theme_file() {
    // Before #1120 the cube chose its set by the viewport ground. A file
    // that turns the ground light on the dark base keeps that: each key
    // it omits takes the light table's value.
    let light_vp = format!(r#""viewport":"{}""#, hex(Tokens::LIGHT.viewport));
    assert_eq!(
        furniture(&parse("dark", &light_vp)),
        furniture(&Tokens::LIGHT)
    );
    let dark_vp = format!(r#""viewport":"{}""#, hex(Tokens::DARK.viewport));
    assert_eq!(
        furniture(&parse("light", &dark_vp)),
        furniture(&Tokens::DARK)
    );
    // A ground of the base's own lightness keeps the base's set.
    let near = format!(
        r#""viewport":"{}""#,
        hex(Color32::from_rgb(0x20, 0x22, 0x26))
    );
    assert_eq!(furniture(&parse("dark", &near)), furniture(&Tokens::DARK));
    // A key the file sets wins over the ground's set.
    let both = format!(r##"{light_vp},"axis_x":"#123456""##);
    let t = parse("dark", &both);
    assert_eq!(t.axis_x, Color32::from_rgb(0x12, 0x34, 0x56));
    assert_eq!(t.axis_y, Tokens::LIGHT.axis_y);
}

#[test]
fn each_view_cube_ink_is_a_theme_key() {
    for name in FURNITURE_KEYS {
        assert!(
            tf::COLOR_NAMES.contains(&name),
            "{name} is a colour key of the theme format"
        );
        let t = parse("dark", &format!(r##""{name}":"#0A0B0C""##));
        let set = furniture(&t);
        let at = FURNITURE_KEYS
            .iter()
            .position(|k| *k == name)
            .unwrap_or(usize::MAX);
        assert_eq!(
            set.get(at),
            Some(&Color32::from_rgb(10, 11, 12)),
            "{name} sets its own ink"
        );
    }
}

#[test]
fn the_facet_veil_keeps_the_hover_colour_of_a_semi_transparent_token() {
    use camcraft_ui::egui_app::ground_ink::facet_veil;
    // The built-in gold gives the veil it gave before #1120.
    assert_eq!(
        facet_veil(GOLD),
        Color32::from_rgba_unmultiplied(255, 217, 102, 90)
    );
    // The probe of the #3993 review: `#FF000080` gave [45, 0, 0, 90],
    // because the stored channels are already multiplied by alpha.
    let half_red = parse("dark", r##""viewport_hover":"#FF000080""##).viewport_hover;
    assert_eq!(facet_veil(half_red).to_array(), [90, 0, 0, 90]);
}

/// The overlay inks of a token table, in the order of `OVERLAY_KEYS`.
fn overlays(t: &Tokens) -> [Color32; 10] {
    [
        t.overlay_panel,
        t.overlay_edge,
        t.overlay_text,
        t.overlay_track,
        t.drop_veil,
        t.empty_hint,
        t.pick_halo,
        t.plate_hole,
        t.plate_dowel,
        t.gcode_line,
    ]
}

const OVERLAY_KEYS: [&str; 10] = [
    "overlay_panel",
    "overlay_edge",
    "overlay_text",
    "overlay_track",
    "drop_veil",
    "empty_hint",
    "pick_halo",
    "plate_hole",
    "plate_dowel",
    "gcode_line",
];

#[test]
fn the_built_in_columns_keep_the_overlay_colours() {
    // The literals of `gen_overlay.rs`, `file_drop.rs`, `shell.rs` and
    // `pick_ink.rs` before #1120, in both columns: these draw sites did
    // not follow the ground.
    let before = [
        Color32::from_black_alpha(200),
        Color32::from_white_alpha(60),
        Color32::WHITE,
        Color32::from_white_alpha(40),
        Color32::from_black_alpha(140),
        Color32::from_white_alpha(150),
        Color32::from_black_alpha(camcraft_ui::pick_ink::HALO_ALPHA),
        Color32::from_rgb(230, 190, 90),
        Color32::from_rgb(120, 200, 255),
        Color32::from_rgb(255, 170, 60),
    ];
    assert_eq!(overlays(&Tokens::DARK), before, "dark column");
    assert_eq!(overlays(&Tokens::LIGHT), before, "light column");
    // The origin triads' outline reads the handle shadow: the same 170.
    assert_eq!(Tokens::DARK.handle_shade, Color32::from_black_alpha(170));
}

#[test]
fn each_overlay_ink_is_a_theme_key() {
    for (at, name) in OVERLAY_KEYS.iter().enumerate() {
        assert!(
            tf::COLOR_NAMES.contains(name),
            "{name} is a colour key of the theme format"
        );
        let t = parse("light", &format!(r##""{name}":"#0A0B0C""##));
        assert_eq!(
            overlays(&t)[at],
            Color32::from_rgb(10, 11, 12),
            "{name} sets its own ink"
        );
    }
    // A key the file omits keeps its base value; the ground rule of
    // the view cube inks does not touch the overlays.
    let dark_vp = format!(r#""viewport":"{}""#, hex(Tokens::DARK.viewport));
    assert_eq!(
        overlays(&parse("light", &dark_vp)),
        overlays(&Tokens::LIGHT)
    );
}
