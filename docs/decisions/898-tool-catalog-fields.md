## #898 - tool catalog fields (part number, vendor, coating, helix, notes)

**Decided.**
- **Model.** `Tool` gains `catalog: ToolCatalog` (`camcraft_tools::catalog_fields`,
  own module because `tools.rs` is already past 400 lines): five optional
  fields, `None` = not stated. Because they live on `Tool`, `AddTool` /
  `ChangeTool` (and so undo/redo, one step per Apply) and the library import
  carry them with no new command. They are inert: no generator, check or post
  reads them, so no golden moves.
- **Files.** `ToolDto` gets five additive keys (`part_number`, `vendor`,
  `coating`, `helix_deg`, `notes`), each `skip_serializing_if = None`: a tool
  without facts saves byte for byte as before, no version step. `.camproj`
  and `.camtools` share `ToolDto`, so both round-trip. An older build skips
  the keys (the tool DTO has no unknown-key refusal) and loads the tool; it
  just loses the facts on its next save. Written-only-when-stated was chosen
  over always-writing so old builds and diffs see no change for existing tools.
- **Validation.** At command creation (`AddTool::new`, `ChangeTool::new`): text
  is trimmed and blank reads as not stated, then refused by tool name when a
  part number / vendor / coating exceeds 80 characters, notes 500, a short field
  holds a control character (notes may hold newlines), or helix is outside
  0..=90 or not finite. `Tool::is_sane` carries the same check, so a hand-edited
  library or project file refuses by tool name too; the DTO trims on read so
  mere padding in a file loads.
- **Editor.** A collapsed "Catalog" header after "Material group" (open when any fact is stated, so the editor keeps its height) (new module
  `tool_catalog_rows.rs`; the Body column is untouched, for #1043). Draft text
  fields; the helix is text so a half-typed number is refused by name at Apply
  instead of silently dropped.
- **MCP.** `add_tool`'s `definition` takes the five keys (same DTO) and
  `get_job` shows them. There is no MCP tool that edits an existing tool at
  all, so editing is filed as `MCP: edit_tool` rather than invented here (#1062).

**Not done, on purpose.**
- #1060's Update/Replace still match by name. Matching on part number first
  changes which tool a library entry overwrites (a rename would become an
  in-place update of a differently named tool); that is a behaviour choice for
  the maintainer, not a safe side effect of a data field. The seam is still the
  match in `ImportTools::with_mode`.
- The setup sheet's tool list does not show the fields yet (#1063); the sheet is a
  post/printing surface of its own.

**Considered, dropped.** A separate `SetToolCatalog` command and a component
beside `ToolDef.number` (one more undo step per edit, a second Apply path);
helix as a drag number (cannot express "not stated").
