## #2727 - The batch gate builds each crate alone and runs its suites in parallel lanes

**What was decided.**

- `plan::batch` starts with one `--no-run` build step for each crate (`debug build: <crate> (--no-run)`),
  not one build step for all the crates.
- `Run` has a `lanes` flag. It is on for the `batch` and `release` labels. The quick gate sets it when it
  falls back to the batch plan. With the flag on, `Run::plan` uses the lane queue of `queue.rs`
  (`run_lanes`, before `run_release`).
- The batch queue takes its order from the newest `timings.tsv` of a `batch-*` or `release-*` run. The
  release queue reads `release-*` runs only, as before.
- The `heavy` tier of #2779 and a quick plan stay one step at a time.

**Why.**

- The batch tier ran about 900 steps one after the other. On a 16-core machine the load was below 3
  (#2727).
- `crates/kernel` has a dev-dependency on itself with the `truck` feature. One `cargo test --no-run` call
  with all the crates gives the shared dependencies the features of all of them. A later step with one
  `-p <crate>` resolves a smaller feature set, so cargo compiles that crate and its dependents again. One
  build call for each crate uses the same package selection as the steps of that crate. The steps then find
  their artifacts ready.
- The lane queue is the code that the release tier uses since #2057 and #2461. It keeps the guarantees of
  the gate: each step runs and has its own log, a red step fails the gate, a failed build stops the run,
  and the checkout is read before each step (VOID).
- The rules of `lanes::alone` apply without a change: the builds, the `ISOLATED` targets (wall-clock
  asserts), each ui step and clippy run alone with the whole job budget.

**Considered and dropped.**

- One parallel prebuild for all the crates (the first proposal of the issue). It is the build that batch
  had. Its feature set is not the feature set of the steps, so the steps compile again.
- One prebuild call with `--workspace` feature flags that copy each step. Cargo has no flag that selects
  the per-package feature resolution in a build of many packages.
- The crate prebuilds in parallel lanes. Two cargo calls on one `target/` wait for the same build lock, so
  they do not overlap. Each build call already uses the whole job budget (`-j`).
- `cargo nextest` (measured on the issue: 18.8 min cold). It is a new tool on each runner, it does not run
  doc-tests, and the known-fail and VOID records of the gate read libtest output. That is a larger change.
- Lanes for each quick plan. A quick plan is short, and its build steps are for each crate already. Only
  the fallback to the batch plan uses the lanes.
- The same split of the debug build of the release tier. The release plan still has one debug build for
  all the crates. This PR does not change the cut gate.

**Cost to change later.** Small. `Run::lanes` is one flag, and `plan::debug_build_crate` is one function.
To go back, give `plan::batch` the one `debug_build` step again and set the flag for `release` only.
