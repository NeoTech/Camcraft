## #4262 - Weight the B letter by |cos A| in the mid-pose swing bound

**What was decided.** The mid-pose walk (`crates/sim/src/verify/midpose.rs`)
bounds how far the tool axis can turn inside an interval from the letters'
sweep. It now counts the B letter at `max |cos A|` over the move, not at
full weight. A and C keep full weight.

**Why.** The axis is `Rz(C) (cos A sin B, -sin A, cos A cos B)`. A turn of
B moves the axis by at most `|cos A|` radians for each radian. The old
bound counted each letter at one radian for each radian. `axis_to_angles`
writes a radial pose under the A line as B180 and the same pose over it
as B0. At A = -90 (the pole) a B 180 -> 0 flip is one pose written two
ways. The old bound read it as a 90 degree swing. The posed sweep grew the
spindle housing capsule (top 300+ mm up the pose) by hundreds of mm, and
the grown capsule reached the machine table 510 mm below. That is the
phantom "spindle housing at move N strikes a fixture" of the report. The
new bound is still a proven upper bound (the derivative norms above), so
no swept volume that the laws enclose gets smaller than the true sweep.

**Considered and dropped.**
- Judge the machine frame with the stack always upright and grow only the
  tip by `off_axis(tip) * dth`. This is tighter, but it changes the
  machine-frame law for all posed moves. The bound fix alone clears the
  reported case and keeps the law's structure.
- Canonicalize the letters before the walk (unwrap B through the pole).
  It hides the cause and does not cover a C letter.
- Name the struck box in the `FixtureHit` message. That changes the
  warning type and its readers (the post gate, MCP, the UI). It is a
  separate follow-up, not part of this defect fix.

**Cost to change later.** One line in `midpose::walk` and the helper
`max_abs_cos`. Every law that reads `Interval::swing` (fixture sweep,
solid, contact, mesh) gets the tighter bound. A revert restores the old,
looser bound.
