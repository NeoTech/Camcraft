## #3330 - Start and exit points: the 5-Axis Contour slice

**What was decided.** The 5-Axis Contour (`OpType::Contour5D`) takes a
start point and an exit point (`PickCapability::Ring`). The picks act on
the waterline ring ladder before the pose stamp
(`finishing3d::waterline_seamed`, `finishing3d::ring_seam`):

- Start: each closed ring of the ladder is re-sequenced to begin at its
  point nearest the pick, in XY. The seams of all levels line up toward
  the pick. The pick's Z is not read: each level has its own ring.
- Exit: each closed ring cuts its full lap, then runs on along its own
  vertices to its point nearest the exit pick, and the tool leaves there.
- An open run (a ring that the overhang column law cut into pieces)
  keeps the ends that its partition gave it. A pick does not change it.
- The cut order of the rings and levels does not change. The cut
  geometry does not change: only the seam position and a re-trace over
  cut material on the exit lap.
- Refused by name, never moved:
  - a pick farther than 1.5 tool diameters from every closed ring;
  - a ladder with no closed ring for the pick;
  - a blocked plunge column at a picked start (without a pick the ring
    loop rotates the entry to a clear vertex; with a pick it refuses);
  - a blocked retract column at a picked exit, judged up to the
    clearance plane (the most conservative retract height).
- The tilt limit and the shank dodge: the same stamp poses the seated
  ladder, so both laws are judged at the pick as at every other ring
  point. The centre-cutting guard (`guard_posed`) also runs on the
  seated path.
- The operation has no leads, so there is no lead-room law to hold.
- An empty pick list passes `None` to the ring loop: every path without
  a pick is byte-identical (pinned at motion scale).

**Why.** The waterline ring loop already treats a closed ring as a cycle
whose start is a choice (the blocked-column rotation). A seat in that
loop keeps every existing law (column, ramp, hop, the stamp, the guard)
on the seated path. A re-sequence of the posed output would have needed
its own copy of the column and pose laws.

**Dropped.**
- MultiAxisContour in the same PR: it is a different planner (the
  drive-curve `contour_curve`, open or closed chains). The issue asks for
  one kind for each PR. It stays `PICK_OWED`.
- A per-ring reach (seat only the rings near the pick): a dome's lower
  rings sit far from a pick near the top, and the seams would then be a
  mix of picked and automatic. One seam line for all closed rings is
  easier to predict.
- A re-order of the rings so that the ring nearest the pick is cut first:
  this changes the cut order, which the slice must not do.
- A new field on `Finish3DParams` (136 struct literals) - the seam is a
  separate argument of `waterline_seamed` / `contour5d_seamed`.

**Cost to change later.** Small: the law is in `ring_seam.rs` and one
hook each for the start and the exit in `rings.rs`. A per-ring reach or a
ring re-order would change only those two places and the pins.

**Review follow-up (PR #4006).**
- The exit retract column law is pinned on the untrimmed ring ladder
  (`finishing3d::ring_seam_tests`). Through `waterline_seamed` the column
  trim (`rings_overhang::trim_blocked_columns`) checks the same column,
  every 0.25 mm, at a radius that is equal or larger. A canopy there opens
  the ring first, so the exit pick has no closed ring and is refused by
  name. The exit law stays as the guard for a ladder without the trim
  (the 5-axis swarf ladder, `emit_rings_with_trim(.., false)`).
- `nearest` states a foot at the end of a segment as the start of the
  next segment. Dropped: a tie rule on the distance alone, which can still
  pick the last segment at `t = 1` by rounding and add one full lap.
