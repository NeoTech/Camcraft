## #1108 - MCP: wire contour op

**Decided:**

- **`tool` is optional by the kind's own word.** `add_operation` asks `KindInfo::mounts_tool()`: a kind that mounts a library tool still requires a non-empty `tool` (unchanged); a kind that mounts none (the Wire Contour) takes an absent or empty `tool` as no tool. A non-empty `tool` on such a kind is passed to `CreateOperation` unchanged, so the refusal is the command's own words ("mounts no library tool ..."), not a second MCP-side message. The schema drops `tool` from `required` (MCP clients read `required` literally) and the description says when it is still needed; the handler keeps enforcing it for mill kinds.
- **No new mount tool.** The existing `load_machine { path }` mounts a `.mch` with `edm.conditions`; the test writes a synthetic one. A library wire EDM is #533's.
- **`list_operation_types` states it as data.** Every kind gets a boolean `mounts_tool`; a kind with `false` also gets a `note`. An agent no longer infers "no tool" from an empty `tool_kinds`.
- **`edit_operation` needed no change.** It already merge-patches the kind's JSON field object through `EditOperationParams`, so `side`, `start` and `condition` patch like any field and the command's gates refuse a bad condition.

**Considered and dropped:** a dedicated `add_wire_contour` tool (a second door to the same command, with its own schema to keep in step); ignoring a `tool` on a wire kind (a silently dropped argument reads as done as asked).

**Cost to change later:** low; the rule lives in one line of `mcp::edit::add_operation`.
