## #4391 - MCP: edit a user holder by its shape fields

**What was decided.**

- **A new tool, not an option of `upsert_tool_component`.** The MCP tool
  `edit_user_holder { id, name?, taper?, clamp?, bore?, gauge_len?,
  nose_d?, largest_d? }` is in its own file
  (`crates/app/src/mcp/holder_shape_tools.rs`). The upsert tool keeps one
  input form (the full component object). Dropped: a `shape` key on
  `upsert_tool_component`, which would give one tool two exclusive input
  forms.
- **The same build and command as the holder form.** The tool reads the
  held shape with `HolderShape::of`, puts the stated fields over it, builds
  the kind and the outline with `HolderShape::kind` and
  `HolderShape::geometry(ADAPTIVE_PORT)`, and runs one
  `EditDocumentComponent` through the History. Undo and redo are the
  existing MCP `undo` and `redo`.
- **The same refusals.** A shape error has the form's text, `holder not
  changed: <ShapeError>`. A broken fit comes from the command and names
  the tool. An empty name has the form's text, `the holder needs a name`.
  An edit that changes nothing gets the command's "already so".
- **A holder that is not a shape outline is refused.** When
  `HolderShape::of` gives `None` (a DXF profile, an adapter, a hand-made
  outline), the tool refuses by name and points to
  `upsert_tool_component`, as the GUI opens such a holder in the generic
  component editor.
- **An unstated `largest_d` on an ER or Weldon clamp follows `nose_d`.**
  These clamps have one body diameter, so a call that changes only the
  nose diameter or the clamp is not refused with "one body diameter". A
  shrink-fit clamp keeps its held `largest_d`.
- **The `camcraft-mcp` skill row is not in this PR.** The skill is a
  process file. The PR body gives the proposed row for the maintainer. The
  generated reference (`docs/agents/mcp-tools.md`) and the user guide name
  the tool, as the drift tests require.

**Cost to change later.** Low: one tool file and one registration line.
