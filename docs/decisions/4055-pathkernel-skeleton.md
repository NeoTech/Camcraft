## #4055 - The path kernel crate skeleton, the stage types and the planner switch

The dependency decision of the new edges (`camcraft-pathkernel` ->
`camcraft-geometry`, `camcraft-motion` -> `camcraft-pathkernel`) is the
record `docs/decisions/4024-path-kernel-design.md` ("Dependency
decision"). This file holds only the choices that the record left open
for this step.

**What was decided**

1. **The switch value is parsed in the crate, read in the app.**
   `PlannerEngine`, `PLANNER_ENV` and `engine_from_switch` are in
   `camcraft-pathkernel` (`engine.rs`): pure values, so the later
   comparison harness and the kind PRs use the same parse. The one reader
   of the process setting is `planner_engine()` in
   `crates/app/src/operation/planner_switch.rs`. It reads the setting one
   time for each process (`OnceLock`) and logs a bad value one time. Tests
   pass the engine directly to `check_engine(kind, engine)`.
2. **The switch is applied in `run_job`, before every arm**, not in each of
   the 16 match arms. Under "new", each phase-1 kind is refused with
   "the new path planner does not plan <kind> yet (CAMCRAFT_PLANNER=new)".
   A kind PR adds its kernel route there. A kind outside phase 1 keeps its
   planner under either value: the switch does not cover it.
3. **Accepted values**: unset, empty or `old` (any case) give the old
   planner; `new` or `kernel` give the kernel. Each other value gives the
   old planner and one diagnostic that names the value.
4. **The Order stage has its own `StartSide { Auto, Inside, Outside }`.** The
   record names motion's `PassStart`, but the crate must not import motion.
   Motion converts `PassStart` into `StartSide`, as it converts `Links`
   into `LinkPolicy`.
5. **`EntryChoice.asked` is an `EntryKind`**, not motion's `Entry3D`, for
   the same reason.
6. **One file for each stage now** (`guide.rs`, `load.rs`, `entry.rs`), not
   the record's directories (`guide/{rings,..}.rs`). A directory comes with
   its first planner (K2, K3, K5), because an empty file is noise.
7. **The step law of a plan is a checked constructor** (`Plan::new`): each
   cut follows an entry or a stay-down link, a lift is followed by an
   entry, and a plan with cuts ends with an exit.
8. **The pin `switch_default_is_the_old_planner` is a heavy-lane test.** It
   generates the 2D Offset Inside on `fig8.step` through the dispatch and
   compares it with the #3750 golden `offset2d_fig8_inside.cl`. One fig8
   generation runs past the 60 s debug budget, as the #3750 pins do. Two
   fast pins in the default suite check the switch itself:
   `switch_default_passes_each_phase1_kind_to_the_old_planner` and
   `switch_new_refuses_a_kind_it_does_not_plan_yet`. The 2D Pocket has no
   app-level golden in the baseline. Its rings are pinned by the motion
   goldens `pocket_band_*` (`pocket_rings_unchanged_by_the_shared_planner`),
   and the switch passes the Pocket kind to the same arm.
9. **A test checks the gate lists against the workspace members**
   (`the_gate_lists_name_every_workspace_crate`, xtask), as the record asks.
   It pins: the root members equal `ALL_CRATES`; `BATCH_CRATES` is
   `ALL_CRATES` without `ui`; `CLIPPY` is `ALL_CRATES` without `kernel` (as
   before this change).

**Why**: the record fixes the place and the default of the switch, but not
how a test sets it, and not where the 16 kinds are listed. One check
before the arms keeps the old arms byte-for-byte unchanged.

**Dropped**: a field on `GenJob` (it reaches every caller of `collect_job`,
and the record forbids a per-project setting); a check inside each arm
(16 edits of old planner code); reading the setting on each call (a bad
value would log on each generation).

**Cost to change later**: low. The switch-over PR changes the default in
`PlannerEngine::default()`; the removal PR deletes `planner_switch.rs`
and its call in `run_job`.
