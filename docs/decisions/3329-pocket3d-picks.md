## #3329 - Start and exit points: the 3D Pocket slice

**What was decided.** The 3D Pocket (`OpType::Pocket3D`) takes a start
point and an exit point. Its capability is the existing
`PickCapability::Region`: the descent place of the first pass and the end
of the last pass. The picks seat the passes of each level that the
strategy already planned (`machining3d::level_seam`,
`machining3d::pocket3d_picked`). They never re-order the passes.

- Start: on each level, the first ring of the first pass starts at its
  point nearest the start point. It cuts its whole lap back to that
  point, then runs on along its own cut, in its own direction, to its
  old start. From there the level is the automatic path, move for move.
  No link changes.
- Start on zigzag rows: an open row chain starts at its end nearest the
  start point. The chain runs the other way.
- Exit: on each level, the last ring of the last pass laps from its own
  start as before. Then it runs on along its own cut to its point
  nearest the exit point. An open zigzag chain ends at its end nearest
  the exit point. The operation's exit is the end of the last pass of
  the last level that cuts.
- A level of one ring with the two picks: the lap from the start point
  runs on straight to the exit point. This is simple and safe: the
  run-on is a re-trace of the cut at the same depth.
- The descent at the new start goes through the ladder's own entry rule
  (helix, ramp, arc, plunge and their stand-ins). The pick does not add
  a motion.
- The ladder reads the X and Y of a pick. Each level descends at the
  start point, so a Z does not select a level. A pick with a Z gets one
  note (`PICK_Z_UNREAD`).
- No pick is the automatic path, byte for byte: the application calls
  the old generator, and `pocket3d_picked` with an empty seam is
  `pocket3d_open_prev` / `pocket3d_offset_prev`, move for move.

**Why the passes are not re-ordered.** The ring order is the engagement
law of the strategy: core first under Auto, from the island under
Inside, from the wall under Outside. A pick that moved a ring to the
front would cut a full-width slot or turn climb into conventional
cutting. A ring of a ring strategy is not reversed for the same reason.
Only the zigzag rows reverse: they cut both ways, so a reversed chain
cuts the same mix.

**Why the run-on back to the old start.** A re-seated first ring that
ended at the new start would need a new link to the next pass. The
run-on keeps each link of the automatic path. The cost is at most one
lap of the first ring for each level, and one part lap of the last
ring. The run-on re-traces a cut at the same depth: it removes no new
material.

**Refused by name, never moved:**

- a start point farther than 1.5 tool diameters from the start of the
  first pass of a level (the reach of the contour, the 5-axis and the
  Deburr slices), with the Z of the level;
- a descent that the entry rule puts farther than that reach from the
  start point (the rule found no room for its motion near the pick);
- an exit point farther than that reach from the end of the operation;
- any pick on a level that the Adaptive (HSM) plan spirals: the plan
  starts in its own entry bore and rides proved arcs, which a re-seat
  would break;
- in the application, before generation: a pick under the Adaptive
  strategy, and a pick with Rest roughing on (the rest pass cuts after
  the last level and ends where its last crumb ends);
- a plunge or retract column at a point that a pick moved that comes
  within the tool radius of the part.

**The column law.** At each point that a pick moved (the new start of a
re-seated ring, the new end of a run-on, the two ends of a reversed
chain, the descent place that the entry rule chose), the tool's cylinder
from the cut height up to the clearance plane must keep the tool radius
(less 0.01 mm) from every facet. The check is exact: each facet is
clipped to the half-space above the foot, then `stock_sim::dist_seg_tri`
measures the distance from the column's axis segment to what is left.
There is no sampling.

- Why clip at the foot: the floor under the tool is at the foot. Without
  the clip each column on a floor is a hit.
- Why 0.01 mm: a ring tangent to a wall sits a few microns inside the
  radius (the region offset flattens a round join to 5 um, and the
  lattice snaps to 1 um). The sim's chord budget covers both.
- The ladder's islands law already keeps each ring point clear above
  its level (the shadow of an overhang is in the island). So no ladder
  fixture reaches the column refusal. It is a second, independent
  check, pinned by a unit test.

**What was dropped.**

- Re-order the passes to cut first the pass nearest the pick: refused
  above (engagement law).
- Rotate the first ring and link from the new start to the next pass:
  each link would be new and must be proved again; the run-on keeps the
  proved links.
- Apply the exit only on the last level: the ladder does not know which
  level cuts last until it ends. Each level ends near the exit point,
  and the reach law is judged at the end of the operation.
- Read the Z of a pick to choose a level: the issue's table says each
  level descends at the start point.
- Seat the Adaptive (HSM) plan: its entry bore and arcs are the plan's
  own proof. A later slice can pass the start point to the planner as
  its bore place.

**Not in this slice.** The UI has no 3D Pocket start / exit rows yet;
the picks are set through MCP `edit_operation` and the project file, as
for the 5-axis and the Deburr slices. The page's "Not taken" lines of
the 3D Pocket go away.
