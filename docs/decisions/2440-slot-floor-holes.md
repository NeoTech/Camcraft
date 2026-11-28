## #2440 - Holes in a slot floor, and a hole tangent to the slot wall

**Decided.**

- The slot reader (`recognize::shape::slot_of`) refuses a floor with a
  boss in it, not a floor with any inner ring. An inner ring is a boss when
  a facet off the floor on one of its edges stands above the floor
  (`walls::Mesh::rises_from`). A hole through the floor has its wall below
  the floor, so it does not break the slot. `islands` still counts every
  inner ring, as before.
- A level face whose boundary walk met a knot (a hole tangent to the outer
  ring shares one vertex with it) had only `Open` loops, so it had no
  outer ring and was not read at all. The new `recognize::pinch` takes the
  walk's split rings as the outer ring and its holes, but only when it can
  prove them:
  - each ring is simple, and each span of each ring is a boundary edge of
    the face;
  - each inner ring has a vertex off the outer ring, and those vertices
    stand inside the largest ring;
  - the largest ring's area less the inner rings' areas is the face's
    facet area (relative slack 1e-3).
  Any other knotted face stays refused, as before.

**Why.** The reported part (p01-flange) has two through holes tangent to
the slot wall (holes at x 39..45, wall at x 45). The face had no outer
ring, and `detect_pockets` returned no feature. With the holes moved off
the wall, the slot read as an open pocket, because any inner ring refused
the slot. A hole drilled through a slot floor is common, and the slot
cycle machines the channel the same way with or without it.

**Dropped.**

- Split knotted rings in `geometry::feature::build_loops`. That walk is
  shared by every pick and region builder. The `Open` refusal is its
  honesty rule, and a face's identity (`FaceId`) is built from its rings,
  so the change would move the ids of saved picks. The proof is kept in
  the one reader that needs it.
- Refuse a slot only when an inner ring lies near its centreline. A boss
  anywhere in the channel is cut by the slot cycle, so any boss refuses.

**Effect on programs.** None directly: `detect_pockets` lists more slots,
and only a `create` makes an operation. A floor with a boss reads as
before.

**Cost to change later.** One predicate and one small file in
`recognize`; the geometry walk is unchanged.
