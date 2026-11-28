## #3647 - MCP tools for tool components, assemblies and the component library

**What was decided.**
- Eight MCP tools in three new files of `crates/app/src/mcp/`
  (`component_tools.rs`, `assembly_tools.rs`,
  `component_library_tools.rs`), chained into the table with one line
  (`component_tools::all()`), because `mcp/tools.rs` is already over
  400 lines.
- A component crosses the MCP door in the `.camproj` wire form
  (`project::ComponentDto`, `deny_unknown_fields`) and is built again
  through `component_from_dto`. A stack crosses as `LinkDto`. One form
  for the file, the library and MCP means one parser and one set of
  refusals. A listed row nests the wire form under `component`, so an
  agent can pass it back to `upsert_tool_component` unchanged.
- `upsert_tool_component` refuses a stated `library_ref`: only
  `sync_tool_component` writes a link, as in the GUI law of #3645.
- `compatible_with` takes one component id. The id is looked up in the
  document, then the user library, then the built-ins. Each kept row
  carries `fits: [{as, port}]` from `component_fit::compatible_parents`
  and `compatible_children`, so one query answers both "the holders
  that take this insert" and "the inserts that this holder takes".
- The library file is the user's own by default. An optional `library`
  argument names another `.camcomponents` file, as `library` does on the
  cut recipe tools (#3073). The tests use it, so no test writes the
  user's real library and no test sets a process environment variable.
- The library is read only when it is needed: a listed user row, a
  compatibility query, or a document row with a link. An unreadable
  library refuses a listing of user rows by name. For the sync status of
  a document row it shows `unreadable` with the reason, and never
  `missing_from_library`.
- Live approval gate: `save_library_component` and
  `delete_library_component` are `FileWrite`. A stated `library` on
  `list_tool_components`, `get_tool_assembly` and `sync_tool_component`
  is `FilePathAccess`. The new tools are not on the read-only allow-list
  of the local model fallback.
- No script verb. The camcraft-mcp skill does not ask for one, and the
  script host has no component verbs to follow.

**Why.** The issue asks for strict, named refusals and for the same
undoable commands that the GUI uses. The wire form and the commands of
#3644 and #3645 already hold those laws, so the MCP layer only parses,
dispatches and reports.

**Considered and dropped.**
- A separate MCP DTO for a component. Dropped: a second form would drift
  from the file form.
- An `upsert_tool_component { from_library }` that copies and links in
  one step. Dropped: it would be two commands. An upsert followed by a
  sync gives the same result in two undo steps.
- Setting `CAMCRAFT_COMPONENTS` in the tests. Dropped: the environment
  is shared by the parallel tests of one binary, and `set_var` is
  `unsafe` in edition 2024.

**Cost to change later.** Low. The tools are thin; the laws are in the
commands and the wire form.
