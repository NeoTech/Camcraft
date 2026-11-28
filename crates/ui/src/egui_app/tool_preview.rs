//! The selected tool's to-scale preview (#2002).
//! #3894: `paint` draws into a given rect, so the tool library window's
//! larger side view (`tool_library_dialog::side_view`) adds its dimension
//! lines over the same drawing.
//! #4452: a thread mill's cutting length shows its teeth
//! (`tool_library_dialog::thread_form`), as the tool library does.
//! Provenance: our own preview extracted from pages.rs.

use camcraft_app::{
    tools,
    units::{self, Unit},
};
use egui::{Align2, Color32, FontId, Pos2, Sense, Shape, Stroke, Ui, Vec2};
fn note_ink() -> Color32 {
    crate::egui_app::theme::current().tool_note
}

/// The live preview: the tool drawn to scale from `tools::Tool::profile`,
/// the same band stack the collision checker sweeps, so what you see is
/// what `verify` would collide.
///
/// Pure state→paint: nothing here queues a command; it renders the draft
/// the fields beside it are editing. Tip at the bottom (z = 0 upward),
/// equal aspect both axes - "to scale" means to scale. A half-typed draft
/// renders a stub and a word, never a NaN.
///
/// #2007: `steel` replaces the cutter, neck and shank outline - the drawn
/// cutter of the Profile tab, while it validates. It does not hold the
/// holder, so it comes with the holder as `body` bands.
pub(super) fn draw(ui: &mut Ui, t: &tools::Tool, u: Unit, body: Option<&[(f64, f64, f64)]>, steel: Option<&[(f64, f64)]>, wide: bool) {
    const SIDE: f32 = 260.0;
    let (resp, painter) = ui.allocate_painter(
        Vec2::splat(if wide { SIDE } else { ui.available_width().clamp(1.0, SIDE) }),
        Sense::hover(),
    );
    let rect = resp.rect;
    painter.rect_filled(rect, 4.0, super::tool_model_view::tool_ground());
    if let Some(s) = paint(&painter, rect, t, body, steel) {
        painter.text(
            Pos2::new(s.axis_x, rect.max.y - 4.0),
            Align2::CENTER_BOTTOM,
            format!("Ø {}", units::len(t.diameter, u, 2)),
            FontId::proportional(10.0),
            note_ink(),
        );
        painter.text(
            Pos2::new(rect.min.x + 6.0, rect.min.y + 6.0),
            Align2::LEFT_TOP,
            format!("{} {} overall", units::len(s.z_max, u, 1), u.suffix()),
            FontId::proportional(10.0),
            note_ink(),
        );
    }
}

/// #3894: where `paint` put the tool - the axis, the tip line and the
/// scale (px per model unit) - so a caller can draw dimension lines over
/// it. `z_max` is the drawing's overall height.
#[derive(Clone, Copy, Debug)]
pub(super) struct Scale {
    pub axis_x: f32,
    pub base_y: f32,
    pub scale: f32,
    pub z_max: f64,
}

impl Scale {
    /// The screen point of model height `z` at radius `r` (negative: left).
    pub(super) fn at(&self, z: f64, r: f64) -> Pos2 {
        Pos2::new(self.axis_x + (r as f32 * self.scale), self.base_y - (z as f32 * self.scale))
    }
}

/// The tool to scale inside `rect` (the background is the caller's). A
/// half-typed draft paints a stub and a word and gives `None`, never a NaN.
pub(super) fn paint(
    painter: &egui::Painter,
    rect: egui::Rect,
    t: &tools::Tool,
    body: Option<&[(f64, f64, f64)]>,
    steel: Option<&[(f64, f64)]>,
) -> Option<Scale> {
    const PAD: f32 = 18.0;
    let axis_x = rect.center().x;
    // A body replaces the parameter holder, as in the collision band stack.
    let outline = if let Some(s) = steel {
        s.to_vec()
    } else {
        let plain = if body.is_some() { tools::Tool { holder_len: 0.0, ..t.clone() }.profile() } else { t.profile() };
        // #4452: a thread mill shows its teeth here too, as in the tool
        // library's views (display only, inside the crest cylinder).
        super::tool_library_dialog::thread_form::toothed(t, &plain).unwrap_or(plain)
    };
    // #371: a body's bands (above, len, r) widen the frame to hold them.
    let bands = body.unwrap_or_default();
    let r_max = outline
        .iter()
        .map(|q| q.1)
        .chain(bands.iter().map(|b| b.2))
        .fold(0.0f64, f64::max);
    let z_max = outline
        .iter()
        .map(|q| q.0)
        .chain(bands.iter().map(|b| b.0 + b.1))
        .fold(0.0f64, f64::max);
    if r_max <= 0.0 || z_max <= 0.0 || !r_max.is_finite() || !z_max.is_finite() {
        painter.line_segment(
            [
                Pos2::new(axis_x, rect.min.y + PAD),
                Pos2::new(axis_x, rect.max.y - PAD),
            ],
            Stroke::new(1.0, note_ink()),
        );
        painter.text(
            Pos2::new(axis_x, rect.center().y),
            Align2::CENTER_CENTER,
            "enter dimensions",
            FontId::proportional(11.0),
            note_ink(),
        );
        return None;
    }
    let base_y = rect.max.y - PAD;
    let scale = (((rect.width() - 2.0 * PAD) / (r_max as f32 * 2.0))
        .min((rect.height() - 2.0 * PAD) / z_max as f32))
    .max(f32::EPSILON);
    let at = |z: f64, r: f64| -> Pos2 {
        Pos2::new(axis_x + (r as f32 * scale), base_y - (z as f32 * scale))
    };
    // The axis first, so everything steel-colored lies over it.
    let pal = crate::egui_app::theme::current();
    painter.line_segment(
        [
            Pos2::new(axis_x, rect.min.y + 6.0),
            Pos2::new(axis_x, rect.max.y - 6.0),
        ],
        Stroke::new(1.0, pal.tool_axis),
    );
    let zc = t.flute_len.max(0.0);
    let zs = zc + t.shank_len.max(0.0);
    let steel = [
        pal.tool_flute, // cutting band
        pal.tool_shank, // shank
        pal.tool_body,  // holder
    ];
    // Band fills, one quad per outline pair: each z-monotone symmetric
    // slice is convex, so it fills honestly - the whole stepped silhouette
    // is not (concave at every band step) and egui fills convex only.
    // Radius steps (z0 == z1) carry no area; the slice above paints them.
    for w in outline.windows(2) {
        let (z0, r0) = w[0];
        let (z1, r1) = w[1];
        if !(z0.is_finite() && z1.is_finite() && r0.is_finite() && r1.is_finite()) || z1 <= z0 {
            continue;
        }
        let zm = (z0 + z1) * 0.5;
        let band = (usize::from(zm >= zc) + usize::from(zm >= zs)).min(2);
        painter.add(Shape::convex_polygon(
            vec![at(z0, -r0), at(z0, r0), at(z1, r1), at(z1, -r1)],
            steel[band],
            Stroke::NONE,
        ));
    }
    // #371: the modelled body's revolved envelope replaces the holder.
    for &(above, len, r) in bands {
        if !(above.is_finite() && len.is_finite() && r.is_finite()) || len <= 0.0 || r <= 0.0 {
            continue;
        }
        painter.rect(
            egui::Rect::from_two_pos(at(above, -r), at(above + len, r)),
            0.0,
            crate::egui_app::theme::with_alpha(pal.tool_holder, 70),
            Stroke::new(1.0, pal.tool_holder),
            egui::StrokeKind::Inside,
        );
    }
    // The outline as one closed polyline (a stroke has no convexity rule):
    // right side up, then the mirrored left side back down. Axis points
    // (r = 0) appear on the right only - the mirror skips them so the tip
    // and top close with a single straight segment.
    let mut pts: Vec<Pos2> = Vec::with_capacity(outline.len() * 2);
    for &(z, r) in &outline {
        pts.push(at(z, r));
    }
    for &(z, r) in outline.iter().filter(|q| q.1 > 0.0).rev() {
        pts.push(at(z, -r));
    }
    painter.add(Shape::closed_line(
        pts,
        Stroke::new(1.0, pal.tool_edge),
    ));
    Some(Scale { axis_x, base_y, scale, z_max })
}

/// The name main's preview tests (#2005) call: the full-width editor preview.
#[cfg(test)]
fn draw_tool_preview(ui: &mut Ui, t: &tools::Tool, u: Unit, body: Option<&[(f64, f64, f64)]>) {
    draw(ui, t, u, body, None, true);
}

#[cfg(test)]
#[path = "tests/tool_preview.rs"]
mod tool_preview_tests;
