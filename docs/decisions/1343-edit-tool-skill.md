## #1343 - Document the shipped edit_tool MCP contract

**Decided:** Add the missing `edit_tool` row beside `add_tool` in the
camcraft-mcp skill. Describe its partial tool DTO, merge-patch semantics,
response, undo/redo, staleness and refusals from the shipped implementation.

**Why:** Agents need the same tool-edit contract that the server already
exposes. The separate number, body and cutter commands keep their ownership.

**Considered and dropped:** Changing the MCP implementation or combining
other missing rows would expand this documentation fix. A PROCESS_VERSION
bump belongs to the merging session under the current process rule.

**Cost to change later:** Update this row when the shipped tool contract
changes; this decision adds no runtime behaviour or dependency.
