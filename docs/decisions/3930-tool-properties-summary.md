## #3930 - U4b: the tool Properties summary, the picker out of the shape window, the window's Body section

**What was decided.**
- A saved tool's Properties page is a read-only summary
  (`egui_app/tool_summary.rs`): type, sizes, holder, rpm, feed, material
  group, tool number, and the one side preview. Its door "Edit in tool
  library" opens the tool library window on the tool
  (`State::open_tool_in_library`); the Properties subject does not change.
  The summary has no Apply.
- The new-tool form keeps the full grid and Add tool. The window has no
  editor for a tool that is not in the document yet, so a removal would
  leave no way to create a tool by hand.
- "Edit tool shape" stays on the summary. The shape window edits through
  Properties' draft (`tool_ui`), so its door stays beside that draft.
- The holder picker left the tool shape window. The window's assembly
  group (#3895) is the one place that draws it.
- The window's Tool tab has a folded "Body" section under the assembly
  group: the stand-in `holder_d` / `holder_len` rows and the body section
  (Import body…, the placement, Pick tip / Pick axis, Apply body, Convert
  to profile, the pieces). It draws from the window's own body draft
  (`ToolEditorView::body`), swapped into `tool_body_ui` by
  `with_library_draft`, as #3895 did for the holder draft.
- A turning tool shows the Body section in the window too (no picker).
- The rest of Properties' old Cutting data group (the tool's own spindle
  rpm and feed, the lathe rows, centre cutting) follows the table on the
  window's Recipes tab (`tool_settings_grid::cutting_base`), so no edit
  path is lost.

**Why.** The issue's "Done means": the summary with the door, no picker in
the shape window, the Body section from the window's own draft. The three
parts are one move of the same fields; one PR keeps each field in exactly
one editor at every commit. Commands: no new command. Each landing is the
command it was before (`ChangeTool`, `ReshapeTool`, `SetToolNumber`,
`SetToolBody`, `SetToolBody::library`, the piece commands), so the
collision input changes nowhere else (safety floor). No MCP tool is owed.

**Considered and dropped (held for a follow-up).**
- The Body tab leaving the tool shape window. It is the input of "Convert
  to profile" beside the Profile tab, and about 30 tests drive it
  (`tool_body`, `tool_body_assembly`, `tool_body_lathe`, `tool_body_floor`).
  The two body drafts stay apart, so the two places do not mix.
- Picking the tip and the axis on the window's large 3D preview. The Body
  section keeps its own pick view (`tool_body_page::placement_view`).

**Cost to change later.** Small: the summary is one file, and the Body
section is one function in `assembly_group.rs`.
