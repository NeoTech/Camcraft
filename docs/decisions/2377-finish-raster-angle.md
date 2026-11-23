## #2377 - 3D Finish Parallel direction angle

**Decided.** The Parallel strategy takes a finite angle in degrees from
0 inclusive to 360 exclusive. Positive angles turn counterclockwise
from +X in the work coordinate frame used for generation. The Passes
page shows the field for Parallel. A change uses the existing undoable
parameter edit. MCP reads and writes the same parameter. Changing the
pattern keeps the angle in the operation. Each other pattern uses its
own direction.

**Why.** The generator uses the angle to orient its sampling rows.
The existing support queries read the same facets, and the existing
boundary, stock, and verification code receives the path in its
expected frame. Exact quarter-turn directions avoid trigonometric
rounding on the axes. An angle of zero uses the original +X raster.
A project without the key takes that same default, and its existing
generation stamp remains valid. The additive key needs no file
version change.

**Alternatives considered.** Rotating the model for the requested
angle would also require moving the stock, boundaries, and translated
diagnostics through that frame. Normalizing supplied values would
silently accept angles outside the documented interval. Applying the
angle to every pattern would change the meaning of Parallel Multi,
Scallop, and the other strategies that already define their paths.

**Cost to change later.** A different frame or angle convention would
require a project migration to preserve saved directions. An angle
for another strategy can use the same command and project field if
that strategy defines its angle in the same way. Its generator,
stamp, UI, and tests would then need to apply the angle.
