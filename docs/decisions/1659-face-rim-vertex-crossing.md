## #1659 - A Face row through a rim vertex crosses the rim

**What was decided.** The Boundary wall reading of the gouge law (b)
(`crates/sim/src/verify/section.rs`, `wall_gouge`, the branch with no
model) now counts two types of rim crossing. The first type is a proper
crossing through the interior of an edge (as before). The second type is
new: the trace goes through a loop vertex, and the two neighbours of that
vertex are on opposite sides of the trace (`Trace::through` in
`arc_wall.rs`, for a segment, an XY arc and an XZ / YZ footprint). A move
that crosses the rim is a Face row that overruns an open rim. Thus the law
does not read it against the region edges.

**Why.** The stock outline of the issue has vertices on its straight
sides, at y 60 and at x 0. The last Face row (y 60) goes exactly through
the vertices (-2, 60) and (62, 60). Thus it crosses no edge interior. The
law read the row as a move in the region, and the row touched the vertex
at distance 0. The result was `Gouge` on move 42, with no part in the
document. The first row (y 0) gets no warning because no vertex is on its
line. A vertex on a straight side is a correct loop. The law must read the
geometry, and not the vertex layout.

**What was considered and dropped.**
- Remove the collinear vertices from the stock outline. Dropped: any
  boolean or any section can put vertices on a side. Then the same false
  warning comes back from a different source.
- Make the tolerance larger, or read a touch at the rim as clean. Dropped:
  the task does not permit a tolerance larger than `GOUGE_TOL`, and a move
  that only touches a vertex (both neighbours on one side) is the centre
  on the wall. It stays a gouge.
- Read a move with an end outside the region as clean. Dropped: a Contour
  with no model runs fully outside its region, and its wall reading must
  stay.

**Safety.** The change makes only one more type of move a rim crossing,
in the branch with no model. In that branch no part exists. With a part,
the keep law of #1607 and the solid law still judge the Face. The pins
show that a Face into a part top, and a last row 0.02 mm into a part
wall, are still refused, and that the post stops.

**Cost to change later.** Small: one helper (`crosses_rim`) and one trace
method.
