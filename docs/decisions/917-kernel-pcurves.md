## #917 - The kernel makes the pcurves that a STEP file does not give

**What was decided.**
- The solidcore pin moves to the fork's commit
  `2016386442dfee3b2a0c3a1bfdd4017c5254dac8` (#2781), by the procedure of
  `916-solidcore-step-bench.md`. The record at that commit has a `Blend`
  surface. The mesh bound and the mesh deviation give no closed form for
  it (`None`), as for a spline, so the measured deviation decides.
- The kernel crate implements the solidcore `Trims` seam
  (`solidcore_ops::pcurve::Trims`) in `crates/kernel/src/solidcore_trims.rs`
  (`KernelTrims`). `read_step` now calls
  `solidcore_step::topology::read_solids_with(&exchange, &KernelTrims)`
  instead of `read_solids`.
- A face that has an edge without a closed-form pcurve (a free-form edge on
  a spline face, for example a twisted B-spline flank) goes to the solidcore
  kernel alone, as an open one-face sheet. The uses without a pcurve go in
  bare (`solidcore_convert::brep::to_kernel_with_bare_uses`). The kernel
  makes their UV trims as it makes each trim it needs, and
  `solidcore_convert::brep::to_record` gives them back.
- The route is the maintainer's Solidcraft `SmlibTrims`, reused with his
  permission and moved onto the public API at the pinned solidcore commit.
- The trims run on the same kernel state as the import (`SolidcoreKernel::k`),
  not on a fresh kernel. Its tolerances, opt-in fixes and cancel token then
  hold for the trims too.
- Nothing is taken unchecked. Each of these is refused by name ("kernel
  pcurves: ..."), and the reader then refuses the solid:
  - an edge that lies more than 0.01 mm off its surface
    (`MAX_EDGE_GAP_MM`), whatever tolerance the file gives the edge. A
    smaller gap raises the edge and vertex tolerances to
    `SURFACE_GAP_MARGIN` times the gap, as the reader does;
  - an edge interval that the kernel moved past the edge's reach (the
    larger of its vertex and edge tolerances);
  - a made pcurve whose lift onto the surface misses its 3D edge by more
    than the held edge tolerance, but never more than `MAX_EDGE_GAP_MM`
    (33 samples, ends included);
  - a face, edge or vertex tolerance that is not a finite number of zero or
    more, and a lift sample whose distance is not finite (`f64::max` drops a
    NaN, so each is checked before it is used).
  The reader checks the whole record again after this
  (`record_check::agreement`). The interval and lift checks are in
  `crates/kernel/src/solidcore_trims/checks.rs` (the 400-line rule).
- New dependency edge: `camcraft-kernel -> solidcore-ops`, at the same
  pinned commit as the other solidcore crates. It is a crate of the solidcore
  workspace and is already in `Cargo.lock` through `solidcore-step`. No new
  third-party crate. No new edge between our own crates.
- The bench (`crates/kernel/tests/step_agreement.rs`) pins the new
  verdicts. `crates/kernel/tests/step_twisted_flank.rs` pins #2780 at the
  kernel: `twisted_flank.step` reads closed at 8000 mm3; the variant with a
  vertex off its curves is refused by name (observed: the surface would
  move 0.89 mm, over the reach of 0.51 mm), and so is
  `bspline_discontinuous.step` (the gap cap below).
- The two twisted-flank fixtures do not reach `KernelTrims`: the reader
  makes their pcurves itself (a probe found no call). `fillet_corner.step`,
  `bar_cross_holes.step` and `fig8.step` reach it, and
  `crates/kernel/tests/step_closed_form.rs` pins that route against closed
  forms. In `step_twisted_flank.rs`, only `bspline_discontinuous.step`
  reaches it (the gap cap).
- An inch file reaches it too: `fillet_corner_in_inches_reads_at_its_closed_form_volume_in_mm`
  edits the length unit of `fillet_corner.step` to the inch in memory and
  reads 25.4^3 times the closed form, in mm. The gap cap and the lift
  tolerance are in mm after the unit change.
- The two `Milling-test*.step` bench rows move from "disagree: bounding box"
  to "agree" (Refs #939). The port's box at 2016386 stops at the flat at
  Y = 19.719, as the file's vertices and both meshes do. This PR does not
  close #939; its owner confirms the fix there.
- The bench helper `step_util::solidcore_mesh` reads with `KernelTrims` too,
  so the sc-tess mesh is judged on the same record as the import.

**Why.** At the pinned commit the reader refuses a face whose free-form
edges have no pcurve ("pcurves failed on ..."). Real files have such faces:
a user STEP file with a twisted B-spline flank was refused. The kernel
already makes trims for its own booleans and blends, so the same code makes
these. The maintainer chose this route (option 2) over the alternatives
below.

**Considered and dropped.**
- Keep truck as a fallback for the faces that solidcore refuses. Dropped:
  two readers give two sets of answers, and the maintainer asked to remove
  truck.
- Our own pcurve fit (project 3D samples onto the surface, fit a 2D
  B-spline). Dropped: it is a second trim maker beside the kernel's, with
  its own tolerances to keep in step.
- Wait for the reader upstream to make the pcurves. Dropped: there is no
  date, and the user file stays refused until then.
- A fresh kernel for each face. Dropped: it loses the opt-in fixes and the
  cancel token of the import.

**Cost to change later.** Low. The seam is one trait. If the reader later
makes the pcurves itself, `read_step` calls `read_solids` again and
`solidcore_trims.rs` is deleted; the bench pins show any change of verdict.

### The lift tolerance (held edge tolerance, at most 0.01 mm)

**Chosen.** `lifts_onto` checks a made pcurve against
`min(held, MAX_EDGE_GAP_MM)`. `held` is the edge's tolerance from the file,
or as `cover_gaps` raised it. A loose file tolerance (or a raised one, which
can reach `SURFACE_GAP_MARGIN` times 0.01 mm) does not pass a pcurve that
misses its edge by more than the cap on an edge's gap.

**Measured** (a temporary probe, removed before the commit):

| Fixture | Held tolerance (mm) | Largest lift gap (mm) |
|---|---|---|
| fillet_corner.step | 1e-6 | at most 2.9e-7 |
| bar_cross_holes.step | 2e-5 | 1.2e-5 |
| bspline_discontinuous.step | 1e-6 | refused at the gap cap first |

On each fixture the held tolerance is under 0.01 mm, so the rule changes no
verdict. fig8, fillet_corner, bar_cross_holes and twisted_flank read at the
same volumes, and the two refused fixtures stay refused.

**Dropped.** The held tolerance alone: it follows the file, and a file can
give any tolerance.

### The edge-gap cap (0.01 mm)

**Chosen.** `MAX_EDGE_GAP_MM` = 0.01 mm. An edge whose measured gap to its
face's surface is over the cap is refused: "kernel pcurves: edge N lies G mm
off the face's surface, over the cap of 0.01 mm". The reader adds the solid
and the face (`#16 at read: pcurves failed on 1 of 6 faces; first: face
#53: ...`).

**Why.** With the reach of 1 mm, `bspline_discontinuous.step` (#1697) read
as a closed solid of 1.125 mm3: its edges stand 0.25 mm off a 1 mm part, and
the record raised their tolerances to 0.375 mm to cover that. On main it was
refused by name, so the 1 mm reach loosened a verdict. The maintainer's
Solidcraft keeps the same 1 mm limit (`MAX_TOLERANCE` = 1.0), so it would
take the file too. The cap is set from the largest gap that `KernelTrims`
measured on the fixtures that read:

| Fixture | Largest gap (mm) |
|---|---|
| fig8.step | 3.5e-12 |
| fillet_corner.step | 8.2e-13 |
| bar_cross_holes.step | 1.2e-6 |
| twisted_flank.step | no bare edge measured |
| bspline_discontinuous.step | 0.25 (refused) |

0.01 mm is about 8000 times the largest good gap and 25 times under the
0.25 mm step. It is not more than the sim's verdict tolerance (twice the
0.005 mm chord, #689), so an edge that is kept cannot move a verdict.

**Dropped.**
- The reader's 1 mm reach alone: it keeps the 0.25 mm step as a solid.
- A cap on the raised tolerance (`SURFACE_GAP_MARGIN` times the gap): the
  same test with a margin in it; the gap is the measured fact.
- A second guard on the reader's notes ("tolerance raised to ..."): the
  0.375 mm note came from `cover_gaps` itself, so the cap removes its cause.
  Not added, to keep one rule in one place.

### Bench verdicts against truck (fillet_corner, bar_cross_holes)

**Chosen.** The rows of `fillet_corner.step` and `bar_cross_holes.step` are
"disagree: volume, area" | "watertight, within tolerance". `fig8.step` is
"agree" | "watertight, within tolerance".

| Fixture | Main | Now |
|---|---|---|
| fig8 | solidcore refuses: #14 ... / no exact solid | agree / watertight |
| fillet_corner | solidcore refuses: #1289 ... / no exact solid | disagree: volume, area / watertight |
| bar_cross_holes | solidcore refuses: #655 ... / no exact solid | disagree: volume, area / watertight |
| bspline_discontinuous | both refuse (#16) / no exact solid | the same |

**Why.** Truck reads both files as open meshes (121 and 1090 open edges), so
its volume is short (20651.68 against 26816.49 mm3; 45816.21 against
78225.68 mm3). That is a defect of the truck path, which the app switch
(#917, PR B) removes. To pin the exact volumes apart from truck,
`crates/kernel/tests/step_closed_form.rs` checks each against:
- a closed form from the fixture's own dimensions, relative 1e-5:
  - fillet_corner: a 30 mm cube less three R 3.125 edge rounds and a
    spherical corner, 27000 - 3 (1 - pi/4) r^2 (30 - r) - r^3 (1 - pi/6) =
    26816.4939 mm3 (read 26816.49);
  - bar_cross_holes: a R 25 x 40 bar less two blind radial R 2.5 holes with
    flat bottoms at the file's z = 16.9707 and -16.9679, the hole volume
    integrated over the hole's disc = 78225.6805 mm3 (read 78225.68);
- solidcore's own closed sc-tess mesh, within the bench tolerance (1e-3);
- the bounding box.

**Dropped.** "agree" with a looser tolerance: it would hide truck's open
mesh. Dropping the rows: the bench must name each fixture.

### Port verdicts that the pin move changed (solidcore 55d0e76 -> 2016386)

The narrow batch gate found three kernel pins that the pin move changed.
KernelTrims is not the cause: none of these tests reads STEP.

- **Rim chamfer (`blend`).** The default path now gives the cone (upstream
  WR-421, solidcore#420). Chosen: the "not yet" test becomes
  `rim_chamfer_default_path_matches_the_cone`, as the test asked. It heals
  the result (`heal` hands out only a closed solid) and measures it against
  the cone's closed form, in volume and box. Dropped: a deleted test, and
  an unchanged "not yet" test with a loose bound.
- **Box at a NaN origin (`corrupt_input`).** The port now refuses it
  (`Err`); 55d0e76 built a non-solid box. Chosen: pin `Err`, with a comment
  that names the rev. The refusal is the stricter verdict. The other 14
  rows do not change.
- **Zone tolerance edits (`import_corpus`).** The writer now prints the
  zone tolerance as `1e-5`, not `0.0000100000000000`. The corpus edit was a
  literal `replacen`, so the zero, negative and NaN rows read the intact
  box. Only the NaN row failed, because the other two rows pin "Ok". The
  reader does not accept a NaN tolerance: `read_smb` refuses a non-finite
  one by name (it did before this change), but the edit did not reach it.
  Chosen:
  - the edit finds the header line by its label (`zone_tolerance`);
  - the corpus test asserts that each edit changed the file;
  - the new test `non_finite_zone_tolerance_is_refused_by_name` asserts the
    refusal text of `read_smb` and `import_smb` for NaN, inf and -inf.

  All pinned rows stay as they were, with the "replaced by 1e-4" notes of
  the zero and negative rows. Dropped: a second refusal in the importer.
  `read_smb` already refuses before the port parses, and `import_smb` calls
  it first. Also dropped: a refusal of a zero or negative tolerance. The
  importer replaces those values and reports the change in a pinned note;
  that is the earlier decision for the INTACT rows.
