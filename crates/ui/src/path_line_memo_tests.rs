//! #2201: the toolpath line memo's two pins (docs/decisions/2238-frame-
//! cost-rule.md rule 5): still calls build once, and each input of the
//! key makes a new build that equals a fresh `path_lines::cache_lines`.
//! Provenance: our own frame-cost law; no vendor material consulted.

use std::rc::Rc;

use camcraft_app::cl::{Move, MoveKind, Toolpath as ClPath};
use camcraft_app::geometry::Point3;
use camcraft_app::model;
use camcraft_app::schema::Toolpath;
use camcraft_app::verify::Warning;
use flecs_ecs::prelude::Entity;

use super::*;
use crate::egui_app::colorvision::Semantic;

fn path(len: f64, warnings: Vec<Warning>) -> Toolpath {
    let a = Point3::new(0.0, 0.0, 5.0);
    let b = Point3::new(0.0, 0.0, -1.0);
    let c = Point3::new(len, 0.0, -1.0);
    let moves = vec![Move::new(MoveKind::Plunge, a, b, Some(300.0)), Move::new(MoveKind::Feed, b, c, Some(900.0))];
    Toolpath::new(ClPath { moves, ..ClPath::default() }, warnings)
}

fn fresh(tp: &Toolpath, pal: &Semantic, dim: f32, arm: f64) -> Vec<[f32; 6]> {
    let mut v = Vec::new();
    assert!(crate::path_lines::cache_lines(&mut v, tp, pal, dim, arm));
    v
}

fn ops() -> (flecs_ecs::prelude::World, Entity, Entity) {
    let w = camcraft_app::world::new_world();
    let a = model::spawn(&w, "op a");
    let b = model::spawn(&w, "op b");
    (w, a, b)
}

#[test]
fn still_calls_build_the_lines_once() {
    let (_w, a, _) = ops();
    let (tp, pal) = (path(100.0, Vec::new()), &Semantic::OFF);
    let before = path_line_builds();
    let first = lines(a, &tp, pal, 1.0, 0.6);
    for _ in 0..5 {
        let again = lines(a, &tp, pal, 1.0, 0.6);
        assert!(Rc::ptr_eq(&again.lines, &first.lines), "a still call shares the lines");
    }
    assert_eq!(path_line_builds() - before, 1, "six calls, one build");
    assert_eq!(&*first.lines, &fresh(&tp, pal, 1.0, 0.6)[..], "a hit is the fresh build");
    assert!(first.complete);
}

#[test]
fn each_input_of_the_key_builds_again() {
    let (_w, a, _) = ops();
    let pal = &Semantic::OFF;
    let base = path(100.0, Vec::new());
    let first = lines(a, &base, pal, 1.0, 0.6);
    let mut builds = path_line_builds();
    let mut rebuilt = |what: &str| {
        let now = path_line_builds();
        assert_eq!(now - builds, 1, "{what}: one new build");
        builds = now;
    };

    // The path: another move list.
    let longer = path(140.0, Vec::new());
    let l = lines(a, &longer, pal, 1.0, 0.6);
    rebuilt("path");
    assert_eq!(&*l.lines, &fresh(&longer, pal, 1.0, 0.6)[..]);
    assert_ne!(&*l.lines, &*first.lines);

    // The warnings: the same moves, a gouge on the feed.
    let warned = path(100.0, vec![Warning::Gouge { at: 1 }]);
    let l = lines(a, &warned, pal, 1.0, 0.6);
    rebuilt("warnings");
    assert_eq!(&*l.lines, &fresh(&warned, pal, 1.0, 0.6)[..]);
    assert_ne!(&*l.lines, &*first.lines, "the warned feed wears the gouge colour");

    // The palette.
    let red_green = &Semantic::RED_GREEN;
    let l = lines(a, &base, red_green, 1.0, 0.6);
    rebuilt("palette");
    assert_eq!(&*l.lines, &fresh(&base, red_green, 1.0, 0.6)[..]);

    // The dim factor (an unselected path under "Show all toolpaths").
    let l = lines(a, &base, pal, 0.45, 0.6);
    rebuilt("dim");
    assert_eq!(&*l.lines, &fresh(&base, pal, 0.45, 0.6)[..]);
    assert_ne!(&*l.lines, &*first.lines);

    // The chevron arm.
    let l = lines(a, &base, pal, 1.0, 2.0);
    rebuilt("arm");
    assert_eq!(&*l.lines, &fresh(&base, pal, 1.0, 2.0)[..]);
    assert_ne!(&*l.lines, &*first.lines);

    // The viewport ground that a dimmed colour fades toward.
    let ground = crate::egui_app::theme::current().viewport;
    let _ = lines_on(a, &base, pal, 0.45, 0.6, ground);
    rebuilt("dim again");
    let _ = lines_on(a, &base, pal, 0.45, 0.6, egui::Color32::from_rgb(250, 250, 250));
    rebuilt("ground");

    // Back to the first inputs: the answer of the first build.
    let back = lines(a, &base, pal, 1.0, 0.6);
    rebuilt("back");
    assert_eq!(&*back.lines, &*first.lines);
}

#[test]
fn each_operation_keeps_its_own_lines_and_undrawn_ones_are_dropped() {
    let (_w, a, b) = ops();
    let pal = &Semantic::OFF;
    let (pa, pb) = (path(100.0, Vec::new()), path(60.0, Vec::new()));
    let before = path_line_builds();
    for _ in 0..3 {
        let _ = lines(a, &pa, pal, 1.0, 0.6);
        let _ = lines(b, &pb, pal, 0.45, 0.6);
    }
    assert_eq!(path_line_builds() - before, 2, "two operations, two builds");
    keep_only(&[a]);
    assert_eq!(path_line_entries(), 1, "the undrawn operation holds no lines");
    let _ = lines(a, &pa, pal, 1.0, 0.6);
    assert_eq!(path_line_builds() - before, 2, "the drawn operation keeps its lines");
    keep_only(&[]);
    assert_eq!(path_line_entries(), 0);
}

#[test]
fn append_stops_at_lines_cut_short() {
    let mut out = Vec::new();
    let full = Lines { lines: Rc::from(vec![[0.0; 6]; 2]), complete: true };
    assert!(append(&mut out, &full));
    let cut = Lines { lines: Rc::from(vec![[1.0; 6]; 2]), complete: false };
    assert!(!append(&mut out, &cut), "a cut path stops the overlay");
    assert_eq!(out.len(), 4);
}
