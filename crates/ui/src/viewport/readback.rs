//! Diagnostic readback of the viewport color target.
//! Provenance: our existing texture readback, from the wgpu documentation.

use super::View3D;
use egui_wgpu::wgpu;

impl View3D {
    /// Read the current color target back to RGBA8 bytes (row-major, no
    /// padding) with its extent. Tests and diagnostics only - the app never
    /// calls it. Requires the 4-bytes-per-pixel formats we use; `None`
    /// anywhere the GPU refuses (no readback path, map failure).
    #[must_use]
    pub fn snapshot_rgba(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Option<(Vec<u8>, u32, u32)> {
        let (width, height) = (self.width, self.height);
        let row_bytes = (u64::from(width) * 4).next_multiple_of(256);
        let size = row_bytes * u64::from(height);
        if size == 0 || size > 64 * 1024 * 1024 {
            return None;
        }
        let buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("viewport3d-snap"),
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("viewport3d-snap-copy"),
        });
        enc.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self._color_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buf,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(row_bytes.try_into().unwrap_or(u32::MAX)),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        queue.submit([enc.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        buf.slice(..)
            .map_async(wgpu::MapMode::Read, move |res| {
                let _ = tx.send(res);
            });
        if device.poll(wgpu::PollType::wait_indefinitely()).is_err() {
            return None;
        }
        if !matches!(rx.recv(), Ok(Ok(()))) {
            return None;
        }
        let raw = buf.slice(..).get_mapped_range().ok()?;
        let mut out = Vec::with_capacity(usize::try_from(u64::from(width) * u64::from(height) * 4).ok()?);
        for row in 0..height {
            let off = usize::try_from(row_bytes * u64::from(row)).ok()?;
            out.extend_from_slice(&raw[off..off + (width * 4) as usize]);
        }
        drop(raw);
        buf.unmap();
        Some((out, width, height))
    }
}
