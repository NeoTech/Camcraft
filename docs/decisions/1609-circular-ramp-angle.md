## #1609 - Circular ramp entry rides the ring at the ramp angle

**Decided.** On the circular kernel (the ladder that opts in with
`Motion::with_ring_zig`), the entry Ramp without a user ramp angle no
longer posts the legacy single chord. It plans the zig-zag ramp along the
ring at the transition's ramp angle (3 deg by default), through the same
`ramp_fallback` that the helix's stand-down already uses. When the ring is
too short for that angle, the descent is `Descent::Sub` with the reason
"the ring is too short to ramp at the angle": an axial dive at the plunge
feed, with an `entry:` note.

**Why.** A ring starts and ends at the same point. So the chord from the
tool's XY to the pass start has no length at each level change (a 90 deg
feed at the cutting feed), and the first entry of an inner ring is a short
chord from the outer ring (76 deg). Both were called a ramp and carried no
note. The zig-zag along the ring holds the angle and rides the ring's arcs
(#1558's `push_zig_ring`).

**Considered and dropped.**
- Change the legacy chord arm for every ladder (pocket, contour, bore). It
  would move posted output of every Ramp entry with no angle set, and the
  issue is the circular kernel's. Not checked here whether other ladders
  with closed passes show the same 90 deg chord; that is a separate
  question.
- Clamp the chord to the angle by moving its start back. A ring has no
  straight room for that: the chord would leave the ring.

**Cost to change later.** One match arm in `descent::plan_arm`. Remove it
and the circular Ramp entry goes back to the chord.
