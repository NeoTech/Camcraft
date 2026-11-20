## #2864 - Outside groove and part-off catalog case

**What was decided:** Add one original revolved bar with a 6 mm outside groove and a separate part-off plane.
Use the existing Groove Rough, Groove Finish, and Parting commands with the catalog GT-0300 blade.
Keep a 0.25 mm radial rough allowance so the posted program shows the finish cut.
Use Z15 as the parting blade center, as the existing API specifies. The 3 mm kerf spans Z13.5 to Z16.5.
Embed the mesh in the saved project and commit the reproducible script, generator, mesh, and NC program.

**Why:** The focused case shows the three requested operations without other turning operations.
The existing command API can build and verify the case. The change needs no machining algorithm changes.
The catalog tests check project regeneration and script reproduction against the committed NC program.

**Considered and dropped:** A cone, thread, facing pass, and profile passes would add operations outside this issue.
External geometry would make the fixture depend on another source. The original meridian keeps its provenance clear.
A linked mesh would make project opening depend on the current directory, so the script removes that link.

**Cost to change later:** Edit the original generator or script, rebuild the project and NC program, and update the operation assertions.
Run the catalog checks and inspect each changed NC block. No format migration or new dependency is required.
