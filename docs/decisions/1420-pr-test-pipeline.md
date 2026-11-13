## #1420 - PR test pipeline: the review's build and tests on the self-hosted Linux runner

Settled by the maintainer (2026-11-13, on the issue: "then that is the
decision, workflow_dispatch"): `pr-test.yml` has a `workflow_dispatch`
trigger only (inputs: the PR number), runs on `[self-hosted, linux]`, checks
out `refs/pull/<N>/head` itself, trial-merges it with main, uses a read-only
token and no secrets, and has a `concurrency` group per PR. The PR-reviewer
starts it after reading the diff; "read, not built" PRs are never sent; no
runner online, or a run that is cancelled or times out, falls back to the
local build or "not built", never a pass. The points below were open and
are decided here.

**The run is pinned to the head the reviewer read.** Decided: a second
required input `sha`; the workflow refuses unless `refs/pull/<N>/head` is
exactly that commit, and the run is named `pr-test #<N> <sha>` so the
reviewer finds its own run with `gh run list`. Why: `gh workflow run`
returns no run id, and a push between the read and the run would otherwise
test code nobody read. Dropped: looking the run up by creation time (races
with a second review) and testing whatever the head is at run time.

**Detection and dispatch sit in the PR-reviewer agent, not the skill.**
Decided: the agent checks the runner, starts the run after its read, waits
and reads the result; `review-prs` only says so. Why: the decision needs
the diff (the "read, not built" rule), which only the agent reads; the
dispatcher would have to read it twice. Dropped: the skill starting the run
and handing the agent a run id.

**Runner online, and nothing else.** Decided: `gh api
repos/NeoTech/Camcraft/actions/runners` lists a runner with `status ==
online` whose labels include `self-hosted` and `linux` (case ignored); busy
counts as online (the run queues). The endpoint needs admin access: a
collaborator whose token cannot read it (403) is treated as "none online"
and builds locally. A `cut.yml` run queued or in progress (issue 1413) also
means local, so a review never delays a cut's release gate. Dropped: a
repository variable that switches the runner on (one more thing to forget).

**Defence in depth in the workflow.** Decided: the workflow itself refuses,
before any build, a diff that touches `build.rs`, `.github/`, `scripts/`,
`.cargo/`, a `Cargo.toml` or `Cargo.lock`, or a `rust-toolchain` file, and a
trial merge that conflicts. It is stricter than the reviewer's rule (any
`Cargo.toml` change, not only a new dependency): the cost is a local build
for a manifest-only change, the gain is that a mistaken dispatch still runs
no build script or toolchain switch a human has not read. Dropped: trusting
the dispatcher alone.

**Readings that only hold on Windows.** Decided: the Linux run does not
check the Windows arm of per-OS pinned values (issue 1402) or anything under
`cfg(windows)`. When the diff adds or changes a `cfg(windows)`,
`cfg(target_os = ...)`, `cfg!(windows)` line or a per-OS pinned reading,
the reviewer also runs the touched suites locally (narrow: those binaries
only) and the review says both. Why: the maintainer's machine and most
users are on Windows; a green Linux run of a Windows-only branch says
nothing. Dropped: a Windows runner (hosted bills 2x; a second self-hosted
machine is not there) and ignoring those readings.

**Concurrency across PRs on one runner.** Decided: `concurrency:
pr-test-<N>`, `cancel-in-progress: false` (a second start for the same PR
waits; a cancelled run would be no verdict). Runs of different PRs queue on
the runner by themselves, one job at a time. The reviewer waits at most 30
minutes for a queued run to start; past that it cancels its run and builds
locally. A started run has a 90-minute job timeout. Dropped: one global
group (a cut-time `cut` group exists for that; PR runs need no order
between them) and cancel-in-progress (it would turn a slow review into a
cancelled one, which is not a pass).

**UI suites on headless Linux.** Decided: they run on the runner, inside
`xvfb-run -a` when no display is set and the runner has `xvfb`, with Mesa's
software drivers for a GPU-less adapter (the same requirement as the cut
runner's, issue 1413). The pixel-snapshot lanes are `#[ignore]`d out of
quick and batch anyway (release lane, #255). A failure in a ui suite that
looks like the environment (no display, no adapter) is re-run locally for
that suite before it is called a blocker; any other failure is a finding.
Dropped: keeping every ui suite local (it would keep a local build for most
UI PRs, the case this issue is for).

**Clippy is a separate step on the touched crates.** Decided: quick has no
clippy step (batch and release do), so the workflow runs `cargo clippy -p
<touched crates> --all-targets -- -D warnings`, the crates read from the
merged diff (`crates/<dir>/Cargo.toml`'s name, plus `xtask`). Shell, no
xtask helper: the mapping is three lines and needs no unit test of its own.
Dropped: an `xtask` flag that adds clippy to quick (changes the local quick
tier everyone runs).

**No local build when the runner ran.** Decided: on the runner path the
reviewer reads the PR with `git fetch origin pull/<N>/head` and `git show`
and makes no worktree and no `target/`. Why: the issue's point is the 10-20
GB per review. The cost to change: none - the local path is unchanged and
remains the fallback.
