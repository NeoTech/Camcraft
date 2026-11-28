//! #1120 review of #4073: the kept holder outline follows a theme change
//! of its inks, and the same inputs build nothing.
//! Provenance: our own tests over the built-in holder library.

use super::*;
use crate::egui_app::theme::{self, Tokens};

/// The stroke and fill colours of the kept shapes.
fn inks_of(d: &Drawing) -> Vec<Color32> {
    d.shapes
        .iter()
        .flat_map(|s| match s {
            Shape::Path(p) => match p.stroke.color {
                egui::epaint::ColorMode::Solid(c) => vec![p.fill, c],
                egui::epaint::ColorMode::UV(_) => vec![p.fill],
            },
            Shape::LineSegment { stroke, .. } => vec![stroke.color],
            _ => Vec::new(),
        })
        .collect()
}

#[test]
fn a_theme_change_of_the_inks_builds_the_holder_outline_again() {
    let rows = holders::rows();
    let Some(row) = rows.iter().find(|r| r.entry.points(0.0, r.entry.gauge_len).is_ok()) else {
        panic!("the library has a holder with an outline");
    };
    let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0));
    let draw = || drawing(&row.name, &row.entry, rect, Unit::Millimeters, true);
    theme::set_current(&Tokens::DARK);
    let first = draw();
    let old = inks_of(&first);
    assert!(old.contains(&Tokens::DARK.tool_body) && old.contains(&Tokens::DARK.tool_dim), "the theme's inks: {old:?}");
    let b = builds();
    let _ = draw();
    assert_eq!(builds(), b, "the same inputs build nothing");

    let mut t = Tokens::DARK;
    t.tool_axis = Color32::from_rgb(1, 2, 3);
    t.tool_body = Color32::from_rgb(4, 5, 6);
    t.tool_edge = Color32::from_rgb(7, 8, 9);
    t.tool_dim = Color32::from_rgb(10, 11, 12);
    theme::set_current(&t);
    let next = inks_of(&draw());
    assert_eq!(builds(), b + 1, "new inks build the outline once");
    for (what, c) in [("axis", t.tool_axis), ("body", t.tool_body), ("edge", t.tool_edge), ("dimension", t.tool_dim)] {
        assert!(next.contains(&c), "the {what} line has the new ink: {next:?}");
    }
    assert!(!next.contains(&Tokens::DARK.tool_body), "no old body ink stays: {next:?}");
    let _ = draw();
    assert_eq!(builds(), b + 1, "and then holds");
    theme::set_current(&Tokens::DARK);
}
