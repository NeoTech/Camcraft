## #3303 - Gate stall watchdog and the runner-stall report

**What was decided.**

- `cargo xtask gate` watches each step (`xtask/src/watchdog.rs`). Output is
  the growth of the step log plus the bytes of a piped stdout. The child and
  each descendant (cargo, rustc, the test executables) write to the same log
  handle, so the output of a descendant counts.
- A debug step with no output for 15 minutes is a stall.
  `CAMCRAFT_GATE_STALL_MIN` sets the time in whole minutes, with the parse
  rules of `CAMCRAFT_GATE_JOBS` (a bad value uses the default and prints a
  diagnostic).
- A `--release` or `--ignored` step gets 4 times the time (60 minutes by
  default).
- At a stall the gate reads the process tree (`ps` on Unix,
  `Win32_Process` on Windows). It kills the tree (`kill -KILL` on each pid,
  `taskkill /F /T` on Windows). It prints `STALL <minutes>min <step>`, the
  last 20 lines of the step and the processes that were alive. It records a
  `STALL` row and goes on with the other steps.
- The exit code is 3 for `STALL` rows only. A `FAIL` row or a void run
  gives 1, as before.
- `post-merge.yml`: the step "Batch gate tier" has `timeout-minutes: 45`.
  The job cap of 90 stays. "Gate summary" appends `gate step TIMEOUT after
  <n> min; last started step: <line>` when the step failed, has no summary
  and ran 44 minutes or more. The gate job exports `runner.name`.
- `post-merge-report.sh` has three red cases:
  - `FAIL` rows, or a `FAIL in` progress line of a step that ended before
    the gate stopped: a red main, as before, also with `STALL` rows or the
    timeout line. Without a summary the issue lists the `FAIL in` lines.
  - No `FAIL` row, but a `STALL` row or the timeout line: a runner stall.
    The script writes the warning `runner stall: <step> on <runner>` and
    exits 0. It opens and comments on no issue.
  - No `FAIL` row, no `STALL` row, no timeout line (for example xtask did
    not build): a red main with the generic text, as before.

**Why.**

- Four of the stops in #3303 were hangs of 51 to 53 minutes in "Batch gate
  tier" that a person had to cancel. They had no FAIL row. A stop that ends
  by itself and names the step gives the runner owners a place to look.
- 15 minutes: a normal batch run takes about 21 minutes in total. AGENTS.md
  moves a debug test that runs longer than about 60 s to the heavy lane, and
  libtest prints a line when each test ends and a "running for over 60
  seconds" line. A debug step that is silent for 15 minutes is not a normal
  step.
- 4 times for release and `--ignored` steps: one heavy test can run for many
  minutes and libtest prints only one line at 60 s. A release link with LTO
  is quiet too. Per-test progress output from libtest is not stable (the
  `--report-time` and JSON formats need nightly), so a longer time is the
  safe choice. The cut can raise it with the variable.
- Kill the tree, not only the child: a killed `cargo` leaves its test
  executable alive, and the lane then stays held. The tree is read at the
  stall, by parent pid. A daemon that left the tree (the sccache server) is
  not killed.
- The kill walks parent pids. Between the read of the process list and the
  kill there is a small pid-reuse window. A process that left the tree (a
  double fork, `setsid`) lives on, and in the capture path of
  `direct::cargo` such a process can keep the stdout pipe open. A process
  group (Unix) or a job object (Windows) closes these gaps: that is the
  follow-up #3493.
- STALL is decided from the watchdog's result (`End::Stalled`), never from
  the log text, so a test that prints the watchdog's line cannot change a
  FAIL row to a STALL row.
- A `FAIL in` line counts without a summary (review of #3489): the gate
  prints it when each step ends, and the summary comes only at the end. A
  hang after a red step must not hide the red step. The report script uses
  no `grep -q` in a pipe: under `pipefail` its early exit gives the writer
  SIGPIPE on a large summary, and a match then reads as a miss.
- The report opens nothing on a stall. Reasons:
  - A stall is a fault of the machine, not of the commit. A "main is red"
    issue would send a worker to code that is correct.
  - A comment on #3303 needs a fixed issue number in the script, and #3303
    closes with this PR.
  - The token of the report job does not change (`issues: write` only); a
    re-run would need `actions: write`.
  - The coordinator logs the stall on #3303 and re-runs the run one time
    (AGENTS.md **Runner stops**).
- The timeout line needs the elapsed time: GitHub gives a timed-out step
  the outcome `failure`, the same as any other red step. 44 minutes is one
  minute below the step timeout.

**Considered and dropped.**

- A stall time that is the same for all steps (for example 60 minutes):
  then the 45-minute step timeout always comes first in the batch, and the
  step name is lost.
- A step timeout only: it names no step and kills the whole gate.
- A re-run by the report job: it needs `actions: write` and a runner label
  for "another machine", which does not exist.
- A comment on #3303 from the script: see above.
- `CAMCRAFT_GATE_STALL_MIN` in seconds, for the tests: the unit tests pass
  the time as a parameter instead. `set_var` is unsafe in edition 2024 and
  races with the tests that run in parallel.

**Cost to change later.** Low. The default and the factor are two
constants in `xtask/src/watchdog.rs`. The timeout is one line of
`post-merge.yml`, and the 44-minute check of "Gate summary" follows it.
