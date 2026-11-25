## #3153 - Add a 2D Chamfer tip offset

**What was decided:** The operation keeps the authored chamfer width and
angle as the edge-break promise. A positive tip offset moves
the centerline toward the selected air side and lowers the virtual CL
tip to `top_z - (minimum_cone_radius + width) * tan(angle)`.

Closed regions use the usual Side law: Outside offsets the region
outward and Inside offsets it inward. Auto retains the generator's
Outside law. An open run uses model sections to identify its physical
air side and the wall's Inside or Outside class. The selected Side must
agree with that class; otherwise the operation refuses by name instead
of moving the cone into material.

The zero-radius branch bypasses every geometric offset call. The
operation therefore keeps the former point order, move positions and
staleness bytes. Project v80 omits a zero `minimum_cone_radius` and
requires a newer reader only because a positive value changes a posted
path.

The user-facing label is "Tip offset". The existing
`minimum_cone_radius` key remains internal and persisted for the
operation's typed and project parameter surfaces.

**Why:** The selected edge-break and the cone contact band answer
different geometry questions. The EdgeBreak must remain on the original
selected loop or chain, with `width * tan(angle)` as its depth. The
cone tip must move farther down to place contact radii at `r` and
`r + width`. A geometric offset at zero can change tessellation, so it
is not a compatibility no-op.

Open chains do not have an unambiguous winding of their own. Model
sections remain the safety source for their air side. The model section
also states the wall class, so Side remains a checkable intent rather
than a direction inferred from the chain's stored order.

**Dropped:** A combined "chamfer depth" field was not added. It would
let the virtual tip and the authored edge break disagree. A general
replacement for real tip geometry was not added; flat, ball and worn tip
compensation remains in #895. The implementation uses the existing
Clipper offset and open-run side helper instead of a new geometry
algorithm.

**Cost to change later:** A different open-chain Side convention would
change how explicit Inside and Outside are validated. It would require
new motion and application refusal tests. A later real-tip model must
keep the ideal-cone band law as a distinct case and must preserve the
original EdgeBreak credit.


### PR #3251 review fixes

**What was decided:** At zero tip offset, both the closed-loop and
open-chain routes keep the legacy edge-break depth calculation,
`top_z - tip_z`, including its floating-point rounding. Positive offsets
keep the authored width-derived depth. Complete captured legacy paths
pin the zero-offset results rather than comparing two paths generated
by the updated implementation.

A positive closed-loop offset must preserve every original boundary
span. Reverse the offset with sharp joins and clip every original span
to its boundary tube. The 0.011 mm allowance covers two 5 µm
flattenings and the micron lattice; it contains no chamfer-width
allowance that could hide a lost span.

**Why:** Equal loop counts can hide a narrow branch that disappears
while its parent loop survives. The generator must refuse that partial
ride before it declares an edge break on the complete selected edge.
Legacy floating-point bytes are part of zero-offset compatibility.

**Dropped:** Loop-count comparison alone cannot establish complete
boundary preservation. A coverage allowance based on chamfer width
could accept a missing selected span and is therefore not used.

**Versioning:** Project format v80 is incompatible with v79 readers (v79 is the #3368 tip-form step, taken on main first).
The fix round records that existing format break with a
`BREAKING CHANGE:` commit footer and corrects the PR body to v80.
