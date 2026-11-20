## #2868 - Indexed and tilted pocket example

**What was decided:** Add an original 30 × 30 × 35 mm block with three 8 × 8 × 2 mm pockets.
Use whole-model 2D Pocket operations at A90 B0, A−90 B0, and A30 B30.
Use the built-in `Tilt-head 5-axis` machine and the 6 mm `EM-0600-4FL` cutter with 15 mm flutes.
Fit box stock to the model with zero margin and embed the model in the project.
Require verification before the script saves the project or posts the NC program.

**Why:** The two opposite faces show single-axis indexing, and the sloped face shows two-axis indexing.
The compact stock keeps the full projected stock envelope above the tilted floor within the cutter's flute length.
The maximum distance is approximately 14.122 mm.
The built-in machine and embedded mesh make the case portable.
The example uses existing commands and keeps all verification checks enabled.

**Considered and dropped:** A 60 × 60 × 70 mm design encountered a false collision refusal caused by mixed coordinate frames.
Issue #3004 records that verifier defect separately.
This example does not change the verifier.
Picked tilted source faces are not supported, so the operations use the whole model.
Production algorithm changes are outside this example task.

**Verification scope:** The column stock simulation cannot judge posed operations.
The geometric gouge and collision checks still run.
The example has no holder or fixture model, and no physical CNC machine was run.
Verification does not prove complete stock removal.
The cutter leaves material in the sharp inside corners of the square pockets.

**Cost to change later:** Regenerate the mesh, project, and NC program after a geometry or tool change.
Update the pinning tests and the case README to match.
