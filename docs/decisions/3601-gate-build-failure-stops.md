## #3601 - A failed test build stops the quick gate and names the metadata error

The quick step `ui build (--no-run)` failed at 16 jobs with rustc metadata
errors (E0786, E0460, E0462). The crate in the error changed from run to
run. 4 of 4 runs failed at 16 jobs, and 4 of 4 passed at 4 jobs. The cause
is not proven. This PR is a partial fix (`Refs #3601`). The issue stays
open for the cause. The decisions were taken unattended, under the
maintainer's standing word "choose recommended".

### 1. What a failed per-crate build does - chosen: stop the plan

- **Decided.** In a plan that runs one step at a time (quick), a failed
  step whose name ends in ` build (--no-run)` stops the plan. This is the
  rule that `debug build (--no-run)` had before. The summary lists each
  later step as `SKIP ... not run (build failed: <step>)`. The gate exits
  red, and the FAIL row names the build step.
- **Why.** Each later step needs the binaries of that build. In the
  reported runs the 8 ui suites ran after the failed build and failed with
  E0463. That hid the one real failure among 9 red rows.
- **Rerun after a stop (review of PR #3708).** Each stopped step gets a
  log with the one line `not run (build failed: <step>)`. `--failed`
  reads that line as a failed step (`select::log_failed`). So
  `--failed gate-logs/<run>` runs the build step and each stopped step,
  not the build step alone. The summary row of a stopped step is `SKIP`,
  not `FAIL`, so the post-merge report and the light-lane note do not
  count it as a test failure.
- **Dropped.** Stop only the steps of the same crate: the steps of other
  crates use the same target directory, and a bad build output there can
  give the same error. A plain stop is simpler and agrees with the
  whole-gate build.

### 2. The metadata error - chosen: one diagnostic block, no automatic action

- **Decided.** When the log of a failed build step (its arguments have
  `--no-run`) has
  `found invalid metadata files for crate`, `E0786`,
  `found possibly newer version of crate` or `E0460`, the step prints one
  `BUILD FAULT` block. The block names the crate of the error and says that
  the build output is bad, not the tests. It gives two commands to try,
  one at a time:
  1. the same gate with `CAMCRAFT_GATE_JOBS=4`;
  2. `cargo clean -p <package>`, where the package is the `-p` value of the
     step (`camcraft-ui` for the ui step).
- **Why the package of the step.** The crate in the error can be a crate
  that is not a package of the workspace graph (the sysroot crate `test`),
  so `cargo clean -p test` is not valid.
- **Dropped.** An automatic retry and an automatic clean. The cause is not
  known, and a retry can hide a real file race. The job budget rule
  ("Gate job budget", #2814) does not change. A `-j` cap for the
  `--no-run` steps is a possible later fix, after a reproduction.
- **Dropped.** E0462 (`found staticlib ... instead of rlib`) in the trigger
  list: the brief named the two error codes above. E0462 came in the same
  runs together with E0786 or E0460, so the block still shows.

### 3. The seed lead - read only, no fix

`xtask/src/seed.rs` copies the units of registry and git packages from
another checkout into a new worktree, with the modification times of the
source (`copy_tree`). Can this give E0460? The reading says: not likely as
the cause of #3601.

- The seed runs one time for each worktree (the marker
  `target/debug/.camcraft-seeded`), before the first step, with the cargo
  locks of the two target directories held. It cannot run during a build.
- Each unit is copied under a temporary name and then renamed. A unit is
  complete or absent.
- It never copies a local package (`camcraft_app`, `camcraft_ui`), and it
  cannot touch the sysroot crate `test`. One reported error was E0786 for
  `test`.
- The seed does not depend on the job count. The failures follow the job
  count (16 fails, 4 passes).
- One residual path: the seed replaces a unit that is already in the
  worktree with the copy of the same hash. If a registry crate does not
  build reproducibly, a crate that was built against the replaced file can
  then see a different SVH (E0460). This can only occur on the first gate
  after a seed (or after a full `cargo clean`, which removes the marker).
  To exclude it on the reporter's machine: look for the marker and for the
  line `target seed: seeded ...` in the failing log, then run with
  `CAMCRAFT_NO_SEED=1`.
