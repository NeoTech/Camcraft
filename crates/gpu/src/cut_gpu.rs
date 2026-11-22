//! The GPU half of the stock removal kernel (#2627, GPU stock sim S2):
//! the pipelines of `cut.wgsl`, the upload of a checked batch, the
//! fill / cut / decode passes, and a read-back with a time bound. Every
//! failure (a limit, a validation, internal or out-of-memory error, a
//! failed map, a poll past its time bound) is an `Err` with the reason;
//! the caller then takes the CPU law (`cut_run::cut_or_cpu`).
//! Provenance: our own dispatch over wgpu's public compute API.

use std::sync::mpsc;
use std::time::Duration;

use crate::cut::{CutInput, CutItem};
use crate::probe::Compute;

const SHADER: &str = include_str!("cut.wgsl");

/// Work items in one dispatch of `cs_cut`: small enough that one
/// dispatch of the golden-section path (about 34 M pose evaluations)
/// stays far under a 2 s driver watchdog on a slow device.
pub const ITEMS_PER_DISPATCH: u32 = 256;

/// The longest wait for one batch to come back. Past it the batch is a
/// failure and the CPU law answers.
pub const CUT_WAIT: Duration = Duration::from_secs(10);

/// The kind of a kernel failure, so the caller's latch reads a value and
/// not the text of the reason.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CutErrorKind {
    /// A wait ended without an answer: the poll passed [`CUT_WAIT`], or
    /// the read-back map did not complete. A device that does not answer
    /// once is not asked again.
    NoAnswer,
    /// Any other failure: a limit, a validation, a memory error, or a
    /// batch that the kernel refuses.
    Other,
}

/// A failure of the GPU half: the reason and its kind.
#[derive(Clone, Debug, PartialEq)]
pub struct CutError {
    pub kind: CutErrorKind,
    pub message: String,
}

impl CutError {
    /// A failure that is not a timeout.
    pub fn other(message: impl Into<String>) -> CutError {
        CutError { kind: CutErrorKind::Other, message: message.into() }
    }

    /// A wait that ended without an answer.
    pub fn no_answer(message: impl Into<String>) -> CutError {
        CutError { kind: CutErrorKind::NoAnswer, message: message.into() }
    }

    /// The reason to turn the GPU path off for the rest of the process,
    /// or `None` when this failure leaves the path on.
    pub fn off_reason(&self) -> Option<String> {
        (self.kind == CutErrorKind::NoAnswer)
            .then(|| format!("{}; the GPU stock cut is off for this process", self.message))
    }
}

impl From<String> for CutError {
    fn from(message: String) -> CutError {
        CutError::other(message)
    }
}

impl std::fmt::Display for CutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

/// Bytes of one `Item` in `cut.wgsl` (five 16-byte rows).
const ITEM_BYTES: u64 = 80;

/// Bytes of `Params` in `cut.wgsl`.
const PARAMS_BYTES: u64 = 48;

/// The compiled kernel for one device: build it one time, run it often.
pub struct CutKernel {
    bgl: wgpu::BindGroupLayout,
    fill: wgpu::ComputePipeline,
    cut: wgpu::ComputePipeline,
    decode: wgpu::ComputePipeline,
}

/// The error class of a wgpu error, for the fallback reason.
pub(crate) fn gpu_error(err: wgpu::Error) -> String {
    let kind = match &err {
        wgpu::Error::OutOfMemory { .. } => "out of memory",
        wgpu::Error::Internal { .. } => "internal",
        wgpu::Error::Validation { .. } => "validation",
    };
    format!("stock cut kernel: GPU {kind} error: {err}")
}

/// Run `f` inside the three error scopes; any captured error wins.
fn scoped<T, E: From<String>>(device: &wgpu::Device, f: impl FnOnce() -> Result<T, E>) -> Result<T, E> {
    let oom = device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
    let internal = device.push_error_scope(wgpu::ErrorFilter::Internal);
    let validation = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let result = f();
    let mut failure = None;
    for scope in [validation, internal, oom] {
        if let Some(err) = pollster::block_on(scope.pop()) {
            failure.get_or_insert(gpu_error(err));
        }
    }
    match failure {
        Some(err) => Err(E::from(err)),
        None => result,
    }
}

fn entry(binding: u32, ty: wgpu::BufferBindingType, dynamic: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer { ty, has_dynamic_offset: dynamic, min_binding_size: None },
        count: None,
    }
}

impl CutKernel {
    /// Compile the three pipelines on `device`.
    pub fn new(device: &wgpu::Device) -> Result<CutKernel, String> {
        scoped(device, || {
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("camcraft-gpu stock cut"),
                source: wgpu::ShaderSource::Wgsl(SHADER.into()),
            });
            use wgpu::BufferBindingType as B;
            let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camcraft-gpu stock cut"),
                entries: &[
                    entry(0, B::Uniform, true),
                    entry(1, B::Storage { read_only: true }, false),
                    entry(2, B::Storage { read_only: false }, false),
                ],
            });
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("camcraft-gpu stock cut"),
                bind_group_layouts: &[Some(&bgl)],
                immediate_size: 0,
            });
            let pipe = |name: &str| {
                device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some(name),
                    layout: Some(&layout),
                    module: &module,
                    entry_point: Some(name),
                    compilation_options: Default::default(),
                    cache: None,
                })
            };
            Ok(CutKernel { fill: pipe("cs_fill"), cut: pipe("cs_cut"), decode: pipe("cs_decode"), bgl })
        })
    }

    /// Cut `input` on `device`: the dense f32 layer of `cut_cpu`'s
    /// layout. Blocks for at most about [`CUT_WAIT`]. `input` must have
    /// passed [`CutInput::check`]; it is checked again here.
    pub fn run(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        compute: &Compute,
        input: &CutInput,
    ) -> Result<Vec<f32>, CutError> {
        let n_tex = input.check()?;
        let plan = Layout::of(device, compute, input, n_tex)?;
        scoped(device, || {
            let (read, at) = self.submit(device, queue, input, &plan)?;
            read_back(device, &read, at, plan.out_bytes)
        })
    }

    fn submit(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        input: &CutInput,
        p: &Layout,
    ) -> Result<(wgpu::Buffer, wgpu::SubmissionIndex), String> {
        use wgpu::BufferUsages as U;
        let buf = |label, size, usage| {
            device.create_buffer(&wgpu::BufferDescriptor { label: Some(label), size, usage, mapped_at_creation: false })
        };
        let params = buf("stock cut params", p.params_bytes, U::UNIFORM | U::COPY_DST);
        let items = buf("stock cut items", p.item_bytes, U::STORAGE | U::COPY_DST);
        let keys = buf("stock cut keys", p.out_bytes, U::STORAGE | U::COPY_SRC);
        let read = buf("stock cut read", p.out_bytes, U::MAP_READ | U::COPY_DST);
        queue.write_buffer(&params, 0, &p.params);
        queue.write_buffer(&items, 0, &item_bytes(&input.items, p.item_bytes)?);
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camcraft-gpu stock cut"),
            layout: &self.bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &params,
                        offset: 0,
                        size: wgpu::BufferSize::new(PARAMS_BYTES),
                    }),
                },
                wgpu::BindGroupEntry { binding: 1, resource: items.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: keys.as_entire_binding() },
            ],
        });
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("stock cut") });
        {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("stock cut"), timestamp_writes: None });
            pass.set_bind_group(0, &group, &[0]);
            pass.set_pipeline(&self.fill);
            pass.dispatch_workgroups(p.tex_groups[0], p.tex_groups[1], 1);
            pass.set_pipeline(&self.cut);
            for (k, count) in p.chunks.iter().enumerate() {
                pass.set_bind_group(0, &group, &[p.offset(k)?]);
                pass.dispatch_workgroups(*count, 1, 1);
            }
            pass.set_bind_group(0, &group, &[0]);
            pass.set_pipeline(&self.decode);
            pass.dispatch_workgroups(p.tex_groups[0], p.tex_groups[1], 1);
        }
        enc.copy_buffer_to_buffer(&keys, 0, &read, 0, p.out_bytes);
        let at = queue.submit([enc.finish()]);
        Ok((read, at))
    }
}

/// The buffer plan of one batch, checked against the device limits.
struct Layout {
    params: Vec<u8>,
    params_bytes: u64,
    stride: u64,
    item_bytes: u64,
    out_bytes: u64,
    tex_groups: [u32; 2],
    chunks: Vec<u32>,
}

impl Layout {
    fn of(device: &wgpu::Device, compute: &Compute, input: &CutInput, n_tex: usize) -> Result<Layout, String> {
        let limit = compute.max_storage_buffer_binding_size.min(compute.max_buffer_size);
        let over = |what: &str, bytes: u64| format!("stock cut kernel: the {what} need {bytes} bytes, over the device limit {limit}");
        let out_bytes = (n_tex as u64).saturating_mul(4);
        let n_items = input.items.len() as u64;
        // an empty batch still binds a one-item buffer
        let item_bytes = n_items.max(1).saturating_mul(ITEM_BYTES);
        if out_bytes > limit {
            return Err(over("texels", out_bytes));
        }
        if item_bytes > limit {
            return Err(over("work items", item_bytes));
        }
        let max_dim = compute.max_compute_workgroups_per_dimension.max(1);
        let groups = u32::try_from(n_tex.div_ceil(64)).map_err(|_| "stock cut kernel: too many texel groups".to_string())?;
        let row = groups.min(max_dim);
        let rows = groups.div_ceil(row.max(1));
        if rows > max_dim {
            return Err(format!("stock cut kernel: {groups} texel groups pass the dispatch limit {max_dim}"));
        }
        let mut chunks = Vec::new();
        let n = u32::try_from(n_items).map_err(|_| "stock cut kernel: too many work items".to_string())?;
        let mut first = 0u32;
        while first < n {
            chunks.push((n - first).min(ITEMS_PER_DISPATCH));
            first += ITEMS_PER_DISPATCH;
        }
        let stride = u64::from(device.limits().min_uniform_buffer_offset_alignment).max(PARAMS_BYTES).next_multiple_of(16);
        let blocks = chunks.len().max(1);
        let params_bytes = stride.saturating_mul(blocks as u64);
        let mut params = Vec::new();
        params.try_reserve_exact(params_bytes as usize).map_err(|e| format!("stock cut kernel: no host memory: {e}"))?;
        params.resize(params_bytes as usize, 0);
        let g = &input.grid;
        let n_tex32 = u32::try_from(n_tex).map_err(|_| "stock cut kernel: too many texels".to_string())?;
        for (k, block) in params.chunks_exact_mut(stride as usize).enumerate() {
            let count = chunks.get(k).copied().unwrap_or(0);
            let first = u32::try_from(k).unwrap_or(u32::MAX).saturating_mul(ITEMS_PER_DISPATCH);
            let words = [g.dims[0], g.dims[1], g.dims[2], n_tex32, first, count, row, g.floor_key(), g.cell.to_bits(), g.cap.to_bits()];
            for (dst, w) in block.as_chunks_mut::<4>().0.iter_mut().zip(words) {
                dst.copy_from_slice(&w.to_le_bytes());
            }
        }
        Ok(Layout { params, params_bytes, stride, item_bytes, out_bytes, tex_groups: [row, rows], chunks })
    }

    fn offset(&self, k: usize) -> Result<u32, String> {
        u32::try_from(self.stride.saturating_mul(k as u64)).map_err(|_| "stock cut kernel: the uniform offset overflows".into())
    }
}

fn item_bytes(items: &[CutItem], bytes: u64) -> Result<Vec<u8>, String> {
    let mut v = Vec::new();
    v.try_reserve_exact(bytes as usize).map_err(|e| format!("stock cut kernel: no host memory: {e}"))?;
    for it in items {
        let row = |xyz: [u32; 3], w: u32| [xyz[0], xyz[1], xyz[2], w];
        let f = |p: [f32; 3]| p.map(f32::to_bits);
        let words = [
            row(f(it.a), it.r.to_bits()),
            row(f(it.b), it.flank.to_bits()),
            row(it.base.map(|v| v as u32), u32::from(it.ball)),
            row(it.lo, it.margin.to_bits()),
            row(it.hi, 0),
        ];
        for w in words.iter().flatten() {
            v.extend_from_slice(&w.to_le_bytes());
        }
    }
    v.resize(bytes as usize, 0);
    Ok(v)
}

/// Map the read buffer and wait at most [`CUT_WAIT`] for it.
fn read_back(
    device: &wgpu::Device,
    read: &wgpu::Buffer,
    at: wgpu::SubmissionIndex,
    out_bytes: u64,
) -> Result<Vec<f32>, CutError> {
    let (tx, rx) = mpsc::channel();
    read.slice(..).map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    device
        .poll(wgpu::PollType::Wait { submission_index: Some(at), timeout: Some(CUT_WAIT) })
        .map_err(|e| CutError::no_answer(format!("stock cut kernel: the GPU did not answer within {CUT_WAIT:?}: {e}")))?;
    map_wait_end(rx.recv_timeout(Duration::from_millis(100)))?;
    let out = {
        let view = read
            .get_mapped_range(..)
            .map_err(|e| CutError::other(format!("stock cut kernel: the read-back range failed: {e:?}")))?;
        if view.len() as u64 != out_bytes {
            return Err(CutError::other(format!("stock cut kernel: the read-back has {} bytes, not {out_bytes}", view.len())));
        }
        let mut out = Vec::new();
        out.try_reserve_exact(view.len() / 4).map_err(|e| CutError::other(format!("stock cut kernel: no host memory: {e}")))?;
        out.extend(view.as_chunks::<4>().0.iter().map(|c| f32::from_le_bytes(*c)));
        out
    };
    read.unmap();
    Ok(out)
}

/// Classify the read-back map wait: a wait that ends without the map is
/// a device that did not answer (the kind that the latch reads), while a
/// map that fails with an error is not a stalled device.
fn map_wait_end(wait: Result<Result<(), wgpu::BufferAsyncError>, mpsc::RecvTimeoutError>) -> Result<(), CutError> {
    match wait {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(CutError::other(format!("stock cut kernel: the read-back map failed: {e}"))),
        Err(_) => Err(CutError::no_answer("stock cut kernel: the read-back map did not complete")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_read_back_wait_that_ends_is_a_no_answer() {
        // The map wait that ends is the same kind as the poll wait: the
        // latch must fire for its text, not only for "did not answer".
        let ended = map_wait_end(Err(mpsc::RecvTimeoutError::Timeout));
        assert_eq!(ended.expect_err("the map wait ended").kind, CutErrorKind::NoAnswer);
        assert!(map_wait_end(Ok(Ok(()))).is_ok(), "a map that completed is not a failure");
    }
}
