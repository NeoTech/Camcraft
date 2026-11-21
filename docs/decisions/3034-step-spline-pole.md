## #3034 - A triangle corner at a spline pole takes its u from the triangle

**What was decided.** The machining-mesh deviation measure
(`camcraft_kernel::mesh_deviation`) now looks for a corner that sits on a
pole of a spline face. At a pole, the derivative along one parameter is
zero, so every value of that parameter gives the same point. The new
module `mesh_pole` moves the free parameter of such a corner to the mean
of the other two corners' values. The two other corners are taken on one
side of the period. The move is kept only when the surface point stays
within 1e-9 mm. Analytic faces do not use the corner parameters and do not
change.

**Why.** On upstream/main (8be100737) the QA part q02 (a SolidCraft mould
with an ellipsoid boss) did not import. The refusal was "face Some(10)
stands 5.089e-1 mm off its surface, tolerance 5e-3 mm". The mesh was good:
its sag against the ellipsoid was about 0.004 mm. The planner gave the
boss apex node u = 0 (and u = 1), and the ring under it u near 0.5. The
measure interpolated u across half the period inside each fan triangle.
It then projected from a hint on the far side of the surface, and the
bound of that triangle became 0.29 mm. The bound did not change with a
finer chord, which showed that the measure was wrong and not the mesh.
With the fix, q02 imports as 48780 facets with no open edge. Its volume
is 97163.50 mm^3 (SolidCraft gives 97165.69 mm^3). It slices at
z = 19.6875, the height of the reported refusal.

**Dropped.**
- A fix in solidcore-tess, so that the planner gives the apex a `PoleNode`.
  solidcore is an external git dependency at a pinned rev. A change there
  needs a new rev, and the measure must stay correct for any plan that it
  gets.
- A larger tolerance, or no deviation check on spline faces. That hides
  a real gouge.
- A fall back to the tolerance weld in the slicer. The slice was never
  reached: the import refused first.

**Pins.** `crates/kernel/tests/mesh_deviation_pole.rs` (a hand-built fan on
a NURBS hemisphere, and the quarter-sphere solid end to end at the
kernel), `crates/app/tests/import_step_spline_pole.rs` (import, volume and
slices of `testdata/models/sphere_quarter_pole.step`), and the
`mesh_pole` unit tests. The STEP bench (`step_agreement`) has a row for the
new fixture, with `Truck::NotRecorded`. The q02 STEP itself is not in the tree: it is on
the QA host, and this session could not copy it.

**Cost to change later.** Small. The change is one call in
`mesh_deviation::face` and one module of about 60 lines. If solidcore
marks poles itself later, `settle` finds no corner to move on those faces,
and nothing changes.
