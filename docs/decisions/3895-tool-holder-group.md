## #3895 - U4: the holder, gauge length and stick-out in the tool library window

**What was decided.**
- The window gets an assembly group in its Tool tab
  (`tool_library_dialog/assembly_group.rs`). It calls the same picker
  fields as the shape window (`tool_holder_picker::holder_fields`), adds
  the read-only total, the re-seed note, a taper that is not the
  machine's `tooling.spindle_mount`, and what Apply holder would refuse.
  Apply holder is the existing `SetToolBody::library`. No new command, no
  MCP tool owed.
- The window's holder pick is its own draft (`ToolEditorView::holder`),
  swapped into `tool_profile_ui` by `with_library_draft`, as U2 did for
  `tool_ui`. Properties' shape window keeps its own draft.
- #808 item 2: when the stored tool's flute plus shank length changes
  under a pick, the projection draft takes the new stickout and is marked
  (`ToolProfileUi::reseeded`). The stored `HolderRef` changes only on
  Apply holder.
- The preview draws the draft holder through the command's own path
  (`holder_library::resolve`, `fits_shank`, `tool_body::build_library`,
  `tool_body::fits`), inside the preview's memo, so a still frame builds
  nothing. A pick that this path refuses keeps the landed body, and the
  refusal shows in the group before the click.
- The preview's stick-out of a library holder is its stated projection.

**Why.** The issue's "Done means" asks for the holder, gauge length and
stick-out in the window, one undo step, the re-seed and the shank warning.
Each change lands through the same undoable command as before, so the
collision input changes nowhere else (safety floor).

**Considered and dropped (held for a follow-up).**
- The Properties page as a summary with "Edit in tool library", and the
  holder picker leaving the shape window. Both move about ten tests that
  drive the shape window's picker and the Properties holder rows
  (`tool_holder.rs`, `tool_holder_reference.rs`, `structural`,
  `page_layout_windows`, `tool_properties`). The shape window keeps its
  picker until then, with its own draft, so the two places do not mix.
- The folded Body section in the window. The body import draft
  (`tool_body_ui`) is still Properties' draft; drawing it in the window
  would mix the two drafts.
- A mark on each picker row whose taper is not the machine's. The group
  warns on the pick instead; the row labels stay as the tests read them.

**Cost to change later.** Small: the group is one file, and the follow-up
moves the remaining surfaces to it.
