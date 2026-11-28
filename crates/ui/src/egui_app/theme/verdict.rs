//! The verdict palette table (#1120): the colours that carry meaning -
//! the Deviate verdict classes, the move kinds, the selection lift, the
//! boundary roles and the warning / error / missing text inks - for each
//! colour-vision mode, on a dark and on a light viewport. They are part
//! of the token table, but they stay ours: no theme-file key sets them,
//! because each set is held to the CIEDE2000 gate of `colorvision`.
//! Values moved unchanged out of `colorvision.rs` and `ground_ink.rs`;
//! `theme_files::verdict_inks` pins each one.
//! Provenance: our own palette numbers (M13.4, #698); no vendor material
//! consulted.

use egui::Color32;

use crate::egui_app::colorvision::{Semantic, TextInks};

impl TextInks {
    /// The house inks, exactly what the pages drew before: egui's own
    /// dark-visuals warning (the theme builds on `Visuals::dark()` and
    /// never respells it), the page red and the chip grey.
    pub const HOUSE: TextInks = TextInks {
        warn: Color32::from_rgb(255, 143, 0),
        error: Color32::from_rgb(0xFF, 0x59, 0x59),
        missing: Color32::from_rgb(0xB8, 0xB8, 0xC8),
    };
}

impl Semantic {
    /// The house palette, exactly what the shell drew before M13.4
    /// (the overlay's own consts and the shaders' literals).
    pub const OFF: Semantic = Semantic {
        clean: [0.42, 0.50, 0.62],
        rest: [0.98, 0.72, 0.20],
        gouge: [0.90, 0.18, 0.14],
        unjudged: [0.45, 0.46, 0.48],
        feed: crate::interact::FEED_RGB,
        rapid: crate::interact::RAPID_RGB,
        plunge: crate::interact::PLUNGE_RGB,
        select_hot: [0.98, 0.72, 0.20],
        select_warm: [0.55, 0.35, 0.05],
        bound_include: [0.20, 0.75, 0.95],
        bound_exclude: [0.92, 0.25, 0.60],
        bound_open: [0.98, 0.62, 0.12],
        text_dark: TextInks::HOUSE,
        text_light: TextInks::HOUSE,
    };

    /// Protan and deutan share the red-green confusion line, so one
    /// remap serves both (each is still gated under its own eye):
    /// rest goes lemon, gouge a dark oxblood, feed yellow against the
    /// unchanged blue rapid, plunge a dark burnt orange - lightness
    /// does the work the lost red-green axis did - and the unjudged
    /// grey lifts clear of the steel.
    pub const RED_GREEN: Semantic = Semantic {
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
        // Warning on the yellow end, refusal on the violet end of the
        // surviving blue-yellow axis; the missing chip reads as a warning.
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

    /// Tritan loses blue-yellow: rest moves to pink (red-cyan survives),
    /// gouge to a deeper red, rapid to a dark slate so lightness parts
    /// it from the green feed.
    pub const TRITAN: Semantic = Semantic {
        clean: [0.42, 0.50, 0.62],
        rest: [1.00, 0.45, 0.70],
        gouge: [0.80, 0.05, 0.05],
        unjudged: [0.80, 0.80, 0.80],
        feed: crate::interact::FEED_RGB,
        rapid: [0.20, 0.40, 0.60],
        plunge: crate::interact::PLUNGE_RGB,
        select_hot: [1.00, 0.45, 0.70],
        select_warm: [0.56, 0.25, 0.40],
        bound_include: [0.10, 0.70, 0.75],
        bound_exclude: [0.90, 0.15, 0.20],
        bound_open: [0.95, 0.95, 0.95],
        // Red-cyan survives: warning goes cyan, refusal stays red.
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

    /// The house palette on a light viewport: the move kinds and the
    /// boundary roles as tuned on screen (#698). Verdict and selection
    /// inks are unchanged: they paint the stock and the selected part,
    /// not lines on the ground.
    pub const OFF_LIGHT_GROUND: Semantic = Semantic {
        feed: [0.08, 0.36, 0.12],
        rapid: [0.01, 0.31, 0.90],
        plunge: [0.90, 0.13, 0.13],
        bound_include: [0.10, 0.80, 1.00],
        bound_exclude: [1.00, 0.20, 0.70],
        bound_open: [1.00, 0.65, 0.05],
        ..Semantic::OFF
    };

    /// Protan/deutan on a light viewport: the same blue-yellow axis as
    /// `RED_GREEN`, darkened to clear 3:1 - gold feed and exclude against
    /// the blue rapid and include; plunge and open go near-black, the
    /// lightness end the dark ground's white open and pale inks held.
    pub const RED_GREEN_LIGHT_GROUND: Semantic = Semantic {
        feed: [0.55, 0.42, 0.00],
        rapid: [0.01, 0.31, 0.90],
        plunge: [0.22, 0.08, 0.00],
        bound_include: [0.00, 0.30, 0.85],
        bound_exclude: [0.55, 0.42, 0.00],
        bound_open: [0.12, 0.12, 0.12],
        ..Semantic::RED_GREEN
    };

    /// Tritan on a light viewport: red-cyan survives - the deep green
    /// feed, red plunge and exclude, a teal include; rapid and open go
    /// near-black so lightness parts them from the green.
    pub const TRITAN_LIGHT_GROUND: Semantic = Semantic {
        feed: [0.08, 0.36, 0.12],
        rapid: [0.05, 0.05, 0.12],
        plunge: [0.90, 0.13, 0.13],
        bound_include: [0.00, 0.40, 0.45],
        bound_exclude: [0.80, 0.05, 0.10],
        bound_open: [0.12, 0.12, 0.12],
        ..Semantic::TRITAN
    };
}
