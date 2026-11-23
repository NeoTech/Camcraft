## #3073 - Saved cut recipes

**What was decided.**

- A cut recipe holds the operation kind with all its parameters, the tool
  by name and the workpiece material word in force when it was saved. The
  caller names the new source. Parameters can contain geometry, so the
  command refuses recipes that require new geometry before it creates an
  operation. The refusal names the recipe.
- Recipes live in a user library file, `recipes.camrecipes`, in a
  `camcraft-recipes/1` envelope. The folder is `CAMCRAFT_RECIPES`, else
  `<platform data dir>/camcraft/recipes`. This is the fixture library's
  shape. Saving a recipe is a file edit with no undo step, as for a saved
  fixture.
- `ApplyCutRecipe` is one undoable command. It wraps `CreateOperation`,
  so every creation gate applies. Undo removes the new operation.
- Refusals, each with the recipe's name: the tool is not in the document;
  the material word is not in the table; the material is not the
  material in force in the active setup; a named material has no matching
  material in force; the parameters carry geometry that is not rebound.
  Geometry includes hole lists, cross holes, authored coordinates, picks,
  and a tilted plane origin.
- MCP tools: `save_cut_recipe`, `list_cut_recipes`, `apply_cut_recipe`.
  Each takes an optional `library` path; omitted, the user library.

**Why.** Repeat jobs move between projects, so a library outside the
project file is the useful home. A recipe tuned for one material gives
wrong feeds and steps in another. A refusal is safer than a silent
operation with those feeds, and the user can change the stock material.
An absent destination material does not confirm the recipe feeds. The
command requires a present match when a recipe names a material.

A new source does not replace coordinates or entity picks inside saved
parameters. Reusing them can cut at the previous source location. The
command refuses these recipes until an interface supplies new geometry.
It keeps the stored settings so the library format does not change.

**Considered and dropped.**

- Recipes stored in the project file: they would not carry to the next
  job.
- Coolant and area controls in the recipe: coolant depends on the
  machine, and area controls depend on the geometry. A new operation
  takes the machine's default coolant.
- A material mismatch or absent material as a warning only: wrong feeds
  can break a tool.
- Copying embedded geometry to a new source: the coordinates and picks
  can still identify the previous part.
- Automatic geometry matching in this fix: no general correspondence
  exists between the previous part and the new source. A named refusal
  prevents a silent guess.
- A GUI panel for recipes: left for a follow-up issue. The command and
  MCP tools are the base it calls.

**Cost to change later.** Low. The envelope has a version, and the
command is a thin wrapper over `CreateOperation`.
