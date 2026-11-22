## #2627 - GPU stock sim S2: the WGSL removal kernel in camcraft-gpu

**What was decided.**

- **No new crate edge.** The kernel takes plain values: `CutInput` =
  `CutGrid { dims, cell, cap }` and `CutItem`s (the chord relative to its
  brick origin in f32, radius, flank, claim margin, ball flag, the brick's
  lattice offset and its local cells `lo..=hi`). `camcraft-gpu` keeps no
  `camcraft-*` dependency (decision 2620; `xtask/src/heavy_plan_tests.rs`
  still holds). The issue text asked for a `camcraft-gpu -> camcraft-sim`
  edge; it is dropped: the kernel needs only numbers, and AGENTS.md says
  the caller passes values in.
- **camcraft-sim gives the items.** `twin_items(work, meta)` turns the
  twin's work list into `TwinItem { tile: TileChord, base, lo, hi }`, the
  same numbers field by field. `run_twin` now runs over those items, so
  the CPU twin and the GPU kernel read one prep. The viewport caller of
  #2629 (in `camcraft-ui`, which sees the two crates) maps `TwinItem` to
  `CutItem`.
- **Parity through a recorded file.** `crates/sim/tests/stock_cut_twin.rs`
  writes `crates/gpu/tests/fixtures/cut_twin_{ball,flat}_2627.txt` (the
  items, the twin's layer and the f64 layer, as f32 bits, sparse) and
  fails when they are stale (`CAMCRAFT_BLESS=1` writes them again: the
  `facet_grid_2619` pattern). `crates/gpu/tests/cut_parity.rs` reads them.
- **A CPU law in camcraft-gpu.** `cut::cut_cpu` copies the twin's f32
  math operation for operation. It is pinned bit for bit against the
  recorded twin layer. It is the fallback of the GPU path and the oracle
  of the spot check. Dropped: a closure from the caller for the fallback
  (the spot check needs a per-cell law, so the crate needs its own copy).
- **Kernel shape.** One workgroup (8 x 8) for each work item, the eight z
  cells of a column in a loop; the claim is `atomicMax` on the twin's
  order-preserving u32 key. Three entry points in one dispatch: fill (the
  `-4 cap` floor key), cut, decode (key -> the bits of its f32, in place).
  The output buffer is then the f32 layer of `pack_cut`'s layout
  (x fastest, then y, then z).
- **Watchdog and time bound.** At most `ITEMS_PER_DISPATCH = 256` items
  in one dispatch of `cs_cut`. The read-back waits at most
  `CUT_WAIT = 10 s` (`PollType::Wait` with a timeout), then the map gets
  100 ms. A wait that ends without an answer (the poll timeout, or a map
  that did not complete) is an `Err` with the kind `NoAnswer`; the kind,
  not the message text, turns the GPU path off for the process: a device
  that does not answer once is not asked again. A map that fails with an
  error is the kind `Other` and leaves the path on.
- **Spot check on every GPU batch.** The deepest claimed cell of up to 64
  evenly spaced items, and 8 fixed places across the lattice, are
  evaluated by the CPU law over every item that covers them. A GPU value
  more than half the winning item's claim margin away (values clamped at
  `-cap`) rejects the batch: the CPU law answers, the reason is logged,
  and the GPU path is off for the process. Cost: at most 72 x items
  containment tests, plus up to 64 x 512 cell evaluations for the deepest
  cell of each sampled item.
- **Validation before any work.** Each item is checked (finite values,
  coordinates under 1e6 mm, a positive radius, a flat with a flank, a
  non-negative margin, cells inside one brick and inside the lattice);
  the lattice is inside the 2^23 texel budget. A bad input is an `Err`
  for both laws, by name. The kernel's own bounds test is a second guard
  only.
- **Kernel compiled once.** `CutKernel::new(device)` builds the three
  pipelines; `cut_or_cpu` caches one kernel next to the shared device.
  A caller with its own device (the render device of #2629) holds its own
  `CutKernel`.
- **Display only.** No verdict, post gate or posted G-code reads this
  kernel. The verdict path (`simulate_document`) is not touched.

**Why.** The kernel's input is numbers; an edge for them would tie the
leaf crate to the simulation crate and block a later `sim -> gpu` edge.
The fixture keeps the test in the crate that it tests (AGENTS.md
"A test lives with the code it tests"). The spot check and the latch
turn a wrong GPU (a driver bug, a fused multiply that leaves the margin)
into a logged CPU result, never a silent wrong stock.

**Measured.** See PR body: the f32 GPU result against the f64 cut and
against the twin, with timings, on this machine's adapter.

**Cost to change later.** A different item layout changes `cut.wgsl`,
`cut_gpu::item_bytes` and the fixture writer together; the fixture test
fails first. Moving the CPU law out of this crate needs the edge that
this entry drops.
