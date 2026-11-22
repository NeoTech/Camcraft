## #2870 - One small model for the five-axis catalog case

**Status:** Held and incomplete.
The project generates, and `cam.verify()` returns a clean result.
Posting refuses the first tilted feed because the opening rapid moves do not establish the initial A angle.
Issue [#3300](https://github.com/NeoTech/Camcraft/issues/3300) tracks this blocker.
The branch has no committed NC program and no passing catalog gate.

**What was decided:** Use one closed faceted cylinder for Swarf, 5-Axis Contour, and 5-Axis Flow.
The cylinder has a 12 mm radius and extends from X = -24 mm to X = 24 mm.
Its original analytic generator uses 16 circumference sections and six length sections.
The OBJ generator writes each vertex once and indexes the shared vertex list to avoid the import defect tracked in [#3298](https://github.com/NeoTech/Camcraft/issues/3298).
The operations cut between Z = 11 mm and Z = 5 mm, with clearance at Z = 25 mm.
Use the Tilt-head 5-axis machine and a BN-0600 ball-nose cutter with a 60 mm flute and a 6 mm shank.
Select no holder and disable the optional Contour and Flow shank-avoidance search.

**Why:** One model gives all three strategies a common tilted wall and surface.
It avoids neighboring bodies that these per-source checks do not test.
The coarse mesh and small path set limit the intended catalog runtime.
The long flute and narrow shank keep the modeled tool clear above the selected wall region.
Generation, verification, and posting remain required steps in the script.
The README states the limits of this modeled tool assembly and the checks.

**Considered and dropped:** Separate wall and cap bodies would show more geometry, but would add a neighboring body outside each operation's source.
A second cap demonstration also added work without improving the three required strategy examples.
A finer mesh and denser paths would increase the case runtime without adding a required behavior.
Physical machine validation and posed stock-removal simulation are outside this example's scope.
An unrelated positioning operation or a machine without rated rotary limits would hide the posting defect.
Both workarounds were dropped.
No safety check is bypassed.

**Cost to change later:** Edit the generator and script, regenerate the mesh, project, and NC program, then repeat the catalog checks.
Additional bodies need checks that include those bodies before the example can claim assembly clearance.
No application behavior, dependency, or safety check changes for this case.
