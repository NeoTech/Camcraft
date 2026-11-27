## #3194 - Quick ops: sketch recipes as profiles, one undo step, MCP tools

**Decided.**

- `SketchRecipe` lives in `camcraft-geometry` (`sketch_recipe.rs`). It compiles a rectangle (optional corner radius), a circle, an obround or a regular polygon to exact `Line`, `Arc` and `Circle` primitives in a `SketchData`, on the layer `outline`.
- The outline is centred on the profile origin, closed and counterclockwise. A polygon has one flat at the bottom, parallel to X. The placement (`position`, `angle`) moves it on the stock face, as for each other profile.
- Each size is 0.1 mm to 100000 mm. A polygon has 3 to 1000 sides. A straight edge is absent (exactly 0) or at least 0.1 mm long. A corner radius from 0 to half the smaller side is accepted. An obround with its width larger than its length is refused. Each refusal names the shape and the field. Nothing is clamped.
- `Sketch` gets `recipe: Option<SketchRecipe>`. `SketchDto` stores it as `recipe` (tagged by `shape`, `deny_unknown_fields`). `PROJECT_VERSION` goes from 84 to 85, as a compatibility break, by the pattern of v82-v84.
- On load, the stored recipe is compiled again. It must give the stored primitives bit for bit, else the file is refused as corrupt by the profile's name.
- `CreateSketchRecipe`, `EditSketchRecipe` and `AddQuickOp` are new undoable commands. `EditSketchRecipe` writes the new outline. The operations on the profile go stale through their existing stamp, which hashes the outline.
- `AddQuickOp` is one `BatchCommand` of `CreateSketchRecipe` and `CreateOperation` (2D Contour, `SrcRef::Entity(profile)`). The operation's gates need the profile in the world. So the constructor applies the profile, builds the `CreateOperation`, and inverts the profile before it returns. A refusal leaves no world change and no history entry.
- The quick op uses the active setup, because `CreateOperation` always joins the active setup.
- MCP: `add_sketch_recipe`, `edit_sketch_recipe` and `add_quick_op` in a new file `mcp/sketch_recipe_tools.rs`. `add_quick_op` takes `depth` (top at the stock top, bottom at that depth) and an optional `params` patch over the 2D Contour defaults. `get_job` profiles show `recipe` (null on a drawing or a text).

**Why.**

- Geometry code has no ECS and no file I/O, so it belongs in `camcraft-geometry`. The app holds only the document, the file, the commands and MCP.
- A centred shape gives the same placement law for each shape, and the user moves it with the stock-face placement that exists.
- The exact load check keeps one law: the geometry of a recipe profile is its recipe's outline. A file that was edited by hand cannot carry a recipe that does not describe its cut.
- The trial apply in `AddQuickOp::new` keeps all `CreateOperation` gates in force without a second copy of them. It also keeps the history clean: a `History` group with an undo on refusal would leave a redo entry.

**Dropped.**

- The compact UI form on the Stock page. The text profile (#2679) also has no UI form, and a UI PR needs screenshots that this headless run cannot make. Filed as #4464 (P7).
- A script verb for each command. The MCP tools call the commands directly, as `move_dxf_profile` does.
- A row in `.claude/skills/camcraft-mcp/SKILL.md`. That file is a process file. The generated `docs/agents/mcp-tools.md` and the guide table list the tools.
- Clamping a bad dimension, and a silent conversion of an obround with width larger than length.
- A recipe that wins over the stored outline on load (it would change the geometry of a saved file in silence if the compile changed).

**Cost to change later.** A change of the compiled outline (for example, a different start point) makes the files saved before the change refuse on load. Such a change needs a `PROJECT_VERSION` step that compiles each stored recipe again.
