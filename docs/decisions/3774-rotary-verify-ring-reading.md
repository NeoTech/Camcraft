## #3774 - rotary_verify takes the seam-ring move count of #3496

**What was decided.** The test
`a_rough_over_an_oversized_bar_peels_from_the_stock_radius`
(`crates/app/tests/rotary_verify.rs`) pins the d12.4 matching bar at
2,110 moves, not 2,064. No code changes.

**Why.** The run alone, debug, `--exact`:

| Commit | d12.4 bar | d20 bar | Result |
|---|---|---|---|
| `7868d3786` (`6eff704f2^1`) | 2,064 | 7,648 | pass |
| `6eff704f2` (merge of PR #3496) | 2,110 | 7,578 | fail at the 2,064 pin |
| `436d38ffb` (main) | 2,110 | 7,578 | fail at the 2,064 pin |

PR #3496 (#3168) removed the `seam.is_none()` guard in
`machining3d::hsm::plan_level`, so each Auto/Zigzag rotary level now ends
with a seam ring. This adds moves on purpose. All the safety asserts of
the test still pass on the new path: the program verifies with no
blocking warning, the top level starts at the stock radius, and the
program posts. The d20 bar stays under the 9,000-move air-lap bound.
The pinned count recorded the toolpath before the rings, so it is
obsolete. The air-lap law that the pin guards did not change.

**Considered and dropped.**
- Fix forward: no rule broke. The only failing assert is a move count.
- Revert PR #3496: it would remove the wall-cusp fix of #3168 for a
  stale number.
- A range instead of an exact count: the exact count keeps the guard
  against a silent change of the matching bar's ladder.

**Cost to change later.** One number in one test.
