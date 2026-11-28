//! #1120 review of #4073: the kept row drawing follows a theme change of
//! its inks, and a still frame with the same inks builds nothing.
//! Provenance: our own tests over a shipped catalog tool.

use super::*;
use crate::egui_app::theme::{self, Tokens};
use camcraft_app::tools::catalog;

/// A shipped tool with a cutting band and a shank.
fn shipped() -> Tool {
    match catalog().into_iter().map(|(_, t)| t).find(|t| t.flute_len > 0.0 && t.shank_len > 0.0) {
        Some(t) => t,
        None => panic!("the catalog has a tool with a cutting band and a shank"),
    }
}

/// The fill colours of the kept shapes.
fn fills(shapes: &[Shape]) -> Vec<Color32> {
    shapes
        .iter()
        .filter_map(|s| match s {
            Shape::Path(p) => Some(p.fill),
            _ => None,
        })
        .collect()
}

#[test]
fn a_theme_change_of_the_inks_builds_the_row_drawing_again() {
    let tool = shipped();
    theme::set_current(&Tokens::DARK);
    let first = shapes_of("thumb ink probe", &tool);
    assert!(fills(&first).contains(&Tokens::DARK.thumb_flute), "the band has the theme's flute ink");
    let b = builds();
    let _ = shapes_of("thumb ink probe", &tool);
    assert_eq!(builds(), b, "the same tool and inks build nothing");

    let mut t = Tokens::DARK;
    t.thumb_flute = Color32::from_rgb(1, 2, 3);
    t.tool_shank = Color32::from_rgb(4, 5, 6);
    theme::set_current(&t);
    let next = shapes_of("thumb ink probe", &tool);
    assert_eq!(builds(), b + 1, "new inks build the drawing once");
    let f = fills(&next);
    assert!(f.contains(&t.thumb_flute) && f.contains(&t.tool_shank), "the drawing has the new inks: {f:?}");
    assert!(!f.contains(&Tokens::DARK.thumb_flute), "no old flute ink stays: {f:?}");
    let _ = shapes_of("thumb ink probe", &tool);
    assert_eq!(builds(), b + 1, "and then holds");
    theme::set_current(&Tokens::DARK);
}
