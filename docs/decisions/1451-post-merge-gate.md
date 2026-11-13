# 1451 - post-merge batch gate on the self-hosted runner

- **Yield rule: skip when a cut.yml run is queued/running or a same-repo release/v* PR is open.**
  Why: simplest rule that cannot starve a cut on a one-runner pool; the release
  gate of the cut re-tests main anyway. Dropped: job priorities (GitHub has
  none for self-hosted queues), a second runner label (more setup), waiting
  in-job (would hold the runner).
  Cost: merges landing during a cut are not post-merge tested; the cut gate covers them.
- **Reporting: one rolling issue, P1 by default, P0 when a failing step name
  matches golden|gcode|g_code|post.** Read from the gate summary's FAIL rows. A
  heuristic on binary names; a follow-up can map suites to a real "posted
  output" tag. Closed by the next green run; cancelled/skipped runs say nothing.
- **Reporting logic lives in `.github/scripts/*.sh`**, testable with a fake gh,
  not inline YAML.
- **Token scope:** gate job read-only (`persist-credentials: false`, token only in the
  yield step); only the report job has `issues: write`.
- **PROCESS_VERSION 61** (upstream main was 60; #1418 also bumps - settled at merge).
