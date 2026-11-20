## #2736 - 2D Adaptive at optimal load 0.5: a slow path, not a hang

**What was decided.** The cause is a slow path, not a loop that does not
end. The fix makes the door search of a routed microlift return
(`adaptive::route::door`) cheaper and keeps its result exact:

- A door from a point whose own disc holds stock or wall is refused
  before any leg is read. Every leg's capsule holds its start disc, so no
  leg longer than `ZERO_LEG` can pass there.
- The other doors read their candidate cells nearest first and stop at
  the first legal leg. The old search read every free cell nearer than
  its best so far, in raster order. The chosen cell is the same: the
  nearest legal cell, the lower index on a tie.

**Why.** Measured in release on the issue's region (the box part, area
offset 3, D6, load 0.5): the plan took 29.6 s, and 23.7 s of that was the
microlift schedule. In the schedule, 13 routed returns took 18 s, and
the door search took 16 s of it. All 22 doors that failed had a blocked
start disc, and each read the leg law on each free cell within 6 mm (the
grid is 0.05 mm). After the fix the plan takes 9.8 s with output
identical by hash on nine region and load cases.

**Considered and dropped.**

- A refusal of a large optimal load at command creation. The load law
  accepts (0, 0.5], and the plan at 0.5 completes with a legal path.
- Plan the passes one time when the app carries the stock top and plans
  again (`generate.rs`, the Adaptive2D arm). This halves the upright
  case. It is not exact without more work: the trochoid fallback reads
  `material_top`. A follow-up issue holds it.
- Cache the doors inside one `LinkLaw::find`. It saves less than the
  fix and adds state.

**Cost to change later.** Low: `door` is one private function with two
unit tests.
