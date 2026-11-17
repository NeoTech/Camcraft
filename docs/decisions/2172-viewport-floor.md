## #2172 - The viewport floor follows the lowest body (display only)

**Decided.** The ground grid and the bottom of a mill work envelope are
drawn at the "floor": the lowest finite world Z of the drawn bodies (the
models, the fixtures of the shown setups and the active setup's stock).
With no fixture the model and the stock stand on the floor. A fixture
under the stock puts the floor at the fixture base, so the model and the
stock stand on the fixture. When the fixture is removed, the floor comes
back up to the stock. The floor is computed from the world at each frame
(`crates/ui/src/floor.rs`), so undo and redo follow with no extra code.

**Why.** The maintainer saw the model "halv way down, thru the floor" and
fixtures "under the floor". The cause is the reference furniture at world
Z=0, not the bodies. Moving the furniture to the bodies gives the asked
picture and cannot change a stored position. The WCS, the toolpaths, the
collision checks, the simulation, the carried stock and the posted program
read the world, and the world does not change. A test posts the seeded
program before and after the floor readers run and compares the bytes.

**Dropped.** A stored "lift to the floor" placement (an undoable command
that moves the model, the stock and the fixtures in Z). A WCS that is set
at a world position (not at a body feature) would then post different
coordinates for an existing program. That is under the safety floor and
needs a migration law for saved projects. It also needs a new command and
MCP tool. The view-only floor gives the same picture without that risk.

**Lathe.** The lathe view (#1993) shows the spindle axis across the
screen and the envelope as a cylinder about that axis. A floor below the
part has no meaning there, so the lathe floor stays at world Z=0.

**Limits.** A floor below -1e7 or above 1e7 mm, or a non-finite one, falls
back to 0 (a corrupt mesh never sends the grid to infinity).

**Cost to change.** Low: one function. A later stored placement can replace
it; the tests in `crates/ui/src/egui_app/tests/floor.rs` then need a
posted-output pin for the moved bodies.
