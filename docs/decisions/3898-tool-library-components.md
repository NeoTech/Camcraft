## #3898 - Tool library window: My library, user holders and the component stack (U7)

Stacked on #4233 (component library store) and #4201 (component table and
assembly commands). UI ownership is with the account owner and one
contributor; each choice below is conservative and can be reversed.

**What was decided (UI).**
- **"My library" is a third source of the tools list** (`Document`,
  `Catalog`, `My library`), as the #3663 plan says. It lists the
  component library (the built-ins, then the user's file) by group
  (Holders, Adapters and extensions, Cutter bodies, Inserts, Solid tools),
  with a name filter. Below it stands the document's component table with
  the library status and the users of each row. No new tab, menu, chrome
  or theme: the page copies the Holders page's pattern (a group column on
  the left, a grid of rows).
- **Pick a row, then act.** A click on a name picks the row; one row of
  buttons acts on the picked row (Add to document, Copy as user component,
  Show compatible parts, Delete from My library; for a document row: Edit
  component, Sync from library, Save to My library, Delete from document).
  Dropped: a button in each row. It repeats one label many times, which
  the AccessKit tests and a screen reader cannot tell apart.
- **The compatible filter** is "Show compatible parts" on the picked
  component: the parents that take it and the children it takes
  (`compatible_parents` and `compatible_children`) over the library and the
  document. "Show all parts" clears it.
- **The component editor stands in for the page** while it is open (as
  the holder view does on the Holders page), with "Back to the list". Its
  fields: the name; an insert's ISO 1832 code; a holder's gauge length,
  only when its built-in shape is known (by its library link, its
  `builtin:holder:` / `user:holder:` id, or its name), because the outline
  is then the holder library's at that length. A holder without a known
  shape keeps its geometry: only its name edits. Dropped: a free outline
  editor for user holders (larger work; it can follow).
- **The gauge length field is in mm** (the component library is mm by its
  format). Dropped for now: unit conversion of this one field.
- **User holders.** A built-in holder's view gets "Copy as user holder";
  the copy is a new document row `user:holder:<name>` (then ` (2)` ...),
  not linked, and it opens in the editor. The Holders page lists the
  document's holders and adapters ("USER HOLDERS") above the built-ins.
  "Save to My library" writes it to the user's file and links the row.
- **The component stack** stands in the Tool tab's assembly group, below
  the holder pick: a grid of the links (port, turn about Z, stick-out,
  library status, "Edit <part>" for the shared part), "Add component"
  (only the parts that fit the last holding part, or the holding parts of
  an empty stack), "Remove last component", "Apply stack", and "Detach to
  standalone tool" (clears the stack, the tool keeps its record). The
  links are a window draft; Apply is one `SetToolAssembly`. A stick-out of
  0 means "home on the bore stop" (`projection_mm: None`). A solid tool
  added to a stack starts at its flute plus shank length, as U4's
  projection does.

**What was decided (document and file).**
- Every document change is one undoable command of #4201/#4233. "Add to
  document" is `UpsertDocumentComponent` then `SyncDocumentComponent` (to
  record the link) folded into one undo step with `History::begin_group`
  and `commit_group`.
- The user library file is not the document. "Save to My library" and
  "Delete from My library" are `component_library::store` writes with no
  undo, as for a machine file (#4233's store law). The save then links the
  document row through the undoable sync. A built-in copy cannot be saved
  under its `builtin:` id: the user copies it as a user component first.
- The window reads the library file once when it opens and after each
  write of it (`State::component_library`, a counter test pins that a
  still frame reads nothing). The library status, the users, the
  compatible set and the stack's "Add component" offer are cached in
  State (`state_component_views`), not in a page (review of #4271). The
  component table and the stacks that use each row are read once for each
  document version; a set builds again only when that read differs, the
  library read changes, or the filter or the stack draft changes. Chosen
  over a page-owned thread-local keyed on the document version alone
  (`crates/ui/AGENTS.md` "Frame cost"); dropped: a revision counter in the
  app's component table, which would change the app crate for a view
  cache. Counter tests pin a still frame, an unrelated document change
  and each changed input.
- "Copy as user component" of a document row copies the whole row (the
  #3646 cutter facts and running data too) under the new `user:` id, with
  no library link (`UpsertDocumentComponent::with_row`; review of #4271).
  `camcraft_app` also re-exports `milling_cutter` for the test that pins
  it.
- The folder is `store::user_dir()` unless `State::component_lib.dir` is
  set (the tests set a temp folder, so no test writes the user's file).
- App side, minimal and outside `commands/`: `camcraft_app` re-exports
  `component_fit`, `frame`, `insert_iso1832`, `insert_seat` and
  `tool_assembly::Mount`, so the UI does not take a new crate dependency.

**Not done here (open on the issues).**
- A turning tool's stack: the lathe side of the resolver is open on #3643,
  so a milling insert in an indexable cutter stands in for #3648's turning
  insert in the tests.
- Derived read-only values in the stack (gauge length from the assembly),
  and a component swap in place: they can follow.
- MCP tools for these commands are #3647 (no MCP file changes here).

**Cost to change later.** All of it is UI and view state, except the
four re-exports. A different layout changes `components.rs`, `stack.rs`,
`component_edit.rs` and their tests only.
