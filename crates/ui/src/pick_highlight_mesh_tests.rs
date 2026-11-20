//! Pins for the screen-mesh cache of `pick_highlight_mesh` (#2201): each
//! of the three slots (selected, viewport hover, row hover) has a build
//! counter. A still frame tessellates nothing again; an orbit, a pane
//! resize and an ink change each tessellate again, and the painted
//! vertices move or change colour. crates/ui/AGENTS.md, "Frame cost".
//! Provenance: our own unit tests; no vendor material consulted.

use camcraft_app::geometry::{Point3, cube};
use camcraft_app::model;
use camcraft_app::operation::{FeatureSel, GeoSel};
use egui::{Color32, Pos2, Rect, Shape, pos2};

use super::{paint_hover, paint_row_hover, paint_selected};
use crate::pick_highlight::lock;
use crate::points_pick::PointsPick;
use crate::state::{HoverHit, State};

#[derive(Clone, Copy, Debug, PartialEq)]
enum Slot {
    Selected,
    Hover,
    RowHover,
}

const SLOTS: [Slot; 3] = [Slot::Selected, Slot::Hover, Slot::RowHover];
const INK: Color32 = Color32::from_rgb(40, 160, 255);
const OTHER_INK: Color32 = Color32::from_rgb(255, 120, 20);
const HALO: Color32 = Color32::from_rgb(10, 10, 10);
const PANE: Rect = Rect { min: Pos2 { x: 0.0, y: 0.0 }, max: Pos2 { x: 800.0, y: 600.0 } };
const SMALL_PANE: Rect = Rect { min: Pos2 { x: 100.0, y: 50.0 }, max: Pos2 { x: 500.0, y: 350.0 } };

/// A state with one cube model "Part" and its first face in `slot`.
fn picked_cube(slot: Slot) -> State {
    let mut st = State::new();
    let part = model::spawn(&st.world, "Part");
    model::set_facets(&st.world, &part, cube(Point3::new(0.0, 0.0, 0.0), Point3::new(10.0, 10.0, 10.0)).facets);
    let features = model::features_of(&st.world, &part).expect("cube features");
    let fid = features.faces.first().expect("a face").id;
    let sel = GeoSel { entity: "Part".into(), feature: FeatureSel::Face(fid) };
    match slot {
        Slot::Selected => st.points_pick = Some(PointsPick { setup: "Setup1".into(), picks: vec![sel] }),
        Slot::Hover => st.hover = Some(HoverHit { entity: "Part".into(), facet: 0, feature: Some(sel.feature) }),
        Slot::RowHover => st.geo_hover_drawn = Some(sel),
    }
    st
}

fn builds(st: &State, slot: Slot) -> u64 {
    let inner = lock(st);
    match slot {
        Slot::Selected => inner.selected_mesh.builds,
        Slot::Hover => inner.hover_mesh.builds,
        Slot::RowHover => inner.row_hover_mesh.builds,
    }
}

/// Run one egui frame that paints `slot` into `pane` with `ink`, and
/// return the painted mesh vertices (position and colour).
fn frame(ctx: &egui::Context, st: &State, slot: Slot, pane: Rect, ink: Color32) -> Vec<(Pos2, Color32)> {
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_max(pos2(0.0, 0.0), pos2(1000.0, 800.0))),
        ..Default::default()
    };
    let mut out = ctx.run_ui(input, |ui| {
        let painter = ui.ctx().layer_painter(egui::LayerId::background());
        let aspect = f64::from(pane.width() / pane.height());
        let to_px = move |uv: [f32; 2]| {
            pos2(
                pane.min.x + (uv[0] + 1.0) * 0.5 * pane.width(),
                pane.min.y + (1.0 - uv[1]) * 0.5 * pane.height(),
            )
        };
        match slot {
            Slot::Selected => paint_selected(&painter, st, aspect, &to_px, 2.0, 4.0, ink, HALO),
            Slot::Hover => paint_hover(&painter, st, aspect, &to_px, 2.0, 4.0, ink, HALO),
            Slot::RowHover => paint_row_hover(&painter, st, aspect, &to_px, 2.0, 4.0, ink, HALO),
        }
    });
    out.textures_delta.clear();
    fn walk(shape: &Shape, verts: &mut Vec<(Pos2, Color32)>) {
        match shape {
            Shape::Vec(v) => v.iter().for_each(|s| walk(s, verts)),
            Shape::Mesh(m) => verts.extend(m.vertices.iter().map(|v| (v.pos, v.color))),
            _ => {}
        }
    }
    let mut verts = Vec::new();
    for clipped in &out.shapes {
        walk(&clipped.shape, &mut verts);
    }
    verts
}

#[test]
fn highlight_mesh_still_frames_tessellate_once_in_each_slot() {
    for slot in SLOTS {
        let st = picked_cube(slot);
        let ctx = egui::Context::default();
        let first = frame(&ctx, &st, slot, PANE, INK);
        assert!(!first.is_empty(), "{slot:?}: the pick paints a mesh");
        for i in 0..8 {
            let again = frame(&ctx, &st, slot, PANE, INK);
            assert_eq!(again, first, "{slot:?}: frame {i} paints the same mesh");
        }
        assert_eq!(builds(&st, slot), 1, "{slot:?}: a still frame tessellates nothing again");
        for other in SLOTS.into_iter().filter(|o| *o != slot) {
            assert_eq!(builds(&st, other), 0, "{slot:?}: the {other:?} slot does not build");
        }
    }
}

#[test]
fn highlight_mesh_rebuilds_after_an_orbit_and_the_vertices_move() {
    for slot in SLOTS {
        let mut st = picked_cube(slot);
        let ctx = egui::Context::default();
        let before = frame(&ctx, &st, slot, PANE, INK);
        st.cam.orbit(12.0, -4.0);
        let after = frame(&ctx, &st, slot, PANE, INK);
        assert_eq!(builds(&st, slot), 2, "{slot:?}: an orbit tessellates again");
        assert!(!after.is_empty(), "{slot:?}: the orbited pick still paints");
        assert_ne!(after, before, "{slot:?}: the painted vertices follow the camera");
        frame(&ctx, &st, slot, PANE, INK);
        assert_eq!(builds(&st, slot), 2, "{slot:?}: the next still frame reads the cache");
    }
}

#[test]
fn highlight_mesh_rebuilds_after_a_pane_resize_and_the_vertices_move() {
    for slot in SLOTS {
        let st = picked_cube(slot);
        let ctx = egui::Context::default();
        let before = frame(&ctx, &st, slot, PANE, INK);
        let after = frame(&ctx, &st, slot, SMALL_PANE, INK);
        assert_eq!(builds(&st, slot), 2, "{slot:?}: a pane resize tessellates again");
        assert!(!after.is_empty(), "{slot:?}: the resized pane still paints");
        let max_x = |v: &[(Pos2, Color32)]| v.iter().map(|(p, _)| p.x).fold(f32::NEG_INFINITY, f32::max);
        assert!(max_x(&after) < max_x(&before), "{slot:?}: the painted vertices shrink into the smaller pane");
    }
}

#[test]
fn highlight_mesh_rebuilds_after_an_ink_change_and_the_colour_changes() {
    for slot in SLOTS {
        let st = picked_cube(slot);
        let ctx = egui::Context::default();
        let before = frame(&ctx, &st, slot, PANE, INK);
        assert!(before.iter().any(|(_, c)| *c == INK), "{slot:?}: the first frame paints the first ink");
        let after = frame(&ctx, &st, slot, PANE, OTHER_INK);
        assert_eq!(builds(&st, slot), 2, "{slot:?}: a theme (ink) change tessellates again");
        assert!(after.iter().any(|(_, c)| *c == OTHER_INK), "{slot:?}: the new ink is painted");
        assert!(after.iter().all(|(_, c)| *c != INK), "{slot:?}: the old ink is never served");
    }
}
