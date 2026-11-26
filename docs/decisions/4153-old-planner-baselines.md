## #4153 - The old-planner baseline of each phase-1 kind

**What was decided.** The baseline pins of the 15 phase-1 kinds that had
none are golden files of the old planners on `side/path-kernel`:

- 14 kinds at the motion level, in one submodule tree
  `crates/motion/tests/pass_start_levels/kernel_baseline/` (one file for
  each kind). The tree joins the `pass_start_levels` directory suite with
  one `mod` line at the end of `pass_start_levels.rs`. The goldens are
  `crates/motion/tests/golden/kernel_<kind>_<case>.cl`.
- The 2D Pocket at the app level, on `fig8.step`, in
  `crates/app/tests/pocket_pass_start/kernel_baseline.rs` (a submodule of
  `pocket_pass_start`). Five generations run past the 60 s debug budget,
  so the full pin is in the heavy lane with a light twin of one case.

**Why.**

- The motion level calls the planner that the kernel replaces, with no
  ECS between them. The comparison of the switch-over proof is then
  exact and fast (all 15 motion pins run in about 40 s in debug).
- New files with new names, and a `mod` line far from the lines that PR
  #4123 changes, let the two PRs merge in either order.
- The text form is the #3750 form: the CL text at 1 um, one `; note:`
  line for each note. The app pin adds one `; warning:` line for each
  verifier warning. The motion pins have no verifier warnings, because no
  verifier runs at that level (as in the #3750 motion pins).

**Considered and dropped.**

- A new top-level test binary for the baseline: refused by **New tests
  join a directory suite**.
- The fixtures of the 60 x 40 Adaptive pocket: the spiral Adaptive took
  51-56 s in debug on a loaded host. The pins use a 40 x 30 pocket with an
  8 x 8 island.
- A curved chain for the trochoidal slot: the old planner refuses a
  polyline bend and an arc of radius 30 at a width of 12 with "the centre
  line turns too tightly for loops this wide". The baseline records only
  straight chains.
- The app level for each kind: the fig8 generations are slow, and the
  motion level records the same planner output.

**Cost to change later.** Re-record with `WRITE_GOLDEN=1` (commands in
the file headers). The files are additions only and can be deleted after
the switch-over.
