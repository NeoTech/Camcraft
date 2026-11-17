## #1953 - UI refit test plan: named page layout tests, behaviour without positions

**Decided (unattended, issue-worker-high):**

1. **One named layout test for each property page and the shell, in one new
   file** (`crates/ui/src/egui_app/tests/page_layout.rs`): `layout_shell`,
   `layout_shell_light`, `layout_page_<page>` for each `UiTarget` page
   (Stock, WCS, Model, Tool library, Tool, Operation, Setups, and the setup
   pages Settings, Machine, Post, Fixtures, Tools) and
   `layout_window_debug_info`. Each asserts a coarse law only: the page's
   anchor widgets are inside the Properties leaf and stack top to bottom.
   Why: a refit PR then updates one small test with the page's name, and a
   failure says which page moved. Dropped: one exact-rect golden per page
   (it would fail on each spacing change and say no more than a PNG).
2. **The shell skeleton is split.** `structural::assert_shell_skeleton`
   now asserts presence, labels and enabled state only. Its positions moved
   to `page_layout::assert_shell_layout`. The two theme tests
   (`shell_structure_dark` / `_light`) keep their theme-token assertions:
   the theme is the behaviour there, and they compare to `Tokens::*`, not to
   literals.
3. **Hard-coded screen points go through `tests/anchors.rs`.**
   `pane_center(tab)` and `chrome_point(avoid)` replace `pos2(4,4)`,
   `pos2(1100,500)`, `pos2(640,712)` and the `y > 600` status lookup.
4. **Painted strips wait for AccessKit nodes.** The ribbon cells and the
   browser rows are painted, not widgets. About 40 tests reach them by plan
   geometry or row arithmetic. A label query cannot find them until the
   strips publish AccessKit nodes. That is UI code, not test code, so it
   goes to follow-up issues and the PR says `Refs #1953`, not `Fixes`.
5. **The rule goes in root `AGENTS.md`** ("A UI refit PR keeps its tests")
   and is restated in `crates/ui/AGENTS.md`. Why: `crates/ui/AGENTS.md` says
   that it adds no new rule, so the rule itself belongs in the root file.
6. **Item 4 (the quick gate runs the page's UI tests) needs no gate change.**
   `cargo xtask gate quick` already runs the `camcraft-ui` lib tests for any
   change under `crates/ui/src`, and that binary holds the structural twin and
   the page layout tests (about 2 min wall time). A per-page selection would
   save little and could miss a shared-helper change.
7. **No pixel snapshot is touched.** The v0.20.0 reading (issue comment)
   found no UI failure in that release gate; the snapshot cost was in the
   v0.19.0 cut. The refit leaves the PNGs to the one regeneration commit at
   the cut.

**Cost to change later:** low. The page tests are data (a target and its
anchor labels). A refit that renames a widget edits one list.
