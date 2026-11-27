## #4389 - Holder files on the Holders page

**What was decided.**

- A holder file is a `camcraft-components/1` file (`component_library::to_text`
  and `parse`), the format of the user component library. No new format.
- Export writes each holder row of the document's component table whose id is
  not in the `builtin:` space. A holder is an adaptive item whose machine side
  is a spindle taper (steep taper, HSK, PSC, KM), the set that the Holders page
  lists (`commands::holder_file::is_holder`). A document with no user holder is
  refused by name.
- Import adds each row as a user holder, with no library link, in one undo step
  (the new `ImportHolders` command: one snapshot of the component table before
  and after). A row keeps its file id when that id is free. An id that the
  document, the component library or an earlier row of the same file holds
  takes `user:holder:<name>`, then ` (2)`, ` (3)`, as "Copy as user holder"
  does.
- The file is read whole. Every `parse` refusal, a file with no row, and a row
  that is not a holder refuse the whole file by name. Nothing is applied.
- The MCP tools `export_holders` and `import_holders` use the same function and
  the same command. `export_holders` asks for file-write approval in a live
  session.

**Why.** The issue names the component library format. One format keeps one
reader with its all-or-nothing law. One table snapshot is the pattern of the
sibling component commands, and it gives one undo step for any number of rows.

**Considered and dropped.**

- Always a new `user:holder:` id: an export into a new document then changes
  ids that are free, and the round trip is not byte-stable.
- Skip a row that is not a holder: a partial import is silent. A refusal by
  name is the Prime Directive.
- One `UpsertDocumentComponent` per row in a history group: each command
  snapshots the table when it is made, so rows made before the first one runs
  overwrite each other.

**Cost to change later.** Small. The format is the library format, so a later
holder-only envelope needs only a new reader beside `parse`.
