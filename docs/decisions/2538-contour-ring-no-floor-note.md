## #2538 - Whole-model 2D Contour: a ring without a floor level gets its own note

**What was decided.** A band of a whole-model 2D Contour that stops at the
next flat level and still emits floor passes now checks each outer ring.
An outer ring that stops at that level and has no floor level above it gets
a note. The note names the ring and its uncut Z range: `whole-model
contour: the ring at (x, y) has no floor level between Z a and Z b - that
wall is not cut; reduce the stepdown`. The check is in the one arm that the
first band and the in-between band share, so one function covers the two
cases of the issue.

**Why.** The band cut the rings that have a floor level and then continued
with a new outline. The ring without a floor level got no pass and no
note, because `forward_band_notes` drops the notes of outer rings when the
contour continues. That is a silent wrong result (Prime Directive).

**A note, not an error.** The band without any floor level already gives a
note and continues (#2436). The same condition on one ring of a band gets
the same class of message. The rest of the program is correct and the note
tells the user what is not cut and what to do.

**Considered and dropped.**
- Forward the `contour_depth::ladder` note of that ring ("meets the part at
  its first level - not walked at any level"). Dropped: that text names the
  skipped flat levels, which the next band does cut. It does not name the
  Z range that stays uncut.
- Refuse the operation. Dropped: it is not consistent with the note of the
  band that has no floor level at all.
- Add the test to `crates/motion/tests/contour_whole.rs`, as the issue
  says. Dropped: that file has 375 lines and the 400-line law applies. The
  pins are in `crates/motion/tests/contour_whole_no_floor.rs`, with the
  same Gate map.

**Not changed.** The first band with no floor level on any ring (for
example a part with only a top chamfer) still gives no note. The issue does
not include it, and a note there changes the output of each chamfered part.

**Cost to change later.** One function (`note_rings_without_floor`) and two
tests. A change of the note text changes the two pins.
