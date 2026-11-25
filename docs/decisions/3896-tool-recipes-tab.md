## #3896 - U5: the Recipes tab of the tool library window

**What was decided.**
- The window's tool editor gets a Recipes tab
  (`tool_library_dialog/recipes_tab.rs`, arithmetic in `recipes_fill.rs`).
  It draws the same cutting rows as Properties
  (`tool_cutting_rows::cutting_rows_with`). A small hook (`Extra`) adds
  read-only cells after each speed-and-feed row: surface speed
  `Vc = pi x D x n / 1000`, feed per tooth `fz = F / (n x z)` and the
  "document material" mark. An inch document shows sfm and inch.
- The mark is on the plain row that names the document's material
  (`cutting::material_of`). When no row names it, the mark is on the
  default (`Unknown`) row, because that is the row the tool runs. With no
  material stated, no row is marked.
- "Suggest" fills the draft row of the picked material (the document's
  material by default) from `camcraft_app::cutting::compute`, which is the
  pure table of `crates/tools/src/cutting.rs` (`WorkMaterial::window`,
  `rpm_for`, `feed_for`) with the document machine's stated spindle range
  and feed ceiling. It writes only the window's draft. It does not use
  `ApplyCuttingData`, which acts on an operation. A missing row is added;
  the row's stepdown and stepover stay.
- "Copy from tool" copies the plain material rows of another document tool
  into the draft: a row of the same material is replaced, a new material
  joins, and the draft's default row, its recipes and its other rows stay.
  The source's default row is not copied: it is that tool's own running
  speed.
- The tab also has the "Tool material" and "Flutes" rows (draft fields),
  because Suggest and fz need them and the catalog tools state neither.
  The operation page's Feeds tab still edits the same fields.
- Apply stays the editor's one Apply: one `ChangeTool`, one undo step. No
  command is new, so no MCP tool is owed. MCP agents set the same rows
  with `edit_tool` (`cutting_data`).
- The Suggest material and the Copy source are egui temp memory of the
  window (view state, not saved, not undoable).
- The tests of `tool_cutting_rows.rs` move to `tool_cutting_rows_tests.rs`
  (a `#[path]` module, no module-list change), so the widget file gets
  shorter (324 to 225 lines) while it gets the hook.

**Why.** The issue's "Done means": the tab lists the rows with Vc and fz,
Suggest and Copy fill the draft, Apply is one `ChangeTool` undo step, and
the row of the document material is marked. The Suggest helper stays in
the UI over public functions, so the quick gate stays on `camcraft-ui`.

**Considered and dropped (held for a follow-up).**
- "The cutting group leaves the property summary." U4 (#3931) held the
  Properties summary page back to #3930. Properties keeps its Cutting data
  group until that summary lands; #3930 removes it there.
- A Suggest button on each row. One "Suggest for <material>" row keeps the
  table narrow, and it can add a row that is not there yet.
- Vc and fz as editable columns that write back into rpm and feed. The
  issue asks for read-only columns.
- A new `crates/tools` helper for Suggest. `cutting::compute` already does
  the arithmetic with the machine clamps; a second copy could drift.
