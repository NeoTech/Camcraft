## #984 - the STEP bore's heavy test pins platform-robust facts, not exact counts

**What was decided.** `app pocket3d_hsm::the_users_bore_roughs_adaptive_level_by_level_and_verifies_clean`
keeps every safety assertion (no verifier warning, no budget refusal, every
rings level named by Z in the notes and the status line) and replaces two
exact pins with platform-robust ones:

- "exactly one rings level, Z 20.000" becomes: Z 20.000 is among the rings
  levels, every rings level is there for a crossing ("crosses itself"), and
  at most two of the five levels go to the rings (three keep the HSM plan);
- "5530 moves / 5497 planar / 600 arcs" becomes: 5000..=8000 moves, at least
  98 % of them planar feeds, and arcs posted.

**Why.** Two separate causes, both measured on macOS (Apple silicon, release):

1. *Platform.* On the same commit ecb54b7f the bore plans differently on
   two machines: the #850 measure (maintainer's machine, Windows) gives
   5530 / 5497 / 600 with only Z 20.000 on the rings; macOS gives
   5786 / 5753 / 651, also only Z 20.000 but crossing at (-2.33, -2.78).
   The test has been red on macOS since the counts were pinned (6c198278).
   macOS is deterministic run to run (same crossings, same counts), so it
   is not hash order; it is last-place float behaviour. Rust does not fuse
   multiply-adds without being asked, so the most likely source is the
   platform libm (sin, cos, atan2, hypot are not correctly rounded and differ
   between the MSVC CRT, glibc and Apple's libm) under the STEP tessellation
   and the planner, amplified by the planner's knife edges (#882: a 16 nm
   change of the rings flips the slot's strategy; #982).
2. *Input change.* `git bisect --first-parent` from ecb54b7f to f490458a on
   macOS puts the second rings level (Z 10.000 crossing) at the merge of
   #689 (575f02d8, STEP chord cap and the arc fit). #689 changed the bore's
   mesh and its arc output on every platform (macOS: 6503 / 6467 / 222, arcs
   down from 651), and the heavy bore test was not re-measured there.

A planner fix was not taken: the crossing is caught by the planner's own
guard and the level falls back to Offset rings by name, which verifies clean.
That is the safety law working, not a gouge; removing the crossing means
making the planner robust to sub-micron perturbation, which is #882 / #982's
work (effort:max), not a narrow change.

**Considered and dropped.**
- *Pin per platform (`cfg(target_os)`)*: only macOS can be measured here;
  a Windows/Linux pin would be a guess, and it would still break at the next
  toolchain or libm update.
- *Pin the macOS numbers*: makes it red on Windows instead.
- *Drop the counts entirely*: the band still catches a runaway plan or a
  level silently lost; kept.

**Cost to change later.** If the planner is made float-robust (deterministic
libm or exact predicates, #1010), the exact counts can be
pinned again in one line.
