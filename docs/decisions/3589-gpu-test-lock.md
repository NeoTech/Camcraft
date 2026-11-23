## #3589 - camcraft-ui lib tests use the GPU one at a time

**What was decided.** The `camcraft-ui` lib tests that make a wgpu instance
take one process-wide test-only lock (`egui_app::gpu_test_lock::gpu_guard`).
The guard is the first statement of each such test, so it is taken before
the harness or instance exists and released after they drop. The lock is
poison-tolerant: a test that panics with the lock held does not make the
other GPU tests fail. The eight tests are:

- `egui_app::tests::headless_run_succeeds`
- `egui_app::gpu::tests::adapter_probe_names_the_gpu`
- six `egui_app::report_tests` tests that press a kind button (the screenshot
  request makes egui_kittest build its wgpu renderer). The scout pack named
  five; `the_door_opens_the_window_and_a_kind_button_files_the_url` presses
  "Request a feature", which sends the same request, so it is in the list.

**Why.** The product makes one instance at a time, on one thread (scout pack,
`gpu_init.rs`, `adapter.rs`). Only the test binary makes several instances in
parallel test threads. The reporter's crash stopped with one test thread.
A lock is the smallest change: it touches only test code and keeps each test
as it is.

**Dropped.** One shared instance for the test process. egui_kittest builds its
own renderer and device inside `Harness::step` (`LazyRenderer`); a shared
instance needs a custom `wgpu_setup` for each harness and does not cover the
headless run, which makes its device through `camcraft_gpu::device()`. A lock
inside `camcraft_gpu::device_async` changes product code and does not cover
kittest. `--test-threads=1` for the whole binary makes all UI tests slow.

**Pins.** `every_gpu_test_takes_the_lock` reads the source of the eight tests
and requires the guard as the first statement. `the_lock_lets_one_holder_in_at_a_time`
and `a_panic_with_the_lock_held_does_not_block_the_next_test` pin the lock.
No test can prove that a random driver crash is gone: the reporter's repeat
runs are the proof.

**Cost to change later.** Low. A new lib test that makes a wgpu instance adds
one line and one row in `GPU_TESTS`.
