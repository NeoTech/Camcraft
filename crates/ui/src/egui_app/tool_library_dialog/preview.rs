//! #3894 (U3 of #3663): the preview area of the tool library window's
//! editor. Two views, a toggle between them:
//! - 3D: the tool as an orbitable model - its imported body, or a model
//!   revolved from its profile, so it is never empty (`model`, painted by
//!   the shared `tool_model_view`);
//! - Side (#4336: the default): the to-scale side view with dimension
//!   lines (`side_view`).
//!
//! The maintainer asked to keep the 3D preview and to build on it ("as
//! running weirdly shape tools requires models"). Display only: the view
//! and its orbit are view state, and nothing here queues a command.
//! Frame cost: the model and the projected screen mesh are cached on
//! their inputs; a still frame builds and projects nothing.
//! #3897 (U6): the preview stands right of the tabs, as the column of
//! the #3663 mock-up, and two toggles show or hide the dimension lines
//! and the holder. The caller gives the size.
//! #3908: a third toggle, Flutes, shows or hides the helical flutes of
//! the 3D model (display only; `model::ToolModel::shown`). The row may
//! wrap in a narrow preview: the drawing takes the height that is left.
//! #4336 (U10): Side comes first and is the default; the Flutes toggle
//! draws the flute lines of the side view too; the model's caption shows
//! only under the 3D view.
//! #4452: the 3D view paints the cutting part, the shank and the holder
//! each in its own theme ink (`tool_model_flute`, `tool_model_shank`,
//! `tool_model_holder`), shaded by the light; a theme change projects
//! the mesh again, a still frame does not.
//! Provenance: our own pane; no HSMWorks material involved.

use std::cell::{Cell, RefCell};
use std::sync::Arc;

use egui::{Rect, Sense, Shape, Ui, Vec2};

use camcraft_app::geometry::Triangle;

use super::model::{self, ToolModel};
use super::side_view;
use crate::egui_app::tool_model_view::{self as model_view, Part, PartInks, tool_ground, View};
use crate::state::{EditorFace, Orbit, PreviewMode, State};

/// The labels of the two display toggles (#3897).
pub(crate) const DIMS: &str = "Dimensions";
pub(crate) const HOLDER: &str = "Holder";
/// #3908: the label of the flutes toggle.
pub(crate) const FLUTES: &str = "Flutes";
/// The least height the toggles and the caption take above the drawing.
pub(super) const HEAD_H: f32 = 60.0;

/// The caption under the toggle: where the 3D model comes from.
pub(super) fn caption(m: &ToolModel) -> String {
    match &m.body {
        Some(src) => format!("Model: the tool with its body {src}"),
        None => "Model: revolved from the tool profile".to_owned(),
    }
}

/// The view toggle, the display toggles, the caption and the chosen view
/// in a `size` area (the drawing takes the size less [`HEAD_H`]).
pub(super) fn draw(ui: &mut Ui, state: &State, face: &mut EditorFace, size: Vec2) {
    let _s = crate::frame_cost::span("ui/tool_library.preview");
    let m = model::of(state);
    let top = ui.cursor().top();
    ui.horizontal_wrapped(|ui| {
        ui.selectable_value(&mut face.preview, PreviewMode::Side, "Side")
            .on_hover_text("the tool to scale from the side, with its dimensions");
        ui.selectable_value(&mut face.preview, PreviewMode::Model, "3D")
            .on_hover_text("the tool as a model: drag to turn, scroll to zoom, double-click to fit");
        ui.separator();
        ui.checkbox(&mut face.dims, DIMS).on_hover_text("show the dimension lines of the side view");
        ui.checkbox(&mut face.holder, HOLDER).on_hover_text("show the holder; off, the cutter and the shank only");
        ui.checkbox(&mut face.flutes, FLUTES)
            .on_hover_text("show the flutes on the side view and the 3D model; for display only, the checks use the plain tool");
    });
    if face.preview == PreviewMode::Model {
        ui.small(caption(&m));
    }
    let head = (ui.cursor().top() - top).max(HEAD_H);
    let size = Vec2::new(size.x.max(160.0), (size.y - head).max(100.0));
    match face.preview {
        PreviewMode::Model => {
            let (mesh, parts, bounds) = m.shown(face.holder, face.flutes);
            model_3d(ui, mesh, parts, bounds, &mut face.orbit, size);
        }
        PreviewMode::Side => {
            let show = side_view::Show { dims: face.dims, holder: face.holder, flutes: face.flutes };
            side_view::draw(ui, &m, state.units(), size, show);
        }
    }
}

fn model_3d(
    ui: &mut Ui,
    mesh: &Arc<[Triangle]>,
    parts: &Arc<[Part]>,
    bounds: ([f64; 3], [f64; 3]),
    orbit: &mut Orbit,
    size: Vec2,
) {
    let (resp, painter) = ui.allocate_painter(size, Sense::click_and_drag());
    let rect = resp.rect;
    painter.rect_filled(rect, 4.0, tool_ground());
    let Some(view) = View::fit(bounds.0, bounds.1, rect, orbit) else {
        return;
    };
    painter.add(Shape::Mesh(projected(&view, mesh, parts, rect, orbit)));
    let g = model_view::gesture(ui, &resp, &view);
    model_view::apply(orbit, &g);
}

/// What the screen mesh was projected from: the model mesh and its parts
/// (by identity; the key holds them, so their addresses are never reused
/// while they are held), the rect, the orbit and the part inks (#4452).
struct ProjKey {
    mesh: Arc<[Triangle]>,
    parts: Arc<[Part]>,
    rect: Rect,
    orbit: Orbit,
    inks: PartInks,
}

impl ProjKey {
    fn same(&self, o: &Self) -> bool {
        Arc::ptr_eq(&self.mesh, &o.mesh)
            && Arc::ptr_eq(&self.parts, &o.parts)
            && self.rect == o.rect
            && self.orbit == o.orbit
            && self.inks == o.inks
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

/// The screen mesh of `mesh`, projected again only when the mesh, its
/// parts, the rect, the orbit or the theme's part inks changed.
fn projected(view: &View, mesh: &Arc<[Triangle]>, parts: &Arc<[Part]>, rect: Rect, orbit: &Orbit) -> Arc<egui::Mesh> {
    let inks = PartInks::current();
    let key = ProjKey { mesh: Arc::clone(mesh), parts: Arc::clone(parts), rect, orbit: *orbit, inks };
    PROJ.with(|p| {
        let mut p = p.borrow_mut();
        if let Some((k, v)) = p.as_ref()
            && k.same(&key)
        {
            return Arc::clone(v);
        }
        PROJECTS.with(|c| c.set(c.get().saturating_add(1)));
        let v = Arc::new(model_view::project_parts(view, mesh, parts, inks));
        *p = Some((key, Arc::clone(&v)));
        v
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::egui_app::tool_model_view::Part;

    /// #4452: a theme change of a part ink projects the mesh again; the
    /// same inputs do not.
    #[test]
    fn a_part_ink_change_is_a_new_projection_key() {
        let mesh: Arc<[Triangle]> = Arc::from(Vec::new());
        let parts: Arc<[Part]> = Arc::from(vec![Part::Flute]);
        let rect = Rect::from_min_size(egui::Pos2::ZERO, Vec2::splat(100.0));
        let inks = PartInks::current();
        let key = |inks| ProjKey { mesh: Arc::clone(&mesh), parts: Arc::clone(&parts), rect, orbit: Orbit::default(), inks };
        assert!(key(inks).same(&key(inks)), "the same inputs keep the projection");
        let new = PartInks { holder: inks.flute, ..inks };
        assert_ne!(new, inks, "the holder and the flute inks differ");
        assert!(!key(inks).same(&key(new)), "a new holder ink projects again");
    }
}
