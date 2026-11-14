## #1495 - Turn the WCS about its own axes

**Decided:** Compose each quarter turn on the right of the setup orientation
(`q * turn`), so X, Y and Z name the setup's current axes. Describe the buttons
as turning about the setup axes. Keep the existing `SetWcs` command, origin
and undo semantics.

**Why:** The user sees and edits the setup coordinate system. Its visible axes
must determine the turn even when the setup is oblique to the world axes.
Right composition expresses that local rotation without changing the command
or the setup's position.

**Considered and dropped:** Keeping world-axis composition preserves the
reported problem. Adding a new command or API duplicates the existing
orientation update and undo path without providing another needed operation.

**Cost to change later:** Change the composition order, the pinning test
expectations and the UI wording together. No project-file migration is needed.
