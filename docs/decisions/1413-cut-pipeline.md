## #1413 - Cut pipeline: the release gate on a self-hosted Linux runner, then tag and start release.yml

Settled by the maintainer (2026-11-13): Linux only, self-hosted only
(`runs-on: [self-hosted, linux]`), `workflow_dispatch` only, and the new
pipeline *triggers* `release.yml`, it does not replace it. The rest was
decided unattended:

**Where the tag is made.** Two manual starts of `cut.yml` per cut. `stage:
gate` runs on `release/vX.Y.0` before the release PR merges, so a red gate
never leaves an untagged release commit on main; green, it records the
tested commit (`cut-tip-vX.Y.0` artifact). `stage: tag`, after the merge,
takes that tip and tags main's head `m` only when `m^2 == tip` (the merge is
the release PR's, nothing else merged) and `git diff $tip $m` is empty -
AGENTS.md step 4's rule, kept and now checked by the machine. Why: gating
main after the merge would put a red release on main; one run that waits
for the PR merge would hold the runner for hours. Dropped: a tip typed in
by hand (error-prone), `workflow_call` from a merged-PR event (a
`pull_request` trigger, ruled out). Cost to change: the two stages are
separate jobs, either can move.

**How release.yml starts.** `gh workflow run release.yml --ref vX.Y.0` with
the workflow token: a tag pushed with `GITHUB_TOKEN` starts no workflow, a
`workflow_dispatch` from it does, and on a tag ref `release.yml`'s
`publish`/`pages` jobs see `refs/tags/...` exactly as on a pushed tag.
`release.yml` already has `workflow_dispatch`, so it is unchanged - not
even a trigger is added. Dropped: `workflow_call` (the called workflow sees
the caller's `github.ref`, a branch, so `publish` would skip), a PAT (a
long-lived secret for no gain).

**Windows-only pinned readings** (issue 1402's per-OS move counts). The
Linux gate checks the Linux arm only. A local Windows check of those pins is
optional at a cut and named in AGENTS.md step 2; left unchecked, a Windows
arm drifts until someone runs it on Windows, and the release PR says it was
skipped. Why: a Windows runner (hosted, 2x billed, or a second self-hosted
box) is outside the maintainer's "Linux only" and the drift is a test
reading, not shipped output - Linux and macOS arms still catch a real
planner change.

**UI snapshot lane on Linux.** Skipped: the gate runs `cargo xtask gate
release --no-snapshots` (new xtask flag, pinned by
`release_without_snapshots_drops_only_the_pixel_lanes`), and the two pixel
lanes stay a local Windows step at the cut, where the goldens are made and
regenerated after a look at the `.diff.png`. Why: the goldens are Windows
renders; a GPU-less Mesa render on Linux differs in pixels, so the lane
would be red every cut or need a second set of goldens to keep in step.
Their structural twins (AccessKit labels, text strips) run in the ui suites
on Linux. Dropped: Linux goldens (two sets to regenerate, reviewed twice).

**Version bump, zip, installer.** The bump stays local, on the release
branch, with `cargo xtask release --version X.Y.0 --no-tag --skip-gate`
(it is a commit the release PR carries). The published zip, installer and
packages were already built by `release.yml` from the tag; the local zip
and installer are now a hand-test build only. The installer stays
Windows-only in `release.yml`'s Windows job. Nothing moves to the cut
runner but the gate and the tag.

**dry_run input.** Added: the gate prints its plan only, the tag stage runs
every check and stops before the tag and the dispatch, so the maintainer can
try the runner safely after merge.

**Runner safety.** The gate job checks out with `persist-credentials:
false`, so the test code never sees a token; only the tag job has
`contents: write` / `actions: write`. AGENTS.md says how the runner is
registered and what keeps fork code off it (no fork-triggerable trigger,
approval for outside collaborators' workflow runs, never approving a fork
PR that touches `.github/workflows/`).
