## #2629 - GPU stock sim S3a: the incremental cut onto a keyframe layer

**Decided.** #2629 (S3) is too large for one PR (the escalation comment of
2026-11-22 lists five gaps). This PR is slice S3a only: the kernel and the
CPU law can cut a batch onto an earlier layer (a playback keyframe), not
only from the empty floor.

- `cut.wgsl` gets a fourth entry point, `cs_encode`. It turns a layer (the
  f32 bits that `cs_decode` writes) back into order-preserving keys, in
  place. A batch then runs encode, cut, decode on the same buffer.
- `CutKernel::cut_into` runs that on a buffer of the caller (for S3c: the
  viewport's `Scene.cut` buffer on the render device) and returns the
  submission. It reads nothing back. `CutKernel::run_onto` does the same on
  its own buffer and reads the layer back (tests, and the fallback rule).
- `cut_cpu_onto` is the CPU law of the same cut. With the max law it is
  exact: a cut onto the keyframe of A gives the bits of A and B in one batch.
- `cut_onto_or_cpu` / `cut_onto_or_cpu_on` take the existing fallback rule
  of `cut_or_cpu`: the `CAMCRAFT_GPU_COMPUTE=off` switch, no adapter, no
  compute, a lost device, a kernel error or a failed spot check give the
  CPU law, with the reason, logged one time. `spot_check_onto` starts the
  CPU law at each probe from the base value.
- A base layer must have the lattice's texel count and finite values only.
  A NaN or an infinity is refused by name (the key order of a NaN is not a
  depth). `cut_into` cannot check a GPU buffer; a NaN there stays a NaN and
  shows only on the display.
- `cut_gpu.rs` was split along the buffer seam (`cut_buf.rs`: the batch
  plan, the byte forms, the read-back) to keep both files under 400 lines.

**Why.** The escalation named the incremental mode as the first gap: the
S2 kernel always cut from the floor, so a playback could not continue from
a keyframe on the GPU. This is the part that S3b (the worker protocol) and
S3c (the UI dispatch on the render device, the keyframe store) both need.
It changes no posted output and no verdict: the f64 engines still judge.

**#3589.** The new test binary (`crates/gpu/tests/cut_onto.rs`) touches
only `shared_device` (one `wgpu` instance in its process). The off-switch
and device-loss pins run in a child process of the same binary, because
each leaves the shared device off for the rest of its process.

**Dropped.**
- Running `CutKernel::run` on each frame and uploading the result with
  `write_cut`: it keeps the CPU cut in the worker, so playback is not
  faster (the escalation comment says the same).
- A persistent key buffer instead of `cs_encode`: the display buffer must
  hold f32 bits for `trace.wgsl` between frames, so an encode pass on the
  same buffer is one buffer less and one pass more (a pass over the texels,
  cheap against the cut).

**Remains (S3b, S3c).** The worker protocol in `camcraft-app` (send the
twin work for each frame on the GPU path, keep CPU checkpoints for the
fallback and the operations the twin refuses), the UI side in
`crates/ui/src/trace/` (dispatch off the UI thread on the render device,
the keyframe store in a VRAM budget, a fall back to the CPU frames on any
error), the choice by batch size and adapter kind from hardware timings,
and the parity proof on a hardware adapter (this PR ran on WARP only).

**Cost to change.** Low: the new entries are additive; `run` and
`cut_or_cpu` keep their behaviour and their tests.
