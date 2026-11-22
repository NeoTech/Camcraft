//! Windowless wgpu device for the headless run and the GPU test suites.
//! The adapter policy moved down to `camcraft-gpu` (#2620), so the compute
//! seam and the renderer take the same device; this module keeps the old
//! paths. Provenance: re-exports of our own `camcraft_gpu` adapter policy.

pub use camcraft_gpu::{device_async, AdapterKind, HeadlessGpu, REQUIRE_GPU_ENV};
#[cfg(not(target_arch = "wasm32"))]
pub use camcraft_gpu::{device, device_for_test};
