## #889 - tool library files carry the tool body; "Import tool holders" on the import page

**What was decided.** Built to the user's direction (2026-11-11: "make it
an import toggle for the user on the import page").

1. *Format.* A `.camtools` tool's `definition` may carry `body`, the very
   `ToolBodyDto` a `.camproj` tool stores (drawn holder profile, placed
   mesh, or lathe holder with its insert zone). Additive, no version step:
   `camcraft-tools/1` stays, a file without the key reads as before, and an
   older build parses the key into `ToolDto::body` and drops it (it reads
   only the tool record). Export writes `body` for every tool that has one.
2. *Safety.* A body is judged at read exactly as the project loader
   (`project::validate`) and `SetToolBody` judge one: re-derived by this
   build's envelope law (`ToolBodyDto::to_body`) and fitted to its tool
   (`tool_body::fits`); any refusal refuses the whole file naming the tool.
   `ImportTools` judges the fit again at command creation. Never a body the
   holder laws would misread.
3. *Toggle.* "Import tool holders" is a checkbox beside Import library on
   the Tool library page, **ticked by default**: a library that carries
   bodies was exported from a document where someone fitted those holders,
   so taking them is the expected round trip, and a file without bodies
   imports identically either way. It is the `holders` parameter of the one
   undoable `ImportTools::with_holders` command (`ImportTools::new` = on),
   not a field poke. Unticked lands every tool bare (holder cylinder only).
4. *Remembered per user* (the issue's own comment, cf. #877): kept in the
   per-user prefs file as `importToolHolders`, written only when false, so
   every older prefs file reads as on. Not project state, not on history.
5. *Identity for the skip.* A library tool that lands bare (no body, or
   holders off) is "already in the document" when the tool record matches,
   exactly as before - so an old bodyless file never duplicates a tool the
   user has since fitted a holder to. A tool that lands with a body matches
   only when the document tool's body is the same too (compared as the DTO
   a save would write); otherwise it arrives renamed "Name (2)" and the
   document's own tool and holder are never overwritten.
6. *MCP.* `import_tool_library` takes optional `holders` (boolean, default
   true); its schema and description say so; the reply adds `holders` and
   `holders_left_out`. The script verb `cam.importTools(path)` keeps its
   arity (holders on) - changing it would move the generated TS bindings.
7. *Counts.* The status line and MCP reply name how many tools landed with
   holders and how many file holders were left out.

**Considered and dropped.**
- *Showing the count before the import.* The toggle stands on the page
  before any file is picked, so the count arrives in the status line after
  the import instead; a pre-import preview dialog would be a new modal for
  little gain.
- *Storing the holder-library entry name.* A landed library holder is a
  drawn profile (`SPUN_SOURCE`), and the project file stores no entry name
  either; the profile itself is what the laws read and round-trips
  bit-identical. Adding a name is a separate format change.
- *The drawn cutter outline (`ToolCutter`).* Display-only data, outside the
  holder question; left for a follow-up if wanted.
- *A version bump of the library format.* Not needed: the key is additive
  and an older reader over-reports nothing (it imports the tool bare).

**PROCESS_VERSION.** The camcraft-mcp skill line for `import_tool_library`
changed, so this PR bumps `.claude/PROCESS_VERSION` 16 -> 17. Other open
process PRs also claim 17-20; whichever merges later takes the next number
when it merges main.

**Cost to change later.** Low: the default of the toggle is one field
initialiser and one prefs default; the identity rule is one closure in
`commands/tool_import.rs`.
