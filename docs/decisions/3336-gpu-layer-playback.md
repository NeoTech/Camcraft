## #3336 - The Simulate view starts the GPU layer playback (slice S3c of #2629)

**What was decided**

- The Simulate view starts the layer playback of PR #3882 when three
  things are true: the window shell made a cutter on the render device,
  the stock eye has a GPU trace pass, and no eye refused a layer frame in
  this process. In each other case the CPU playback of before starts.
- The cutter runs the S2/S3a kernel (`cut_onto_or_cpu_on`) on the render
  device of the window. It makes no second device.
- A software adapter (`DeviceType::Cpu`: WARP, lavapipe) gets no cutter.
  `CAMCRAFT_GPU_COMPUTE=off` gives no cutter. A device without compute
  gives no cutter. One log line names the reason.
- The kernel is compiled on the playback worker, at the first batch.
- If the eye holds a layer frame but marches it on the CPU, the layer
  playback ends and the CPU playback starts. The status line says so one
  time. The layer playback then stays off for the process.
- `FieldPlay::set_layer` keeps the layer mode off when no operation has
  an f32 twin.
- The PR says `Refs #3336`. It does not close the issue.

**Why**

- The render device: a second device in one process crashed a driver
  (#3589). The shared compute device of `camcraft-gpu` takes the software
  adapter first (#2629, precondition 2).
- No kernel on a software adapter: the review of PR #3532 measured the
  kernel on WARP at 11.9 ms against 1.1 ms for the CPU law on one batch.
- The eye rule: the layer operations are not cut in `FieldView::stock`.
  The CPU march reads that stock. It would draw uncut stock.
- The `set_layer` rule: a layer frame is always the whole layer. The
  bench (`playback_cost_in_the_layer_mode`, release, this machine) read
  11.2 ms for the whole-layer pack on fig8 and 9.1 ms on the Pocket. A
  patch frame of the CPU playback is 0.13 ms. With no layer operation the
  layer mode costs time and gives nothing.
- `Refs`, not `Fixes`: the two reference programs of #3336 (the fig8 2D
  Offset and the 3D Pocket) have arc moves. The f32 twin refuses an
  operation with an arc, so these programs stay on the CPU field. Arcs
  on the kernel are slice S4 (#2628). The issue also needs a reading on
  a hardware adapter. This machine has only llvmpipe.

**Considered and dropped**

- The CPU twin law as the cutter when there is no hardware adapter. It
  was not measured as faster than the field for a real program, and the
  two reference programs do not reach it. Dropped until a reading shows
  a gain.
- Chords in place of arcs for the twin. A chord cuts inside the true
  circle, so the layer would remove more than the f64 field. That breaks
  the one-sided law of the twin. Dropped: #2628 owns arcs.
- A path choice by batch size (#2629, precondition 3). It needs timings
  from a hardware adapter. Dropped here; only the adapter type decides.
- The kernel writing into `Scene.cut` with no read-back (`cut_into`).
  The layer mode of PR #3882 reads each layer back for its keyframes and
  its merge with the field. Dropped: it changes the worker protocol.
- Patch frames for the layer. Dropped: not needed for a correct picture.

**Cost to change later**

- The start rule is one function (`LayerPlayback::start_cutter`) and the
  adapter rule is one function (`kernel_adapter`), both in
  `crates/ui/src/play_layer.rs`. A rule by batch size goes into the
  closure of `gpu_cutter`.
