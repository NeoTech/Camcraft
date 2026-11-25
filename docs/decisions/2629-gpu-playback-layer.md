## #2629 - GPU stock sim S3b: the playback layer mode

**Decided.** Slice S3b of #2629 adds an opt-in layer mode to the
playback (`FieldPlay`, `PlayWorker`).

- At the build, before the first cut, each op gets one owner. A plain op
  (no rotary, turning, undercut or drill) whose tool is a ball or a flat
  end mill, and whose moves are all linear and upright, goes to the layer.
  The f64 field cuts every other op and never a layer op.
- The layer (`camcraft_sim::stock_sim::LayerPlay`) is a dense f32 cut layer
  on the packed frame's lattice. Its value at move k is the max-law fold of
  the layer ops' moves before k, so it has its own keyframes (every stride,
  in a 256 MiB budget) and cuts only when a frame is published. A jump cuts
  the layer once, not at each stride.
- A cutter that the caller gives (`LayerCutter`) cuts each batch onto the
  layer. `cpu_layer_cutter` is the CPU twin law (`run_twin_items_onto`). A
  GPU cutter (S3c) maps the `TwinItem`s to `camcraft_gpu::CutItem` and
  calls `cut_onto_or_cpu_on` with a device of the UI's choice.
- The picture is `max(pack_cut(field), layer)`, always a whole layer. No
  eye then repacks a layer frame from the field.
- A layer fault (a cutter error, a window over the twin's ceiling) turns
  the mode off. The reason is logged, and the field is rebuilt from the
  uncut stock with every op, at the same position (the CPU playback of
  before). The field keyframes start again; a precalc budget is lost.
- The display flank of a flat end mill is the display cut's own law
  (`simulate_display`: the stock top minus the lowest move, plus slack),
  taken once for each op instead of for each batch. Any flank that reaches
  above the stock gives the same visible cut.

**Why opt-in, and why no `camcraft-gpu` edge in `camcraft-app`.**
- The review of PR #3532 says that the shared compute device is the
  software adapter first (WARP runs on the CPU). S3 must use the render
  device or a hardware adapter.
- #3589: two `wgpu` instances alive at the same time crash on an Intel Arc
  adapter. The app has one instance today (the window's). A cutter that
  called `cut_onto_or_cpu` (the shared device) from the worker would make
  a second one.
- So the worker takes the cutter as a parameter, and the UI makes it on
  its render device in S3c. `camcraft-app` gets no new crate dependency.
- In the layer mode `FieldView::stock` does not hold the layer ops' cut.
  Two readers use it: the patch repack of `eye.rs` `full_cut` (layer
  frames are always whole, so it is not reached) and the CPU eye
  (`cpu_eye.rs` `march_job`). Only a caller with the GPU trace eye may
  turn the mode on. Nothing turns it on in this slice, so the product
  does not change.

**The spot-check tie (review note of PR #3878).** `want_at` in
`crates/gpu/src/cut_run.rs` now takes the largest claim margin of each
item whose CPU value is within its own margin of the result. Before, a
base value that won gave margin 0 and a bound of about one ULP. A spring
pass over a keyframe on hardware could then refuse a good GPU layer and
turn the GPU path off for the process. The bound stays on the safe side:
the GPU value stays at least half an item margin under the f64 cut.

**Dropped.**
- Calling `cut_onto_or_cpu` from the worker: the second instance (#3589)
  and the software adapter (the PR #3532 review).
- A shared keyframe value (field and layer in one `Keyframes`): it would
  tie the layer to the precalc seam of the field. The layer is a function
  of k, so separate keyframes are exact.
- A per-batch twin choice (a batch with an arc to the field, the rest to
  the layer): each op has one owner, so the max-law picture has no gap
  and no double cut to reason about.
- Patch frames in the layer mode: a patch needs the layer's dirty bricks
  and a merged patch; whole frames are first. This is a cost item for S3c.

**Remains (S3c).** The GPU cutter on the render device (the item mapping,
off the UI thread), the UI turning the layer mode on only with the trace
eye, the CPU eye reading the packed frame or keeping the mode off, patch
frames for the layer, the path choice from hardware timings, and the
parity proof on a hardware adapter.

**Cost to change.** Low: the mode is opt-in, and the field path keeps its
code and tests (`cut` moved file without a change in the off mode).
