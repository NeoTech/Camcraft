## #1084 - Complete the rotary-stack MCP reference

**Decision:** Correct the existing rotary-stack rows instead of adding duplicates.
Name the readback fields and the tallest-stack rule. State the setter's required
arguments, undo/redo and refusals. List the subplate dimensions for both accessory
tools and the pin-spacing tolerance for placement and resizing.

**Why:** The rows already shipped in `9707408b7` and are on current main. The
current source defines the remaining details. `rotary_tools.rs` requires `setup`
and `nose_to_axis` for the setter; only the getter defaults to the active setup.
The readback returns one tallest stack, or `null` when a stated figure wins.

**Considered and dropped:** Do not make `setup` optional or change Rust behavior.
Do not duplicate the existing `rotary_subplate` kind in `add_accessory`.
PR #4057 changes different rows in the same skill; keep those rows unchanged.
Do not bump `PROCESS_VERSION` or announce on #681 here. Current `AGENTS.md`
assigns both actions to the merging session. Declare `Process change: yes`.

**Cost to change later:** Update the affected rows and their focused Python
contracts when the public schema or stack readback changes. No migration is needed.
