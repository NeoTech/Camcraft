## #2881 - Turning nose offset: a segment claims only where its tangent point is on it

**What was decided.** In `profile_offset` and `profile_offset_in`
(`crates/motion/src/turning/offset.rs`), the line claim of a profile
segment holds only over its claim window. The claim window is the
segment's own station window, shifted by `d*b/sqrt(1 + b^2)` (toward
the outside normal for the OD ride, the inside normal for the bore
ride). At these stations the tangent point of the ball is on the
segment. At other stations the nearest point of the segment is an end
point, and that vertex's circle makes the claim.

**Why.** The old claim held the infinite offset line over the
segment's own window. That is wrong in two directions:

- A steep, short chord (the first chord of a fillet) claimed its
  line far above the bar. The ride went out to X354 on a 30 mm bar.
  The bisection toward that spike wrote many zero-length moves.
- A sloped face did not claim the stations before its inside corner,
  where it is the nearest piece. The ride stood nearer than `d` to
  the face there. The pin measures 0.884 instead of 1.0 at a 45
  degree corner. In the catalog example `cone-thread`, the rough and
  finish rides dropped vertically at the cone-to-flat corner.

The window rule is the exact Minkowski boundary of a segment swept
by a disc. The no-gouge theorem of the module comment is now true at
every station.

**Considered and dropped.**

- Extra candidate stations at each claim window end. At these
  stations the line meets its end circle at the tangent, so the
  curve has no kink there. The extra samples changed four catalog
  programs and added near-duplicate blocks, and the chord settle
  already samples the curve. Dropped.
- A clamp of the ride to the profile maximum plus `d`. It hides the
  wrong claim and does not fix the inside-corner case. Dropped.

**Cost to change later.** Low. The claim is one private struct and
two helpers in one file. The catalog programs `turning/cone-thread`
and `turning/contour-finishes` are the goldens that show the change.
