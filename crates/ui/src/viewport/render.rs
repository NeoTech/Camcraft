//! Offscreen viewport pass and draw-list encoding.
//! Provenance: our existing z-buffered indexed rendering, from public graphics literature.

use super::{View3D, UNIFORM_SIZE};
use egui_wgpu::wgpu;

impl View3D {
    /// Encode the 3D pass into the offscreen target. `eye` is the camera
    /// position in world space, for the specular term; `display` selects
    /// the display mode (the buffers are identical in each mode - only the
    /// draw list, `crate::draw_list::plan`, and the `u.wire` flag differ).
    pub fn render(
        &self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        mvp: &[[f32; 4]; 4],
        eye: [f32; 3],
        display: crate::state::DisplayMode,
    ) {
        self.render_with(queue, encoder, mvp, eye, display, None);
    }

    /// `render` with the stock slot handed to `stock` (M7.2 B3b's
    /// traced stock): when given, it draws in place of the stock sheet's
    /// faces and wire, in shaded mode only, and group 0 is re-bound after.
    pub fn render_with(
        &self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        mvp: &[[f32; 4]; 4],
        eye: [f32; 3],
        display: crate::state::DisplayMode,
        stock: Option<&dyn Fn(&mut wgpu::RenderPass<'_>)>,
    ) {
        let wireframe = display.is_wireframe();
        // The live theme's viewport ground (#698): the clear colour and
        // the wireframe ink follow it, so a light theme gets a light ground.
        let ground = crate::egui_app::theme::current().viewport;
        let light = crate::egui_app::ground_ink::light_ground(ground);
        let wire = crate::egui_app::ground_ink::wire_ink(light);
        let mut data = [0u8; UNIFORM_SIZE as usize];
        let mut o = 0usize;
        for col in mvp {
            for v in col {
                data[o..o + 4].copy_from_slice(&v.to_le_bytes());
                o += 4;
            }
        }
        for v in eye {
            data[o..o + 4].copy_from_slice(&v.to_le_bytes());
            o += 4;
        }
        data[76..80].copy_from_slice(&(f32::from(u8::from(wireframe))).to_le_bytes());
        let p = &self.palette;
        o = 80;
        for c in [p.clean, p.rest, p.gouge, p.unjudged, p.select_hot, p.select_warm] {
            for v in [c[0], c[1], c[2], 1.0] {
                data[o..o + 4].copy_from_slice(&v.to_le_bytes());
                o += 4;
            }
        }
        for v in [wire[0], wire[1], wire[2], 1.0] {
            data[o..o + 4].copy_from_slice(&v.to_le_bytes());
            o += 4;
        }
        // #2958: the overlay geometry - target size, half width, and the
        // quad's reach (half width plus the one-pixel fade).
        let hw = self.line_px * 0.5;
        for v in [self.width as f32, self.height as f32, hw, hw + 1.0] {
            data[o..o + 4].copy_from_slice(&v.to_le_bytes());
            o += 4;
        }
        queue.write_buffer(&self.uniform, 0, &data);

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("viewport3d-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: self.msaa_view.as_ref().unwrap_or(&self.color_view),
                resolve_target: self.msaa_view.as_ref().map(|_| &self.color_view),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: f64::from(ground.r()) / 255.0,
                        g: f64::from(ground.g()) / 255.0,
                        b: f64::from(ground.b()) / 255.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        // One bind group for the frame: every pipeline shares the layout,
        // so switching pipelines mid-pass needs no resource rebinds. Draw
        // order is opaque models, their wire, then translucent stock with
        // its wire on top, then the cut-region fills last among triangles -
        // a light film over everything it overlaps - then the reference-
        // geometry overlay lines on top of all of it. Wireframe keeps the
        // law, swaps the models' color pass for a depth-only one (so the
        // hidden-line cull still holds) and drops the stock faces - the
        // box is a volume of stock, not a surface, and its full wire reads
        // truer than an occluded one.
        // The order itself is `draw_list::plan` (#2818): one list for each
        // display mode, read from the cached index partition.
        pass.set_bind_group(0, &self.bind_group, &[]);
        use crate::draw_list::Pipe;
        let plan = crate::draw_list::plan_with(display, &self.counts(), stock.is_some(), self.shade.effects);
        if self.index_count > 0 {
            pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
        }
        for step in plan.steps() {
            let pipeline = match step.pipe {
                Pipe::Faces => &self.pipes.pipeline,
                // No occlusion buffer: the plain faces (the list never asks).
                Pipe::FacesOccluded => self.shade.bind_occluded(&mut pass).unwrap_or(&self.pipes.pipeline),
                Pipe::GroundShadow => {
                    self.shade.draw_shadow(&mut pass, step.range.clone());
                    pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
                    continue;
                }
                Pipe::FacesDepthOnly => &self.pipes.depth_pipeline,
                Pipe::Edges => &self.pipes.line_pipeline,
                Pipe::HiddenEdges => &self.pipes.hidden_line_pipeline,
                Pipe::Stock => &self.pipes.stock_pipeline,
                Pipe::Region => &self.pipes.region_pipeline,
                Pipe::TracedStock => {
                    if let Some(draw) = stock {
                        draw(&mut pass);
                        pass.set_bind_group(0, &self.bind_group, &[]);
                    }
                    continue;
                }
            };
            let indices = if step.pipe.uses_edge_buffer() { &self.edge_buf } else { &self.index_buf };
            pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
            pass.set_pipeline(pipeline);
            pass.draw_indexed(step.range.clone(), 0, 0..1);
        }
        if self.overlay_count > 0 {
            pass.set_pipeline(&self.pipes.wide_pipeline);
            pass.set_vertex_buffer(0, self.overlay_buf.slice(..));
            pass.draw(0..6, 0..self.overlay_count);
        }
    }
}
