## #2908 - Batch tier lanes for separate runners: the `xtask` part

Decided unattended by the issue worker, 2026-11-20.

**What was decided**

- **A flag, not named groups.** `cargo xtask gate batch --lane N/M` selects
  lane N of M. The workflow chooses M. A named group for each crate does not
  balance: the app crate has most of the 973 steps.
- **A round-robin deal in plan order.** The steps that share the machine are
  dealt first, then the steps that run alone (`lanes::ISOLATED`, ui, clippy).
  Each lane gets an equal share of the two kinds (325, 324 and 324 steps for
  three lanes). Clippy is a step of the deal, so it is in exactly one lane.
- **No timing file in the split.** The longest-first order of `lanes.rs` uses
  `timings.tsv`, which is different on each runner. A split from it could put
  one test binary in two lanes, or in none. The split reads only the plan.
  The order inside a lane still uses the timings of that runner.
- **Each lane builds for itself.** A lane keeps the `debug build: <crate>`
  step of each crate that it tests. The runners share nothing but the commit.
  The compile cache (`sccache`) of each runner pays for the repeated builds.
- **A lane record, and one collector.** A lane run writes `lane-result.tsv`
  into its `gate-logs/batch-<stamp>/`. `cargo xtask gate-verdict <dir>...`
  reads the records of all lanes and prints one verdict. The verdict names
  each lane, its runner, its time, its result and each red step.
- **A lane that did not report is red.** The verdict is red for a lane with
  no record, two records, a void run, fewer steps than its plan, or a record
  that cannot be read. An empty lane is an error at the start.
- **Partial work, `Refs #2908`.** This PR has the `xtask` part and the `gate`
  skill text. The issue stays open for the workflow.

**Why**

The worker's brief does not permit a change in `.github/`. The open PR #2606
changes the `report` job of `.github/workflows/post-merge.yml` and has the
label `needs-maintainer`, and the brief does not permit a stack on such a PR.
The `xtask` part does not depend on the workflow, and the workflow cannot be
written without it.

**What was considered and dropped**

- **The workflow change** (`post-merge.yml`: a lane matrix, one artifact for
  each lane, a collector job in place of the `report` job of #1611). It waits
  for the maintainer's decision on #2606. A proposal is in a comment on #2908.
- **One build job that uploads the test binaries.** The debug `target/` is
  10-20 GB. An upload and a download for each lane costs more than a build
  with a warm compile cache. The absolute paths in the binaries also differ
  between runners.
- **A split balanced by measured time.** It is better balanced, but it is not
  the same on each machine (see above). A committed timing table could do it
  later.
- **The `runners` skill text.** It describes the workflow, which does not
  change in this PR.
- **The macOS runner `m3`** and **the treatment of a push during a run**.
  These are decisions about the workflow and its runner labels.

**What it costs to change later**

The split is one function (`lane_split::assign`). A different deal changes
which lane runs a step, not which steps run: the test
`each_batch_step_runs_in_exactly_one_lane` holds for any deal. The record
format is read only by `gate-verdict`.
