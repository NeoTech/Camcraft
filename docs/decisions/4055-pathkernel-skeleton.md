## #4055 - The path kernel crate skeleton, the stage types and the planner switch

The dependency decision of the new edges (`camcraft-pathkernel` ->
`camcraft-geometry`, `camcraft-motion` -> `camcraft-pathkernel`) is the
record of #4024 in `RECOMMENDED.md` (heading `## #4024`, "Dependency
decision"; the imported items of each edge are in
`docs/decisions/4407-pathkernel-dependency-edges.md`). This file holds only the choices that the record left open
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
7. **The step law of a plan is a transition table** (`Plan::new`, #4123
   review finding 2). The states are clear, ready, cut end and lifted. A
   cut consumes the readiness, so each cut needs its own entry or
   stay-down link. A lift must be followed by an entry. The plan must end
   clear (after an exit, or empty). The table is in the doc comment of
   `Plan::new`, and the suite tests each cell.
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

10. **The quick-plan pin follows the gate maps.** The issue asks that
   `gate quick` on a diff in the new crate lists the crate's suites and
   the `motion` and `app` suites that depend on it. The quick gate selects
   suites of another crate only by their Gate map; it has no
   reverse-dependency fan-out (as for `kernel`). No motion code calls the
   kernel yet, so no motion suite can name a kernel module truthfully. The
   app suite `offset_pass_start` names `pathkernel::engine` (its switch
   pins use it). The xtask pin
   `a_pathkernel_change_plans_its_suites_and_the_mapped_app_suite` checks
   this. Each kind PR that makes motion call the kernel names the kernel
   module in its motion suite's Gate map. A change to `Cargo.toml` or
   `Cargo.lock` (as in this PR) still makes quick a batch. Dropped: adding
   the crate to `SHARED` (the record says no) and a fan-out rule for every
   crate (a gate change outside this issue).

11. **The CL round-trip test skips the annotation tail of a golden.** The
   #3750 baseline goldens (`offset2d_fig8_*.cl`, cherry-picked from
   `9c69f004a`) end with `; note:` and `; warning:` lines after the CL
   text. `camcraft-cl`'s `cl_text_round_trips_every_golden` reads each app
   `.cl` golden as CL text, so it failed on them. That fault is in
   `9c69f004a` itself; no output changed. The test now round-trips the CL
   text and checks that the tail has only those annotation lines. The
   golden bytes stay as recorded. Dropped: a new file extension for the
   baseline goldens (it renames the files that the record keeps as the
   baseline) and recording them again without annotations (that loses the
   notes and warnings that the pins compare).

12. **One rule for the crate: valid by construction** (#4123 review
    finding 3). A value of a public type is always valid. A field that
    could hold an invalid value is private and is set only by a checked
    constructor. An enum variant with data holds a validated newtype
    (`PlanarFrame`, `Disc`, `RingsChoice`, `RowsChoice`, `TraceOffset`,
    `Trochoid`, `ConstantLoad`, `StayPath`, `LiftReason`, `Area`, `Hole`,
    `Spot`, `Positive`, `Engagement`, `Gaps`). A list in a value holds only
    validated records, and its constructor checks the rules between them.
    The only public fields left are of types whose every value is valid
    (`OrderChoice`). `Refusal` maps a blank reason to `NO_REASON` and drops
    a place that is not finite, so the error type is valid too. Dropped: a
    `validate()` call at each boundary (it relies on each boundary
    remembering it, which is the class of the finding).
13. **The order stage may only restart or reverse a curve** (finding 1).
    `Curve::same_cut`: same index, family, closure and circle (within the
    tolerance); a closed curve is a cyclic shift of the input or of its
    reverse; an open curve is the input or its reverse. The start point of
    `OrderedSet` must be the first point of its first curve.
14. **Each proof names its place, and the descent stays in it** (finding
    4). `Proof::InAir(Area)`, `InColumn(Area)`, `InHole(Hole)` and
    `IntoMaterial { area, angle_or_pitch }`. The entry has approach moves
    (above the material, not bound) and descent moves (bound). A descent
    move is inside the proved area within the tolerance, or, for a hole, no
    farther from the centre than the hole radius less the tool radius. A
    hole proof must name one of the holes of the choice (centre and
    radius).
15. **Every record in every list is validated** (finding 5). The limit is
    an engagement in (0, 1]; a reading and an over-limit record are valid
    by their types (finite place, engagement in [0, 1], a reason that is
    not blank). Each reading above the limit is named exactly once, and
    each over-limit record agrees with a reading above the limit (same
    place and load within the tolerance).
16. **The guide set has a total order** (finding 6): index, family, open
    before closed, the canonical point sequence, then the circle. A closed
    curve starts at its lowest point (the least sequence among equal lowest
    points); an open curve runs from its lesser end. A curve of zero length
    and a closed curve that repeats its first point are refused.
17. **The tests of the invariants use the public API**, in the directory
    suite `crates/pathkernel/tests/laws.rs` (one submodule for each
    stage). The record puts the K7 law suite in that directory, so K7
    joins it. The engine parse keeps its unit tests.

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
