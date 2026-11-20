## #2719 - STEP import: a curved face that the fit cannot measure is logged and makes the model reduced fidelity

**What was decided.** When the curved-face fit (`step_fit.rs`, #2600)
cannot read a face's mesh, it returns the reason. The reasons are: no
surface parameters, parameters that do not give the mesh's points, a facet
that is not a triangle, a point index that the mesh does not hold, too
many points, a tolerance that is not positive, and no surface in the shell.
The import then:

- keeps the kernel's facets of that face;
- logs `step: face <n> keeps the kernel's facets unmeasured (<reason>);
  the model is reduced fidelity` at warn level;
- reports the face's height as `UNMEASURED` (1000 file units). The model
  is then reduced fidelity (#864), and verify refuses its operations.

**Why.**

- The issue permits two ways: measure the kept mesh, or mark the model
  reduced fidelity. To mark it is the smaller change, and it is sound: a
  face that was not measured is never verified as exact.
- To measure the kept mesh needs a nearest-point search on the surface.
  For a mesh with no parameters there is no start point for the search, and
  truck's search on an unbounded cylinder or cone without a start point can
  fail. A failed search then needs a second fallback.
- 1000 file units is past every verdict tolerance in each unit from the
  micrometre to the kilometre, and under `model_fidelity::UNKNOWN_MM`
  (1e12 mm), so the "saved before v58" words of that sentinel do not show.

**What was considered and dropped.**

- Measure the kept mesh by a nearest-point search (see above). Dropped for
  now; it is the better result for a user, because a fine kernel mesh would
  stay at full fidelity.
- Refuse the import (an infinite chord fails at `step.rs`): too strong for
  a model that only has one face that is not measured. Dropped.
- Reuse the unknown-fidelity sentinel: its words say the project is old.
  Dropped.

**Cost to change later.** `judged()` in `step_fit.rs` is the one place.
A measure can replace the `UNMEASURED` value there, with no change to the
import. The refusal text that the user sees is the general
`TOO_COARSE` text ("re-import with a larger STEP facet budget"). A larger
budget does not help this face; the log line names the true cause.
