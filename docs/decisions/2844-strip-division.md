## #2844 - a runaway strip face is divided over its boundary nodes

**What was decided.** The kernel seam (`machining_mesh::planned`) calls
`mesh_redivide::redivide` on each plan, before the measure. A face that
the planner gave a runaway count gets a second division: the Delaunay
fill of its boundary loop, over its shared edge nodes only, with no node
inside. The seam keeps that division only when
`mesh_deviation::measure_within` proves it inside the tolerance. If not,
the face keeps the planner's triangles. The measure of the whole plan
runs after that, as before, so the guarantee of a machining mesh does not
change. A kept division is a note in the mesh report.

Readings on the four-turn thread flank at 5e-3 mm (debug):

| Plan | Planner | Boundary division | Measured |
|---|---|---|---|
| chord 5e-3 mm | 274192 triangles | 926 triangles | 7.3e-3 mm, not kept |
| chord 2.5e-3 mm (replan) | 391338 triangles | 1150 triangles | 3.8e-3 mm, kept |

The test time went from 68 s to 35 s. The two planner runs and the
measure of the first plan remain.

**Why.** This is option 3 of the issue comment of 2026-11-20. Options 1
and 2 are changes in `NeoTech/solidcore` and a move of the pin. Option 3
needs only the public fields of `TessPlan` and `FacePlan`. The shared
edge nodes (`fixed`) stay bit for bit, so the neighbouring faces meet
the face as before.

**Decisions inside it.**

- *Which faces.* Only a face with 10 000 planned triangles or more
  (`REDIVIDE_MIN_TRIANGLES`) and 16 or more for each boundary node
  (`REDIVIDE_RATIO`). The two numbers are our judgement, not measured
  optima. They keep the meshes of ordinary models as they are: no golden
  moves. Dropped: a division of each face that passes the measure. It
  would change the facet counts of many models for a small gain.
- *No node inside.* A face that needs inside nodes (a wide twisted or
  bulged patch) fails the measure and keeps the planner's triangles
  (`machining_mesh_redivide.rs` pins this). Dropped: rows of inside nodes
  by the curvature of each direction. That is the anisotropic planner of
  option 1, and it belongs in solidcore-tess.
- *One loop only.* A face with a hole, a pole, a triangle turned over in
  (u, v) (a period), a pinched boundary, or a boundary node that is not a
  shared one is not divided.
- *Own fill, not `solidcore_tess::triangulate_planar`.* The public
  triangulator cuts across a boundary run that is exactly collinear in
  (u, v) (the straight run-out trims of the flank), so its triangles do
  not keep the boundary. The fill here is ear clipping and Lawson flips
  (`mesh_redivide_fill.rs`), bounded at 8192 vertices because ear
  clipping is quadratic. The seam checks that the fill has the planner's
  boundary sides and one sense for each triangle. With these two facts
  the triangles cover the face one time.
- *The fill's plane.* (u, v) scaled by the mean lengths of the two
  partial derivatives at the boundary nodes, so that "near" in the plane
  is near on the surface.

**Not done.** The replan at half the chord remains (the first plan's
boundary division measures 7.3e-3 mm against 5e-3 mm, mostly the chords of
the edges themselves). The planner still makes its isotropic division
before the seam replaces it, so the import time of a threaded part falls
by about half, not by the factor of the facet count. The speed item of
#2780 stays open for that.

**Cost to change later.** Small. When solidcore-tess plans a strip
anisotropically, the planner's count falls under the two thresholds and
`redivide` does nothing. Then the two modules and the one call can go.
