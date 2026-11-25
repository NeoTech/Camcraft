## #3379 - the last unguarded GPU tests of the ui `egui_app` and lib binaries

**What was decided.**

- Three tests of `crates/ui/tests/egui_app.rs` take `headless_gpu::gpu_guard()`.
  They are the tests whose headless run draws, so it makes a wgpu device.
- Three ignored pixel snapshot tests of the lib take the same guard.
  `snapshot` and `snapshot_options` make the renderer of egui_kittest.
- The guard comes before `snapshot_guard()`. `render_all_guide_images` and
  the #1710 evidence test use the same order.
- `gpu_test_lock` pins the two sets. A new test in `egui_app.rs` must take
  the guard, or must be in the list of tests that make no device.

**The shared resource.** Native GPU objects of wgpu: instance, adapter and
device. #3589 and #3605 found that parallel creation and teardown of these
objects ends the process on Windows.

**Why.** The lock and the pins are there already. The change adds no new
lock type, and the default thread count stays.

**Dropped.**

- `--test-threads=1` for these binaries. It makes each test slower.
- A guard inside `headless::run`. The lock is for tests, and it is not
  re-entrant. A test that holds it would stop.
- A guard on the five tests of `egui_app.rs` that make no device.
- A change to `tests/shell.rs`. It has one test that makes a device, so no
  second thread of that binary can make one at the same time.

**Not proven.** No Windows machine was available. The crash itself was not
reproduced. The pins read source text; they do not see the crash.

**Cost to change later.** Low. All changes are in test code.
