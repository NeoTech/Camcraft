//! 3D viewport: indexed triangles with a z-buffer, rendered offscreen and
//! handed to the shell as a texture so the view docks like any panel (the
//! imgui external-texture bridge retired at M1.4.2 M-g; the wgpu types
//! come through `egui_wgpu` now - the same unified wgpu underneath).
//! Provenance: standard orbit camera + z-buffered indexed rendering from
//! public graphics literature.
//!
//! Selection is baked into the vertex stream (an extra float per vertex):
//! `Queue::write_buffer` lands at submit time, so a per-draw uniform tint
//! would silently take the last frame's color - one uniform means one
//! upload per frame and one draw call.


use egui_wgpu::wgpu;

mod camera;
mod init;
mod pipes;
mod readback;
mod render;
mod shader;
mod upload;
mod wide;

pub use camera::{Camera, ProjectionMode};

pub use wide::{
    GUARANTEED_SAMPLES, LIFT_WIDTHS, MAX_LINE_PX, MIN_LINE_PX, SEGMENT_BYTES, effective_samples, line_px,
    offered_samples, segment_bytes,
};

/// vec3 position + f32 selection flag + f32 deviation class. Like `sel`,
/// the class rides baked in the vertex stream (same submit-time law as
/// above) and every `vs_main` pipeline reads all three attributes - the
/// layouts are one const, so a pipeline can never forget the third.
const VERTEX_STRIDE: u64 = 20;

/// The one vertex-buffer law, shared by every solid pipeline.
pub(crate) const VERTEX_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: VERTEX_STRIDE,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &[
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x3,
            offset: 0,
            shader_location: 0,
        },
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32,
            offset: 12,
            shader_location: 1,
        },
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32,
            offset: 16,
            shader_location: 2,
        },
    ],
};

pub(crate) const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// `struct U`: mvp + eye + wire (80 bytes), six palette vec4s, the
/// wireframe edge ink of the live ground (#698), then the overlay line
/// geometry `vp` (#2958: target size, half width, half width + 1 px).
const UNIFORM_SIZE: u64 = 80 + 8 * 16;

/// One triangle-soup part for the GPU: world-space vertices (transform
/// applied by the caller), plus selection and kind flags. Stock parts are
/// drawn blended after the models; region-fill parts after those (see
/// `set_document`'s models → stock → fill partition). A fill part is pure
/// translucent overlay: it never contributes welded wire.
/// The class a model part's vertices carry when it is a fixture the
/// selected op strikes (#339 slice 4): `fs_main` tints it in the gouge
/// colour. Above every stock class (0..=3), which only `fs_stock` reads.
pub const STRUCK_CLASS: f32 = 4.0;

pub struct GpuPart {
    pub verts: Vec<[f32; 3]>,
    pub selected: bool,
    pub stock: bool,
    pub fill: bool,
    /// Per-vertex deviation class (stock_sim's `color_classes` law: 0
    /// clean, 1 rest, 2 gouge, 3 unjudged; 4 the incoming-stock preview,
    /// #406), read only by `fs_stock`.
    /// Empty means "no verdict rendered" - the part uploads class 0
    /// everywhere and paints exactly as an ordinary stock part.
    pub classes: Vec<f32>,
}

pub struct View3D {
    /// Every pipeline of the pass, built for `samples` (#2958).
    pipes: pipes::Pipes,
    shader: wgpu::ShaderModule,
    layout: wgpu::PipelineLayout,
    format: wgpu::TextureFormat,
    /// Samples per pixel of the pass targets: 1 draws straight into
    /// `color_view`; more renders into `msaa_view` and resolves there.
    samples: u32,
    /// The overlay line width in target pixels (#2958).
    line_px: f32,
    bind_group: wgpu::BindGroup,
    uniform: wgpu::Buffer,
    /// The verdict/selection palette the next render uploads (M13.4;
    /// the shell sets it from the world's colour-vision mode).
    palette: crate::egui_app::colorvision::Semantic,
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    /// Total indices; `[0, model_index_count)` are models,
    /// `[model_index_count, stock_index_end)` stock and the rest region
    /// fills (set_document partitions parts models-first).
    index_count: u32,
    model_index_count: u32,
    stock_index_end: u32,
    /// Where the overlay stock parts (the incoming stock, #406) start
    /// inside the stock range: `[overlay_index_start, stock_index_end)`
    /// is drawn even when the traced stock replaces the rest of the
    /// stock range - otherwise the preview vanished whenever toolpaths
    /// existed (hand check of #519 on a real GPU).
    overlay_index_start: u32,
    edge_buf: wgpu::Buffer,
    /// Pairs of u32 line indices; `[0, model_edge_count)` are model edges,
    /// the rest stock edges. Edges are per-part, so a stock face coplanar
    /// with a model face cannot weld the two into a seam-less blob.
    edge_count: u32,
    model_edge_count: u32,
    overlay_buf: wgpu::Buffer,
    /// Overlay segments (instances), not vertices (#2958).
    overlay_count: u32,
    /// The ground shadow pass and the occlusion pass (#2834).
    shade: crate::shade_pass::ShadePass,
    _msaa_tex: Option<wgpu::Texture>,
    msaa_view: Option<wgpu::TextureView>,
    _color_tex: wgpu::Texture,
    color_view: wgpu::TextureView,
    _depth_tex: wgpu::Texture,
    depth_view: wgpu::TextureView,
    width: u32,
    height: u32,
}

/// The distance behind the target the orthographic camera sits at. Its
/// framing is independent of distance (the half-height is `dist * tan
/// (fov/2)`), so nothing useful comes of pushing the eye toward the
/// model - dollying along the view direction with `dist` only swallowed
/// the model's near faces behind the camera as the zoom shrank. The
/// ortho `dist` is a zoom factor; the eye stays at a nominal distance.
pub(crate) const ORTHO_EYE_DIST: f64 = 100.0;

/// The upload order: models, then stock, then the overlay stock (which
/// the traced stock never replaces), then region fills; and the index in
/// that order where the overlay stock starts.
pub(crate) fn draw_order(parts: &[GpuPart]) -> (Vec<&GpuPart>, usize) {
    let mut order: Vec<&GpuPart> = parts.iter().filter(|p| !p.stock && !p.fill).collect();
    order.extend(parts.iter().filter(|p| p.stock && !p.fill && !is_overlay(p)));
    let overlay_from = order.len();
    order.extend(parts.iter().filter(|p| p.stock && !p.fill && is_overlay(p)));
    order.extend(parts.iter().filter(|p| p.fill));
    (order, overlay_from)
}

/// An overlay stock part: drawn in the stock pass but never replaced by
/// the traced stock (today only the incoming stock, #406).
pub(crate) fn is_overlay(p: &GpuPart) -> bool {
    p.classes.first().is_some_and(|c| (*c - crate::incoming_view::INCOMING_CLASS).abs() < 0.5)
}
