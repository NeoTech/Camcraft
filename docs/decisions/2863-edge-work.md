## #2863 - A small plate for edge-work debugging

**What was decided:** Add one original 60 x 40 x 6 mm plate with four
existing command-driven operations. Chamfer and Deburr compare cone edge
breaks. Engrave and Trace compare the same closed boundary at the same
depth. The project embeds the mesh without an external model link.

**Why:** A small reproducible fixture makes the posted paths easy to
inspect and compare. One script builds, verifies, saves, and posts all
four operations. The embedded mesh lets the project open from another
working directory. This change adds no production code.

**Considered and dropped:** Interior text and open strokes would require
behavior beyond the closed boundaries that Engrave and Trace support.
Separate cases would repeat the plate and script without making this
small comparison easier to inspect.

**Cost to change later:** Edit the fixture or script and regenerate its
project and NC program. No production code change is required.
