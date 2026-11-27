# #4101: a build step gets the longer stall time

The v0.27.0 cut gate stopped at its step [01] "debug build (--no-run)".
Three cold `rust-lld` links of the `camcraft-ui` test binaries wrote no
output for 15 minutes, and the stall watchdog killed the step. No test ran.

## Decision

- **Chosen: option 1 of the issue.** `watchdog::limit_for` gives a build
  step the same longer limit (`SLOW_FACTOR`, 4x) that a `--release` or
  `--ignored` step already gets. A build step is a `cargo test` step with
  `--no-run`, or a `cargo build` step. With the default of 15 minutes, a
  build step can be silent for 60 minutes. The plan line now says
  "4x for --release, --ignored and build steps".
- **Why:** a build step is a build, not a test. A link is silent by its
  nature, and a cold link of a large test binary on a loaded runner can
  take longer than a debug test step. A real hang of a build is rare, and
  the 60-minute limit still ends it, below the 75-minute stop of the batch
  step.
- **Only the subcommand counts.** A test binary that is named `build`
  (`--test build`) is not a build step.
- **Clippy keeps the base time.** It checks and does not link, and it
  writes a line for each crate.
- **Dropped: option 2** (count the CPU time of the child processes as
  progress). It is a better signal, but it needs a CPU-time reader for each
  platform (`ps` and `Win32_Process`) and its own tests. It can come later
  if a 60-minute silent link occurs.
- **Dropped: option 3** (`CAMCRAFT_GATE_STALL_MIN` in `cut.yml`). It is a
  workaround, and it also makes each test step wait longer before it is
  stopped.

## Not proven here

The "Done means" of the issue asks for a cold release-tier gate on the cut
runner that passes step [01]. Only a cut runner can show that. The unit
test `build_steps_get_the_longer_stall_time` pins the limit.
