//! Viewport construction and target lifecycle.
//! Provenance: our existing wgpu resource setup, from the wgpu documentation.

use super::{View3D, UNIFORM_SIZE, line_px, pipes, shader::SHADER, wide};
use egui_wgpu::wgpu;

impl View3D {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("viewport3d"),
            source: wgpu::ShaderSource::Wgsl(format!("{SHADER}{}{}", wide::WIDE_SHADER, crate::shade_pass::WGSL).into()),
        });

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("viewport3d-bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("viewport3d-pl"),
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });

        // mat4x4 (64) + eye vec3 + wire (80), then the six palette
        // vec4s (M13.4): UNIFORM_SIZE.
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("viewport3d-uniform"),
            size: UNIFORM_SIZE,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("viewport3d-bg"),
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            }],
        });

        let pipes = pipes::build(device, &shader, &layout, format, 1);

        // 4-byte placeholders keep the empty-document path simple.
        let dummy = |label, usage| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: 4,
                usage: usage | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let vertex_buf = dummy("viewport3d-vbuf", wgpu::BufferUsages::VERTEX);
        let index_buf = dummy("viewport3d-ibuf", wgpu::BufferUsages::INDEX);
        let edge_buf = dummy("viewport3d-ebuf", wgpu::BufferUsages::INDEX);
        let overlay_buf = dummy("viewport3d-obuf", wgpu::BufferUsages::VERTEX);

        let targets = pipes::make_targets(device, format, width, height, 1);
        // #2834 + #2958: the shade pipelines draw in this pass, so they take
        // its sample count (1 until `set_samples`).
        let shade = crate::shade_pass::ShadePass::new(device, &shader, &layout, format, 1);

        View3D {
            pipes,
            shader,
            layout,
            format,
            samples: 1,
            line_px: line_px(1.5, 1.0),
            bind_group,
            uniform,
            palette: crate::egui_app::colorvision::Semantic::OFF,
            vertex_buf,
            index_buf,
            index_count: 0,
            model_index_count: 0,
            stock_index_end: 0,
            overlay_index_start: 0,
            edge_buf,
            edge_count: 0,
            model_edge_count: 0,
            overlay_buf,
            overlay_count: 0,
            shade,
            _msaa_tex: targets.msaa,
            msaa_view: targets.msaa_view,
            _color_tex: targets.color,
            color_view: targets.color_view,
            _depth_tex: targets.depth,
            depth_view: targets.depth_view,
            width: width.max(1),
            height: height.max(1),
        }
    }

    /// The shell-facing color target; re-register/update after resize.
    #[must_use]
    pub fn color_view(&self) -> &wgpu::TextureView {
        &self.color_view
    }

    #[must_use]
    pub const fn extent(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn resize(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat, width: u32, height: u32) {
        let (width, height) = (width.max(1), height.max(1));
        if width == self.width && height == self.height {
            return;
        }
        let targets = pipes::make_targets(device, format, width, height, self.samples);
        self._color_tex = targets.color;
        self.color_view = targets.color_view;
        self._depth_tex = targets.depth;
        self.depth_view = targets.depth_view;
        self._msaa_tex = targets.msaa;
        self.msaa_view = targets.msaa_view;
        self.width = width;
        self.height = height;
    }

    /// #2958: rebuild the pipelines and the multisample targets for
    /// `samples` per pixel. The resolved colour target is kept, so the
    /// shell's egui texture stays bound. The caller passes only a count
    /// the device offers (`effective_samples`). True when it changed.
    pub fn set_samples(&mut self, device: &wgpu::Device, samples: u32) -> bool {
        let samples = samples.max(1);
        if samples == self.samples {
            return false;
        }
        self.pipes = pipes::build(device, &self.shader, &self.layout, self.format, samples);
        // #2834: the shadow and the occluded faces draw in the same pass.
        self.shade.rebuild(device, &self.shader, &self.layout, self.format, samples);
        let (msaa, depth) = pipes::make_sample_targets(device, self.format, self.width, self.height, samples);
        self.msaa_view = msaa.as_ref().map(|t| t.create_view(&wgpu::TextureViewDescriptor::default()));
        self._msaa_tex = msaa;
        self.depth_view = depth.create_view(&wgpu::TextureViewDescriptor::default());
        self._depth_tex = depth;
        self.samples = samples;
        true
    }
}
