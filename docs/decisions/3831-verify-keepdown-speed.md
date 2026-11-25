## #3831 - Share prismatic sections and preserve collision samples

**What was decided.** The verifier shares a model section for geometric
queries inside each prismatic Z slab. The reach law (c) keeps the boundary
samples from every cut plane. It does not replace those samples with
points at fixed spacing.

A prismatic slab is the Z range between two neighbouring vertex levels.
Each crossing facet must be vertical: two vertices have bit-equal XY.
A plane that the path cuts flat at keeps its own section. A plane outside
a prismatic slab also keeps its own section.

The gouge law (b) asks the cheap distance test before the wall probe
`edge_is_wall`. These tests are pure. Their order does not change a result.

**Why.** A keep-down ramp ends each move at a different Z. Repeated model
sections and wall probes made verification expensive. A shared section
reduces repeated geometric queries. However, equal geometric regions do
not imply equal collision samples.

Each plane cuts the diagonal between two wall facets at a different point.
The old reach law pools the vertices and edge midpoints from all planes.
It nudges each point toward that section's loop vertex mean. Thus both the
sample position and its nudge can change with the plane.

The first version replaced these samples with wall points each 0.25 mm.
The review of PR #3977 found a loss of collision coverage. On a 20 mm boss,
a shank near a wall passed with no warning. The plane-exact reading found
500 `ShankHit` warnings. The shank met stock above the finished part, so
the solid law did not replace this stock check. Small bands and near-tangent
contacts also lost findings. The sample limit could widen the spacing.

**Conservative rule.** A shared geometric section is not proof that its
samples cover every plane's samples. Law (c) therefore reads the exact
section for each cut plane that would otherwise use a shared section.
It pools the original boundary samples. Other geometric queries can still
use the shared section. Planes that already have their own section reuse
that section for sampling.

This fallback applies to every shared slab. It does not depend on a minimum
band size, a distance from tangency, or a perimeter limit. No finite fixed
step proves that it preserves every old contact. A future replacement must
prove coverage before it can remove the plane-exact fallback.

**Performance limits.** The first version measured 282 s before the change
and 1.2 s after it on a plate with 38,432 facets and 16 holes. That faster
reading used the unsafe sample replacement. It is not a measurement of
this corrected implementation. The fallback restores plane section work
and the pooled sample set for law (c). The corrected implementation has
not been timed on that plate.

**Considered and dropped.**

- Fixed wall spacing, including a smaller step. It cannot prove coverage
  for narrow bands, near-tangent contacts, or a widened sample step.
- Only corners and midpoints from one shared section. An earlier fixture
  lost 19 of 80 findings with this option.
- Shared sections for flat planes. Their original sample positions must
  remain available, and a level ladder has few distinct planes.
- A lower move count for keep-down ramps. That changes posted G-code and
  is outside this PR.
- A time limit in a test. Section counts and collision findings give
  repeatable assertions across machines.

**Cost to change later.** A future coverage proof can replace the law (c)
fallback. Until then, exact sampling has the previous section and sample
cost. `Rest::plane_exact` remains the test reference. The verifier reads
the path; it does not change the moves or the posted program.
