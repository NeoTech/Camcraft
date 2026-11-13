## #1568 - The STEP face evaluation cap gets measured headroom, and a finer pass over a face proven continuous falls back to the coarser mesh

**Decided.**
- `FACE_EVAL_CAP` (`crates/app/src/geometry/import/step_guard.rs`) goes
  from 25 M to **150 M** evaluations per face per pass, a fixed constant.
- A **refinement** pass (one after a coarser pass met) that trips the cap
  on a face **proven continuous** stops refining and keeps the coarser
  pass's mesh, reported at that mesh's own chord tolerance, with a warning
  in the log - the same way a pass over the facet budget is dropped. The
  import then carries the usual reduced-fidelity note when that chord is
  coarser than the 0.005 mm promise.
- Every other trip still refuses the model by name, as #1536 decided: a
  trip on the first pass (no coarser mesh exists), and any trip on a face
  that is not proven continuous.
- "Proven continuous" (`step_continuity.rs`) is read from the surface's
  definition, never from its mesh: planes, cylinders, cones, spheres and
  tori; a B-spline whose every interior knot multiplicity is at most its
  degree, with finite control points and knots; a NURBS that also has
  every weight finite, positive and at least 1e-9 of the net's largest
  weight (review of PR #1578: a weight of 1e-300 is positive but nearly a
  pole, whose narrow spike a coarser mesh checked at one point per cell
  can miss; real CAD weights stay within a few decades of each other).
  Swept surfaces are not proven. **And** no pass of the import may have
  divided the face outside the surface's own parameter range (a
  millionth of its span of slack). The flag is sticky across passes
  (review): truck derives each pass's range from that pass's tolerance,
  so the coarser pass whose mesh the fallback keeps may be the one that
  went wild while the finer pass that tripped stayed in range.
- When several faces trip in one pass, the pass counts as continuous only
  if EVERY tripped face is proven and in range (review: reporting only the
  first tripped face let a proven face's trip mask a jump face's, and the
  fallback would have kept the jump's smear). The refusal names the first
  tripped face that is not proven.
- When the fallback keeps a coarser mesh, the log says why ("a finer pass
  ran past a face's evaluation cap - a larger facet budget will not
  help") instead of "bounded by memory". The user-facing reduced-fidelity
  refusal still advises a larger facet budget, because the import records
  only the chord, not its cause; carrying the cause there needs a stored
  field and a project-format change, filed as a follow-up.
- A continuous face that trips on its first pass is refused with its own
  wording ("did not finish ... at the coarsest tolerance (a continuous
  surface, too dense to divide)"), not called discontinuous.

**Why.** The corpus (M5's review of #1546: 75 STEP files from testdata
and users' models, release) measured healthy faces far above what #1536's
decision assumed ("a few million"; "a face needing 25 M would be ~7 M
facets from one face" - both wrong: a trimmed NURBS face is divided over
its whole untrimmed range, so its evaluations do not follow its facets).
Peak evaluations on one face in one pass:

| Model | Peak on one face | Note |
|---|---|---|
| 32050.STEP | 23.1 M | default budget, 4.97 M facets; 92 % of the old cap |
| 91251A245 / A716 / A349 socket-head screws | 19.2 / 17.8 / 16.4 M | default budget |
| 10208, 22588B, 91864A770, 92220A263, 26250 | ~7.3 M | default budget |
| 10208.STEP face #1073 | 28.07 M | 60 M budget (`step_budget::CEILING`); converges; the old cap refused it |
| bspline_discontinuous.step (#1536) | runaway | reached 41.7 M uncapped before it was stopped |

150 M is 5.3x the heaviest healthy face measured at the ceiling budget.
A runaway face spends it in 10 s (the #1536 fixture) to 15 s
(sc_logo.step's face #836) in release, through `camcraft-cli mcp`
`import_model`; the default-cap pin takes ~27 s in debug.

The fallback rule rests on a property of truck's subdivision: it splits a
cell while the surface at a point near its middle stands off the bilinear
patch of the corners by more than the tolerance. On a surface continuous
over a closed parameter box (so uniformly continuous) that deviation goes
to zero as the cells shrink, so the subdivision ends at every tolerance.
A face proven continuous that runs out on a finer pass was only
expensive, and the coarser pass's mesh is a converged mesh at its own,
honestly reported tolerance. A face that may jump has no such guarantee:
any pass wider than its gap converges with the gap smeared into a wall.

**Considered and dropped.**
- *Scale the cap with the facet budget.* The facet budget does not bound
  one face's evaluations (23 M on one face of a 4.97 M-facet model); a low
  budget (down to `step_budget::MIN`, 1,000) would lower the cap below a
  healthy face's need; and a broken face's time would grow with a user
  setting (to ~30 s at the ceiling). A fixed cap keeps the refusal time a
  property of the program, not of a setting.
- *250 M (the issue's "about 10x").* Measured, it took 17 s on the
  fixture and 25 s on sc_logo.step - past the 12-15 s the issue asked
  for. With the continuous-face fallback in place, the cap only has to
  cover a face that is not proven continuous, or a first pass, which is
  coarse (a quarter of the model's extent); 5.3x the heaviest measured
  face does that.
- *Keep the coarser mesh whenever it is a closed shell of the right
  volume.* Measured on the #1536 fixture with the proof forced true: the
  smeared jump imports as 12 facets at chord 0.125, a closed box of
  volume 1.0 - exactly the "sound" shape such a check would accept. The
  shell's closure and volume cannot see a smear; the surface's own
  definition can. (A STEP file states no volume to compare against.)
- *Prove continuity from the surface alone, ignoring the divided range.*
  Tried first. sc_logo.step's face #836 is a rational degree-(4,2) corner
  blend with positive weights - proven continuous - and it still never
  converged: truck divided it over u from -2e51 to 1 (a trimming bound
  gone wild at its collapsed corner), far outside the patch, where the
  extrapolated rational surface is not continuous. Without the range
  check the import kept a 140-facet mesh at a 3.75 mm chord: the smear
  #1536 refused. With it, the face is refused by name in 15 s.
- *Detect non-convergence by the grid's growth pattern.* Heuristic, and
  needs per-sweep instrumentation inside truck's private recursion.

**Limits, and the cost to change later.** The cap is one constant;
raising it costs only time on a broken face. The continuity proof is
conservative: a B-spline with an interior knot of full multiplicity
(degree + 1) whose two control rows coincide is continuous but not
proven, and a swept surface is never proven, so a trip on a refinement
pass over either still refuses; a coincident-row check or a swept-curve
proof would widen it. Edge curves are still divided unmetered (#1545).
The corpus files are users' own and are not committed; the pins are
synthetic: an egg-crate bicubic patch that needs 39.3 M evaluations at a
6e-5 chord (`step_guard_tests.rs`; ~8 s release, ~35 s debug, so it
rides the heavy lane, with an instant default-lane twin that the cap stays
at least 5x the 28.07 M face), and lowered caps over the dome and
discontinuous fixtures (`crates/app/tests/import_step_guard.rs`). The
default-cap refusal of the #1536 fixture now takes ~27 s in debug and
stays in the default lane.
