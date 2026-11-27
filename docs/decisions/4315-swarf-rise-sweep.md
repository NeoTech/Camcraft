## #4315 - swarf exit rises: an exact sweep proof, and the plan's rapid rises answer to it

**What was decided.** Two changes in the swarf stamp
(`crates/motion/src/multiaxis/`):

1. The rise proof in `stamp_rise::rapid_rise` is exact. The tool's axis
   swept up the rise is a parallelogram. Its distance to each facet comes
   from the triangle-triangle distance law (new `stamp_sweep.rs`). The
   swept tool must stay at least as far from the part as the start pose
   (`min(start gap, d0, r)`, less 1e-6). The old proof sampled the ball and
   the flank every r/2. It also let the flank stand at `r - tol`.
2. The plan's own RAPID rises under a held lean now answer to the same
   proof. These are the side pass's retracts off each level. Before, the
   stamp gave each plan rapid the held pose with no proof. These rapids
   made the nine `RapidInPart` warnings on the blade. They were not the
   #48 feed rises. A plan rapid rise that is not proved leaves along the
   contact normal first, under the same pose (`stamp_rise::normal_exit`).
   Then it rises and crosses back to the plan's top point. Each leg is
   proved by the exact sweep. The shortest step of a doubling ladder
   (r/16 to 4r) wins. If no step is proved, the rapid stays the plan's
   own, and the verifier's rapid law judges it (it blocks the post).

**Why.** The verifier is right: each of the nine lifts entered the part
(0.013 to 0.24 mm). Two causes were real and each alone kept the gate red:

- the plan rapids had no proof at all (mutation: the route off, all nine
  warnings come back);
- the sampled proof missed a facet edge between samples (mutation: the
  sampled law back, move 2141 comes back. That rise starts under the
  tube's equator).

**Dropped.**

- A looser verifier law. The lifts are real penetrations.
- Demote an unproved plan rapid to an upright feed, so the link law takes
  the turn (the pre-#48 route for feed rises). The full-band continuity
  test refused it: the link's plunge back onto the same ball centre under
  the upright pose stands inside the tube under the equator. It also added
  nine air links.
- Refuse the operation when no exit is proved. The verifier already
  blocks the post for a bad rapid, and an exit that is only conservative
  must not refuse a program that the verifier passes.
- Apply the normal exit to unproved FEED rises too. They keep the link
  law, as before. No case needed a change there.

**Cost to change later.** Small. `rise_kind` is the one decision point.
The exact sweep (`quad_gap`) is a closed form with no tuning. The column
radius is the ball radius, as in `flank_clear`; a shank or holder wider
than the ball is not in this proof (the verifier's stack law judges it).
