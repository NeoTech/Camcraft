## #2624 - Show move arrows and rotate rings together

**What was decided:** The Move button shows arrows and rotate rings together.
The existing Rotate button remains available as a shortcut. The named
`Move value` and `Rotate value` controls select the typed editor only.
Both sets of handles stay visible. A drag selects its corresponding editor.

The change stacks on PR #2662 and uses its typed values and Shift snapping.
The angle snap step keeps the existing configurable default of 90 degrees.
An edge-on ring consumes a press without starting a turn. It shows a hint
to orbit the view. Its accessibility node is disabled. A face-on ring stays
available.

The existing `TurnModelBy` path uses `MoveModel` for the undo step and the
operation staleness checks. The existing MCP tool supplies the same turn.
This change adds no command or MCP tool.

**Why:** The part must turn from its movement gizmo without a mode switch.
Separate typed editors keep distance and angle inputs clear while both
sets of handles remain available. The existing command path preserves undo
and marks toolpaths stale after a turn. The edge-on refusal prevents an
inert drag and tells the user how to make the ring usable.

**Considered and dropped:** Removing the Rotate toolbar button would expand
the change and overlap other toolbar work. A duplicate command or MCP tool
would repeat the existing turn behavior. Guessing an angle from screen
motion on an edge-on ring would make the result depend on an ambiguous
projection. Changing the snap default would alter the behavior from #2662
without a requirement to do so.

**Cost to change later:** Removing the Rotate shortcut requires toolbar and
layout test changes. A different snap default requires a local constant
and test update. A different edge-on interaction requires a new drag law
and behavior tests. These choices change no saved project format.
