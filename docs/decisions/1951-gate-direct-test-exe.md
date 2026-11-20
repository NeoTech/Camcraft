## #1951 - A batch suite step starts its test executable from the build's cargo JSON artifacts

**What was decided.**

- Each batch prebuild (`debug build: <crate> (--no-run)`) has the flag
  `--message-format=json-render-diagnostics`. The executor keeps the JSON messages of the build and makes
  one `cargo metadata --no-deps` call. From the two it records the test executables of the crate
  (`xtask/src/artifacts.rs`).
- A suite step of that crate (`--lib`, `--bins` or `--test <name>`) then starts the recorded executable
  and does not start cargo (`xtask/src/direct.rs`). The step name, the step number, the log file name and
  the plan are the same. `--dry-run` prints the same cargo commands.
- The executable starts as `cargo test` starts it: in the package directory, with the libtest args after
  `--`, and with the variables that cargo sets (`CARGO`, `CARGO_MANIFEST_DIR`, `CARGO_MANIFEST_PATH`, the
  `CARGO_PKG_*` set, `CARGO_BIN_EXE_<bin>`, `OUT_DIR`, the `rustc-env` pairs of the build script and the
  dynamic library path). The list was read from a test process under cargo 1.100.
- The gate writes cargo's `Running <target> (<path>)` line into the step log before the output of the
  executable. `read_log`, the notes and `--failed` read the log as before.
- A step stays a cargo step when one of these is true: its package has no recorded build in this run
  (`--only` or `--failed` selected the step without its build step); its target has no executable in the
  list; it is a `--doc` step; its args are not exactly the known form. The printed command shows which
  way a step ran.
- The executable path comes only from the JSON list. The gate never searches `target/` for one.
- `CAMCRAFT_GATE_NO_DIRECT=1` makes each step a cargo step again. It is for a comparison run, and for a
  platform where a direct run does not behave as `cargo test` does. This PR ran on macOS only.

**Why.**

- Each suite step started cargo to run one binary that was built already. The issue gives about 0.5 s for
  one start on Windows, and the batch plan has several hundred suite steps.
- This PR is stacked on #2788. That PR builds each crate alone, so the executables of the build are the
  same ones that `cargo test -p <crate>` runs. With one build for all the crates the features are
  unified, and the executables are different ones.
- cargo 1.100 puts a test executable in `target/debug/build/<pkg>/<hash>/out/`, not in
  `target/debug/deps/`. A path that the gate computes itself is wrong on one of the two layouts. The JSON
  list is correct on the two.
- When the gate runs under `cargo xtask`, its own process has the `CARGO_MANIFEST_DIR` and `CARGO_PKG_*`
  values of `xtask`. A direct run must set the values of the tested package. If not, a test that reads
  one of them at run time gets the wrong value.

**Considered and dropped.**

- The release tier. Its debug build is one build for all the crates (feature unification). Its steps
  stay cargo steps. A change to builds for each crate changes the plan of the cut gate; that is not in
  this issue.
- The heavy lanes (`--release --ignored`) and the quick tier. They have other build steps and few steps
  for each run. `direct::records` and `direct::of` accept a step only by its args, so a later PR can add
  the JSON flag to those builds.
- A new dependency (`serde_json`, `cargo_metadata`). The gate reads a few fields. `xtask/src/json.rs` is
  a small reader for them.
- A search for `*-<hash>` files in `target/`. It can find an old executable or the GUI binary.
- A hard failure when a target has no recorded executable. cargo runs the step correctly in that case,
  and the printed command shows it.
- A `rustc --print target-libdir` call for the library path. Under `cargo xtask` the path of the gate
  process has the toolchain directories already. The tests link no toolchain dylib.

**Cost to change later.**

- Low. Remove the JSON flag from `plan::debug_build_crate`, and each step is a cargo step again.
- The `gate` skill text still says that each step calls `cargo test`. The worker session could not edit
  `.claude/skills/gate/SKILL.md` (the edit was refused). The text needs an update.
- If a later cargo sets a new variable for a test process, add it in `artifacts::package`.
