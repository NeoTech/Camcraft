## #3605 - one locked, shared GPU device for the camcraft-ui tests

**Decided:** The process GPU lock moves from the lib-only
`egui_app::gpu_test_lock` to `camcraft_ui::headless_gpu` (`gpu_guard`).
A new `headless_gpu::test_gpu(suite)` takes that lock and gives the one
device of the process. The device is made once (`OnceLock`) and is never
dropped. `trace_composite` and `trace_parity` use `test_gpu` in each test.
The lib test `view_pass::fit_target_never_asks_...` uses `test_gpu` too.
The three ignored evidence renderers take `gpu_guard()` first.

**Why:** Each test made and dropped its own wgpu device on a parallel test
thread. A hybrid-GPU Windows laptop (Intel Arc + NVIDIA RTX) ended the
process with 0xC0000005. The lock and the shared device remove both
hazards: no two tests use the GPU at the same time, and no device is
dropped while a second one is made. The `gpu_render` binary used this
pattern already. One home for the lock lets the lib tests and the shared
device take the same lock.

**Dropped:**
- A lock in `camcraft_gpu::device_for_test`. It serialises only creation,
  not the use and the drop of the device.
- `--test-threads=1` for the two binaries. It changes the gate and does not
  protect a new test that someone adds later.
- Skipping the tests on Windows. The maintainer asked for the root cause.

**Cost to change:** Small. The helpers are test support in one file. A
source-scan pin (`the_trace_binaries_share_one_locked_device`) names each
binary that must use `test_gpu`.
