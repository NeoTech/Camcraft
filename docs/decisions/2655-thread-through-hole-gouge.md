## #2655 - Thread mill on a detected through hole: excuse the exit face in ThreadBore

**Decided.** Add `pub exit: Option<f64>` to `ThreadBore` in `crates/sim/src/verify/thread_bore.rs`, populated as `h.through.then_some(h.floor_z)` in `crates/app/src/operation/thread_rims.rs`. In `ThreadBore::wall_radius`, alongside vertical minor-wall facets and mouth facets at or above `self.top - WALL_BAND`, excuse a facet on the hole's minor circle band when `self.exit` is `Some(exit)` and `at_exit` holds: every vertex satisfies `z <= exit + WALL_BAND`, and every vertex on the minor circle satisfies `z >= exit - WALL_BAND`. `excused` also requires `exit` (when present) to be finite and `<= top`.

**Why.** On a through hole (`through: true`), `machining2d::thread_mill` extends the helix one pitch below `floor_z` so the teeth cut a full thread out the bottom exit, and the cutter spans across the plate's bottom face at `floor_z`. `ThreadBore` previously knew only `top` (`entry_z`) and excused horizontal mouth facets only at `z >= top - WALL_BAND`. Every bottom-face triangle sharing the hole's minor rim had `wall_radius` return `None`, so the solid law convicted the lead-in and helix moves crossing `floor_z` with `Gouge` ("cuts past the finish surface").

Requiring `z <= exit + WALL_BAND` on all vertices and `z >= exit - WALL_BAND` on the minor-circle vertices matches flat exit faces and downward-drafted undersides while still convicting blind hole floors (`exit: None`), intermediate shelves between `exit` and `top`, lower parts or shelves below `exit - WALL_BAND`, steps reaching inside the minor circle, and cuts past the model's major cylinder.

**Considered and dropped.**
- Raising `floor` in `thread_mill` so the cutter never crosses `floor_z`: would leave an incomplete thread at the exit of a through hole.
- Checking only `max z <= exit + WALL_BAND` without bounding the on-circle vertices at `exit - WALL_BAND`: would excuse a separate plate or step sitting below the threaded plate if it shared the same hole XY and minor radius.
- Requiring all three vertices inside `[exit - WALL_BAND, exit + WALL_BAND]`: would falsely convict a bottom face drafted downward away from the hole exit.

**Cost to change later.** One `Option<f64>` field on `ThreadBore` and one helper (`at_exit`) in `crates/sim/src/verify/thread_bore.rs`, populated in `crates/app/src/operation/thread_rims.rs`.
