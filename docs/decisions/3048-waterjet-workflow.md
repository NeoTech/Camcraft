## #3048 - Document the Waterjet Profile MCP workflow

**Chosen:** Add the Waterjet Profile workflow before the Plasma Profile workflow.
Show the machine file, complete nozzle definition and operation parameters.
State that the example requires #1883 (PR #3058), which is not yet on main.
Keep the current limits explicit: timings appear in notes, and no post serves waterjet.

**Why:** Agents need the tool calls and parameter keys for the new operation.
The example follows the pending implementation's MCP test and parameter DTO.
The availability statement prevents agents from assuming that main supports it.

**Dropped:** Do not add runtime code or a waterjet post in this documentation change.
Do not use a milling or plasma post as a substitute.
Use standalone Python documentation tests, as the skill already does.
Do not add a Rust dependency for a text contract.

**Cost to change later:** Update the example and its documentation tests when #1883
or #1882 changes the operation contract or post support.
