## #980 - re-measure the STEP bore's heavy Adaptive pin for the 0.18.0 cut (part 1)

**What was decided.** The heavy-lane pin
`pocket3d_hsm::the_users_bore_roughs_adaptive_level_by_level_and_verifies_clean`
is re-measured, test-only: the levels that fall back to Offset rings are now
asserted as exactly {Z 20, Z 10}, each "crosses itself", and the path size as
(6500 moves, 6464 planar feeds, 222 arcs). Every safety assertion stays as it
was: no verify warnings, each rings level named in the notes and the status
line, no "planning budget" / "too detailed" refusal. The issue stays open for
part 2 (get the Z 10 HSM plan back).

**Why.** Measured with the test itself (release, `--ignored`):

| Commit | Result | Fallback levels | moves / planar / arcs |
|---|---|---|---|
| 575f02d8^1 (3bd54fc9, before #839) | pass | Z 20 | 5530 / 5497 / 600 |
| 575f02d8 (merge of PR #839) | fail | Z 20, Z 10 | 6500 / 6464 / 222 |
| f490458a (main at the cut) | fail | Z 20, Z 10 | 6500 / 6464 / 222 |

PR #839 (issue #689) caps the STEP tessellation chord at 0.005 mm (half the
program sim's verdict tolerance) where it was ~0.1% of the model extent, so
the user's bore is meshed much finer and every level's outline changes. On
the finer outline the Z 10 HSM plan would cross itself at (-1.99, -0.13); the
planner's own crossing law refuses it and the Offset rings cut the level,
named in the notes. The output is still verified clean: a quality regression
(one more rings level, fewer fitted arcs), not wrong output. #839 is a
safety fix (bore sagitta charged as a gouge) and is kept.

**Considered and dropped.**
- Fixing the planner now (option 1): no small, certain defect was found in
  the time box; why the growth planner's plan crosses itself on the finer
  outline is planner work (issue part 2) and must not be rushed under a cut.
- Reverting or loosening #839's chord cap: it fixes a false gouge verdict;
  trading a safety fix for a quality pin is backwards.
- Loosening the pin to ranges or `fell.len() >= 1`: the pin exists to trip on
  any planner or mesh change; exact values keep that.

**Cost to change later.** When part 2 restores the Z 10 plan, the pin goes
back to `fell.len() == 1` (Z 20) and the counts are re-measured once more.
