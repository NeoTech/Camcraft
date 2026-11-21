## #3020 - Judge a shallow spot with its cone radius

**What was decided:** Keep the tangency law. Use the matching real cone's
radius at the stock top for linear axial moves with no explicit tool pose.
Apply this radius only in Boundary contexts with declared drill rims and no
supplied model. Use the deepest endpoint to bound both the plunge and the
retract. Keep the nominal radius when the cone top or the shank enters.
Keep the nominal radius for model sections, tool poses, arcs, lateral moves,
other shapes, and missing or mismatched cutter geometry.

**Why:** A 6 mm drill with a 120-degree point reaches a radius of 1.732 mm
at a point depth of 1 mm. That cone stays inside a hole with a radius of
2.5 mm. The nominal 3 mm radius stays above the stock surface. The solid
check, stock simulation, and post checks still refuse actual gouges.

**Considered and dropped:** Suppressing all spot gouge warnings would hide
real wall cuts. Changing the tool diameter would describe the wrong tool.
Reducing the section radius for all operations would weaken other wall
checks. Extending the change to posed tools or other profiles would need
additional geometry proofs and tests.

**Cost to change later:** Extend the footprint calculation for tool poses
or other profiles only with separate safety proofs and pinning tests.
This change adds no dependencies and changes no process files.
