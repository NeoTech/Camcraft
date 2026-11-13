## #1062 - MCP: edit_tool

**Decided.** `edit_tool {name, patch}` merges `patch` (JSON merge patch, RFC 7386, the same rule `edit_operation` uses) over the tool's `get_job` definition, reads it back as a `Tool`, and runs the existing `ChangeTool` - one undo step, every gate and refusal the GUI has.

**Unknown keys** are refused by name by checking that every non-null patch key survives the typed round-trip (the `edit_operation` check, reused). `center_cutting: true` is exempt from that check because it is written only when false.

**`number`, `body`, `cutter` are refused by name.** They are not `ChangeTool`'s fields (`SetToolNumber` and the body/cutter commands own them) and `get_job`'s tool definition does not carry them, so accepting them would silently drop the edit. Considered: also calling `SetToolNumber` for `number` - dropped, it would be a second undo step and `set_tool_number` has no MCP tool yet.

**Why.** Reusing `ChangeTool` and the merge helpers keeps the headless surface identical to the panel with no new rules. Cost to change later: small, one module (`mcp/tool_edit.rs`).

**Process note.** The camcraft-mcp skill's tool table should list `edit_tool`; the skill is a process file, so it is not edited in this PR.
