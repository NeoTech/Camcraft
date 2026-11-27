## #2268 - The playback field and its worker live in camcraft-sim

- **What was decided:** `FieldPlay`, `PlayIndex`, `PlayWorker`, the
  playhead (`progress`, `settle_frac`, `Progress`), the cut dispatch, the
  layer mode, the precalc bindings and the packed frame move from
  `crates/app/src` to `crates/sim/src` (`play*.rs`, `play_worker*.rs`).
  With them move the plain-data parts that `FieldPlay` reads:
  - `FieldView`, `Packed`, `CutLayer`, `PackedBase`, `DEFAULT_TEXELS`,
    `build_stock`, `build_target` and `subdivide` go to
    `camcraft_sim::field_view`;
  - `rotary_pose_tol` and `simulate_rotary_slice` go to
    `camcraft_sim::stock_sim` (`rotary_slice.rs`);
  - `LathePlan` goes to `camcraft_sim::stock_sim` (`lathe_plan.rs`).

  App keeps the world reads and re-exports the moved items at their old
  paths (`stock_play`, `stock_play_worker`, `stock_view`,
  `stock_sim::rotary`, `stock_sim::lathe`):
  - `PlaySeed` is now an app newtype over the sim's `PlayInputs`.
    `PlaySeed::gather` reads the world and converts each `Planned` op to a
    `PlayOpPlan` (the undercut unpack that `from_seed` did before).
  - `FieldPlay::from_seed` and the `PlayWorker::spawn*` functions take
    `impl Into<PlayInputs>`, so each caller that passes a `PlaySeed` does
    not change.
  - `FieldPlay::new` and `FieldPlay::with_budget` read the world, so they
    become the methods of the app trait `stock_play::FieldPlayWorld`.
    Three test files import that trait. No assertion changed.
  - The Gate maps of the playback suites name the new sim modules, so a
    change in `crates/sim/src/play*.rs` selects them in the quick gate.
  - The pins are unit tests in the sim lib (`play_tests.rs`), not a new
    test binary.
- **The clock.** The worker timed its chunks and its publications with
  `camcraft_app::clock::Instant` (the #31 wasm32 seam). camcraft-sim has
  no `web-time` dependency, and this PR adds no dependency. The worker now
  reads the host's clock from `PlayInputs::clock`
  (`camcraft_sim::play::Clock = fn() -> Duration`). App passes
  `clock::monotonic` (the seam's `Instant`, since a fixed origin). The
  chunk and publication laws are the same: each `elapsed()` became
  `now() - t0` with a saturating subtraction.
- **Why:** AGENTS.md "app holds the document, nothing else". Only
  `PlaySeed::gather` and the world-built constructors read the ECS world.
  The #2192 review asked for this move.
- **Considered and dropped:**
  - Pass the rotary and lathe helpers into sim as closures. They use only
    sim types (`Stock`, `SimOp`, `RotaryAxis`, `LatheProfile`,
    `LatheAxis`), so a move is simpler and keeps one copy.
  - Use `std::time::Instant` in the sim worker. The worker thread does not
    start on wasm32 today, but #31 owes a run-inline fallback. Then that
    read would panic in a browser.
  - Add `web-time` to camcraft-sim as a wasm32-only dependency. It needs a
    maintainer decision on a dependency, and the clock is the host's
    concern.
  - Keep app wrapper types with `Deref` to the sim types, so that no test
    file changes. A wrapper is not a re-export, and it hides which crate
    owns the type.
- **Overlap:** stacked on #4380, which split `PlayIndex` and `PlaySeed`
  into `stock_play_index.rs`. This PR moves that file to sim.
- **Cost to change later:** low. The app paths stay as re-exports. A new
  caller can use `camcraft_sim::play` directly with its own `PlayInputs`.
