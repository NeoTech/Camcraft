## #3645 - `.camtools` version 8: tool components and tool stacks (slice 2)

**What was decided.**
- The `.camtools` file gets the tool components and the tool stacks.
  The issue says version 5, but the file is at version 7 on `main`
  (`CUTTER_HAND_VERSION`), so the step is version 8
  (`ASSEMBLY_VERSION`).
- Shape: a definition of an assembled tool holds its stack as
  `assembly` (the `project::assembly_wire` links, as a `.camproj` tool
  states them). The file holds a top-level `components` array with the
  rows that the exported stacks name, in table order, in the one wire
  form of `project::component_wire` (the row's `library_ref` included).
  A row that no exported tool names is not written.
- The version step is conditional, as each earlier `.camtools` step: a
  library with no stack keeps its older number (1 to 7) and the same
  bytes. A library with a stack says 8, so a version 7 build refuses it
  and never drops the stacks without a word. The bare `[{name,
  definition}]` array has no table, so a stack in it is refused by the
  missing row.
- Strict reading, the law of the project loader:
  - the table is read whole first (`components_from_dtos`: each row
    built again through the checked constructors, an unknown key and an
    id twice refused, each refusal by the row's id);
  - each stack must name rows of the table and resolve (the #3643
    resolver), or the file is refused by the tool's name;
  - a row that no stack names is refused by its id, so nothing in the
    file is dropped without a word;
  - a file that says a version before 8 and states `components` or an
    `assembly` is refused: it says it is a file that cannot hold them.
- The tool record and the body stay as the file states them, as on a
  project open (slice 1). The body goes through `as_derived`, so an
  imported assembled tool is the same as the one a project opens.
- The import lands the rows and the tools in the ONE `ImportTools` undo
  step. The table changes first on apply and is restored last on undo.
- **The id rule.** A library row whose id the document already holds:
  - the same content (the whole row: name, item, geometry, the #3646
    cutter facts and running data and the #3643 lathe body; only the
    library link is left out, as for a link-only edit) - the document
    row is reused, and the report names it;
  - a different content - the whole import is refused by the id
    ("rename one of them; nothing imported").
  An import never changes or renames a document row.
- A tool that lands takes the library's stack:
  - an added or a renamed tool gets its stack (or none);
  - an updated or a replaced tool takes the library's stack, and a
    library tool without a stack clears the old one, because the record
    no longer comes from it; undo restores it;
  - the skip of an identical tool compares the stack only when the
    library states one (a version 7 file never clears a stack of a tool
    that it skips). Each row that a skipped tool's stack names goes
    through the same id rule: a changed row refuses the whole import,
    also when other tools of the file are added.
- Holders off (`holders: false`) leaves the stacks and their rows out
  with the bodies: a stack without its holder bodies would derive a
  different tool on the next component edit.
- `ImportReport` names the added and the reused rows; the summary says
  "N tool components added".

**Why.** The issue asks for portable tool libraries with components and
assemblies, refusal by name, and older files that load unchanged. To
reuse a row of the same content keeps one row for each id and does not
change a posted program. To refuse a different row of the same id is
the safe answer: a merge or a silent rename would change the tools that
are built from the document row, or hide which row a stack names.

**Considered and dropped.**
- Rename a clashing library row (`id (2)`) and its links. Dropped: the
  user can not see which row a stack names, and two rows that look the
  same give different tools.
- Let the library row replace the document row. Dropped: it changes
  other tools of the document without the edit law of
  `EditDocumentComponent`.
- Land the rows that no stack names. Dropped: the import has no other
  way to show them, and an export never writes one.
- Keep an old stack on an updated tool when the library states none.
  Dropped: the next component edit would derive the tool again from the
  old stack and change the imported record.

**Left on #3645.** Nothing of the issue text. The MCP tools are #3647
and the UI is #3648.

**Cost to change later.** Low. The format is the slice 1 wire form; the
import law is in `commands/tool_import/components.rs` and is pinned in
`crates/app/tests/tool_library_holders/camtools_v8.rs`.

**Review fix (mkinney, comment 7211141172).** The first form compared
only the name, the item and the geometry, and the skip of an identical
tool did not check the rows of its stack. A same-id row with other
running data was reused, and the imported tool then derived the
document's spindle speed, not the file's. A changed seat position of a
skipped tool's row was ignored. Both now refuse the import by the id,
with one comparison for each import path. Dropped: let a changed row
make the tool "not identical", so that it goes through the update path.
That gives a different answer in each import mode for the same row
clash.
