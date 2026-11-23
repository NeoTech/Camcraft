## #3474 - A short arc whose written ends share one angle is a straight move

**Decided.** The arc guard of #3442 (`arcFateOf` in
`crates/post/src/post/arc_guard.rs`) has one more rule. The written start
and the written end differ, but the written end is on the written start's
ray from the written centre (the same angle), or one end is on the written
centre. If the record turns half a circle or less, the record posts as the
script's straight feed move (`"linear"`). All shipped posts use the rule
through their existing `arcFate` call. The two lathes (`fanuc-lathe`,
`tormach-lathe`) now write that feed move in their meridian arc path:
before, they wrote no block for each answer that was not `"arc"`.

**Confirmed by a post, not by a control.** On the parent branch (#3471)
every mill post wrote such a block, for example:

- Fanuc mm: `G01 X10.011 Y5.` then `G03 X10.01 Y5. Z5.5 I-0.011 J0. F180`.
- LinuxCNC inch: `G1 X0.4011 Y0.2` then `G3 X0.401 Y0.2 Z0.2165 I-0.0011 J0 F7.09`.

The start radius is 0.011 and the end radius is 0.010, at the same angle.
No control ran these blocks. RS-274 / ISO 6983 circular interpolation
reads an arc whose end is at its start angle as a full turn. A control
that accepts the radius difference then cuts a spiral of nearly one turn.
This change does not depend on which reading a control takes: a straight
move is correct for each of them.

**Why a straight move is safe.** The written ends are on one ray only when
the record's angular sweep is below the output step divided by the radius.
Then the chord of the record is shorter than about one output step. The
straight move stays within the output resolution of the programmed arc.

**How the test reads the words.** The rule parses the number of each
written word (the script's own word function: unit, precision, origin,
lathe diameter). It uses only two facts: the cross product of the two
written radius vectors is zero, and their dot product is positive. A
mirrored axis or a scaled axis (a lathe diameter word) does not change
these two facts. So the rule needs no knowledge of the frame of each post.

**A zero sweep in the stream stays an arc.** The stream itself is rounded.
A 359.99 degree record can arrive with its own ends on one ray (stream
sweep 0), as in `radius_arcs.rs`. `emitArcWithRadius` reads a zero stream
sweep as a full turn. The new rule acts only on a positive record sweep,
so such a record stays a full-turn arc, as before.

**Dropped.**
- A check of the written sweep against the record sweep with the turn
  direction (the issue's example). The written cross product's sign flips
  on a post that mirrors an axis, so a sign test would make normal arcs
  straight moves on such a post. The zero test does not flip.
- A rule for a written end that is behind the written start (a written
  sweep of nearly a full turn on a short record). Monotone rounding of the
  ends cannot put the end behind the start about the written centre in
  the cases that we found. A centre word that rounds apart from the
  start could do it in theory. This is not exercised and is left open.
- The rounded absolute centre for a post that writes increments (review
  of PR #3500). The two centres can differ by one output step. Fanuc mm
  wrote `G02 X10.01 Y5. Z5.5 I-0.011 J0.` after `G01 X10.011 Y5.`: the
  control's centre is X10. Y5., and both ends are on its positive X ray.
  The rounded absolute centre is X10. Y5.001, so the rule kept the arc.

**The centre that the control reads.** A post that writes `I J K` as
increments from the start passes a third argument to `arcFate` /
`arcFateOf`: `inc(axis, delta)`, the number of the written increment word
in the scale of that axis's position word. The rule adds it to the
written start. The mill posts pass `arcIncOf(fu)`. The lathes double the
radius `I` to the diameter `X` scale. The Heidenhain posts write an
absolute centre (`I J`, `CC`) and pass no `inc`: their written centre is
the rounded absolute word. Pinned by
`a_same_angle_arc_about_the_written_incremental_centre_posts_as_its_straight_move`
(mm and inch) and the `offset-*` rows of
`arc_fate_reads_a_same_angle_short_arc_as_a_straight_move`.

**Merged with main's full-turn test.** Main (`532d0fa92`) reads a full
turn from ends that are equal in the stream, not from ends within 1e-5
mm. The merge keeps main's exact equality and this branch's `_arcSweep`
helper for the sweep.

**The stream writer decides when the stream has no sweep (second review
of PR #3500).** The app posts through `cnc::serialize`, which writes
whole micrometres. A mm post with three decimals writes those numbers
again. For the arc of the issue the stream then has start X10.011 Y5,
end X10.010 Y5 and centre X10 Y5: a zero sweep. The post rule above keeps
a zero stream sweep as a full-turn arc, so Fanuc mm still wrote `G03
X10.01 Y5. I-0.011 J0.` for a program from the app. The post cannot
decide this: the same record can be a turn of 359.99 degrees. Only the
writer knows the true sweep. `cnc_arc_lattice::short_arc_on_lattice` now
has the same rule on the lattice: the rounded ends differ, they are on
one ray from the rounded centre (or one is on it), and the true sweep is
positive and not more than half a turn. The writer then gives a `Feed`
to the rounded end. A turn of more than half a circle keeps its arc
record. The test is exact: integer cross product zero, integer dot
product positive. Pinned by
`the_arc_of_3474_through_the_stream_writer_posts_as_a_straight_move_on_fanuc`
(a `Toolpath` through `dump` and `serialize`, mm and inch),
`every_mill_post_writes_the_arc_of_3474_through_the_stream_writer_as_its_straight_move`,
the lathe test of `same_angle_stream.rs`, and two tests of
`cnc_arc_lattice.rs`. The post rule stays: it covers a post whose words
are coarser than the stream (inch, or fewer decimals).

**Dropped (continued).**
- A change of the post rule to read a zero stream sweep as a short arc.
  A hand-written stream and the 359.99 degree record of `radius_arcs.rs`
  state a full turn in that form.

- A refusal of the record: it would block a legal program for a move
  under the output step.

**Cost to change.** One function in the PRELUDE (`sameAngleShort`). A
dialect with a different reading changes its own call site only.
