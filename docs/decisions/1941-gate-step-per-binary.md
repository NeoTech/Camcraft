## #1941 - Gate: one step for each test binary in quick, batch and release

**What was decided.**
- The debug suites of the three tiers use the planner pattern of the heavy lanes (#1714). One build step (`cargo test --no-run`) comes first. Then each test target has its own step: `lib`, `bins`, each binary, `doc`. The release tier keeps all four kinds. Batch keeps `lib` and the binaries, and leaves out the `SLOW_BINARIES` (as before). The ui crate keeps its CWD `crates/ui`.
- A step runs `cargo test -p <crate> --test <binary>`. It does not run the exe directly. This is the mechanism of the heavy lanes. It needs no new step kind, and `--only`, `--failed`, log names and the `read_log` note work without change.
- Quick builds one time for each crate (`<crate> build (--no-run)`), but only when the crate has more than one run step.
- Parallelism: quick and batch stay sequential, as now. Each step already uses `--test-threads=6`. The release tier keeps its waves (`lanes.rs`, four at most); the app, kernel and ui steps still run alone. No new limit.
- A binary without `#[ignore]` still gets a debug step. Only the heavy lanes skip such binaries.

**Why.** The maintainer asked for it (#1941): the per-binary steps of the last cut gate named a failure at once and ran efficiently.

**Cost.** Each step starts cargo: about 0.5 s on Windows (7 geometry steps: 5.0 s in all, against 1.5 s for the one grouped call). The batch plan grows from 10 to 713 steps and the release plan from 189 to 911. The extra start time is about 6 min serial in batch on Windows. This has not been measured on the Linux runner.

**Dropped.** Running the test exes straight from the cargo JSON artifacts. It would remove the cargo start, but it needs a second step mechanism. Follow-up issue #1951 (P7).

**Change later.** Only `plan.rs`, `quick.rs` and their tests change.
