//! #3894 (U3 of #3663): the preview area of the tool library window's
//! editor. Two views, a toggle between them:
//! - 3D (the default): the tool as an orbitable model - its imported
//!   body, or a model revolved from its profile, so it is never empty
//!   (`model`, painted by the shared `tool_model_view`);
//! - Side: the to-scale side view with dimension lines (`side_view`).
//!
//! The maintainer asked to keep the 3D preview and to build on it ("as
//! running weirdly shape tools requires models"). Display only: the view
//! and its orbit are view state, and nothing here queues a command.
//! Frame cost: the model and the projected screen mesh are cached on
//! their inputs; a still frame builds and projects nothing.
//! Provenance: our own pane; no HSMWorks material involved.

use std::cell::{Cell, RefCell};
use std::sync::Arc;

use egui::{Rect, Sense, Shape, Ui, Vec2};

use camcraft_app::geometry::Triangle;

use super::model::{self, ToolModel};
use super::side_view;
use crate::egui_app::tool_model_view::{self as model_view, BG, View};
use crate::state::{EditorFace, Orbit, PreviewMode, State};

/// The preview's share of the editor's height, and its bounds, logical px
/// (the width is the editor's). The tabs' rows keep the rest.
const SHARE: f32 = 0.4;
const MIN_H: f32 = 140.0;
const MAX_H: f32 = 320.0;

/// The caption under the toggle: where the 3D model comes from.
pub(super) fn caption(m: &ToolModel) -> String {
    match &m.body {
        Some(src) => format!("Model: the tool with its body {src}"),
        None => "Model: revolved from the tool profile".to_owned(),
    }
}

/// The toggle, the caption and the chosen view.
pub(super) fn draw(ui: &mut Ui, state: &State, face: &mut EditorFace) {
    let _s = crate::frame_cost::span("ui/tool_library.preview");
    let m = model::of(state);
    ui.horizontal(|ui| {
        ui.selectable_value(&mut face.preview, PreviewMode::Model, "3D")
            .on_hover_text("the tool as a model: drag to turn, scroll to zoom, double-click to fit");
        ui.selectable_value(&mut face.preview, PreviewMode::Side, "Side")
            .on_hover_text("the tool to scale from the side, with its dimensions");
        ui.separator();
        ui.small(caption(&m));
    });
    let size = Vec2::new(ui.available_width().max(160.0), (ui.available_height() * SHARE).clamp(MIN_H, MAX_H));
    match face.preview {
        PreviewMode::Model => model_3d(ui, &m, &mut face.orbit, size),
        PreviewMode::Side => side_view::draw(ui, &m, state.units(), size),
    }
}

fn model_3d(ui: &mut Ui, m: &ToolModel, orbit: &mut Orbit, size: Vec2) {
    let (resp, painter) = ui.allocate_painter(size, Sense::click_and_drag());
    let rect = resp.rect;
    painter.rect_filled(rect, 4.0, BG);
    let Some(view) = View::fit(m.bounds.0, m.bounds.1, rect, orbit) else {
        return;
    };
    painter.add(Shape::Mesh(projected(&view, &m.mesh, rect, orbit)));
    let g = model_view::gesture(ui, &resp, &view);
    model_view::apply(orbit, &g);
}

/// What the screen mesh was projected from: the model mesh (by identity;
/// the key holds it, so its address is never reused while it is held),
/// the rect and the orbit.
struct ProjKey {
    mesh: Arc<[Triangle]>,
    rect: Rect,
    orbit: Orbit,
}

impl ProjKey {
    fn same(&self, o: &Self) -> bool {
        Arc::ptr_eq(&self.mesh, &o.mesh) && self.rect == o.rect && self.orbit == o.orbit
    }
}

thread_local! {
    static PROJ: RefCell<Option<(ProjKey, Arc<egui::Mesh>)>> = const { RefCell::new(None) };
    static PROJECTS: Cell<u64> = const { Cell::new(0) };
}

/// The projections made on this thread - a still frame adds none.
#[cfg(test)]
pub(crate) fn projects() -> u64 {
    PROJECTS.with(Cell::get)
}

/// The screen mesh of `mesh`, projected again only when the mesh, the
/// rect or the orbit changed.
fn projected(view: &View, mesh: &Arc<[Triangle]>, rect: Rect, orbit: &Orbit) -> Arc<egui::Mesh> {
    let key = ProjKey { mesh: Arc::clone(mesh), rect, orbit: *orbit };
    PROJ.with(|p| {
        let mut p = p.borrow_mut();
        if let Some((k, v)) = p.as_ref()
            && k.same(&key)
        {
            return Arc::clone(v);
        }
        PROJECTS.with(|c| c.set(c.get().saturating_add(1)));
        let v = Arc::new(model_view::project(view, mesh));
        *p = Some((key, Arc::clone(&v)));
        v
    })
}
