## #917 - kernel and geometry parity for the solidcore STEP path (PR B0)

PR B0 holds the items of #917 that we can pin under the app. PR B (the
switch of the app to solidcore) is stacked on it.

### The coordinate snap (`camcraft_kernel::mesh_weld`)

- **Decided.** Before the exact-position weld of the machining mesh, each
  coordinate goes through one rule:
  - The grid is 1e-3 mm (`SNAP_GRID_MM`). The window is 1e-9 mm
    (`SNAP_WINDOW_MM`).
  - `k = round(c / 1e-3)`, `g = k / 1000` (one IEEE 754 division).
  - If `|c - g| <= 1e-9`, the coordinate becomes `g + 0.0`. Otherwise it
    stays bit for bit. A coordinate that is not finite, or with
    `|k| > 9e15`, stays.
  - `+0.0` replaces `-0.0`. Equal inputs always give equal outputs.
- **Degenerate facets.** A triangle whose corners snap or weld onto a
  repeated vertex has no area. The weld drops it. The side count after
  the weld (`sides`) then refuses the mesh with `NotWatertight` if the
  drop opened a hole. No facet is moved past the window to close a gap.
- **Why.** The evaluator gives 20.000000000000004 for a face at Z 20. The
  app's flat-level and floor logic compares levels exactly, so a level
  read off the mesh by one ulp gives a wrong floor. The window is far under
  any machining tolerance (the smallest tolerance is a millionth of the
  body). A vertex moves at most one window in each axis, sqrt(3) x 1e-9
  = 1.73e-9 mm (`SNAP_MOVE_MM`), so the measured deviation stays true to
  1.73e-9 mm. The mesher adds `SNAP_MOVE_MM` to the measured deviation
  before it compares it with the tolerance, so the judged figure holds
  for the snapped mesh too (review of #2843).
- **Dropped.** A snap to the 1e-6 grid (it would move true model values
  such as 12.5000004). A relative window (a large part would move more).
  A snap after the weld (two near copies of one vertex then weld
  differently).
- **Pinned by** `the_snap_takes_the_last_bit_noise_off_a_grid_value`,
  `a_coordinate_outside_the_window_is_never_moved` (12.5 +/- 1e-6 stays)
  and `the_bore_mesh_stands_on_its_exact_levels`.
- **Test location.** The coordinator asked for the snap pins in
  `crates/geometry/tests`. They are in `crates/kernel/tests` because the
  snap is in the kernel's machining mesh, and `geometry::step` is not on
  `main`. A geometry test would need a new dependency edge
  (geometry -> kernel), which the law does not permit without a decision.
- **Cost to change.** One function and two constants.

### An inward solid is turned outward (`machining_mesh`)

- **Decided.** A watertight, consistently wound mesh of ONE shell (one
  group of triangles joined side to side, `mesh_weld::shells`) with a
  finite negative volume has all its faces pointed inward. The mesh swaps the
  second and third corner of each triangle, negates the volume and adds
  the note "the solid's faces all pointed inward (volume -X mm^3); the
  mesh is turned outward". A zero or NaN volume is still the `Volume`
  refusal. A mesh of two or more shells (a solid with a void) with a
  negative volume is also the `Volume` refusal: the sign of the sum does
  not tell which shell is turned, and a turn of all shells would make the
  void solid (review of #2843).
- **Why.** Main's truck path turned such a solid outward too
  (`step_outward.rs`). A closed shell with all senses turned over is the
  same solid.
- **Dropped.** A refusal (a legal file would be refused). Turning only
  part of the faces (a mixed winding is already refused by `sides`).
- **Pinned by** `an_inward_solid_is_turned_outward_and_said` and
  `a_solid_with_a_void_and_its_outer_shell_inward_is_refused`.

### A breaking edge or swept spline is refused by name (`step_curve_guard`)

- **Decided.** Before the topology read, the reader scans each B-spline
  curve that an `EDGE_CURVE` (through `SURFACE_CURVE`, `SEAM_CURVE`,
  `INTERSECTION_CURVE`) or a swept surface
  (`SURFACE_OF_LINEAR_EXTRUSION`, `SURFACE_OF_REVOLUTION`) uses. It refuses
  a weight that is not positive. It refuses an interior knot whose
  multiplicity is above the degree when the control polygon jumps there
  by more than 1e-9 x max(1, |coordinate|). The message names the role
  ("edge #N" or "swept surface #N"), the curve, the knot, the
  multiplicity, the degree and the jump, and says that the curve needs a
  subdivision.
- **Why.** Such a curve is two curves in one entity (ISO 10303-42 permits
  a multiplicity of degree + 1 only at the ends). solidcore evaluates it
  as one continuous curve and the mesh then has a gap or a wrong edge. A
  repeated knot with no jump is continuous and still imports.
- **Dropped.** A split of the curve at the knot (it changes the model's
  topology, which is the exporter's job).
- **Pinned by** the four tests in `crates/kernel/tests/step_curve_guard.rs`.

### Adaptive refinement of the deviation bound (`mesh_deviation`)

- **Decided.**
  - `measure_within` refines a sub-triangle until its bound is in the
    tolerance, down to depth 12, with a budget of 2^21 new samples for
    one plan (3 for each split). Past either cap the bound found so far
    is the judged one. `measure` keeps depth 3.
  - The bound of each sub-triangle also takes the minimum with
    `dmax + longest side`. The distance to a surface is 1-Lipschitz, and
    no point of a flat triangle is farther from a corner than its longest
    side, so this is a true upper bound.
  - The bound is one-sided. It bounds the distance from each point of
    the mesh to the surface. It does not bound the distance from each
    point of the surface to the mesh. A surface that folds back between
    two samples could stand off the mesh by more than the tolerance with
    no refusal. The algorithm is not changed for this (review of #2843).
- **Why.** The fillet fixture has a pole. A sliver at the pole spans the
  full u range in parameters, so the parametric Hessian bound did not
  shrink with a split. The bound was about 12 times the true deviation and
  a good mesh was refused. The Lipschitz bound shrinks with each split.
- **Dropped.** A depth cap only (it cannot stop a plan with many bad
  triangles from taking a long time). A larger fixed depth for `measure`
  (the bench timings would change).
- **Pinned by** `the_fillet_fixture_meshes_at_the_app_chord`.

#### Changed pins (`crates/kernel/tests/machining_mesh.rs`)

The tighter bound proves plans that the old bound could not prove. Each
fixture that we measured now meshes at its first plan:

| Fixture | Tolerance (mm) | Judged bound (mm) | Replan |
|---|---|---|---|
| Solidcraft sample | 0.005 | 0.0049999913 | no (was yes) |
| Solidcraft sample | 0.1 | 0.0999999405 | no (was yes) |
| fillet_corner.step | 0.005 | 0.0049998025 | no (was refused) |

- `solidcraft_sample_meshes_within_the_app_chord`:
  - old: the notes contain "replanned";
  - new: the notes do not contain "replanned" (the first plan is proven).
- `solidcraft_sample_still_refuses_an_out_of_tolerance_mesh`:
  - old: `coarse_01.largest() > 0.1` (the coarse 0.1 mm plan measured over
    0.1 mm);
  - new: `coarse_01.parametric > 0.1 && coarse_01.largest() <= 0.1` (the
    parametric reading still over-reads; the certified bound proves the
    plan).
  - Kept: the coarse plan exceeds 0.005 mm; the 0.1 mm request passes with
    a parametric reading over 0.1 mm; the spline bump on box.step is still
    refused with `Deviation` at face 0.
- The replan branch is still run by the bump case (replan, then refusal).
  No tracked fixture now passes only after a replan.

### Solid ranges in the mesh (`camcraft_geometry::geometry::mesh_solids`)

- **Decided.** `Mesh` has a `solids` field: one facet range per closed
  solid, empty for one soup. `Mesh::from_solids` refuses ranges that do
  not tile the facets in order. `slice::slice_of(facets, ranges, z)` slices
  each solid on its own and unites the sections when the whole soup meets
  the plane non-manifold. A solid with a volume that is not positive keeps
  the soup's refusal.
- **Why.** A STEP file with solids that share a wall line (the AP214
  sample) gives one soup in which the shared edges are used four times.
  The union fallback (`closed_solids`) must refuse it, because it cannot
  tell the solids apart. The importer knows the solids, so it keeps them.
- **Dropped.** Radial pairing of the facets around a non-manifold edge
  (a guess where the importer has the truth). One part for each solid in
  the app (it changes the model tree that the user sees).
- **Cost.** The literal `Mesh { facets }` no longer compiles; eight app
  tests use `Mesh::from_facets`. A transform that builds a new mesh from
  facets drops the ranges, which gives the old soup behaviour.
- **Pinned by** the five tests in `crates/geometry/tests/mesh_solids.rs`.
  The app reads the ranges in PR B.
