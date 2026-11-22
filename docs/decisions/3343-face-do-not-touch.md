## #3343 - "Do not touch" on Face operations

**What was decided.** A Face operation accepts protected faces and edges.
At each cut level, the generator asks for the cutter-centre keep-out of the
protected items (`AvoidSpec::keepout_region`, for each tip height from the
level up to the clearance plane). It cuts each row short at that keep-out,
with the same interval law that clips rows at bridged holes
(`face_holes::pieces`). Where a row is cut short, or where a row-end step
(sharp or blended) would cross the keep-out, the facer links over it at the
clearance plane: up, across, plunge. The existing rapid router
(`route_links`) raises a link that must go higher, and the existing
`verify_avoid` check and post gate read every Face move. A level whose rows
are all inside the keep-out is skipped and the path notes it. When no level
is left, the operation is refused with the protected items named. A path
note says that the material beside the protected items stays unfaced.

**Why.** The Face generator already had a "forbidden centre region" law
with clearance-plane links. The keep-out is the same kind of region, so
the change reuses a proven path shape and adds no new link geometry. The
keep-out covers tip heights up to the clearance plane, so each plunge and
each lift at a clipped row end is also clear. Lower levels see a larger
keep-out than higher levels, so a lower level never takes a deeper bite in
an area that a higher level skipped. With no keep-out (no protected items)
the generator takes the old code path, and the path is the same to the
byte (pinned in `motion/tests/face_keep.rs` and
`app/tests/do_not_touch_face.rs`).

**Dropped.**
- Post-processing the finished Face path (clip each move after
  generation): it must also re-plan arcs, leads and plunges, and it
  duplicates the row law.
- Refusing every Face that comes near a protected item: safe, but a
  protected boss in the face area is the main use, and the issue asks the
  generator to keep clear.
- An exact test for the blended step against the keep-out: the step is
  tested with a box that holds the sharp step and the semicircle on either
  side. A false "not clear" only adds a link; it never gouges.

**Cost to change later.** Small: the keep-out reaches the facer through
one closure (`FaceKeep`), and the per-level logic is in
`machining2d/face_keep.rs`.
