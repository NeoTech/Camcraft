## #2575 - A new worktree seeds its target/ from another checkout

**Decided.** Before its first step, `cargo xtask gate` copies the dependency units
(`target/debug/{build,deps,.fingerprint}`) of the checkout with the most units into a new
worktree. Cargo then finds `solidcore`, `flecs` and the build-script crates Fresh. The copy:

- skips each package without a `source` in `cargo metadata` (workspace members and path
  dependencies): their sources are in the checkout, and an older copy can read as Fresh
  with other code;
- replaces a unit that is there already (`cargo xtask` builds its own dependencies first),
  else its newer time makes each copied unit above it dirty;
- keeps the modification times, copies outputs before fingerprints, and renames each unit
  into place, so a stopped copy only causes a compile;
- takes cargo's locks of the two directories without a wait, and passes over a checkout
  where a build runs;
- writes `target/debug/.camcraft-seeded` once. `CAMCRAFT_NO_SEED=1` turns it off.

**Why.** sccache 0.19 hashes the `OUT_DIR` value of each env dependency, the cwd and the
raw arguments of a Rust compile. `OUT_DIR` names the worktree, so each build-script crate
misses, and each crate above it misses too. `SCCACHE_BASEDIRS` applies only to C and C++.
Cargo's fingerprints do not name the worktree, so a copied unit is reused. Measured on an
18-core Mac (APFS clones, sccache warm): the first app test took 107 s without the seed
and 42 s with it (9 s seed, 24 s step). 231 units compiled without it, 14 with it.

**Dropped.**
- A `--remap-path-prefix` in RUSTFLAGS: sccache hashes the raw argument, which names the
  worktree, so all keys would change in each worktree.
- A shared `CARGO_TARGET_DIR` or `build-dir`: AGENTS.md forbids the first, and the two lock
  out parallel gates of other worktrees.
- A change to sccache (normalize `OUT_DIR`): outside this repository.
- Copy only the units that the gate needs: cargo has no stable unit graph to name them.

**Cost to change.** One module (`xtask/src/seed.rs`) and one call in `gate`. Removing it
returns the gate to the sccache-only behaviour.
