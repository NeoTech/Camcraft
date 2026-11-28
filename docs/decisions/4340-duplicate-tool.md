## #4340 - Duplicate a tool with its holder and component stack

**What was decided.**

- A new command `DuplicateTool` copies the stored tool. It lands the same
  `ToolRecords` list that `DeleteTool` snapshots: the definition, the holder
  body, the drawn cutter and the component stack. Only the tool number is
  new. A new tool record is added to that one list, so the copy cannot drop
  it.
- The copy's name is `<stem> (2)`, `(3)`, and so on. The stem drops a
  trailing ` (n)`, so the copy of `EM (2)` is `EM (3)`. This is the
  `free_name` style of fixtures and sketches.
- The number comes from `tool_reservations::allocate`, the same rule as
  `AddTool`. "No free tool number" refuses the copy by name.
- The tool editor's Duplicate copies the tool as it is applied, not the
  unapplied draft. This is the same rule as Duplicate on a user holder
  (#4338). The window then opens the copy.
- The MCP tool is `duplicate_tool { name }`. The copy name is not an
  argument: `edit_tool` and later commands use the name in the reply.

**Why.** One command keeps the copy one undo step. The shared record list
keeps the copy and the delete-undo in step.

**Dropped.**

- `AddTool` with the stored tool: it applies the machine's default holder
  and drops the stack (the issue's finding).
- A "Copy of <name>" prefix: names then grow at the front, and a list sorted
  by name does not keep the copy next to its source.
- A copy of the unapplied draft: then Duplicate also applies the draft
  without Apply, and the source and the copy can differ silently.

**Cost to change later.** Small: the name rule is one function in
`crates/app/src/commands/tool_duplicate.rs`.
