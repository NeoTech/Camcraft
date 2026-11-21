## #3150 - The gate pins the env that enters the cargo unit hash

**Cause.** Cargo 1.100 puts each workspace unit in its own directory,
`target/<profile>/build/<package>/<hash>/` (rlib or test exe, pdb,
fingerprint). The hash covers the unit's profile. Read in the shared build
worktree `issue-917` on 2026-11-21:

- The manifests, `Cargo.lock` and `.cargo/config.toml` were the same at
  the last six heads that were gated there.
- The `profile` hash in the fingerprint of the same test unit
  (`world_init_race`) took three values over five gate runs (A, B, A, B,
  C). Each value gave all of about 600 `camcraft-app` units a new
  directory. Third-party units did not change: their profile has no
  incremental setting, and they stayed Fresh.
- The agents that use that worktree ran gates with different env. One
  worker restarted its gate with `CARGO_INCREMENTAL=0`; the others used
  the default (incremental on for workspace members). `CARGO_INCREMENTAL`
  sets `profile.incremental`, which is part of the profile hash.
- Nothing in `xtask/src` sets a profile, features, `--config` or rustflags
  for a gate step. Inside one run, the per-crate `-p` builds (#2727) give
  the lower crates one variant for each feature resolution. That set is the
  same in each run, so a second run reuses it.

**Decided.** Every cargo call that the gate starts (`direct::cargo`, and the
catalog builds that already take the gate cache env) gets
`CARGO_INCREMENTAL=0` and loses the caller's `CARGO_PROFILE_*` and
`CARGO_BUILD_INCREMENTAL` variables (`xtask/src/unit_env.rs`). The gate
prints the pin at its start, names each removed variable, and notes a set
`RUSTFLAGS` / `CARGO_ENCODED_RUSTFLAGS` / `CARGO_BUILD_RUSTFLAGS`. The
summary prints the number of unit directories that the run added (0 on a
second run of the same tree).

**Why incremental off, not on.** sccache cannot cache an incremental unit,
so with incremental on the workspace crates never hit the shared cache.
Incremental data also adds to the size of each unit, and disk is the
defect here. The coordinator already set `CARGO_INCREMENTAL=0` for all
builds in the shared worktree. Cost: an edit of `camcraft-app` recompiles
the crate in full instead of in part.

**Dropped.**
- Remove `RUSTFLAGS` too: a person can set it on purpose (a target CPU, a
  sanitizer). The gate notes it instead.
- Prune the old unit directories after each run: an older unit can be the
  valid one for another head in a shared worktree, and a wrong delete costs
  a full rebuild. A prune command can follow if the pin is not enough.
- Fix the env only in the agent briefs: each new agent or shell can miss
  it again; the gate is the one choke point.

**Cost to change later.** One constant (`unit_env::PINNED`). A change of the
pinned value makes each existing unit stale once.
