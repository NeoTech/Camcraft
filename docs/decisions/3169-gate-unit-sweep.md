## #3169 - The gate removes superseded test-harness units at the end of each run

**Cause.** Read in the shared build worktree `issue-917` on 2026-11-21 (read only):

- One run does not give one target hundreds of hashes. The 634 `camcraft-app`
  unit directories that were left after a sweep are 627 distinct targets: one
  unit for each integration test binary (`crates/app/tests` has about 620
  suites), plus the lib, the bins and the lib tests. Each test binary is its
  own linked executable with its `.pdb`, about 100 MB, so one set is about
  64 GB. Only 5 targets had a second variant, from a cargo call outside the
  gate with another dependency feature resolution.
- The growth is across trees and env. `camcraft-motion` had 747 units for 124
  targets (six variants), `camcraft-post` 554 for 110, `camcraft-sim` 462 for
  92. The variants differ in the profile hash (agents with other env before
  #3154) or in the fingerprints of registry dependencies (other `serde_core`
  feature sets). `Cargo.lock` and the manifests were the same at the heads
  `8b4209215` and `cc04d7bcd`.
- Cargo never removes a superseded variant. So each change of the env or of
  a feature resolution adds a full set, about 60 GB for `camcraft-app`. The
  count "629 to 4386" included these older sets.

**Decided.** At the end of each gate run (green, red or void; never in a dry
run), `xtask/src/unit_sweep.rs` removes a unit directory only when all of
these are true:

- it is under `target/debug/build` or `target/release/build` of the gate's
  own root, and no `CARGO_TARGET_DIR`, `CARGO_BUILD_TARGET_DIR` or
  `CARGO_BUILD_BUILD_DIR` points elsewhere;
- its one target fingerprint is a test-harness target (`test-lib-`,
  `test-integration-test-`, `test-bin-`);
- another variant of the same target (profile, package, fingerprint name) was
  written in this run, and this variant was written before the run started;
- cargo's locks of each profile directory are free (the sweep holds them while
  it deletes), and no other process names `<root>/target` in its command line
  (another gate's xtask, a cargo or rustc there, a test executable);
- no level of the path (`target`, the profile, `build`, the package, the unit,
  `fingerprint`) is a symlink, a junction or another reparse point. A linked
  profile directory or `target/` stops the whole sweep; a linked package or
  unit is skipped. The unit is checked again just before its delete.

The process check matches each form of `<root>/target`: the path as given,
its absolute and clean form, and the canonical form (a subst drive, a
junction, `/var` and `/private/var` on macOS). On Windows the match ignores
case. A root form that cannot be resolved, a process list that cannot be read,
or a cargo lock that cannot be probed counts as "unsure": nothing is removed.

A failed delete is a warning line, never a gate failure. The summary
prints the number of directories and the GB removed, or why the sweep did not
run. `CAMCRAFT_GATE_SWEEP=off` (or `0`) turns it off.

**Why at the end, and in any outcome.** A variant that this run wrote is the
one that this tree uses. The superseded variant is used again only when the
worktree goes back to a tree with the earlier resolution; then cargo builds
those targets again (minutes) instead of keeping 60 GB. A red test does not
make the new units wrong, so the outcome does not change the choice.

**The lock of `cargo xtask` itself.** `cargo xtask` is `cargo run -p xtask`.
Cargo holds the build lock of `target/debug` only while it compiles, and frees
it before it starts the binary. The proof by reading: each gate step starts a
cargo build into the same `target/debug` while `cargo xtask` is alive; if the
outer cargo held the lock, every gate would wait for it without end. So the
lock probe at the end of a run sees only other builds.

**Why only test-harness units.** A test executable is built only when its
own package is selected, and the gate selects one package for each step, so a
target has one valid variant in a tree. A library can have two valid variants
in one tree (for `-p geometry` and for `-p motion`); removing one costs a
compile in the next run. Libraries are small next to the test executables.

**Dropped.**
- Fewer `camcraft-app` test binaries (one harness with modules): this is the
  root of the size, but it changes about 620 files and the gate maps. It can
  be a separate issue.
- Remove every unit that this run did not touch: a quick run builds only some
  crates, and the units of the others are still valid for the tree.
- Keep the newest variant by time alone: a run that reuses an older variant
  as Fresh would lose it and build it again in the next run.
- `debug = "line-tables-only"` for tests: smaller `.pdb`, but it changes the
  profile, so each unit is stale once, and it changes debugging for persons.

**Cost to change later.** One module; `CAMCRAFT_GATE_SWEEP=off` disables it
without a code change.
