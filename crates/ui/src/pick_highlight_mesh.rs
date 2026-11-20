//! Single-mesh epaint tessellation and screen-space mesh caching for selected
//! and hovered pick highlights.
//!
//! Provenance: our own single-mesh stroke batcher over `egui::epaint::Tessellator`.

use crate::pick_highlight::{
    HighlightInner, ResolvedHighlights, WorldPolyline, lock, resolved_hover, resolved_row_hover,
    resolved_selected,
};
use crate::state::State;
use std::sync::Arc;

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct MeshKey {
    resolved_id: u64,
    aspect_bits: u64,
    mvp_bits: [[u32; 4]; 4],
    p0_bits: [u32; 2],
    p1_bits: [u32; 2],
    ppp_bits: u32,
    clip_bits: [u32; 4],
    px_bits: u32,
    halo_px_bits: u32,
    ink: egui::Color32,
    halo: egui::Color32,
    include_vertices: bool,
}

pub(crate) struct CachedMesh {
    key: MeshKey,
    /// `Shape::Mesh` holds an `Arc<Mesh>`: a cache hit sends a clone of
    /// the `Arc`, not a deep copy of the vertices.
    mesh: Option<Arc<egui::Mesh>>,
}

/// One screen-mesh slot: the last mesh and its key, and the number of
/// times this slot tessellated. The count pins the cache: a still frame
/// adds nothing to it (crates/ui/AGENTS.md, "Frame cost").
#[derive(Default)]
pub(crate) struct MeshSlot {
    cached: Option<CachedMesh>,
    pub(crate) builds: u64,
}

fn push_screen_polyline_runs(
    poly: &WorldPolyline,
    mvp: &[[f32; 4]; 4],
    to_px: &impl Fn([f32; 2]) -> egui::Pos2,
    out: &mut Vec<(Vec<egui::Pos2>, bool)>,
) {
    let n = poly.pts.len();
    if n < 2 {
        return;
    }
    let proj: Vec<Option<egui::Pos2>> = poly
        .pts
        .iter()
        .map(|&p| {
            crate::interact::project_uv(mvp, p)
                .map(to_px)
                .filter(|q| q.is_finite())
        })
        .collect();
    if let Some(none_idx) = proj.iter().position(Option::is_none) {
        let mut run: Vec<egui::Pos2> = Vec::new();
        let start = if poly.closed { (none_idx + 1) % n } else { 0 };
        for step in 0..n {
            let idx = (start + step) % n;
            match proj[idx] {
                Some(pt) => {
                    if run.last().copied() != Some(pt) {
                        run.push(pt);
                    }
                }
                None => {
                    if run.len() >= 2 {
                        out.push((std::mem::take(&mut run), false));
                    } else {
                        run.clear();
                    }
                }
            }
        }
        if run.len() >= 2 {
            out.push((run, false));
        }
        return;
    }
    let mut screen = Vec::with_capacity(n);
    for pt in proj.into_iter().flatten() {
        if screen.last().copied() != Some(pt) {
            screen.push(pt);
        }
    }
    if poly.closed && screen.len() >= 3 {
        out.push((screen, true));
    } else if screen.len() >= 2 {
        out.push((screen, false));
    }
}

#[allow(clippy::too_many_arguments)]
fn build_batched_mesh(
    painter: &egui::Painter,
    resolved: &ResolvedHighlights,
    include_vertices: bool,
    mvp: &[[f32; 4]; 4],
    aspect: f64,
    to_px: &impl Fn([f32; 2]) -> egui::Pos2,
    px: f32,
    halo_px: f32,
    ink: egui::Color32,
    halo: egui::Color32,
) -> Option<egui::Mesh> {
    let mut paths: Vec<(Vec<egui::Pos2>, bool)> = Vec::with_capacity(resolved.polylines.len());
    for poly in &resolved.polylines {
        push_screen_polyline_runs(poly, mvp, to_px, &mut paths);
    }
    if include_vertices && !resolved.vertices.is_empty() {
        let mut uv_segs = Vec::with_capacity(resolved.vertices.len() * 4);
        for &w in &resolved.vertices {
            crate::vertex_pick::vertex_marker(mvp, w, aspect, &mut uv_segs);
        }
        for (a, b) in uv_segs {
            let pa = to_px(a);
            let pb = to_px(b);
            if pa.is_finite() && pb.is_finite() && pa != pb {
                paths.push((vec![pa, pb], false));
            }
        }
    }
    if paths.is_empty() {
        return None;
    }
    let ppp = painter.ctx().pixels_per_point();
    let options = painter.ctx().tessellation_options(|o| *o);
    let mut tess = egui::epaint::Tessellator::new(ppp, options, [1, 1], Vec::new());
    tess.set_clip_rect(painter.clip_rect());
    let mut mesh = egui::Mesh::default();
    let stroke_pass = |tess: &mut egui::epaint::Tessellator,
                       mesh: &mut egui::Mesh,
                       stroke: egui::Stroke| {
        for (pts, closed) in &paths {
            let shape = if *closed {
                egui::epaint::PathShape::closed_line(pts.clone(), stroke)
            } else {
                egui::epaint::PathShape::line(pts.clone(), stroke)
            };
            tess.tessellate_path(&shape, mesh);
        }
    };
    if halo_px > 0.0 {
        stroke_pass(&mut tess, &mut mesh, egui::Stroke::new(halo_px, halo));
    }
    if px > 0.0 {
        stroke_pass(&mut tess, &mut mesh, egui::Stroke::new(px, ink));
    }
    (!mesh.vertices.is_empty() && !mesh.indices.is_empty()).then_some(mesh)
}

#[allow(clippy::too_many_arguments)]
fn paint_batched(
    painter: &egui::Painter,
    state: &State,
    resolved: &ResolvedHighlights,
    include_vertices: bool,
    aspect: f64,
    to_px: &impl Fn([f32; 2]) -> egui::Pos2,
    px: f32,
    halo_px: f32,
    ink: egui::Color32,
    halo: egui::Color32,
    get_slot: impl Fn(&mut HighlightInner) -> &mut MeshSlot,
) {
    if !aspect.is_finite()
        || aspect <= 0.0
        || (resolved.polylines.is_empty() && (!include_vertices || resolved.vertices.is_empty()))
    {
        return;
    }
    let mvp = state.cam.mvp(aspect);
    let p0 = to_px([0.0, 0.0]);
    let p1 = to_px([1.0, 1.0]);
    let clip = painter.clip_rect();
    let key = MeshKey {
        resolved_id: resolved.id,
        aspect_bits: aspect.to_bits(),
        mvp_bits: mvp.map(|r| r.map(f32::to_bits)),
        p0_bits: [p0.x.to_bits(), p0.y.to_bits()],
        p1_bits: [p1.x.to_bits(), p1.y.to_bits()],
        ppp_bits: painter.ctx().pixels_per_point().to_bits(),
        clip_bits: [
            clip.min.x.to_bits(),
            clip.min.y.to_bits(),
            clip.max.x.to_bits(),
            clip.max.y.to_bits(),
        ],
        px_bits: px.to_bits(),
        halo_px_bits: halo_px.to_bits(),
        ink,
        halo,
        include_vertices,
    };
    {
        let mut inner = lock(state);
        if let Some(cached) = &get_slot(&mut inner).cached
            && cached.key == key
        {
            if let Some(m) = &cached.mesh {
                painter.add(egui::Shape::mesh(Arc::clone(m)));
            }
            return;
        }
    }
    let mesh = build_batched_mesh(
        painter,
        resolved,
        include_vertices,
        &mvp,
        aspect,
        to_px,
        px,
        halo_px,
        ink,
        halo,
    )
    .map(Arc::new);
    if let Some(m) = &mesh {
        painter.add(egui::Shape::mesh(Arc::clone(m)));
    }
    let mut inner = lock(state);
    let slot = get_slot(&mut inner);
    slot.builds = slot.builds.saturating_add(1);
    slot.cached = Some(CachedMesh { key, mesh });
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_selected(
    painter: &egui::Painter,
    state: &State,
    aspect: f64,
    to_px: &impl Fn([f32; 2]) -> egui::Pos2,
    px: f32,
    halo_px: f32,
    ink: egui::Color32,
    halo: egui::Color32,
) {
    let resolved = resolved_selected(state);
    paint_batched(
        painter,
        state,
        &resolved,
        false,
        aspect,
        to_px,
        px,
        halo_px,
        ink,
        halo,
        |inner| &mut inner.selected_mesh,
    );
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_hover(
    painter: &egui::Painter,
    state: &State,
    aspect: f64,
    to_px: &impl Fn([f32; 2]) -> egui::Pos2,
    px: f32,
    halo_px: f32,
    ink: egui::Color32,
    halo: egui::Color32,
) {
    let resolved = resolved_hover(state);
    paint_batched(
        painter,
        state,
        &resolved,
        true,
        aspect,
        to_px,
        px,
        halo_px,
        ink,
        halo,
        |inner| &mut inner.hover_mesh,
    );
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_row_hover(
    painter: &egui::Painter,
    state: &State,
    aspect: f64,
    to_px: &impl Fn([f32; 2]) -> egui::Pos2,
    px: f32,
    halo_px: f32,
    ink: egui::Color32,
    halo: egui::Color32,
) {
    let resolved = resolved_row_hover(state);
    paint_batched(
        painter,
        state,
        &resolved,
        true,
        aspect,
        to_px,
        px,
        halo_px,
        ink,
        halo,
        |inner| &mut inner.row_hover_mesh,
    );
}

#[cfg(test)]
#[path = "pick_highlight_mesh_tests.rs"]
mod tests;
