## #3645 - Tool components: the project file, the user component library and the library sync (slice 1)

**What was decided.**
- This PR is the first slice of #3645. It does the `.camproj` table and
  stacks, the user library store with its built-ins, the sync status and
  the undoable `SyncDocumentComponent`. The `.camtools` export and import
  of components is left on #3645 (see below).
- One wire form for a component row, `project::component_wire`, serves
  the project file and the library file. A row is
  `{id, name, item, geometry, library_ref}`. `item` is one of `insert`
  (the ISO 1832 code text), `solid` (the mount code and the tool record
  as a `.camproj` tool states it), `tool_item` (the mount code and the
  seats as ISO 5608 letters, size and thickness) and `adaptive` (the two
  connection codes). The codes are written as their text, which parses
  back to the same value (#3642). `geometry` holds the port frames, the
  body (`base` and `[z, r, arc]` points), the cutting point and the bore
  stops. Each empty key is not written.
- Strict reading: `deny_unknown_fields` on each record, every row built
  again through the checked constructors, an id twice refused, and each
  refusal names the row by its id. A component's tool record must not
  state `body`, `holder`, `cutter`, `number` or `assembly`.
- `.camproj` v84: a top-level `components` table and an `assembly` on
  `ToolDto` (the #3644 `ToolAssembly`: an id and a `Mount` per node). The
  loader reads the table and each stack before any world write, checks
  that each link names a row and that each stack resolves (the #3643
  resolver), and then lands them through `table_set` and `set_assembly`.
  This replaces the clear-on-open line of #3644.
- The derived `ToolDef` and `ToolBody` are not derived again on open.
  They are saved as before and they are what the document judged and
  posted. A newer resolver, or a hand edit of a tool record (still
  permitted, #3644), must not shift a posted program or make a saved
  project fail to open. The next component edit derives them again.
- The version step: `PROJECT_VERSION` 83 -> 84. Every file this build
  writes says 84, so a v83 build refuses it and never drops the table
  silently. Files of v83 and older open unchanged (no table, no stacks;
  a #2016 `holder` reference reads as before). This repository marks
  each such step as a compatibility break (`BREAKING CHANGE` footer, as
  for v82 and v83), and the issue asks for it, so the commit is
  `feat(tools)!:`.
- `library_ref {id, content_hash}` on a document row. The hash is FNV-1a
  64 over the compact wire JSON of the name, the item and the geometry
  (`fnv1a64:<16 hex>`). The id is not content, so a document row can
  link to a library copy with a different id. FNV-1a is public domain
  and already in the tree (`stock_print.rs`); no new dependency.
- An open never applies a library copy. `component_library::sync_status`
  compares the document row with its library copy: `NotLinked`,
  `UpToDate`, `UpdateAvailable { library_hash }` or
  `MissingFromLibrary`. Only the explicit sync changes the document.
- `SyncDocumentComponent` puts the library copy into the row (the row
  keeps its document id), records the link, and goes through the edit law
  of `EditDocumentComponent` (`with_row`): every tool built from the row
  is derived again and judged by every `ChangeTool` law, all in one undo
  step. A sync that changes only the link (the first link of the same
  content) derives no tool again. A sync of a row that is already the
  linked copy is refused, so the history gets no empty step.
- A hand edit (`EditDocumentComponent::new`) keeps the row's link, so
  the status then shows that the document copy differs.
- The user library: `camcraft-components/1`, `units: "mm"`, a
  `components` array of rows without `library_ref`. The folder is
  `CAMCRAFT_COMPONENTS` (an absolute path), else
  `<platform data dir>/camcraft/components`, the file
  `components.camcomponents`, as the fixture library resolves its folder.
  A save writes a temp file in the same folder and renames it.
- Built-in components are read-only and are not in the user file. Their
  ids start with `builtin:` (`builtin:holder:<entry>`,
  `builtin:tool:<catalog name>`), and the parser and the save refuse
  that id space in a user file. A holder library entry with a taper is
  an adaptive item (the taper, then the ER size or `WELDON <bore>`) with
  `ComponentGeometry::holder`. A standalone straight-shank chuck states
  no shank size, so it is not a built-in. A starter catalog tool is a
  solid tool on `CYL <shank>`.

**Why.** The issue asks for portable projects that never shift posted
G-code without a word, and for strict, named refusals. One wire form
keeps the project and the library from drifting apart.

**Considered and dropped.**
- Derive the records again on open and refuse on a difference. Dropped:
  a hand edit of an assembled tool, or a newer resolver, would make a
  saved project fail to open.
- A conditional version step (only a file with a table says 84, as v79
  did). Dropped: the ladder folded the last conditional step, and the
  repository steps every file now.
- `serde` on the `camcraft-tools` records. Dropped: #3642 kept the model
  free of a file format; the DTOs are in `app::project`.
- A hash from the standard library hasher. Dropped: its output is not
  stable between Rust versions.

**Left on #3645 (not in this slice).**
- `.camtools` with components and assemblies. The issue says v5, but the
  file is at version 7 on `main` (`CUTTER_HAND_VERSION`), so the step is
  version 8. The import must land the rows in the table through the
  `ImportTools` undo unit, which is a change to `commands/tool_import`.
- The MCP tools are #3647 and the UI is #3648.

**Cost to change later.** Low to medium. The wire form is the contract:
a change to a key needs a project version step. The commands and the
status are small and are pinned in
`crates/app/tests/tool_library_holders/component_files.rs` and
`component_sync.rs`.

## The #3646 cutter facts and running data on file (merge of #4249)

**Chosen.** #4249 (#3646) gave `DocumentComponent` two fields: `milling`
(the catalog facts of a multi-seat cutter body) and `running` (an
insert's tool material and cutting rows). The v84 component row carries
them as two optional keys, `milling` and `running`. Each key is strict:
an unknown key is refused, and the reader applies the same row law as
the component commands (`row_refusal`: cutter facts only on a tool item,
running data only on an insert). A `.camtools` v8 row uses the same wire
form. The user component library (`camcraft-components/1`) does not
hold the two keys and refuses a row that states them. A sync keeps the
document row's own facts and running data, as a hand edit does. The
content hash stays over the name, the item and the geometry.

**Why.** The #4249 PR body asks the second PR to merge to carry the two
fields into this file format. Without them, a saved multi-seat cutter
loses its facts and its insert loses its cutting rows on reopen. The
library format stays as it is so that the library, its hash and its
built-in rows do not change in a merge.

**Considered and dropped.**
- The library holds the facts too, with the facts in the content hash.
  Dropped for now: it changes `LibraryComponent` and every caller in
  the stacked PRs (#4268, #4271, #4280). A follow-up issue can add it.
- Accept and drop the keys in a library file. Dropped: a silent loss.
- Save a document row with the facts to the library and drop them.
  Dropped: a silent loss. `LibraryComponent::from_row` refuses such a
  row by name, and each save path (MCP, UI) goes through it.

## The #3643 lathe body on file (merge of #4278)

**Chosen.** #4278 (#3643) gave `DocumentComponent` a `lathe_body`
field: a lathe component's body as facets in its own frame. The v84
component row carries it as an optional key, `lathe_body`, with one key
`facets` (`[[x, y, z]; 3]` each). The key is strict: an unknown key is
refused, the facets are built again through `LatheBody::mesh` (no
facet, or a non-finite corner, is refused by the row's name), and the
reader applies the same row law as the component commands
(`row_refusal`). A `.camtools` v8 row uses the same wire form. The user
component library (`camcraft-components/1`) does not hold the key: it
refuses a library file row that states it, and
`LibraryComponent::from_row` refuses a document row with a lathe body by
name. Follow-up #4295 covers adding these row facts to the library. A
sync keeps the document row's own lathe body, as a hand edit does. The
content hash stays over the name, the item and the geometry.

**Why.** The #4278 PR body asks this PR to save and load `lathe_body`
with the other row fields. Without it, a saved turning stack loses its
holder body on reopen, and the next derive gives a different body.

**Considered and dropped.**
- Write the boxes that made the body. Dropped: `LatheBody` keeps only
  its facets, so the boxes are not known at save time. The facets give
  the same value back, and the file is byte-stable.
- Accept and drop the key in a library file. Dropped: a silent loss.
