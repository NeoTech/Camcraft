## #2620 - GPU compute seam: the camcraft-gpu crate, its probe, its fallback and one parity test

**What was decided.**

- A new crate `crates/gpu` (`camcraft-gpu`) holds the headless compute
  seam. It has no `camcraft-*` dependency. It owns:
  - the windowless device policy (software adapter first, then hardware,
    #615), moved down from `camcraft-ui`'s `headless_gpu`;
  - the capability probe (`probe`): `DownlevelFlags::COMPUTE_SHADERS` and
    the device limits, read from the device; WebGL2 / GL below ES 3.1 is
    "no compute" by name;
  - one shared device for each process (`shared_device`), made on the
    first request, never dropped, marked lost by the device-lost callback;
  - the off switch `CAMCRAFT_GPU_COMPUTE=off` (also `0`, `false`, `no`);
  - the first kernel, conservative facet binning (`bin_facets`), the GPU
    twin of the f64 support facet grid of #2619.
- The fallback rule: each entry answers `Result`. An `Err` names the reason
  and is logged one time for each process (`camcraft::gpu` target); the
  caller runs its CPU code. A GPU answer is only a filter (a superset of
  the f64 cells); the CPU f64 law stays the judge.
- Both binning paths check one 256 MiB host memory budget before they
  allocate cell lists. The budget includes empty cells, CSR offsets,
  facet-cell entries, upload bytes and read-back copies. Host allocations
  use fallible reservation. The CPU twin now returns `Result` too.
- GPU error scopes capture `Validation`, `Internal` and `OutOfMemory`.
  The scopes remain active through submission and read-back. Each scope
  is popped before the function returns, including a failed dispatch or
  map. The caller receives a named error and can use its CPU path.
- The binning boxes go to the GPU as tile-relative f32, each corner moved
  outward by `(extent + s) * 2^-18` and one more f32 step. This bound is
  far above the f32 cast error (2^-24) plus WGSL's 2.5 ULP division error,
  so the GPU cells hold every f64 cell entry.
- `wgpu` is a direct workspace dependency, pinned `=30.0.1`: the version
  that `egui-wgpu` 0.36.2 already locks. It moves with the egui set
  (#658, #1118). `Cargo.lock` gains only the `camcraft-gpu` entry.
- One new crate edge: `camcraft-ui -> camcraft-gpu`.
  - Items: `device`, `device_async`, `device_for_test`, `AdapterKind`,
    `HeadlessGpu`, `REQUIRE_GPU_ENV` (six; `headless_gpu.rs` re-exports
    them, so no UI call site changed).
  - Why the value cannot be passed in: these are the device policy itself,
    not values. Two copies of the policy would let the renderer and the
    compute seam choose different adapters.
  - What rebuilds: a change in `camcraft-gpu` rebuilds `camcraft-ui` and
    its binaries. `camcraft-gpu` itself depends only on `wgpu`, `tracing`
    and (native) `pollster`, which `camcraft-ui` already had.
- No `camcraft-motion -> camcraft-gpu` edge and no edge the other way. The
  parity test reads a recorded fixture,
  `crates/gpu/tests/fixtures/facet_grid_2619.grid`. Motion's lib test
  `gpu_parity_fixture_matches_the_grid` writes it with `CAMCRAFT_BLESS=1`
  and otherwise fails when the f64 grid changes and the file is stale.
- The recorded fixture has every box edge on a cell border. There, a cell
  map that moves up by one cell still gives a superset, so the fixture
  alone does not catch it. A second parity test,
  `twin_and_gpu_bins_hold_the_f64_cells_of_off_border_boxes`, bins 200
  boxes with edges inside cells on a far origin. It computes the f64 cells
  in the test with the #2619 law (floor, then clamp). A mutation of the
  shader or the twin cell map by `+ 1.0` makes it fail.
- No `camcraft-app` edge in this slice: the wiring of a real caller is
  #2621.
- The pipeline is compiled for each call. No pipeline cache yet: the one
  caller is a test. A cache comes with the first production caller.
- The dispatch blocks (`pollster`). An async dispatch is left to #2623.
- wasm32: the crate builds for `wasm32-unknown-unknown`. The blocking parts
  (`device`, `device_for_test`, `shared_device`, `bin_facets`, `bin_on`)
  are native only by `cfg`, as `headless_gpu` was before.
- `xtask` gate lists (`ALL_CRATES`, `BATCH_CRATES`, `CLIPPY`) name `gpu`,
  so quick, batch and release run its tests and clippy.

**Why.** The issue asks for a seam that later kernels (#2621-#2626) plug
into, with a probe, a CPU fallback and a parity test, and with tests green
on a machine with no GPU. The device policy had to move down: the seam
must not depend on the UI crate, and a second policy copy would drift.

**What was considered and dropped.**

- A dev-dependency of `camcraft-gpu` on `camcraft-motion` for the parity
  test: an edge for one test. Dropped for the recorded fixture.
- Keep the device policy in `camcraft-ui` and make the GPU crate depend on
  it: the wrong direction (domain on UI). Dropped.
- Take wgpu through `egui-wgpu`'s re-export: it pulls the egui stack into a
  headless compute crate. Dropped.
- A CSR output built on the GPU (count, prefix sum, fill): three passes and
  a sort for ascending indices. A per-cell bit set gives ascending, unique
  indices from one pass with `atomicOr`. Dropped for this slice; the plan
  refuses a bit set past the storage limit, and the caller falls back.
- A compact representation for empty grids alone: it does not bound
  memory for nonempty grids. Dropped for one budget on both paths.
- Keep the default GPU error handler: an allocation or internal error
  can cause a panic. Dropped for error scopes that return the failure.
- A test that needs a GPU to pass: refused by the issue. The GPU half of
  the parity test prints a loud SKIP without an adapter, and fails only
  with `CAMCRAFT_REQUIRE_GPU=1`.

**Cost to change later.** The kernel and its plan are two files. A
pipeline cache is a `OnceLock` next to the shared device. If the egui set
moves wgpu, the root pin moves with it in the same PR. The host budget
can change in one constant. A caller of the CPU twin must handle its
`Result`. A new GPU error class needs a scope and a named conversion.
