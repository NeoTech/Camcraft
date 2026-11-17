## #2246 - the area preview is triangulated once per draft, not on each frame

The maintainer's words (2026-11-16): "2235, selecting eddges performs
well.. that "visualized" outside area in orange tho.. It dropped
framerate".

**What was decided.** The Boundary group's preview (`boundary_draft::
Preview`) now keeps the triangles of its fill (`Preview::fill`, an
`Arc<[[Point2; 3]]>`). `refresh_preview` makes them once, when it makes
the region, under the same key: the draft parameters, the draft area, the
geometry source and the document version. The scene (`world::collect`)
reads the stored triangles. Before, it ran `Region::triangles` on each
frame (15.1 ms of the 30.0 ms still frame of the #2238 probe's state c,
release).

**Why.** The fill is a pure function of the preview region, and the
preview already has a key that changes when the region can change. One
key for the two values cannot go stale apart. Display only: no cut
geometry, no G-code, no golden changes.

**Dropped.**
- A `frame_memo` cache keyed by the region: it needs an equality compare
  or a hash of the region on each frame, which is a cost of its own.
- Triangulation on a worker thread: it is not necessary while the build
  happens only on a draft or document change.

**Cost to change later.** Small. The fill is a display value in UI state
and is not in the project file.
