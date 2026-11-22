//! The one compute device of the process (#2620): made on the first
//! request, cached for the life of the process, never made for each
//! operation or each frame. It is never dropped: a device made and dropped
//! in one process can race its sibling processes in the driver
//! (`crates/ui/tests/gpu_render.rs`, the shared device note).
//! Provenance: our own policy over wgpu's public device API.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use crate::adapter::{device, HeadlessGpu};
use crate::switch::env_switch;

static SHARED: OnceLock<Result<HeadlessGpu, String>> = OnceLock::new();
static LOST: AtomicBool = AtomicBool::new(false);

/// The process's compute device, or why there is none: the switch is off,
/// no adapter, the probe found no compute, or the device was lost. A lost
/// device is not made again: the CPU path runs for the rest of the process.
pub fn shared_device() -> Result<&'static HeadlessGpu, String> {
    env_switch()?;
    let made = SHARED.get_or_init(|| {
        let gpu = device()?;
        gpu.device.set_device_lost_callback(|reason, msg| {
            LOST.store(true, Ordering::SeqCst);
            tracing::warn!(target: "camcraft::gpu", "GPU compute device lost ({reason:?}): {msg}");
        });
        Ok(gpu)
    });
    let gpu = made.as_ref().map_err(Clone::clone)?;
    if LOST.load(Ordering::SeqCst) {
        return Err(format!("the GPU device {} was lost", gpu.name));
    }
    gpu.compute.as_ref().map_err(|e| format!("{e} ({})", gpu.name))?;
    Ok(gpu)
}
