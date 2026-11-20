## #2713 - A refined STEP face has no flat cap facets

**What was decided.** The STEP importer flips edges after it splits a curved
face (`geometry/import/step_fit.rs`, `relax`). The rule is Lawson's edge
flip, measured in space: an inside edge whose two opposite angles sum to
more than a half turn is replaced by the other diagonal of its two facets.
A flip is made only where the two new facets keep their winding in the
parameter plane and the new edge passes the chord-height limit of the
split. No point moves. No boundary edge changes. Only a face that the split
of #2600 changed is flipped.

**Why.** The test `feature_edge_g1_join` was red on `main` for the fixture
`bar_cross_holes.step` (116 split pieces). The join law was not the cause.
The split of #2600 measures facet edges only. It left flat "cap" facets on
the bar's wall: three points on the cylinder, the apex over the middle of
the long edge. Each edge of such a facet is within the limit, but the
facet's normal stood up to 65 degrees off the radius (2 147 of 131 834
wall facets over 25 degrees). The feature graph read the edges of these
facets as creases, and some facets as faces of their own. The pick showed
3 701 edges on the bar; most were on the smooth wall. Those false edges
were the split pieces. After the flip the worst wall normal is 6.5 degrees
off, the bar has 642 pickable edges and 0 split pieces.

**Considered and dropped.**

- *Keep the fixture out of the test.* The issue permits it. It makes the
  gate green, but the user still sees false edges on the wall of each bar
  or tube with a cross hole, and facet normals that are wrong by 40 to 65
  degrees stay in the mesh.
- *Change the join law (`feature/g1_seam.rs`, `feature/trail.rs`).* The
  law is correct for real edges. A law that joins false creases makes them
  longer, not absent.
- *Make the crease rule of the feature graph ignore thin facets.* This
  changes face and chain ids of every model (saved selections, goldens) to
  hide a defect of one importer.
- *Flip in rounds over the full face.* The first version did this. The
  import of the bar took 18 s in a debug build (2.6 s before). The queue
  version builds the edge table one time; the import takes 5.5 s.

**What it costs.** The import of a face that the split changed takes
longer (the bar: 2.6 s to 5.5 s in a debug build). The facet count does
not change. A face that the kernel meshed correctly is not changed, so
other models import as before. The flip is capped at 8 flips for each
facet of the face; a cap facet that is left then stays as it was.
A cap facet whose long edge is on the face's boundary cannot be flipped
and stays.

## #2713 - Add the missing kernel fixture row in this PR

**What was decided.** Add the triage row for `bar_cross_holes.step` to
`crates/kernel/tests/step_agreement.rs` in this PR. The import refuses with
`solidcore refuses: #655 at read: pcurves failed on`. The mesh reports
`no exact solid to judge against`. The test `step_bar_cross_holes` pins
these results.

**Why.** The review found that the fixture has no row. The agreement test
must cover every STEP fixture. The issue requires both this test and
`feature_edge_g1_join` to pass.

**Considered and dropped.** File a separate issue for the missing row.
That leaves the agreement test red after this PR.

**What it costs.** Run the kernel agreement suite explicitly. A later
change to the fixture requires a new triage of its agreement row.

## #2713 - Keep the measured bound when merging #2718

**What was decided.** Keep the split and flip limit at
`tol / kind.inside()`. Keep the measured height from #2718 and include the
largest height of every new diagonal accepted by the flip. Multiply this
bound by `kind.inside()` when reporting the mesh error.

**Why.** The split measures the edges before the flip. A new diagonal can
have a larger chord height than those edges. The reported bound must cover
the edges that the flip adds, as well as the edges already measured.

**Considered and dropped.** Keep the old measured bound without the new
diagonals. That bound can understate the error of the mesh after a flip.

**What it costs.** Each accepted flip updates the largest height. No extra
surface evaluation or pass over the mesh is needed. The bound can retain
the height of an edge that a later flip removes, which is conservative.
