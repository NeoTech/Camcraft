## #2762 - The fill triangulation bridges a hole to a walk vertex first

**What was decided.** `merge_hole` in `crates/geometry/src/geo2d.rs` (the hole
bridge of `Region::triangles`) tries the walk vertices first, nearest
first. The computed ray hit on an edge is now the last candidate. Before, the
ray hit came first.

**Why.** The four red frame-cost UI tests on main came from the product
code, not from the test helper. After #2760 the fig8 mesh is finer. The 2D
Adaptive Outside region of the biggest fig8 patch has a hole whose rightmost
vertex looks onto a steep, short outer edge. The ray hit on that edge rounds
to a point 3e-16 inside the material. The ear clipper then reads that bridge
point as strictly inside the ears beside it and stops with "no ear in a
non-triangle walk", so `preview_fill` gave `None`. A small region with the same
edge gave a fill with a lost part (1814 of 2223 mm2) and no error. A bridge to
an existing vertex adds no new point. Eberly's construction ("Triangulation by
Ear Clipping", hole bridging) shows that a walk vertex with x >= r.x is
visible when the ray hits an edge. The same candidate checks (no proper
crossing, on material) still accept or refuse each bridge.

**Considered and dropped.**
- Change the test helper to pick another patch or Inside. Dropped: the
  region is legal and the product gave a wrong preview. A helper change would
  hide the defect.
- An epsilon in the ear tests (`tri_contains`, `is_left`). Dropped: it
  changes every ear decision of every fill, and a tolerance can accept a
  wrong ear.
- Remove the ray candidates. Dropped: they stay as the last resort for a case
  that no vertex can bridge.

**Cost to change later.** Local to `merge_hole`. The pin is
`region_triangles_bridge_a_hole_to_a_steep_edge` in
`crates/geometry/tests/geo2d.rs`. The triangulation is not display-only:
`motion::entry::dive_room` takes the centroid of the largest clear triangle
as the plunge or helix point. A region with a hole can thus get a different
dive point. Each candidate point is still checked with `Region::contains` and
`boundary_clears`, so a changed point stays inside the room. The motion
suites and the golden CL and G-code suites were run to show the effect.
