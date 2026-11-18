## #2016 - A tool's library holder is a reference to the entry

**Decided.** A tool that sits in a built-in library holder stores a
reference: the entry's exact name, the gauge length and the projection
(`camcraft_tools::holder_library::HolderRef`, on `ToolBody::library`).
The project file and the `.camtools` file write it as the tool key
`holder: {entry, gauge_len, projection}` (plus `machine_default: true`
for an assumed holder, #828) and write no `body`. One function turns a
reference into a shape, `tool_body::build_library`; the command
(`SetToolBody::library`, `AddTool`), the project loader and the tool
library reader all call it. `tool_body::judged`, `stack`, `digest`, the
simulation and the preview read the body it returns, as before.

**The document keeps the resolved body beside the reference.** The
`ToolBody` component still holds the outline, facets and bands. They
are derived data: nothing but `build_library` writes them for a library
holder, and no file stores them. The laws therefore read exactly the
steel they read before, with no second resolution path. Resolving on
every `judged` call was dropped: it would add a refusal path to every
law and gain nothing, since both lengths are stored in full.

**Both lengths are stated in the reference**, also when the caller left
them out (the entry's gauge length, the tool's flute plus shank length).
A later edit of the tool's lengths therefore does not move the holder;
`tool_body::fits` refuses at use when the holder no longer fits, as it
did for the drawn copy.

**Same shape, pinned.** For every library entry, at its default and at
other lengths, picked and assumed: the judged tool, the envelope, the
band stack, the stamp term and the facets of the referenced holder
equal those of the drawn copy an older build stored
(`holder_reference::every_entry_reads_the_same_bands_by_reference_as_its_old_copy`).
The stamp term is unchanged, so no operation goes stale.

**Migration: an older file is not rewritten.** A file without `holder`
loads as it did. A drawn copy of a library holder stays a drawn
(custom) profile with the same outline; a `holder_d`/`holder_len`
cylinder stays the tool's own numbers. Neither is turned into a
reference: the copy may have been edited, and matching an outline back
to an entry is a guess. The user picks the entry again to get a
reference.

**Version step.** Project files write v70 (`PROJECT_VERSION = 70`),
after v68 (#2441) and v69 (#1517) on `main`, and `.camtools` files write
version 4 (`HOLDER_REF_VERSION = 4`, after `CUTTING_STEPS_VERSION = 3`)
when a tool names a library holder (keeping earlier minimum versions
when no tool names one). `v0.22.0` reads project v62 and `.camtools` 2,
and pre-#2016 builds read up to project v69 and `.camtools` 3; both
refuse v70 and version 4 instead of dropping `holder` and judging the
tool's smaller holder cylinder.

**Refusals.** A reference to a name the library does not hold refuses
the file by tool name, and so does a grouped pick ("BT40 Weldon", which
names no bore), lengths the entry cannot be drawn at, a holder that
reaches into the flutes, an unknown key in the reference, and a tool
that states both `holder` and `body`. No other holder is put in its
place (#808). The shank fit is checked when the holder is set, not on
load: a file the app saved must open, and a drawn copy was not checked
on load either.

**A change of the library's own numbers now reaches old projects.**
A reference is drawn from the build's tables. If an entry is corrected,
every tool that names it follows. That is the point of a reference; a
copy kept the old numbers.

**Editing.** The outline of the referenced holder applied unchanged
(Apply profile, MCP `points`) keeps the reference. One edited point
makes it a custom drawn profile, in one undo step.

**Tool library import.** The import's "same tool" test still compares
the shape (the body as a save wrote it before). A drawn copy and a
reference with the same outline are therefore the same tool and are
skipped, not added a second time as "Name (2)".

**The holder fields.** With a referenced holder the tool settings show
the holder diameter and length the laws read (the cylinder that
encloses the entry) as text, not as two fields. The tool record's own
`holder_d`/`holder_len` are not overwritten: they are what the tool
goes back to when the holder is cleared, and writing them would move
the tool's stamp. An assumed (machine default) holder keeps the two
fields, because #828 floors it at them.

**MCP.** `set_tool_holder` with `library` writes the reference;
`get_job` and the reply show it as `holder.library`. A `holder`
reference inside `add_tool`'s `definition` is refused by name (the
definition's body was never read there; dropping a reference would land
the machine's default holder in its place). `edit_tool` refuses a
`holder` patch like `body`.

**Dropped.** Removing `holder_d`/`holder_len` from the tool record (every
law and every file reads them; the stand-in cylinder of a bare tool
stays). A reference field on `Tool` itself (the body component already
carries undo, the file key and the delete hook). User-defined library
entries (the reference names built-in entries only; a user library is
separate work).

**Breaking.** A project or tool library saved with a library holder is
not readable by v0.22.0 and older (project v70, `.camtools` version 4).
`ToolBody` has a new public field.

**Cost to change.** Small. The reference type, `build_library`, the
`holder` key (`project/tool_body_wire.rs`) and its readers.
