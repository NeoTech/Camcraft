## #3925 - Re-pin the tabbed contour case after #3678 pecks its tab window dives

**Decided.** `crates/app/tests/plunge_depth_tab.rs` now pins two cases.

1. The catalog case `2d/contour-tabs` as it is. The verifier names no
   `PlungeTooDeep` dive. Each of the four dive columns that reach the last
   level stops at Z 4, 2 and 0. No step bites more than the 2 mm stepdown
   below the Z 6 stock top or the depth before it.
2. The same case with the contour top 4 mm under the stock top, set with
   `EditOperationParams`. The levels are Z 2 and Z 0. The ring start dives
   to the first level, which never pecks (#3678). Each later window of the
   last level pecks from a floor one stepdown over the first level (Z 4),
   so its first peck also lands on Z 2. The stock top is Z 6, so the four
   dives bite 4 mm against the 2 mm stepdown. The verifier names each one,
   and none holds the post.

**Why.** PR #3866 (#3678) and PR #3847 (#3679) merged one minute apart. The
toolpath is correct; the old expectation of three 6 mm dives was out of
date. Case 2 keeps an end-to-end pin of the law through the app: the
stepdown wiring (`cutting_rows::plunge_stepdown`), the generate step and
the cached warnings. The sim crate tests keep pinning the law itself.

**Dropped.**

- A new catalog case or project file for the deep dive. The edit of one
  height on the existing case is smaller and needs no recorded program.
- A synthetic toolpath fed straight to the verifier. That skips the app
  wiring, and `crates/sim/tests/plunge_depth.rs` already covers it.
- A scan of every catalog case for a remaining deep dive. No case in
  `examples/` or `examples/catalog/{2d,3d,multiaxis}` names one now.

**Found.** Under carried stock the first peck of a later window bites 4 mm.
The #3678 floor (one stepdown over the first level) does not see the stock
above the op's top. The generator note "Carried stock" and the verifier
both name it. Follow-up #3926 records it; this PR does not change it.

**Cost to change.** Small. If a later change pecks from the stock top under
carried stock, case 2 changes on purpose (fewer findings) and needs its
counts updated.
