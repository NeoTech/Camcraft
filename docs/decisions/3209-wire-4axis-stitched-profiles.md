## #3209 - Wire 4-axis: stitched runs and a die face's opening are profiles

**What was decided.**

- The 4-axis wire cut groups the picked open runs by plane. The runs of
  one plane go through the stitcher that the 2-axis cuts use
  (`stitch_open_runs`). When they close into exactly one ring and every
  run is in that ring, the ring is the profile of that plane.
- Runs that do not close are refused by name ("do not close into one
  ring"). Runs that close into two or more rings are refused by name.
- One open run alone on a plane stays open. The generator refuses it as
  before (an open profile has no die or punch side).
- A face with more than one closed ring gives its one opening (a `Hole`
  ring) when the operation is on the die side. In all other cases a
  face with more than one ring is refused, and the text now says that
  the edge chains of one ring are stitched.
- The stitcher runs once for each plane, never over all picks. Two
  outlines of the same shape at two heights have the same XY ends, and a
  walk over all picks could weld a top run to a bottom run.

**Why.** The issue's three refusals sent the user in a circle: the face
refusal said "pick the edge chain", and the chain refusal said "pick one
face". The 2-axis cut already stitches the same picks, so the 4-axis cut
uses the same weld law and the same tolerance.

**Dropped.**

- A punch face with an opening reads its outer ring. Dropped: the punch
  side states no rule for which ring of two it cuts, and a guess can cut
  the wrong outline. The user picks the edge chains of the ring.
- A face with several openings on the die side picks the largest one.
  Dropped for the same reason.
- Curved wall patches as profiles. Dropped: a wall is not a plane
  outline. The refusal now names what to pick instead.

**Cost to change later.** Small. The rules are in `read`, `plane_ring`
and `profile` of `crates/app/src/operation/wire_ruled.rs`.
