## #3129 - Zigzag levels of the avoid pocket and the 3D pocket close with boundary rings

**What was decided.** Two more pass plans now use
`pocket_zigzag::rest_passes` for Zigzag and Auto, as the plain 2D pocket
does after #3021:
- `pocket_avoid` (2D pocket with "Do not touch"), at each level.
- `machining3d::hsm::plan_level`, the 3D pocket's level plan when the
  strategy is not Adaptive and the level has no rotary seam.

The rows stand 0.01 mm inside the level's cutter-center area. One closed
ring then follows each boundary loop of that area: the outer wall, each
island, and each keep-out edge.

**Why.**
- Rows touch a wall or an island only at their ends, so a cusp stays
  between each two ends over the full depth. The pins measured, before
  the change: 0.755 mm on a rounded corner of the avoid pocket (0.919 mm
  with blended turns), and a 2.0 mm strip on the boss wall of a 3D level.
- `rest_passes` is the plan that the plain 2D pocket and the rest pocket
  already use. One plan for all Zigzag levels keeps the wall result the
  same in all three operations.
- The 3D plan already read Auto as Zigzag (`plan_passes`). The change
  keeps that reading.
- `rest_passes` is made `pub(crate)` and re-exported as
  `zigzag_ringed_passes`, so that `machining3d` reads it through the
  `machining2d` door, as it reads `plan_passes`.

- The rotary wrap (Rotary Offset) uses the same `plan_level`, always
  with a seam. Its levels keep the plain rows. The developed area of a
  wrap level ends at the seam, so a ring on each boundary loop would also
  cut the seam line, at both ends of the strip. That is one physical line,
  cut twice, and not a wall.

**Dropped.**
- Rings on the rotary wrap levels in this PR. They need a ring that
  skips the seam edges. Issue #3168 tracks them.
- A change inside `plan_passes`. It has more callers (the 2D Adaptive
  fallback, the 3D rest pass and shells), and #3130 changes its caller in
  `pocket.rs`. A change at the two call sites is smaller and does not
  overlap #3130.
- A crossing-law post for blended turns in `pocket_avoid` (the plain
  pocket's `hold_crossings`). The blended pin shows no crossing pair, and
  `pocket_avoid` rounds its turns with the level area as the legal region.
  If a crossing shows up later, the post check blocks the program, and the
  law can be added then.

- A raw count of posed moves in the Multi-Axis Rough test
  (`posed <= 20`). The closing ring round the boss adds a pose on each
  ring chord. The test now checks that each pose stays within the ball's
  reach of the boss, which is the intent of the old bound.

**Cost to change later.** Low. Each plan selects `rest_passes` in one
branch. The pins in `crates/motion/tests/pocket_wall_finish_levels.rs`
hold the wall result.

### Review correction: plain pocket entries and link assertions

**What was decided.** A restart for a tool that is not center-cutting
now checks its posted entry against prior feed moves. It tries the helix,
then a ramp along the pass. It refuses the operation if neither entry
passes the check. The plain pocket tests keep the inset rows and the
separate closing rings. Their entry counts describe that plan.
The KeepDown test measures the reduction in rapid moves as well as gaps
that contain nonplanar moves.

**Why.**
- A closing ring restart can descend through the same depth interval as
  the rows' earlier entry. Room inside the region does not prove that
  those descents miss each other. The restart now uses the existing
  `moves_cross_prior` check on a staged entry before it posts that entry.
  The `center_cut_law` pocket test checks the crossing pairs directly.
  `a_closing_ring_restart_does_not_cross_the_rows_transition_ramp`
  also requires the round pocket to succeed with a named ramp fallback.
  A refusal cannot satisfy that pin.
- The square has two passes at each level: the inset rows and the outer
  ring. It has four dives over two levels, instead of two. The row Ys
  are 5.01, 10.01, 15.01, 20.01, and 24.99 mm. The wall ring adds
  Ys of 5 and 25 mm. Five 19.98 mm rows, 19.98 mm of connectors,
  and the 80 mm ring give 199.88 mm, plus the entry link. The test checks
  these values and keeps its center-area checks. A ring entry can stand
  on the center boundary, so its entry check includes that boundary.
- The island splits the inset rows into five passes. Its outer wall and
  island add two rings. The test therefore expects seven dives at each
  level, or fourteen over two levels. It keeps the island distance and
  chord checks.
- `pass_gaps` does not count a successful planar KeepDown feed as a gap.
  Fewer rapid moves than the retract plan show that a link stayed down.
  The test also requires fewer rapid moves than a plan whose 0.001 mm
  stay-down limit forces each link to lift. Verification and the checks
  for named lifts remain in place.

**Dropped.** Accepting an entry only because it fits the region would
keep the crossing defect. The old entry counts, row positions, and path length describe
rows without closing rings. Restoring that plan would restore the wall
cusps. Counting only nonplanar gaps would miss successful KeepDown links.

**Cost to change later.** Low. The three motion suites hold the entry,
geometry, crossing, and link behavior.
