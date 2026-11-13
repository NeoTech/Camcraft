# 1451 - post-merge batch gate on the self-hosted runner

- **Yield rule: skip when a cut.yml run is queued/running or a same-repo release/v* PR is open.**
  Why: simplest rule that cannot starve a cut on a one-runner pool; the release
  gate of the cut re-tests main anyway. Dropped: job priorities (GitHub has
  none for self-hosted queues), a second runner label (more setup), waiting
  in-job (would hold the runner).
  Cost: merges landing during a cut are not post-merge tested; the cut gate covers them.
- **Reporting: one rolling issue, P2 by default, P1 only when a failing suite
  is a wrong-output suite.** AGENTS.md "Priorities" (P1..P9, P0 retired) puts
  "a gate on main is red" at P2 and wrong posted output at P1; AGENTS.md
  governs over the issue's wording, which asked for P1/P0. P0 is never used.
  The wrong-output suites are an explicit, anchored list of whole test-binary
  names in the script (`OUTPUT_SUITES`: golden_cl, solid_law_goldens,
  recorded_post_streams, recorded_streams, the `*_post` NC golden suites,
  post's `script_<dialect>` suites, ...), read from the FAIL rows' "failed:
  tests/<name>.rs" notes. Dropped: a substring match on golden|gcode|post
  (caught the whole post crate step and words like "compost"). An open P2
  issue is raised to P1 by a later output-suite red, never lowered by the
  script. Closed by the next green run; cancelled/skipped runs say nothing.
- **Only `source:agent` issues are found, commented on and closed**, so a
  green run never closes a human-filed issue with the same title.
- **Concurrency group per ref:** a dispatch of a non-main ref runs in its own
  group and never cancels main's gate.
- **Reporting logic lives in `.github/scripts/*.sh`**, testable with a fake gh,
  not inline YAML.
- **Token scope:** gate job read-only (`persist-credentials: false`, token only in the
  yield step); only the report job has `issues: write`.
- **PROCESS_VERSION 61** (upstream main was 60; #1418 also bumps - settled at merge).
