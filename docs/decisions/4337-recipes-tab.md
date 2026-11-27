## #4337 - Tool library dialog U11: the Recipes tab

**Decided.**

- The tab opens with one line: "One recipe for each work material." and the
  row that operations take. When no row names the stock material, the line
  says that the Unknown row runs. When no material is stated, it says so.
- A star (★, U+2605, in the body ink) stands in a narrow lead column on the
  row of the stock material in force: the active setup's stock material,
  else the document material (`stock_material::in_force`). Operations
  resolve the material in this order, so the star marks the row they take.
  U5 marked the row of the document material only.
- The derived values are one dimmed line for each table row, under the
  table: "= 226 m/min surface speed, 0.050 mm per tooth (Aluminium 6061)".
  A value that cannot be derived names its missing input. The U5 columns
  "Vc" and "fz" are gone: they stood right of Stepover and scrolled out of
  view in the narrow editor column.
- The lines are computed only when their input changes. The input is the
  list of (material, diameter, rpm, feed, flutes, unit) keys. The cache is
  `ToolLibraryView::recipes` (`RecipesView`). `computes` counts the misses
  for the frame-cost test.
- The buttons stand on one row under the lines: "Add material…" (a menu of
  the materials without a row), "Suggest feeds and speeds" (a menu: the
  stock material first, with the star, then each material that has a row)
  and "Copy recipes from tool…" (a menu of the other document tools; a tool
  without material rows is disabled and says why). A note with the units
  follows. Tool material and Flutes (the inputs of Suggest and of the chip
  load) follow the note.
- No command is new. Add, Suggest and Copy write the window's draft, as U5
  did. The editor's one Apply lands the draft as one `ChangeTool`, so each
  of them is one undo step after Apply. The tests pin this for each button.
- With the Recipes tab's `Extra`, the shared table does not draw its own
  "Add material row" button. Properties' Cutting data group keeps it.

**Why.** The mockup (screen 3) and the maintainer's design rules on #4245:
flat, structure from type, no boxes. The star and the lines use the
existing body and weak inks, no new colour token. Menus keep the button row
to three buttons, as in the mockup, and still let the user pick the
material or the tool.

**Dropped.**

- A row selector with one derived line for the selected row. It needed a
  column right of Stepover, which scrolls out of view in the editor column.
  One line for each row needs no selection and matches the mockup.
- An undoable command for each button. The editor's draft-and-Apply model
  (U2, U5) already makes each edit one undo step, and a command for each
  draft change would give two undo steps for one edit.
- A gold star colour. No theme token has it, and the rule is "existing
  theme tokens only".

**Cost to change.** Small: the star and the line text are constants and
one function in `recipes_tab.rs` and `recipes_derived.rs`.
