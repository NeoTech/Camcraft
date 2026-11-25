## #3886 - pocket3d_hsm bore pin follows the solidcore bump of PR #3802

**What was decided.** The heavy test
`the_users_bore_roughs_adaptive_level_by_level_and_verifies_clean` pins the
current output: mesh 5232 facets (Linux x86_64) and the counts
(4688, 4660, 1980) on Windows x86_64, Linux x86_64 and macOS arm64. The
default-suite exactness tests now also reject the reading from before the
bump (3158, 3130, 1141).

**Why.** The first bad commit is the merge of PR #3802 (df899f029, the
solidcore bump). On Linux x86_64, release, with a print-only probe:

| Commit | Facets | (moves, planar, arcs) | Warnings |
|---|---|---|---|
| 940cc5d0e (parent of df899f029) | 6228 | (3158, 3130, 1141) | 0 |
| df899f029 (#3802 merge) | 5232 | (4688, 4660, 1980) | 0 |
| baad348f1 (main) | 5232 | (4688, 4660, 1980) | 0 |

PR #3802 changes no planner code (Cargo.lock, `crates/kernel/Cargo.toml`,
`geometry/src/step_diag.rs`). Each level, before and after the bump, takes
one of two HSM plans that the planner already made: plan A (397 planar
feeds, 72 arcs, 334.31 mm) or plan B (1933 planar feeds, 918 arcs,
999.92 mm). Before the bump only Z 10 took plan B; after it Z 15 and Z 5
take plan B. The new mesh moves the plan choice, not the plan. Every level
verifies clean, no level falls back to rings, and no planning budget note
shows. The Windows triple is the same as the Linux one (issue comment of
2026-11-24 at 6d0381f19 and 2e2d40cb3).

**Considered and dropped.**
- A fix of plan B in this PR: it is an older HSM planner defect (it was in
  the pin before the bump, at Z 10), and an algorithm change is
  `effort:max`. Filed as #4010.
- A widened band in place of an exact pin: it hides the next real change.
- A plain `#[ignore]` or a known-fail marker: the output is the planner's
  correct current output, not a failure.
- Revert of the solidcore bump: the maintainer asked for the bump.

**Not measured.** macOS arm64 after the bump: the pin follows Windows and
Linux, because macOS equalled them before. The facet count on Windows: the
test does not assert it there.

**Cost to change later.** One constant (`BORE_PIN`) and the facet pin.
When #4010 makes the plan stable, both move again.
