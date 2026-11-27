//! The verdict palettes are in the token table (#1120): the six
//! colour-vision sets and the house text inks keep the values that
//! `colorvision.rs` and `ground_ink.rs` wrote before the move. They stay
//! ours: no theme-file key sets them.
//! Provenance: our own tests over our own palette table; the expected
//! numbers are the literals of the two files before the move.
//!
//! A module of the `theme_files` suite (#3187: no new test binary).

use camcraft_ui::egui_app::colorvision::{Semantic, TextInks, palette_for};
use camcraft_ui::egui_app::theme_files as tf;
use camcraft_app::schema::ColorVision;
use egui::Color32;

const HOUSE: TextInks = TextInks {
    warn: Color32::from_rgb(255, 143, 0),
    error: Color32::from_rgb(0xFF, 0x59, 0x59),
    missing: Color32::from_rgb(0xB8, 0xB8, 0xC8),
};

const OFF: Semantic = Semantic {
    clean: [0.42, 0.50, 0.62],
    rest: [0.98, 0.72, 0.20],
    gouge: [0.90, 0.18, 0.14],
    unjudged: [0.45, 0.46, 0.48],
    feed: [0.20, 0.85, 0.35],
    rapid: [0.30, 0.50, 0.95],
    plunge: [0.92, 0.30, 0.25],
    select_hot: [0.98, 0.72, 0.20],
    select_warm: [0.55, 0.35, 0.05],
    bound_include: [0.20, 0.75, 0.95],
    bound_exclude: [0.92, 0.25, 0.60],
    bound_open: [0.98, 0.62, 0.12],
    text_dark: HOUSE,
    text_light: HOUSE,
};

const RED_GREEN: Semantic = Semantic {
    clean: [0.42, 0.50, 0.62],
    rest: [0.98, 0.85, 0.20],
    gouge: [0.75, 0.10, 0.10],
    unjudged: [0.80, 0.80, 0.80],
    feed: [0.95, 0.85, 0.25],
    rapid: [0.30, 0.50, 0.95],
    plunge: [0.75, 0.30, 0.00],
    select_hot: [0.98, 0.85, 0.20],
    select_warm: [0.55, 0.48, 0.10],
    bound_include: [0.25, 0.45, 1.00],
    bound_exclude: [0.95, 0.90, 0.30],
    bound_open: [0.95, 0.95, 0.95],
    text_dark: TextInks {
        warn: Color32::from_rgb(255, 230, 51),
        error: Color32::from_rgb(204, 140, 255),
        missing: Color32::from_rgb(255, 230, 51),
    },
    text_light: TextInks {
        warn: Color32::from_rgb(140, 77, 0),
        error: Color32::from_rgb(140, 0, 217),
        missing: Color32::from_rgb(140, 77, 0),
    },
};

const TRITAN: Semantic = Semantic {
    clean: [0.42, 0.50, 0.62],
    rest: [1.00, 0.45, 0.70],
    gouge: [0.80, 0.05, 0.05],
    unjudged: [0.80, 0.80, 0.80],
    feed: [0.20, 0.85, 0.35],
    rapid: [0.20, 0.40, 0.60],
    plunge: [0.92, 0.30, 0.25],
    select_hot: [1.00, 0.45, 0.70],
    select_warm: [0.56, 0.25, 0.40],
    bound_include: [0.10, 0.70, 0.75],
    bound_exclude: [0.90, 0.15, 0.20],
    bound_open: [0.95, 0.95, 0.95],
    text_dark: TextInks {
        warn: Color32::from_rgb(77, 217, 217),
        error: Color32::from_rgb(255, 128, 128),
        missing: Color32::from_rgb(77, 217, 217),
    },
    text_light: TextInks {
        warn: Color32::from_rgb(0, 107, 107),
        error: Color32::from_rgb(179, 0, 26),
        missing: Color32::from_rgb(0, 107, 107),
    },
};

const OFF_LIGHT_GROUND: Semantic = Semantic {
    feed: [0.08, 0.36, 0.12],
    rapid: [0.01, 0.31, 0.90],
    plunge: [0.90, 0.13, 0.13],
    bound_include: [0.10, 0.80, 1.00],
    bound_exclude: [1.00, 0.20, 0.70],
    bound_open: [1.00, 0.65, 0.05],
    ..OFF
};

const RED_GREEN_LIGHT_GROUND: Semantic = Semantic {
    feed: [0.55, 0.42, 0.00],
    rapid: [0.01, 0.31, 0.90],
    plunge: [0.22, 0.08, 0.00],
    bound_include: [0.00, 0.30, 0.85],
    bound_exclude: [0.55, 0.42, 0.00],
    bound_open: [0.12, 0.12, 0.12],
    ..RED_GREEN
};

const TRITAN_LIGHT_GROUND: Semantic = Semantic {
    feed: [0.08, 0.36, 0.12],
    rapid: [0.05, 0.05, 0.12],
    plunge: [0.90, 0.13, 0.13],
    bound_include: [0.00, 0.40, 0.45],
    bound_exclude: [0.80, 0.05, 0.10],
    bound_open: [0.12, 0.12, 0.12],
    ..TRITAN
};

#[test]
fn each_colour_vision_palette_keeps_its_values_in_the_token_table() {
    assert_eq!(TextInks::HOUSE, HOUSE, "the house text inks");
    for (name, got, want) in [
        ("OFF", Semantic::OFF, OFF),
        ("RED_GREEN", Semantic::RED_GREEN, RED_GREEN),
        ("TRITAN", Semantic::TRITAN, TRITAN),
        ("OFF_LIGHT_GROUND", Semantic::OFF_LIGHT_GROUND, OFF_LIGHT_GROUND),
        ("RED_GREEN_LIGHT_GROUND", Semantic::RED_GREEN_LIGHT_GROUND, RED_GREEN_LIGHT_GROUND),
        ("TRITAN_LIGHT_GROUND", Semantic::TRITAN_LIGHT_GROUND, TRITAN_LIGHT_GROUND),
    ] {
        assert_eq!(got, want, "{name}");
    }
    // Each mode on each ground still wears the same set.
    for (mode, dark, light) in [
        (ColorVision::Off, OFF, OFF_LIGHT_GROUND),
        (ColorVision::Protan, RED_GREEN, RED_GREEN_LIGHT_GROUND),
        (ColorVision::Deutan, RED_GREEN, RED_GREEN_LIGHT_GROUND),
        (ColorVision::Tritan, TRITAN, TRITAN_LIGHT_GROUND),
    ] {
        assert_eq!(*palette_for(mode, false), dark, "{mode:?} on a dark viewport");
        assert_eq!(*palette_for(mode, true), light, "{mode:?} on a light viewport");
    }
}

#[test]
fn no_theme_key_sets_a_verdict_colour() {
    for key in ["clean", "rest", "gouge", "unjudged", "feed", "rapid", "plunge", "select_hot", "bound_include", "text_dark", "warn", "error", "missing"] {
        let text = format!(r##"{{"format":"camcraft-theme/1","descriptor":{{"name":"V"}},"base":"dark","colors":{{"{key}":"#123456"}}}}"##);
        let err = tf::parse_text(&text, "verdict").err().unwrap_or_default();
        assert!(err.contains(&format!("unknown token \"{key}\"")), "{key}: the verdict colours stay ours: {err}");
    }
}
