## #1815 - Rest machining group on the 2D Pocket page

**What was decided.**
- Place the `"Rest machining"` group inside 2D Pocket's `Passes` tab using
  `layout::group(ui, "Rest machining", Some(&mut armed), ...)` so the
  header switch defaults off (`rest: None`) and hides all four parameters
  until armed.
- Extract the group into a dedicated `crates/ui/src/egui_app/rest_pocket_page.rs`
  module instead of adding ~80 lines onto `mill2d_page.rs` (already at 402
  lines).
- Pass the mounted tool's diameter via `Cx::tool_diameter` in
  `params_page.rs` so toggling Rest machining ON seeds
  `previous_tool_diameter = (d * 2.0).max(1.0)` when a valid tool is mounted
  (falling back to `12.0` mm), with `0.0` for `previous_corner_radius`,
  `previous_stock_to_leave`, and `allowance`.
- Use concise non-ellipsizing row labels inside `LABEL_W = 120.0` with full
  descriptions on hover via `prop_row_hint`: `"Prev. tool Ø"`,
  `"Prev. corner rad."`, `"Prev. stock left"`, and `"Rest allowance"`.
- Show inline `consequence` notes when rest machining is active alongside
  an incompatible `PocketStrategy` (`rest_strategy`) or `approach.predrilled`,
  while keeping validation and named refusals on `Apply` through
  `EditOperationParams` (`#1489`).

**Why.**
- Keep draft edits aligned with every other operation parameter page: toggling
  and typing edit only the page draft; nothing commits to the world until
  `Apply`, preserving undo/redo and `#1489` named command-door refusals.
- Visual inspection of the initial render showed longer prose labels (`"Previous
  tool diameter"`, etc.) truncating with `...` at `LABEL_W = 120.0`; compact
  labels matching `tool_cutter_params` (`Ø`) read cleanly without clipping.

**Considered and dropped.**
- Exposing rest machining on 2D Adaptive or 2D Offset in this PR: `#1489`
  only implements rest machining for 2D Pocket (`OpParams::Pocket`), so
  `Adaptive2D` and `Offset2D` remain unchanged.
- Live clamping `previous_tool_diameter` in the UI instead of letting
  `EditOperationParams` refuse it: violates issue `#1815` and the command-door
  refusal law ("refused by name on Apply").

**Cost to change later.** All UI widgets live in one 82-line file
(`rest_pocket_page.rs`), so relabelling fields or adding an operation-level
"from previous operation" picker is a single-module edit.
