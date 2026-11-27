//! Document and overlay uploads, display settings, and draw partitions.
//! Provenance: our existing viewport uploads and public indexed-rendering practice.

use super::{GpuPart, View3D, MIN_LINE_PX, MAX_LINE_PX, draw_order, segment_bytes};
use egui_wgpu::wgpu;

impl View3D {
    /// Replace the GPU document. Parts are world-space triangle soup (the
    /// caller applies rigid transforms) tagged with selection and stock
    /// kind; partitioning for the draw order happens here, not in the
    /// caller, so a sloppy caller still gets models-first, stock-last.
    pub fn set_document(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, parts: &[GpuPart]) {
        let (order, overlay_from) = draw_order(parts);

        let mut verts: Vec<u8> = Vec::new();
        let mut idx: Vec<u32> = Vec::new();
        let mut edges: Vec<u32> = Vec::new();
        let mut vertex_count = 0u32;
        let mut model_indices = 0u32;
        let mut stock_end = 0u32;
        let mut overlay_start = None::<u32>;
        let mut model_edges = 0u32;
        for (pi, part) in order.iter().enumerate() {
            if pi == overlay_from {
                overlay_start = Some(idx.len() as u32);
            }
            let sel: f32 = if part.selected { 1.0 } else { 0.0 };
            for (vi, v) in part.verts.iter().enumerate() {
                for c in v {
                    verts.extend_from_slice(&c.to_le_bytes());
                }
                verts.extend_from_slice(&sel.to_le_bytes());
                // Absent class = clean, per the field's law; a class list
                // shorter than the vertex list is a caller bug we paint
                // over as steel, not a crash.
                let cls = part.classes.get(vi).copied().unwrap_or(0.0);
                verts.extend_from_slice(&cls.to_le_bytes());
            }
            let n = part.verts.len() as u32;
            idx.extend(vertex_count..vertex_count + n);
            // CAD-style wire per part: crease and boundary edges only, and
            // never welded across parts - a stock face coplanar with a
            // model face must not erase either solid's outline. Region
            // fills are a pure translucent overlay; their outline already
            // rides the overlay lines, so they add no wire.
            if !part.fill {
                edges.extend(feature_edges(&part.verts).into_iter().map(|i| i + vertex_count));
                stock_end = idx.len() as u32;
                if !part.stock {
                    model_indices = idx.len() as u32;
                    model_edges = edges.len() as u32;
                }
            }
            vertex_count += n;
        }
        if verts.is_empty() {
            self.index_count = 0; // render() then draws nothing, just clears
            self.model_index_count = 0;
            self.stock_index_end = 0;
            self.overlay_index_start = 0;
            self.edge_count = 0;
            self.model_edge_count = 0;
            return;
        }
        let vertex_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("viewport3d-vbuf"),
            size: verts.len() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&vertex_buf, 0, &verts);
        let idx_bytes: Vec<u8> = idx.iter().flat_map(|i| i.to_le_bytes()).collect();
        let index_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("viewport3d-ibuf"),
            size: idx_bytes.len() as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&index_buf, 0, &idx_bytes);
        let edge_bytes: Vec<u8> = edges.iter().flat_map(|i| i.to_le_bytes()).collect();
        let edge_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("viewport3d-ebuf"),
            size: (edge_bytes.len() as u64).max(4),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        if !edge_bytes.is_empty() {
            queue.write_buffer(&edge_buf, 0, &edge_bytes);
        }
        self.vertex_buf = vertex_buf;
        self.index_buf = index_buf;
        self.index_count = idx.len() as u32;
        self.model_index_count = model_indices;
        self.stock_index_end = stock_end;
        self.overlay_index_start = overlay_start.unwrap_or(stock_end).min(stock_end);
        self.edge_buf = edge_buf;
        self.edge_count = edges.len() as u32;
        self.model_edge_count = model_edges;
    }

    /// Replace the overlay line set: pairs of `[x, y, z, r, g, b]` vertices
    /// drawn non-indexed after every solid - the WCS triads, world axes and
    /// cached toolpaths (the producers cap the list before it gets here).
    pub fn set_overlay(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, lines: &[[f32; 6]]) {
        // #2958: one 32-byte instance per segment, drawn as a wide quad.
        let data = segment_bytes(lines);
        if data.is_empty() {
            self.overlay_count = 0; // render() then skips the draw entirely
            return;
        }
        let buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("viewport3d-obuf"),
            size: data.len() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&buf, 0, &data);
        self.overlay_buf = buf;
        self.overlay_count = u32::try_from(lines.len() / 2).unwrap_or(u32::MAX);
    }

    /// The cached index partition that the draw list reads (#2818).
    #[must_use]
    pub const fn counts(&self) -> crate::draw_list::Counts {
        crate::draw_list::Counts {
            index_count: self.index_count,
            model_index_count: self.model_index_count,
            stock_index_end: self.stock_index_end,
            overlay_index_start: self.overlay_index_start,
            edge_count: self.edge_count,
            model_edge_count: self.model_edge_count,
            shadow_vertex_count: self.shade.shadow_count,
            occlusion_vertex_count: self.shade.occlusion_count,
        }
    }

    /// Replace the cached shade data (#2834): the ground shadow mesh and
    /// the occlusion values of the model vertices of the last
    /// `set_document`. Call it with each new document.
    pub fn set_shade(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, shade: &crate::shade_pass::Shade) {
        self.shade.set(device, queue, shade);
    }

    /// The shading switches that the next `render` reads (#2834). They pick
    /// draw list steps; they do not touch a buffer.
    pub fn set_effects(&mut self, effects: crate::draw_list::Effects) {
        self.shade.effects = effects;
    }

    /// The overlay segment count of the last upload.
    #[must_use]
    pub const fn overlay_segments(&self) -> u32 {
        self.overlay_count
    }

    /// #2958: the overlay line width in target pixels (`line_px` holds it
    /// to `MIN_LINE_PX..=MAX_LINE_PX`). A uniform: nothing re-uploads.
    pub fn set_line_px(&mut self, px: f32) {
        self.line_px = if px.is_finite() { px.clamp(MIN_LINE_PX, MAX_LINE_PX) } else { MIN_LINE_PX };
    }

    #[must_use]
    pub const fn line_width_px(&self) -> f32 {
        self.line_px
    }

    #[must_use]
    pub const fn samples(&self) -> u32 {
        self.samples
    }

    /// The sample count of the ground shadow and occluded face pipelines
    /// (#2834). They draw in the 3D pass, so it is always `samples()`.
    #[must_use]
    pub const fn shade_samples(&self) -> u32 {
        self.shade.samples()
    }

    /// The verdict/selection palette the next `render` uploads (M13.4's
    /// colour-vision remap). A uniform, not a vertex re-bake: a mode
    /// switch re-colours without touching the geometry buffers.
    pub fn set_palette(&mut self, p: &crate::egui_app::colorvision::Semantic) {
        self.palette = *p;
    }
}

/// Wireframe of a triangle soup: boundary edges and creases only, facet
/// seams inside a flat or smoothly-bent run stay hidden. The rule itself
/// lives in the app layer (`geometry::feature`, shared with M1.5 face and
/// chain topology); this is the adapter into the GPU index list. Welding
/// is per part, as the caller splits the soup that way.
fn feature_edges(verts: &[[f32; 3]]) -> Vec<u32> {
    let pts: Vec<[f64; 3]> = verts
        .iter()
        .map(|v| [f64::from(v[0]), f64::from(v[1]), f64::from(v[2])])
        .collect();
    camcraft_app::geometry::feature::feature_edges(&pts)
        .into_iter()
        .flat_map(|e| [e.i, e.j])
        .collect()
}
