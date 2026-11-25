## #3893 - Tool library dialog: U2 edit a tool in the window

**Decided.** A tool opens in the tool library window's own editor. The
list folds into a rail (`tool_library_dialog/rail.rs`) on the left, and
the editor stands on the right: the heading, a Tool tab
(`tool_tab.rs`), a Details tab (`details_tab.rs`) and Apply. The window
keeps its own draft in `ToolLibraryView::editor` (`ToolEditorView`:
the `ToolUi` draft, its sync key and the tab). Properties keeps
`State::tool_ui`.

**Why.**
- The plan of #3663 asks that the dialog and Properties can show two
  different tools. One shared draft cannot do that: each frame,
  Properties writes its tool's name into `tool_ui`, and `sync_tool_ui`
  then reads the other tool again, so a half-typed value in the window
  is lost.
- `State::with_library_draft` swaps the window's draft into `tool_ui`,
  and its sync key into `tool_synced`, only while the window draws and
  applies. Then the existing rows (`tool_settings_grid::geometry`,
  `kind_rows`, `catalog`) and the existing Apply helpers
  (`State::apply_tool_numbers`, `tool_settings::push_number_if_changed`)
  run without a copy. A new kind row (#3368, #3369) lands in one place.
- The window's draft has its own sync key, `(name, doc_version)`.
  Typing does not change `doc_version`, so the draft is read again only
  after a command or when another tool opens.
- Apply queues the same commands as Properties' Apply: `ChangeTool`
  (or `ReshapeTool` when a drawn cutter rides the change), and
  `SetToolNumber` only when the number changed. A geometry and catalog
  edit is one undo step. A number change is a second step, as in
  Properties today. No new command, so no MCP tool is owed.
- The Document list's "Edit" opens the tool in the window (U1 sent it to
  Properties until this step). A new window opens on the tool that
  Properties shows, if any. A pick in the window (Edit, the rail)
  changes neither the selection nor the Properties subject.
- "Edit tool shape" opens the existing shape window. That window edits
  through Properties' draft (`tool_ui`). So when Properties shows a
  different tool, Properties first follows the window's tool. Else the
  shape window and Properties would show two tools in one draft.
- When the open tool leaves the document (deleted, renamed), the editor
  closes and the window shows the full list.

**Dropped.**
- One shared draft with a "which tool" flag (the two drafts fight over
  each frame; the plan's risk).
- A copy of the geometry and catalog rows for the window (two places to
  change for each new kind).
- "New tool…" in the window: it still opens the blank form in
  Properties. The issue did not ask for it in U2.
- The holder, the cutting data and the drawing in the window: U3 to U5.
  Properties keeps all its groups until then.

**Cost to change.** Small. The swap is in one function
(`with_library_draft`), and the tabs are two short files.
