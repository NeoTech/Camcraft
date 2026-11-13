## #890 - a tool's cutting data per workpiece material

**Decided.**
- **Model.** `Tool` gains `cutting: Vec<CuttingRow>` (`camcraft_tools::cutting_rows`,
  own module; `tools.rs` is past 400 lines). A row is `{material, rpm, feed}`
  keyed by the built-in `WorkMaterial` table - the same groups the document's
  material (M13.2) states - so a row matches the document exactly, never by
  fuzzy text. The tool's own `material_group` / `rpm` / `feed` stay what they
  were: the running numbers the post reads (`posting::posted_rpm` reads
  `Tool::rpm`, the tap cycle reads `rpm x pitch`) and the fallback when no row
  names the material. In effect today's single row is the default entry and
  the table holds the others.
- **Why the running numbers stay the posted ones** (rather than resolving rpm
  by material at post time): `Tool::rpm` is read in about 46 places, generation
  among them (the tap's feed). Resolving it per material at the post would let
  the posted S and the generated tap feed disagree, and would move posted
  speeds for an existing project with no command in the history. So a row
  moves numbers only through a command, one undo step each:
  - `SetWorkMaterial` re-seeds every tool with a row for the new material
    (`ChangeTool`) and the milling feed of every operation on such a tool
    (`EditOperationParams`). The status line says how many moved.
  - `AddTool` into a document with a stated material starts the tool on its row.
  - `ChangeTool` that edits the table but not rpm/feed loads the row into the
    running numbers. An Apply that states rpm/feed itself keeps them (so the
    Feeds tab's spindle row and `ApplyCuttingData` are untouched).
  - `EditOperationLinks` moving an op to a tool with a row takes the row's feed.
  - Quick-add (ribbon), MCP `add_operation` and the feature recognizer seed a
    new op's milling feed beside `seed_helix_for_tool`. An explicit MCP
    `params` patch still wins.
  - No row for the material: nothing moves. That is today's behaviour.
  Nothing runs on load: a project loads with the exact rpm/feed it saved
  (pinned by `loading_never_moves_the_numbers_an_existing_project_runs`).
- **Files.** `ToolDto` gets an additive `cutting_data` list, `skip_serializing_if
  = Vec::is_empty`. A tool without rows saves byte for byte as before, so there
  is no project version step (pinned: save -> load -> save is byte-identical).
  An older build skips the key and keeps posting the tool's own rpm/feed, which
  are exactly what this build last ran. It loses only the table, on its next
  save. The material is a word (`mild_steel`, ...). An unknown word refuses
  the file by name (serde `try_from`), never a silently dropped row. `.camproj`,
  `.camtools` and the MCP `definition` share `ToolDto`, so all three carry it.
- **Validation.** At command creation (`AddTool` / `ChangeTool`), by the tool's
  name: every row has a finite rpm > 0 (a torch's row states 0, the #670 law)
  and a finite feed > 0, and no material appears twice. `Tool::is_sane` carries
  the same law, so a hand-edited file or library refuses at read.
- **Editor.** A collapsed "Cutting data by material" header under the Feed row
  (new module `tool_cutting_rows.rs`; the Body column is untouched for #1043).
  It has rows of material combo, rpm and feed with Remove, and "Add material
  row", which takes the next unused material and starts from the running
  numbers. It is a draft like every tool field: Apply/Add carry it through
  `ChangeTool`/`AddTool`.
- **MCP.** `add_tool`'s `definition` takes `cutting_data`, and `get_job` shows
  it (same DTO). There is still no MCP tool to state the material (#732, open)
  or edit an existing tool (#1062). Both are existing issues, not new ones.

**Interaction with #1060 (tool library Update/Replace).** Import still matches
tools by name. An Update writes the library tool, rows and running numbers
alike, as the library states them. It does not re-seed from the document
material, because the library's own numbers are what the user asked to import.
The next `SetWorkMaterial`, or a table edit, seeds as usual. Replace removes
tools (and so their tables) like before. Rows are validated on import by the
same `is_sane` law.

**Not done, on purpose.**
- No per-material stepdown/stepover (the issue's "optionally"). These are
  operation geometry, not tool data, and seeding them would move toolpaths.
  That is a separate decision.
- Drilling, turning and plasma ops take no feed from a row (`cutting::feed_mut`
  answers only the milling feed). The running rpm still follows the row for
  every kind.
- The free-text `material_group` is not matched against the document material.
  It stays a label.
- The setup sheet does not list the table.

**Considered, dropped.** Resolving rpm/feed by material at post time (see
above). A separate `SetToolCuttingData` command (one more undo path, the same
data `ChangeTool` already carries). Free-text material keys (they cannot match
the document's material reliably).
