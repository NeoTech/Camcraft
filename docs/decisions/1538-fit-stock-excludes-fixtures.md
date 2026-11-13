## #1538 - Fit to Models moves into the app as one parts-only rule, with an MCP tool

**What was decided.** The box, cylinder and per-side stock fits and the Stock page's "Models:" readout measure the setup's parts only: `stock_fit::part_bounds` (the #1529 `setup_align::part_aabb_in` set) and a new radius twin `stock_fit::part_radius_about_in`, both leaving out every model that carries a fixture definition (vises, accessories, plates, tagged meshes, in any setup). The fit itself (bounds, shape, the stock's kept rotation, the undoable `SetStock`) moved out of the GUI's `State` into `camcraft_app::stock_fit::fit_command`, which the GUI's Fit button and a new MCP tool `fit_stock` (box / cylinder / per-side) both call. The WCS pane's "model bounds" anchors (`interact::anchor_reference`) read the same parts-only box, so a G54 set from model bounds lands on the part's corner, never the vise's (added at the coordinator's request). Collision checks are untouched: they still read every fixture.

**Why.** The GUI was the only place the fit existed, so MCP had no way to fit stock and the rule could not be pinned headlessly; one function keeps both surfaces on the same set and the same refusal text. A stock that wraps the vise is wrong output a user would cut air with (or clamp a block that is far too big).

**Considered and dropped.**
- Swap `solid_aabb_in` for `part_aabb_in` at the three GUI call sites only. Dropped: leaves MCP without a fit and the logic in the UI layer (AGENTS.md layering).
- Also leave a `stock_source` "raw stock only" model out of the fit. Dropped for now: #1529's heights use the same `part_aabb_in` set; changing one without the other would split "the model" in two. A follow-up can change both together.

**Cost to change later.** Low: one 84-line app module, one MCP tool file, three UI call sites.
