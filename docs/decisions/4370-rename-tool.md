## #4370 - Rename a tool with an undoable command

**What was decided.**

- A new command `RenameTool { from, to }` changes the `Name` of the tool
  entity and the `tool` field of each operation that names it, in one
  undo step. The entity stays, so the tool keeps its definition, number,
  holder body, drawn cutter and component stack.
- The operations are the only document records that hold a tool name. A
  search of the schema, the project file and the commands found no other
  stored reference: holders are keyed by holder entry, component stacks
  by component id, tool numbers live on the tool.
- A name is not an input of a toolpath. An operation that reads Ok before
  the rename gets its stamp taken again, so it reads Ok after. A Stale or
  a never-generated operation keeps its stamp. Undo does the same in
  reverse, so the stamps come back exactly.
- Refused by name at `new`, before the history: an empty name (after a
  trim), a control character, an unknown name, a non-tool, the same name,
  a name that another entity has (`NameConflict`), a setup's stock or
  work offset name, a name that an operation still names (the operation
  of a deleted tool would silently take this tool), and a name that the
  machine reserves (`tool_reservations::for_tool_as`).
- The tool editor's Name row is a text field. The typed name is a draft
  of the window (`ToolLibraryView::rename`), keyed by the tool. Apply
  queues `RenameTool` after the other draft commands. A rename alone
  queues only `RenameTool`, one undo step. A rename with field edits is
  the `ChangeTool` step and the rename step, as a number edit is today.
- A typed name makes the draft dirty (`library_draft_changed`), so
  Duplicate waits for Apply (#4340). Back and Cancel drop the typed name
  with the other draft fields. A refused name keeps the typed text.
- The views that show a tool by name follow the tool entity when its name
  changes: the library editor, the Properties subject and draft, and the
  tool shape window. This runs once in `drain_actions`, so it covers
  Apply, undo, redo and an agent's `rename_tool`.
- The MCP tool is `rename_tool { name, new_name }`. It replies with the
  old and the new name and the operations that followed.

**Why.** One command keeps the rename one undo step and keeps each
operation on its tool. A refusal at `new` keeps a bad name out of the
history.

**Dropped.**

- Rewriting the cut recipe library, the machine reservations and the
  exported `.camtools` files: they are outside the document, and a recipe
  that names the old tool is already refused by name when it is used.
- Rewriting user scripts: they are text that the user owns.
- A rename that also accepts a dangling operation's name and binds that
  operation to the tool: it changes what an operation cuts with, silently.
- Grouping Apply's `ChangeTool` and `RenameTool` into one batch: the
  window has no group command today; the separate steps undo in order.

**Cost to change later.** Small: the command is one file,
`crates/app/src/commands/tool_rename.rs`.
