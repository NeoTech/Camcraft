## #2210 - 2D area controls in an aligned setup

**Decided:** remove the refusal "area controls are not supported yet in an
aligned setup" for the four verified 2D kinds (Face, 2D Contour, 2D Pocket,
2D Adaptive). No new transform layer is added.

**Why:** each world read of the area law (`operation::boundary_ctx::build`)
already goes through the setup frame since #326 and #1743: the picked
faces and edges (`selection_geometry_in` with `pose_in_setup`), the
stitched open chains, the part material and its silhouette
(`setup_align::facets_in`) and the heights. The stock outline is
furniture and correctly stays unaligned. The refusal was a guard from
before those doors were verified for 2D. The new tests
(`crates/app/tests/setup_align_2d_area.rs`) turn an L-shaped part 90
degrees about Z and prove that the aligned document cuts exactly what the
same part with the turn baked into its mesh cuts. The cut avoids the
turned L, and it reaches a region that the unturned and the -90 L both
cover.

**Dropped:**
- A second transform of the area controls at generate time: it would
  turn the picks two times, because the reads are already in the setup
  frame.
- Keep the refusal with a better message: the maintainer reported the
  refusal itself as the P1 defect, and the reads are correct.

**Known gap, not changed:** `boundary_picks::extend_to_loop` (a UI helper
that picks "the most horizontal face") reads face normals through
`model::transform_of`, not the setup frame. For a turn about Z the result is
the same; for a flip or a free angle it can choose a different face. It only
seeds a selection; it does not cut.

**Cost to change later:** low. The refusal is one block in
`setup_align::generation_problem`; restoring it is a revert.
