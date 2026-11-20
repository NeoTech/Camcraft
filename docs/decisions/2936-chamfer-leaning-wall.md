## #2936 - a wall that leans in the frame is not a modelled chamfer

**Decided**

Fix the check of #2887 (PR #2894). Do not change the fixture of #1700.

`partly_bevelled` in `crates/motion/src/machining2d/chamfer_ride.rs` now
returns `None` when the top loop is the wall loop moved whole by one plan
vector (`leans`). The ride is then the wall at the corner depth, as it was
before #2887.

**Why**

The fixture is a 70 x 70 x 20 box stock with no part. The operation frame
is tilted 10 degrees. The top section (Z10) and the corner section (Z9) are
the same rectangle. The corner section is moved 0.176 mm
(`1 mm * tan(10 deg)`) along one axis.

No edge of that box carries a chamfer. The check of #2887 compared each
segment with the wall loop. It read the two sides that touch the moved
corners as "bevelled" and the middle of the other sides as "sharp". The
refusal text was not true for this model. A legal program was refused.

A loop that carries a bevel on part of its length is never a moved copy of
its wall. Its bevelled side moves in. Its sharp sides stay. The opposite
side would have to move out, and a chamfer does not do that. Thus the new
test cannot hide a case of #2887.

The test does not depend on tessellation. The vector is the difference of
the two area centroids. A collinear vertex does not change an area
centroid. Each segment of each ring (two ends and the middle) must then be
on the other ring within `LEAN_SLACK` (1e-4 mm). The sections are on a
1e-6 mm grid, and `MATCH_SLACK` (0.05 mm) is the smallest bevel that the
check of #2887 reads.

**Considered and dropped**

- Change the fixture (other tilt, or picks). The fixture is a correct
  chamfer on a plain box. A change would hide the wrong refusal from the
  suite, and a user would still get it.
- Compare signed distances (a bevel is always inside the wall). One side of
  a leaning box is inside its wall too. A sign cannot tell the two apart.
- Remove collinear vertices and compare the vertices one by one. The result
  would depend on a collinearity tolerance.

**Not decided here**

A part with a draft wall on some sides only, in an upright frame, has the
same two sections as a part with a modelled chamfer on those sides. The
check of #2887 still refuses it and asks for picks. That is the safe
reading. This PR does not change it.

**Cost to change later**

Small. The test is one private function with one call. Remove the call and
the check of #2887 is as it was.
