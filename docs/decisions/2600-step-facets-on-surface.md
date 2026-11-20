## #2600 - A STEP face's facets are held to its surface

**What was decided.** The STEP importer now measures the mesh it keeps. For
each curved elementary face (cylinder, cone, sphere, torus) it measures the
chord height of every facet edge inside the face against the face's own
surface. An edge that stands more than twice the pass tolerance off the
surface is split at the surface, round by round, until none does
(`geometry/import/step_fit.rs`). An edge on the face's boundary is never
split, so the shell stays closed. A chord height that cannot be refined
away (the facet budget, the round cap) becomes the chord error that the
import reports. The model then carries reduced fidelity (#864) and the
gate refuses to verify against it.

**Why.** The issue says that tilted operations are not judged for gouges.
That is not the cause. A tilted operation is judged at generation, in the
plane's frame, by the keep law (`contain_into`, #1603). The test
`a_tilted_slot_into_a_meshed_bar_is_refused` shows this on unchanged code.
The cause is the reported part. It is a D50 bar with two cross holes, from
a STEP file. The kernel (truck) triangulates a trimmed face in its
parameter plane. A cylinder's parameter grid has two rows, and the two are
on the boundary, so no grid point is inside the face. With a hole in the
wall, each rim edge was joined to the hole's rim a quarter turn away. The
facets ran through the solid, up to 25 mm inside the true cylinder. The
importer reported the tolerance that it had asked for, not the error of
the mesh that it got. Every gouge law (generation, the program sim, the
upright operations too) then judged the cut against a part that was too
small. The slot at A0 was convicted only because the mesh is correct next
to the hole at the top.

**Considered and dropped.**

- *Hold every program that the stock sim does not judge.* This is the
  second option of the issue. It refuses every 3+2 and rotary program,
  although the generator-time laws judge them (`posting_sim.rs`, pinned by
  `post_not_judged.rs`). It also does not fix the report: the upright sim
  reads the same wrong mesh. The maintainer can still decide on an
  override design; this PR does not change that policy.
- *Only report the measured error.* Smaller, but each bar or tube with a
  cross hole is then refused as "imported too coarsely", and a larger
  facet budget does not help. Refinement gives a correct mesh.
- *Stretch the parameter plane of elementary faces* (as #1602 does for
  B-spline faces), or give the triangulation interior rows. This gives the
  better mesh with fewer facets. It changes the triangulation of every
  cylinder face of every model, and its effect on the goldens is not
  known. It is follow-up work.
- *Measure B-spline, NURBS and swept faces too.* Their grid is #1602's
  and one evaluation is more expensive. Not measured here; stated in the
  module header.

**What it costs.** Refinement by edge bisection makes many facets where
the kernel's mesh was wrong: the reported bar goes from 1 084 facets to
about 133 000, and its import takes longer (about 5 s in a debug build). A
face that the kernel meshes correctly is not changed. The slack of two
tolerances keeps a mesh that meets the kernel's own test as it is; a lower
slack would change more meshes.
