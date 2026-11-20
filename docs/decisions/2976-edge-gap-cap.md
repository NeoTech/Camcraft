## #2976 - The STEP edge-gap cap follows the tolerance that the file states

**What was decided.** The maintainer chose option A (reduced fidelity) on
2026-11-20, after the measurement below. The worker set the details.

1. **Edge cap.** An edge may lie off its face's surface by up to
   `cap = clamp(2 x stated, 0.01 mm, 0.02 mm)`
   (`camcraft_kernel::step_edge_gap::EdgeGapCap`).
   - `stated` is the distance uncertainty of the file, in mm. Solidcore's
     units module reads it from `GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT` and
     converts it with the unit of the measure. An inch file converts.
   - When the file has more than one context, the least value is used,
     because the face's own context is not known at the check.
   - A file that states no uncertainty, or a value that is zero, negative,
     NaN or infinite, keeps the fixed cap of 0.01 mm
     (`MAX_EDGE_GAP_MM`, unchanged).
   - No file raises the cap past `EDGE_GAP_LIMIT_MM` = 0.02 mm.
   - The floor of 0.01 mm means that a file that states 1e-6 mm keeps
     0.01 mm. No tracked fixture changes its verdict.
   - The lift check of a made pcurve uses the same cap (it used 0.01 mm).
2. **Refusal text.** A gap over the cap is refused by name, with the gap,
   the cap and the source of the cap. For example: `edge 0 lies 0.0210 mm
   off the face's surface, over the cap of 0.02 mm; the file states a
   tolerance of 0.01 mm, and the reader accepts twice that, from 0.01 mm
   to 0.02 mm`. A file without a tolerance gets `...; the file states no
   tolerance`. The start of the old text stays, so the existing pins
   (`bspline_discontinuous.step` at 0.25 mm) did not change.
3. **Looser mesh.** The machining mesh cannot meet the chord on a face
   whose edges lie farther off the surface than the chord (finding below).
   The importer (`camcraft_geometry::step::read`) then meshes the model
   one more time at `min(2 x stated, 0.02 mm)`. This happens only when all
   of these are true:
   - the mesher refused at the chord with `FaceCapped`, `EdgeCapped` or
     `Deviation`;
   - the file states a tolerance;
   - a kept edge gap is larger than the chord;
   - the looser tolerance is coarser than the chord.

   The chord that the importer hands out is that looser tolerance. Thus
   the existing reduced-fidelity law (#864: `step::coarse`, `coarse_mm`,
   the `ReducedFidelity` component, `model_fidelity`) flags the model and
   judges each check against it. The fidelity law is not derived a second
   time. No schema or project-file change was necessary.
4. **Notes.** These are the notes that the user sees:
   - The importer note, in plain words. The geometry crate writes the
     facts: "This file's edges sit up to 0.018 mm off their surfaces. The
     file states a tolerance of 0.010 mm. The model is used at reduced
     accuracy (0.020 mm)." The app adds what the fidelity law does at that
     chord (`geometry::import::step::loosened_consequence`, which reads
     `model_fidelity::excess`, `excess_floor` and `DEV_TOL`, the same law
     as `verdict_tol`):
     - excess over `DEV_TOL` (every chord over 0.015 mm, so 0.02 mm and the
       inch fixture's 0.01524 mm): "A program cannot be posted while this
       model is in the document. Export the file again with a tighter
       tolerance to post one."
     - excess inside `DEV_TOL`: "Gouge checks against this model are
       stricter, and an operation that they cannot check is refused at the
       post."
   - Review fix: the first text said "checks allow 0.020 mm". That was
     false. The checks are stricter, or the post refuses.
   - The kernel note for each kept gap over 0.01 mm: "an edge lies
     0.0180 mm off its face's surface; the file states a tolerance of
     0.0100 mm, so the reader accepts up to 0.0200 mm". The cap is never
     made larger without a word.
   - The "up to" gap is the larger of two values: the gaps that the kernel
     measured, and the edge tolerances that the reader raised, divided by
     its least margin of 1.5. This is an upper bound on the gap.

**Why.**
- `Lobe.step` (a user file, private, not in the repository) states
  0.01 mm. Its worst edge measures 0.0102 mm. A cap of 1 x stated refuses
  a file that is inside its own tolerance by any honest reading: the gap
  is sampled at 20 points, and the writer rounds its numbers. Two times
  covers this and stays small.
- The limit of 0.02 mm is 12 times under the 0.25 mm step that must stay
  refused. It is twice `DEV_TOL`.
- The safety argument does not depend on the edge cap alone. The machining
  mesh bounds each face against its true surface at the chord that the
  importer hands out (`machining_mesh.rs` deviation and capped-sag
  checks). A model meshed at 0.02 mm is flagged reduced fidelity, and
  every downstream check (program verdict, gouge gate, stock) uses that
  chord through `model_fidelity`.

**Measurement** (local harness, not committed; the gap of each bare edge,
as `cover_gaps` measures it, with the cap off for the run):

| File | Stated (mm) | Edges measured | Largest gap (mm) | > 0.001 | > 0.005 | > 0.01 |
|---|---|---|---|---|---|---|
| Lobe.step (private) | 0.01 | 28 | 1.015e-2 | 14 | 10 | 1 |
| fig8.step | 0.01 | 1 | 3.5e-12 | 0 | 0 | 0 |
| fillet_corner.step | 1e-6 | 2 | 8.2e-13 | 0 | 0 | 0 |
| bar_cross_holes.step | 2e-5 | 4 | 1.19e-6 | 0 | 0 | 0 |
| bspline_discontinuous.step | 1e-6 | 3 | 0.25 | 3 | 3 | 3 |
| twisted_flank, Milling-test (0.01), Milling-test-inch (0.01 converted), sample_ap214 (1e-14), 8 more (1e-6) | - | 0 | - | - | - | - |

With the cap off, `Lobe.step` reads as one solid. The reader raised 7
more edges to tolerances of up to 1.507e-2 mm. The machining mesh refuses
the solid at the chords 0.005, 0.01, 0.0125 and 0.015 mm ("face 1 capped",
sag 9.9e-3 to 1.56e-2 mm). It meshes at 0.02 mm: 10640 triangles,
deviation 1.947e-2 mm. With this PR, `Lobe.step` imports at a chord of
0.02 mm, with the note. The mesh volume is 113192.5 mm3, and the exact
kernel volume is 113295.5 mm3.

**Cost.**
- 0.02 mm is coarser than the verdict tolerance of the stock simulation
  (`DEV_TOL` = 0.01 mm) and four times the import chord (0.005 mm).
- On such a model, a gouge of less than about 0.02 mm is not judged.
- #864 already handles a coarse mesh this way: the program verdict
  tolerance is `DEV_TOL` minus the excess (`chord - CHORD_CAP`). At a chord
  of 0.02 mm the excess is 0.015 mm, which is more than `DEV_TOL`. Thus no
  verdict tolerance is left (`model_fidelity::verdict_tol` is `Err`). The
  program gate refuses an operation on this model by name when its own
  floor does not cover the excess (`op_problem`, `excess_floor`).
  `VouchModelFidelity` does not apply, because the chord is a recorded
  measurement.
- `gate_problems` adds the `verdict_tol` refusal when no operation is
  refused by itself. Thus at a chord of 0.02 mm the post refuses each
  program while the model is in the document, also when each operation
  leaves stock that covers the excess.

**The advice of the reduced-fidelity texts (review fix).** The texts
said "re-import with a larger STEP facet budget". For a model that is
coarse because of its edge gaps, that advice is wrong. The document does
not record why a model has reduced accuracy. Issue #2996 adds that record;
then each text names the one cause. Until then each text names the two
remedies: "re-import with a larger STEP facet budget, or export the file
again with a tighter tolerance". This applies to `model_fidelity::TOO_COARSE`,
the MCP note (`model_fidelity::facts`), the refusal of a kind that cannot
read the excess (`op_problem`), the vouch refusal of a recorded chord, the
model-page hint and the `import_model` tool description. Dropped: a stored
cause in this PR (a schema and project-file change that #2996 makes
anyway).

**Posting waits for #2996.** The maintainer decided on 2026-11-20 to
widen the verdict tolerance for such a model in a follow-up (#2996).
Until then, on a model that opens at a chord of 0.02 mm, the user can:
- open the model, see it, measure it and set up stock and fixtures;
- make and generate operations (the gate names each operation that it
  cannot verify);
- read the reduced-fidelity flag and the note.

The user cannot post a program while this model is in the document.
Export the file again with a tighter tolerance to post one.
- The user sees the reduced-fidelity flag on the model page, in the MCP
  reply and in the project file, and the note names the gap and the
  file's tolerance.

**Considered and dropped.**
- A larger fixed cap for each file (for example 0.02 mm). Dropped: a file
  that states 1e-6 mm would accept a gap 20000 times its own tolerance.
- Keep the refusal (option D). Dropped by the maintainer: the user cannot
  open a file that is inside its own tolerance.
- 1 x stated. Dropped: `Lobe.step` measures 2 % over its own stated value.
- A file that states a tolerance over the limit gets the fixed 0.01 mm
  cap (the first brief). Dropped for a clamp at 0.02 mm (the maintainer's
  rule). The two keep every gap over 0.02 mm out.
- A loose mesh for every refusal of a file that states a tolerance.
  Dropped: the looser mesh is only for a model whose measured edge gaps
  explain the refusal. A planner limit on a sound model stays a refusal.

**Limits found (not exercised as a pass).**
- A file that states a large tolerance (5 mm, `edge_gap_021_stated_5mm`)
  does not reach the kernel's gap check. The reader fits those pcurves
  itself at the loose tolerance. No gap is measured, so the importer has
  no reason to mesh looser. The machining mesh refuses the 0.021 mm step
  at the chord by name. This is safe, but a 0.018 mm gap in such a file
  is also refused.
- A file that states no tolerance, with a gap between the chord and
  0.01 mm (`edge_gap_009_unstated`), passes the edge check. The mesh at
  the chord refuses it, as before #2976.
- The new fixtures are in `testdata/models/edge_gap/`, not in
  `testdata/models/`. The STEP agreement bench (`step_agreement.rs`) needs
  a recorded truck reading for each top-level fixture. The truck reader
  was removed in #2790, so a new fixture has no such reading, and a row
  with an invented reading would be false.

**What it would cost to change later.** The rule is in one kernel module
(`crates/kernel/src/step_edge_gap.rs`: `STATED_FACTOR`,
`EDGE_GAP_LIMIT_MM`, `MAX_EDGE_GAP_MM`) and one retry in
`crates/geometry/src/step.rs`. The pins in `crates/kernel/tests/step_edge_gap.rs`,
`crates/geometry/tests/step_edge_gap.rs` and
`crates/app/tests/import_step_edge_gap.rs` name the numbers.
