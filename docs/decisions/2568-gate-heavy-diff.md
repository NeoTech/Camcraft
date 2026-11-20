## #2568 - `cargo xtask gate heavy`: the heavy lanes a diff reaches

**What was decided**

- A new gate tier, `cargo xtask gate heavy [--base main]`, maps the diff to
  the release `--ignored` heavy lanes that it reaches and runs only those
  (`xtask/src/heavy.rs`). The steps are the release tier's own heavy steps,
  with the same names and commands, so `--only`, `--failed` and the
  known-fail checks work as in a cut.
- The reach is by crate, through the dependencies between our crates. They
  are read from `crates/*/Cargo.toml` (normal, dev and build dependencies).
  A source change in `motion` reaches every heavy target of `motion` and of
  `app`. A change in `kernel` reaches each heavy lane.
- Narrower cases: a changed suite file reaches its own heavy binary only.
  Shared test code of a crate reaches that crate only. A post script,
  machine file, catalog case or fixture reaches the binaries that read it
  (the quick tier's own mappers). A workspace manifest, `Cargo.lock`,
  `.cargo/` or toolchain file reaches each heavy lane.
- An unreadable crate manifest is an error, not a smaller plan.
- Process rule **Heavy lanes at review** (AGENTS.md "Testing"): the author
  runs the command on the final head before the PR is flagged ready, and the
  review comment lists the lanes that ran. "Not run" is never a pass.
- The reviewer checks heavy reach after either the runner path or local
  tests. A missing, stale or incomplete author result requires a local
  heavy run or a blocker.

**Why**

- The v0.24.0 cut gate was red on a heavy test that no earlier gate ran
  (#2568). The quick tier selects by Gate map and skips `--ignored` tests.
- A Gate map names the modules that a suite covers directly. It does not
  follow a change in a lower crate into the suites of a higher crate. That
  is the case that the issue names, so the heavy reach does not use the
  Gate maps for source changes. A crate-level reach can run too much. It
  cannot miss a lane.
- The runner runs quick and clippy only. A common heavy check prevents its
  result from bypassing the heavy lanes.

**Considered and dropped**

- Narrowing a source change by Gate map (as quick does): dropped, because
  it misses the cross-crate case.
- A flag on the quick tier (`gate quick --heavy` already runs changed
  `#[ignore]`d test fns): kept as it is. It does not reach unchanged heavy
  tests.
- `--diff` as a flag (the issue's example): dropped. The tier always works
  from the diff, with `--base` as in quick.
- The shared lanes of the release executor (parallel queue): not used. The
  heavy tier runs its steps one at a time. A later change can turn them on.
- `pr-test.yml` running the heavy tier: not in this PR. A workflow change
  needs the maintainer (`needs-maintainer`), and that workflow refuses a PR
  that changes `xtask/`. The wrappers `scripts/gate.sh` and
  `scripts/gate.ps1` do not forward `heavy` yet; `cargo xtask gate heavy`
  is the command.

- Keeping the heavy check only in the local test step: dropped, because
  the runner path skips that step.

**Cost**

- A change in `motion`, `geometry` or `kernel` source runs most heavy lanes
  in release before the PR is ready. This is a cost on purpose (maintainer,
  2026-11-18: "yes file all 3 changes and move them to P4"). The rule
  "never hours per change" still holds for each edit: the heavy tier runs
  one time, on the final head.
- To change the reach later, edit `heavy::reached`. The pins are in
  `xtask/src/heavy_plan_tests.rs`.
