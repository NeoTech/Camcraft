## #2440 - Holes in a slot floor, and a hole tangent to the slot wall

**Decided.**

- The slot reader (`recognize::shape::slot_of`) refuses a floor with a
  boss in it, not a floor with any inner ring. An inner ring is a boss when
  there is material above the floor inside it (`walls::Mesh::rises_from`).
  Two tests find such material:
  - a facet off the floor on one of the ring's edges stands above the
    floor;
  - a facet off the floor reaches above the floor and overlaps the inside
    of the ring, seen from above (`geo2d::overlap::triangle_overlaps_ring`,
    with a 1e-5 mm slack on the ring's boundary).

  The second test finds a boss that stands in a recess below the floor.
  All facets on such a ring go down, so the first test alone does not see
  the boss (review of PR #4320). A hole through the floor has nothing above
  the floor inside it, so it does not break the slot. `islands` still
  counts every inner ring, as before.
- A level face whose boundary walk met a knot (a hole tangent to the outer
  ring shares one vertex with it) had only `Open` loops, so it had no
  outer ring and was not read at all. The new
  `geometry::feature::face_rings` takes the walk's split rings as the
  outer ring and its holes, but only when it can prove them:
  - each ring is simple, and each span of each ring is a feature edge
    between the face and another face;
  - each inner ring has a vertex off the outer ring, and those vertices
    stand inside the largest ring;
  - the largest ring's area less the inner rings' areas is the face's
    facet area (relative slack 1e-3).

  Any other knotted face stays refused, as before.
- The proof and the overlap test are in `camcraft-geometry`, not in
  `app`. They use only the feature topology and 2D points, so AGENTS.md
  (**`app` holds the document, nothing else**) puts them there. The app
  reader calls them. No new crate dependency is added.

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
  so the change would move the ids of saved picks. `face_rings` is a
  separate reader beside the walk.
- Refuse a slot only when an inner ring lies near its centreline. A boss
  anywhere in the channel is cut by the slot cycle, so any boss refuses.
- Test only the highest point inside the ring against the floor with a
  height probe. The facet overlap test needs no new data structure and
  also finds an overhang above the ring.

**Effect on programs.** None directly: `detect_pockets` lists more slots,
and only a `create` makes an operation. A floor with a boss reads as
before, also when the boss stands in a recess.

**Cost to change later.** One predicate in `recognize::walls`, one
function in `geometry::feature` and one in `geo2d`; the geometry walk is
unchanged.
