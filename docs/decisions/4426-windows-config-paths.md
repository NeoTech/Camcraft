## #4426 - Build expected config paths with native separators

- **Decided:** build the hierarchical config test's expected paths with one
  `join` per component. Keep its exact path-count and key/privacy assertions.
- **Why:** on Windows, `join(".cargo/config.toml")` retains `/`, while the
  diagnostic builds `.cargo` and the file name separately and prints `\`.
  The existing test fails even though it reports the correct files.
  This follows the same test correction as #4120.
- **Before reading:** the unchanged main at
  `4d891498e7270eabba1b82809d08228b465c0aec` fails on Windows with
  `cargo test -j 4 -p xtask unit_env::config::tests::hierarchical_configs_report_paths_and_key_hints_not_effective_values -- --exact --nocapture`.
  The assertion at `config.rs:98` finds zero matching paths instead of one.
- **Dropped:** changing runtime diagnostics or normalizing their text in the
  assertion. Neither is needed, and normalization could hide a wrong path.
  The independent failures in #4427 and #4428 remain outside this issue.
- **Cost to change later:** this affects only three expected test paths.
  Runtime behavior, dependencies and the test inventory stay unchanged.
